# Rust Build-Budget Measurement (Program 27 Phase 0, Task 3)

**Date:** 2026-07-29
**Commit:** `412de59e` (dev tip at measurement time)
**Box:** solo dev box, 12 cores / 31 GB RAM (13 GiB used, 683 MiB free, 18 GiB
buff/cache, 19 GiB swap in use at measurement time — other agents were active
concurrently on unrelated work; this task's cargo invocations ran serially,
single-flight, per the machine-safety rule)
**Toolchain:** `rustc 1.91.1` / `cargo 1.91.1` (pinned via `rust/rust-toolchain.toml`,
channel `1.91.1`)
**Workspace measured:** the only Rust we have — `rust/` (3 crates:
`babylon-md`, `babylon-tui`, `babylon-tui-python`)

## Measurements

All three runs used bash's `time` builtin (GNU `time`/`/usr/bin/time -v` is not
installed on this box — wall/user/sys/cpu% below come from the builtin, which
does not report peak RSS).

| Measurement | Command | Wall time | User | Sys | CPU% |
|---|---|---|---|---|---|
| **Cold build** | `cargo clean && cargo build` | **30.57s** | 203.87s | 32.46s | 773% |
| **Incremental build** | `touch crates/babylon-tui/src/app.rs && cargo build` | **1.46s** | 1.31s | 0.95s | 155% |
| **Test wall** | `cargo test -p babylon-tui` | **11.90s** | 62.10s | 15.51s | 652% |

Cold build compiled 28 dependency crates + the 3 workspace crates, dev profile
(`unoptimized + debuginfo`), from a fully clean `cargo clean` (752.2 MiB
removed). The dependency graph pulls in `pyo3`/`pyo3-macros` (the
`babylon-tui-python` PyO3 bridge), `ratatui`, `rustworkx-core`, `h3o`,
`log4rs`, and the `hypergraph-rs` git dependency (built from a pinned rev, not
cached as a registry crate) — the heaviest compile units by wall-clock order
in the tail output were `ratatui`, `h3o`, `sprs`/`rustworkx-core`, and the PyO3
macro crates.

`cargo test -p babylon-tui` compiled the test binary (reusing the just-built
dev artifacts — no full recompile needed) plus ran 12 unit tests (all `ok`,
0.00s runtime) and the doctest harness (0 doctests); nearly all of the 11.90s
wall time is test-binary link + doctest-harness compile, not test execution.

## Workspace stats

- `cargo tree | wc -l` → **469** lines (469 total dependency-tree entries
  across the 3-crate workspace, transitive closure, dev profile)
- `du -sh rust/target` → **2.2G** (dev-profile build artifacts after both the
  cold and incremental builds above)
- Workspace crate count: **3** (`babylon-md`, `babylon-tui`, `babylon-tui-python`)

## Engine-scale extrapolation

Program 27 Phase 1+ plans an **8-crate engine workspace** (`babylon-graph`,
`babylon-kernel`, and further BSL/engine crates layered on top of today's
3-crate client workspace). This extrapolation is **naive linear scaling by
crate count** — real compile time is dominated by the *heaviest* dependency
subtree and by cross-crate monomorphization, not a flat per-crate cost, so a
true 8-crate cold build could land anywhere from sub-linear (if the new
engine crates share most of today's already-compiled dependency graph — likely,
since `rustworkx-core`/`h3o`/`sprs` are exactly the kind of numeric/graph deps
the engine crates will also need) to super-linear (if PyO3 boundary crates or
a new heavy numeric dependency — e.g. `faer` per the numeric-closure audit,
Task 5 — get added per crate). It is used here only to **bound the decision**
between "do nothing" and "invest in build-speed tooling now," not as a
committed prediction.

| Scale | Crates | Naive linear cold-build estimate (crate-count × today's rate) |
|---|---|---|
| Today (client only) | 3 | 30.57s (measured) |
| Planned engine workspace | 8 | **~81s** (30.57s × 8/3) |
| Client + engine combined | 11 | **~112s** (30.57s × 11/3) |

Even the pessimistic 11-crate combined estimate (~112s cold, well under 2
minutes) stays inside a tolerable *cold* CI/from-scratch budget; the number
that actually matters for the day-to-day inner loop is the **incremental**
figure, and today's 1.46s single-file-touch incremental build has enormous
headroom before any linker/codegen intervention is warranted.

## Recommended posture

**Do not adopt `mold` or the cranelift dev backend yet.** Measured incremental
build time (1.46s) is far below the 30s threshold that would justify the
codegen-backend investment named in the plan, and cold build (30.57s) is not
an inner-loop cost — it only recurs on `cargo clean`, a fresh clone, or a
dependency bump. Concrete posture for Phase 1+:

1. **Linker:** defer `mold`. It is not installed on this box (`which mold` →
   not found) and the plan's own trigger condition ("only if measured
   incremental exceeds 30s") is not met — current incremental is ~20× under
   that bar. Revisit if the 8-crate engine workspace's incremental build (once
   it exists) crosses 10s, well before the 30s cranelift trigger, as an early
   warning.
2. **`opt-level`:** `rust/Cargo.toml` does not yet declare a workspace
   `[profile.dev]` override — it should, once the engine crates land, to pin
   `opt-level = 1` for engine crates specifically (numeric-heavy code is
   unusably slow at `opt-level = 0` at runtime, and `opt-level = 1` costs
   little at compile time relative to `0`). This is a Phase-1 action item, not
   a Phase-0 change (Phase 0 makes no engine-crate changes).
3. **`share-generics`:** already the default behavior for `dev`/`test`
   profiles on non-nightly stable `cargo` (`-Zshare-generics` is nightly-only;
   the stable equivalent, incremental compilation's codegen-unit sharing, is
   already active — this workspace already benefits from it with no config
   change needed).
4. **Cranelift dev backend:** explicitly NOT adopted per the plan's own
   trigger ("only if measured incremental exceeds 30s") — measured is 1.46s,
   ~20x under threshold. Do not add the nightly-only `-Zcodegen-backend=cranelift`
   toolchain requirement to a `rust-toolchain.toml` currently pinned to stable
   `1.91.1` for no measured benefit.
5. **Inner-loop budget:** state it explicitly for Phase 1 planning — **incremental
   builds must stay under 10 seconds** (an order of magnitude below the 30s
   cranelift trigger, leaving headroom for the engine crates before any
   codegen-backend intervention becomes worth its stable-toolchain cost). Cold
   builds have no hard budget in Phase 0 — they are not a per-commit cost —
   but should stay under ~2 minutes at the full projected client+engine scale
   per the extrapolation above; re-measure once the first engine crate lands
   rather than trusting the naive linear extrapolation further.

## Caveats

- No `/usr/bin/time -v` available on this box, so peak RSS during compilation
  was not captured — if memory pressure becomes a concern at the 8-crate
  scale, a follow-up measurement with `/usr/bin/time -v` (installable via the
  distro's `time` package, or `valgrind --tool=massif` as a heavier
  alternative) should be taken.
- Other agents were running concurrently on unrelated work during this
  measurement (13 GiB RAM in use, 19 GiB swap in use at start) — this task's
  own `cargo` invocations ran serially per the machine-safety rule, but
  ambient system load was not zero. The absolute wall-times above may be
  mildly pessimistic versus a fully idle box; the *relative* posture
  conclusions (incremental far under the 30s trigger) are robust to that
  noise margin.
- This measures the **client** Rust workspace only — the planned engine
  workspace does not exist yet, so all engine-scale numbers here are
  extrapolation, not measurement, and are labeled as such throughout.
