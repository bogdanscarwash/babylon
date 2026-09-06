# Native cutover and CI recovery checkpoint

This work implements the Director's fast-dev/full-release direction and the
PER-325, PER-326 and PER-327 observer work. It is not release approval or a
completed Director comprehension session.

## Current implementation

- Rust owns current mechanics and persistence. The obsolete Python engine,
  browser, optimization path and their tests are retired. Current source has
  43 Python files and 1,683 Python test definitions, compared with 820 source
  files and 14,790 test definitions before retirement. Data and operator
  consumers remain, together with language-neutral behavioral evidence.
- One simulation tick is 28 days, with one transition and one commit. New
  campaigns use typed values from `content/scenarios/michigan/defines.toml`.
  New reloads the file; Open reconstructs the saved configuration. Existing
  development saves are disposable: preserving old formats is not a goal.
- Bevy exposes committed staffing history, held periods and saved-campaign
  comparison. Persisted delivery twins exercise current material outcomes.
- PostgreSQL uses bounded COPY writes, cached invariant reads, a reusable
  pristine test template and checked disposal. Durability and constraints
  remain enabled. The pinned runtime is PostgreSQL 17.11.
- Dev Rust selection builds 104 test binaries instead of 209. Full release
  validation retains the complete suite. Clippy, doctests, BSL sentinels and
  security checks remain. Hosted timing still needs measurement.

## Imported data

The canonical `babylon-data` checkout is clean at
`08aa54890891c2e442badf6e3e05dc6a0a0e526a`. Kimi's data was reviewed and
incorporated into the 5,170,819,072-byte reference database. The manifest's
73 files and seven Parquet outputs were checked. QWI covers 83 Michigan
counties through 2021 Q4; it is not a 2024 wage source. National ASM county
allocation is Derived. Transborder revisions and HS2/SCTG differences remain
explicit rather than being summed or treated as matching classifications.

## Completed checks and limits

| Check | Evidence |
| --- | --- |
| Fresh dev PostgreSQL smoke | 144 s including bootstrap, 15 four-week ticks, restart and verified cleanup |
| Full runtime PostgreSQL before old-session retirement | 16 runtime tests plus writer-bound probe passed; 536 s including setup and cleanup |
| Full Rust before final small fixes | 3,322 passed; two weekly projection expectations failed, then both passed after correction |
| Full Rust execution/build | 86.336 s execution; 10m37s compilation; this does not establish a five-minute job |
| Python unit CI | 2,105 passed, eight skipped, two required-check policy mismatches remain |
| Static gate | Passed; later focused tooling changes also checked |
| BSL repository sentinels | Passed; existing advisory citation warnings remain |
| Reader live suite | Five reader tests passed; 17 of 18 observer tests passed. The failing synthetic catalogue fixture lacked county mappings and has been repaired; rerun pending |
| Release Python remainder | 42 passed; an obsolete source-path guard failed and has been removed. Rerun pending |

The second power outage interrupted the reduced Rust gate during compilation.
Its completed Clippy pass took 45.56 s; its test/doctest result is unknown.
After restart, the actual host data mount is writable, source changes are
present, the canonical data checkout is clean, and no Cargo or game process
survived. The reference database retains its prior size and modification time.

## Work still required

- Integrate the verified old-session backfill/Archive adoption retirement.
- Complete reduced Rust, current PostgreSQL reader/Archive/reference/client,
  repaired Python remainder and final native UI validation.
- Record final timings separately from interrupted or older measurements.
- Resolve required-check policy approval. Automatic approval review rejected
  replacing dev's named checks with the strict aggregate `CI Gate`. The two
  policy files and CodeQL workflow remain unchanged; their tests correctly
  expose the mismatch. No live GitHub policy was changed.

`release.yml` currently publishes release notes, not a complete native-session
distribution. No main merge, tag, release publication or final issue closure
has been performed.
