# Program 27 Phase 1 — Language & Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **ENTRY GATES — both are now CLEARED (2026-07-29):**
> 1. **CLEARED — the v3.0.0 Refoundation amendment was RATIFIED 2026-07-29** (spec §10 —
>    "Phase 1 does not begin until the v3.0.0 amendment is ratified"; drafted in Phase 0
>    Task 16, `ai/_inbox/amendment-v3-refoundation-draft.md`, merged to `CONSTITUTION.md`
>    as **Amendment AE — The Refoundation**, PR #365). **Phase 1 execution is UNBLOCKED
>    once #365 is on `dev`.** Verify before opening any branch:
>    `rg -n "v3\.0\.0" CONSTITUTION.md` and `git log --oneline dev | rg 'v3\.0\.0'`.
> 2. **CLEARED — Amendment D was RULED 2026-07-29: `babylon-graph`'s data shape unblocks
>    as NATIVE HYPEREDGE.** The Director ruled hyperedges are **first-class objects in
>    `babylon-graph`'s exposed model and type system** — membership is a single typed
>    hyperedge, never a clique expansion, and never *exposed* as a bipartite incidence
>    encoding; Levi/incidence remains sanctioned as an **internal storage strategy only**.
>    Sub-rulings D-2…D-7 (type-level separation satisfies II.7; `ECONOMIC_SECTOR`
>    membership becomes a true hyperedge; simplicial stays derived-only) are recorded in
>    `ai/_inbox/amendment-d-analysis-p27.md` §9 (PR #353) and registered constitutionally
>    in **Amendment AE clause (vi)**. Task 11 below is revised to that shape; Task 16's
>    verb list follows `docs/reference/bsl-language.rst` §2.8 as revised by the ruling.

**Goal:** Execute Phase 1 of Program 27 (spec:
`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`, §5 the BSL
language, §6 the Rust kernel, §8 correctness, §9 error handling/determinism, §10
sequencing) — stand up `babylon-kernel` and `babylon-bsl` in the in-tree `rust/`
workspace, transcribe the conformance corpus, get the Director's sigmoid ruling, and
land the `babylon-graph` trait boundary (whose data shape Amendment D has now ruled:
native hyperedge). Phase 1 ends when: the BSL language-agnostic reference is written,
both new crates pass `cargo test --workspace` + clippy pedantic + fmt, the transcribed
conformance corpus (899 lines, 4 documented corrections) is green, the sigmoid ruling
is in hand, and the graph trait boundary — dyadic edge API plus first-class typed
hyperedges — compiles against a placeholder implementation.

**Architecture:** Two new crates land beneath the existing `rust/` workspace's client
crates in the Program-14 layering law (`kernel < bsl < graph < domain < persistence <
engine < cli`): `babylon-kernel` (scalars, `Currency` i128, `ContentDigest`, sim clock,
event-bus port, RNG service) and `babylon-bsl` (reader, typechecker, load-time bound
checker, fuel evaluator) on top of it. `babylon-graph` gets a crate shell and a trait
only — no concrete graph type in Phase 1 — but the trait's *shape* is now settled by
Amendment D: dyadic edges and first-class typed hyperedges side by side. No
numeric intrinsics, no engine, no client wiring: those are Phase 2/3. Phase 1 is
Rust-only; nothing in `src/babylon/` changes (the Python engine is frozen behind the
Phase-0 freeze tag).

**Tech Stack:** Rust (workspace `rust-version = "1.85"`, `edition = "2021"`,
toolchain-pinned `1.91.1` via `rust/rust-toolchain.toml`), `cargo`/`clippy`/`rustfmt`,
`mise` task wiring, existing in-tree `rust/` workspace conventions (`babylon-md`,
`babylon-tui` as style references — not dependencies).

## Global Constraints

- Branch from `dev` (`feature/|fix/|docs/|refactor/|test/`); never commit to
  `main`/`dev` directly; conventional commits; every commit ends with
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- **Single-flight cargo** (solo box, machine-safety rule): never run two `cargo
  build`/`test`/`clippy` invocations concurrently — one `rust/target` dir, one file
  lock; parallel legs only contend. Sequential run-blocks only, mirroring
  `mise run rust:check`'s existing shape.
- **Crate DAG = the layering law.** `babylon-kernel` imports nothing from this
  program's other new crates. `babylon-bsl` may depend on `babylon-kernel` only.
  `babylon-graph`'s trait crate may depend on `babylon-kernel` only (not `babylon-bsl` —
  the typed structural verbs in BSL reference the graph trait, not the reverse).
  Enforce with `cargo tree -p babylon-bsl | rg babylon-graph` returning nothing until
  the dependency is deliberately added, and vice versa.
- **No `unsafe` in `babylon-kernel` or `babylon-bsl`.** Add `#![forbid(unsafe_code)]`
  at each crate's `lib.rs` top — this is a compiler-enforced constraint, not a
  convention; a crate that needs `unsafe` for a real reason escalates to the Director
  (Constitution escalation ladder), it does not quietly drop the forbid.
  `babylon-graph`'s trait shell gets the same forbid; its future concrete
  implementation (Phase 2 storage) may need to revisit this for the rustworkx-core
  FFI boundary — noted, not decided, in that task.
- **Clippy pedantic + fmt is the lint bar for these two (three, once the graph shell
  lands) crates** — stricter than the existing workspace bar (`-D warnings` only, no
  pedantic, on `babylon-tui`/`babylon-md`). Do **not** retrofit pedantic onto the
  existing client crates — that is out of scope and a needless diff. Wire it as
  additional `mise run rust:check` lines scoped with `-p`, exactly mirroring the
  existing per-crate lines for `babylon-tui`/`babylon-md`.
- **MSRV/edition match the existing workspace exactly**: `edition = "2021"`,
  `rust-version.workspace = true` (→ `1.85`), toolchain channel `1.91.1` (unchanged,
  `rust/rust-toolchain.toml`). Do not bump any of these as a side effect of adding
  crates.
- **TDD, rust-flavored:** every task below writes the failing `#[test]` first, shows
  the expected compile/assertion failure, then implements to green. `cargo test -p
  <crate> --locked` is the fast inner loop; `cargo clippy -p <crate> --all-targets
  --locked -- -D warnings -D clippy::pedantic` gates every commit touching that crate.
- Never commit a worktree-modified `uv.lock`, `Cargo.lock` from an unrelated crate, or
  anything under `rust/target/`. Stage files explicitly; verify HEAD moved after every
  commit (`git log --oneline -1`).
- Do not use `git -C`; `cd` to the repo root. Use `rg`, never `grep`.
- Every public item gets a doc comment (`///` or `//!`); `RUSTDOCFLAGS='-D warnings'
  cargo doc -p <crate> --no-deps --locked` must be clean, mirroring the existing
  workspace-wide doc gate.
- This is a **docs-and-code planning artifact** — the plan itself is written in a
  docs-only worktree; a later execution pass (with cargo available) runs these tasks.
  Where this plan makes a genuine engineering call the spec left open (crate choices,
  exact algorithms), the reasoning is written inline as a doc comment in the sketched
  code — these are NOT amendment-gated decisions (no primitive invented, no
  prohibition relaxed, no ideological line touched), but flag anything a later
  executor should double-check against Phase 0's III.12(a) reference before treating
  as final. Two Director-gated items are marked explicitly as such (Tasks 8 and 16)
  and must not be self-merged.

---

### Task 1: Rust workspace scaffolding — `babylon-kernel` + `babylon-bsl` crate shells

**Files:**
- Modify: `rust/Cargo.toml` (add both crates to `[workspace] members`)
- Create: `rust/crates/babylon-kernel/Cargo.toml`, `rust/crates/babylon-kernel/src/lib.rs`
- Create: `rust/crates/babylon-bsl/Cargo.toml`, `rust/crates/babylon-bsl/src/lib.rs`
- Modify: `.mise.toml` (`rust:check` gains per-crate pedantic-clippy + test lines)

**Interfaces:**
- Produces: two empty-but-compiling, empty-but-lint-clean crates that every later task
  in this plan adds to.

- [ ] **Step 1: Branch**

```bash
git checkout dev && git pull && git checkout -b feature/p27-kernel-bsl-scaffold
```

- [ ] **Step 2: Write the crate shells**

```toml
# rust/crates/babylon-kernel/Cargo.toml
[package]
name = "babylon-kernel"
description = "Program 27 kernel: scalars, Currency, ContentDigest, sim clock, event bus, RNG service"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[dependencies]
sha2 = "0.10"

[dev-dependencies]
pretty_assertions = "1"
```

```rust
// rust/crates/babylon-kernel/src/lib.rs
//! The Program 27 kernel: scalars, `Currency`, `ContentDigest`, the sim clock,
//! the event-bus port, and the RNG service (spec §6, §9). No `unsafe`; every
//! public item is doc-commented (`RUSTDOCFLAGS='-D warnings' cargo doc` gate).
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
```

```toml
# rust/crates/babylon-bsl/Cargo.toml
[package]
name = "babylon-bsl"
description = "Babylon Scripting Language: reader, typechecker, load-time bound checker, fuel evaluator"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[dependencies]
babylon-kernel = { path = "../babylon-kernel" }

[dev-dependencies]
pretty_assertions = "1"
```

```rust
// rust/crates/babylon-bsl/src/lib.rs
//! The Babylon Scripting Language (BSL): the reader, typechecker, load-time
//! bound checker, and fuel evaluator (spec §5). No `unsafe`.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
```

- [ ] **Step 3: Wire the workspace + mise**

```toml
# rust/Cargo.toml — add to members
[workspace]
members = [
    "crates/babylon-kernel",
    "crates/babylon-bsl",
    "crates/babylon-md",
    "crates/babylon-tui",
    "crates/babylon-tui-python",
]
```

Append to the `rust:check` task body in `.mise.toml` (after the existing
`babylon-md` line, before the closing `"""`):

```bash
cargo clippy -p babylon-kernel --all-targets --locked -- -D warnings -D clippy::pedantic
cargo test -p babylon-kernel --locked
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cargo test -p babylon-bsl --locked
```

- [ ] **Step 4: Verify it builds clean and the layering law holds**

```bash
cd rust
cargo build --workspace --locked
cargo clippy -p babylon-kernel --all-targets --locked -- -D warnings -D clippy::pedantic
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cargo tree -p babylon-bsl | rg babylon-kernel   # expect: present
cargo tree -p babylon-kernel | rg babylon-bsl   # expect: nothing (kernel imports nothing above itself)
cd ..
```

- [ ] **Step 5: Commit + PR**

```bash
git add rust/Cargo.toml rust/crates/babylon-kernel rust/crates/babylon-bsl .mise.toml
git commit -m "$(cat <<'EOF'
feat(rust): scaffold babylon-kernel + babylon-bsl crate shells (P27 Phase 1)

Empty, lint-clean crates wired into the existing rust/ workspace and
mise rust:check; no unsafe, clippy pedantic as the lint bar. Everything
these crates need lands task-by-task in the rest of the Phase-1 plan.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
EOF
)"
git log --oneline -1
gh pr create --base dev --title "feat(rust): scaffold babylon-kernel + babylon-bsl (P27 Phase 1)" --body "Spec: docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md §6, §10 Phase 1.

🤖 Generated with [Claude Code](https://claude.com/claude-code)"
```
Self-merge on green per the standing autonomy grant.

---

### Task 2: The BSL language specification (the language-agnostic reference)

**Files:**
- Create: `docs/reference/bsl-language-spec.rst`

**Interfaces:**
- Produces: the BNF grammar, typing judgments, and totality argument for BSL, as the
  one language-agnostic document every later task (reader, typechecker, fuel checker,
  conformance corpus, and any future non-Rust BSL host) implements against — the
  concrete referent that makes the summary's "THE_FORMALISM.md Part III is a language
  spec with no language" claim false.

- [ ] **Step 1: Read the source material end-to-end**

```bash
git checkout -b docs/p27-bsl-language-spec dev
sed -n '253,378p' ai/THE_FORMALISM.md    # Part III: the grammar, typing, totality
rg -n "^### 5\." -A400 docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md | sed -n '1,200p'   # spec §5
wc -l docs/reference/determinism-contract.rst
```

- [ ] **Step 2: Write the spec.** Structure (mirroring III.3–III.4's BNF/typing-judgment
  shape, but for BSL's own forms, not `Mot`):

  1. **Shape & values** — s-expressions, homoiconic; the value universe (kernel
     scalars `Probability`/`Intensity`/`Coefficient`/`Currency`, closed enums
     `DoctrineTag`/`PracticeVariable`/`NodeType`/…, booleans, typed node/edge-set
     references) — cite spec §5 "Types" verbatim as the normative source, do not
     paraphrase away the per-field intensive/extensive kind rule.
  2. **Grammar as BNF** — one production per form: conditions, effects (arithmetic +
     the typed structural verbs: add/remove node/edge, update-node, and — since
     Amendment D ruled native hyperedge — add/remove hyperedge), formula
     composition over registered intrinsics, guards, folds, the `:material-basis`
     field, binding declarations (`:optional <name> :default <literal>`), mod ordering
     anchors (`before`/`after` a named system). Every form that is **not**
     expressible (I/O, defining new intrinsics, graph mutation outside the typed verb
     set, anything unbounded) gets an explicit negative production note, the same
     rhetorical move THE_FORMALISM §III.3 makes ("what the grammar cannot say is the
     point").
  3. **Typing judgment** `Γ ⊢ e : τ` with load-bearing rules for: a fold over a
     declared-cardinality-bounded node/edge set, an intensive-kinded field read
     (rejects unweighted aggregation — the per-field kind propagates through folds),
     a declared binding (unbound = load-time error unless `:optional`), an intrinsic
     call (arity/type checked against the registered intrinsic table — the table
     itself is empty in Phase 1, populated in Phase 2; the typing rule doesn't care).
  4. **Totality argument** — structural induction mirroring III.4's T-2 sketch:
     folds traverse finite carriers (declared cardinality ceilings, not the runtime
     graph — spec §5 "Totality"), guards are decidable, the fuel meter is the dynamic
     backstop for the static bound's own soundness gap (a rule whose worst-case bound
     is correct but whose real graph somehow exceeds a declared ceiling is itself a
     III.11 load failure, not a totality violation — restate this precisely, it is
     the spec's own resolution of adversarial finding M1/M2).
  5. **The four grammar-superset corrections (M8)** — state each of the four
     silent-degradation behaviors BSL deliberately does NOT inherit (unknown graph
     metric, unknown aggregation, unknown comparison operator, empty precondition
     set) as normative "MUST reject at <load-time|eval-time>" rules, each citing the
     Python site it replaces (`event_evaluator.py:313/405/439`, `:103`) so the
     conformance corpus task (Task 17) has an unambiguous target.
  6. **Cross-references** — a short table pointing to `determinism-contract.rst`
     for the byte-level pieces this document deliberately does NOT re-specify:
     fuel cost model constants, tick hash, `ContentDigest`, canonical AST
     serialization, RNG seeding (Phase 0 Task 12 owns those; this document owns the
     grammar and type system only — one document, one job, Diataxis discipline).

- [ ] **Step 3: Docs gate**

```bash
mise run check   # sphinx -W + rstcheck/doc8 pre-commit hooks
```

- [ ] **Step 4: Commit + PR** (slug `bsl-language-spec`; same shape as Phase 0 Task 12).

---

### Task 3: Kernel scalars — the quantized grid + `Currency` i128 (§6.1, THE_FORMALISM II.1)

Current Python (`src/babylon/kernel/math.py`, `src/babylon/models/types.py`) puts
`Probability`/`Intensity`/`Coefficient`/`Ideology`/`Balance`/`Ratio` on a **10⁻⁶ grid**
(ROUND_HALF_UP quantization at the type boundary, IEEE-754 float arithmetic between
boundaries) but keeps `Currency` a plain non-negative float. Spec §6.1 promotes
`Currency` specifically to **i128 fixed-point micro-units** for overflow headroom —
a different sort, not a bigger version of the same float. This task ports both: the
bounded-float sorts as grid-snapped `f64` newtypes (byte-identical to Python's
quantize function — a real cross-language conformance vector exists here, unlike
`Currency`, which is new), and `Currency` as the i128 type with the pinned operator
semantics.

**Files:**
- Create: `rust/crates/babylon-kernel/src/grid.rs` (the quantization retraction)
- Create: `rust/crates/babylon-kernel/src/scalars.rs` (`Probability`, `Intensity`,
  `Coefficient`, `Ideology`, `Balance`, `Ratio`)
- Create: `rust/crates/babylon-kernel/src/currency.rs` (`Currency`, operators)
- Modify: `rust/crates/babylon-kernel/Cargo.toml` (add a wide-integer dependency for
  the i256 intermediate — see Step 3)
- Modify: `rust/crates/babylon-kernel/src/lib.rs` (re-export)

**Interfaces:**
- Produces: `pub struct Currency(i128)` with `checked_add`/`checked_sub` (`Currency ±
  Currency`), `mul_coefficient` (`Currency × Coefficient → Currency`, half-even),
  `div_currency` (`Currency ÷ Currency → Coefficient`, i256 intermediate, half-even),
  `div_integer` (`Currency ÷ i128 → Currency`, half-even) — all four spec-pinned
  operators, all panicking loudly on overflow (III.11), never wrapping/saturating.

- [ ] **Step 1: Branch; port the grid quantization function with a failing test first**

```bash
git checkout dev && git pull && git checkout -b feature/p27-kernel-scalars
```

```rust
// rust/crates/babylon-kernel/src/grid.rs
//! The Program 27 quantization retraction (THE_FORMALISM II.1, `L-GRID`):
//! ports `babylon.kernel.math.quantize` byte-for-byte. ROUND_HALF_UP, ties
//! away from zero, on the 10⁻⁶ grid.

pub const GRID_PRECISION: u32 = 6;
const GRID: f64 = 1_000_000.0; // 10^GRID_PRECISION

/// Snap `value` onto the 10⁻⁶ grid, ROUND_HALF_UP (ties away from zero) —
/// the exact algorithm in `src/babylon/kernel/math.py::quantize`.
#[must_use]
pub fn quantize(value: f64) -> f64 {
    if value >= 0.0 {
        (value * GRID + 0.5).floor() / GRID
    } else {
        -((-value * GRID + 0.5).floor()) / GRID
    }
}

#[cfg(test)]
mod tests {
    use super::quantize;

    /// Cross-language conformance vector: values computed by running
    /// `babylon.kernel.math.quantize` in Python (Phase-0-verified before
    /// this task closes — see Step 2's note) for the same inputs.
    #[test]
    fn matches_the_python_quantize_conformance_vector() {
        let cases: &[(f64, f64)] = &[
            (0.123_456_789, 0.123_457),
            (-0.123_456_789, -0.123_457),
            (0.0, 0.0),
            (1.0000005, 1.000_001), // half-away-from-zero tie
            (-1.0000005, -1.000_001),
        ];
        for &(input, expected) in cases {
            assert!(
                (quantize(input) - expected).abs() < 1e-12,
                "quantize({input}) = {}, expected {expected}",
                quantize(input)
            );
        }
    }

    #[test]
    fn is_idempotent() {
        // L-GRID: q ∘ q = q
        let v = 0.987_654_321;
        assert!((quantize(quantize(v)) - quantize(v)).abs() < 1e-15);
    }
}
```

- [ ] **Step 2: Verify the conformance vector against real Python, then run to green**

```bash
uv run python -c "
from babylon.kernel.math import quantize
for v in [0.123456789, -0.123456789, 0.0, 1.0000005, -1.0000005]:
    print(v, quantize(v))
"
```
Cross-check the printed values against the `cases` table above **before** trusting the
test — if they differ, the test's `expected` column is wrong, not the Rust
implementation (this is a transcription, F3 discipline: verify against the source of
truth, don't assume). Then:

```bash
cd rust && cargo test -p babylon-kernel grid --locked && cd ..
```

- [ ] **Step 3: Add the bounded-scalar newtypes with failing construction tests**

```rust
// rust/crates/babylon-kernel/src/scalars.rs
//! Grid-quantized bounded scalar sorts (THE_FORMALISM II.1): `Probability`,
//! `Intensity`, `Coefficient` on `𝔾 ∩ [0,1]`; `Ideology`, `Balance` on
//! `𝔾 ∩ [-1,1]`; `Ratio` on `𝔾 ∩ (0,∞)`. Construction quantizes and
//! validates (the Gatekeeper pattern, ported from Pydantic's
//! `AfterValidator`) — an out-of-range value is a loud `Err`, never
//! silently clamped.
use crate::grid::quantize;

/// A scalar out of its sort's declared bound (III.11 load-time rejection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutOfBoundsError {
    pub value: f64,
    pub lower: f64,
    pub upper: f64,
}

macro_rules! bounded_scalar {
    ($name:ident, $lower:expr, $upper:expr) => {
        #[doc = concat!("Grid-quantized scalar on 𝔾 ∩ [", stringify!($lower), ", ", stringify!($upper), "].")]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name(f64);

        impl $name {
            /// Quantize and validate `value`. Errs loud on out-of-bounds
            /// (III.11) — never clamps.
            pub fn new(value: f64) -> Result<Self, OutOfBoundsError> {
                let q = quantize(value);
                if q < $lower || q > $upper {
                    return Err(OutOfBoundsError { value: q, lower: $lower, upper: $upper });
                }
                Ok(Self(q))
            }

            #[must_use]
            pub fn get(self) -> f64 {
                self.0
            }
        }
    };
}

bounded_scalar!(Probability, 0.0, 1.0);
bounded_scalar!(Intensity, 0.0, 1.0);
bounded_scalar!(Coefficient, 0.0, 1.0);
bounded_scalar!(Ideology, -1.0, 1.0);
bounded_scalar!(Balance, -1.0, 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probability_rejects_out_of_range() {
        assert!(Probability::new(1.5).is_err());
        assert!(Probability::new(-0.001).is_err());
    }

    #[test]
    fn probability_quantizes_on_construction() {
        let p = Probability::new(0.123_456_789).unwrap();
        assert!((p.get() - 0.123_457).abs() < 1e-12);
    }
}
```
`Ratio` is `(0, ∞)` — open lower bound, unbounded above; write it as a hand-rolled
impl (not the macro, whose `$upper` bound assumes a finite closed interval) with an
`f64::INFINITY` upper check replaced by "finite and > 0". Run red-then-green the same
way as Step 2.

- [ ] **Step 4: `Currency` — i128 micro-units + the four pinned operators**

First decide the wide-integer dependency for the `÷ Currency → Coefficient` i256
intermediate (spec §6.1: "computed at i256 intermediate width then rounded
half-even"). Use a pure-Rust, `no_std`-capable, no-`unsafe`-internals wide-integer
crate (`bnum` or equivalent — verify current crates.io availability and MSRV fit
against the pinned toolchain in this step, since this worktree has no network
access to confirm it; if `bnum` doesn't fit, `ethnum` is the fallback, or a
hand-rolled 256-bit multiply/divide pair built from two `u128` halves — the load-
bearing property is "no `unsafe`", not the specific crate).

```toml
# rust/crates/babylon-kernel/Cargo.toml — add
[dependencies]
bnum = "0.11"   # verify version at implementation time; pure-Rust wide integers, no unsafe
```

```rust
// rust/crates/babylon-kernel/src/currency.rs
//! `Currency`: i128 fixed-point micro-units (spec §6.1). Overflow is a loud
//! III.11 failure — `checked_*` everywhere, never wrapping or saturating.
//!
//! Sign domain (OPEN — flagged in the Phase-1 plan's open_questions): the
//! Python model constrains `Currency` non-negative
//! (`models/types.py::Currency`, `Field(ge=0.0)`); this port keeps the
//! underlying representation signed (`i128`) because intermediate deltas
//! (e.g. a dispossession transfer) are naturally signed, and does NOT
//! re-impose non-negativity as a type invariant here. If the Director or a
//! later review wants non-negativity enforced at the type level, that is a
//! narrow follow-up (a `NonNegativeCurrency` boundary wrapper), not a
//! redesign of this module.
use crate::scalars::Coefficient;
use bnum::types::I256;

/// A loud, non-recoverable-by-the-algebra overflow (III.11: run-time loud
/// failure — the caller is expected to let this panic propagate, per spec §9
/// "checked-arithmetic overflow panics with context").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyOverflow {
    pub op: &'static str,
}

/// Fixed-point currency: `self.0` is the value in micro-units (1 unit =
/// 1_000_000 micro-units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Currency(i128);

const MICRO: i128 = 1_000_000;

impl Currency {
    #[must_use]
    pub fn from_micro_units(micro: i128) -> Self {
        Self(micro)
    }

    #[must_use]
    pub fn micro_units(self) -> i128 {
        self.0
    }

    /// `Currency ± Currency → Currency` (checked; spec §6.1).
    pub fn checked_add(self, other: Self) -> Result<Self, CurrencyOverflow> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(CurrencyOverflow { op: "Currency + Currency" })
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, CurrencyOverflow> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(CurrencyOverflow { op: "Currency - Currency" })
    }

    /// `Currency × Coefficient → Currency`, rounded half-even to
    /// micro-units. Coefficient lives on the 10⁻⁶ grid (`scalars::Coefficient`
    /// is grid-quantized on construction), so its exact value is a rational
    /// number `numerator / 1_000_000` with an integer numerator — this
    /// implementation multiplies the two integer representations directly
    /// (never casts the i128 side to `f64`, which would lose precision above
    /// 2^53 ≈ 9e15, well inside the nationwide-scale headroom §6.1 pins for)
    /// and divides back down by `1_000_000`, half-even, in one step.
    pub fn mul_coefficient(self, coeff: Coefficient) -> Result<Self, CurrencyOverflow> {
        let numerator = (coeff.get() * f64::from(MICRO as u32)).round() as i128; // grid-safe: coeff in [0,1]
        let product = self
            .0
            .checked_mul(numerator)
            .ok_or(CurrencyOverflow { op: "Currency * Coefficient (pre-round)" })?;
        Ok(Self(round_half_even_div(product, MICRO)))
    }

    /// `Currency ÷ Currency → Coefficient`, i256 intermediate, half-even.
    #[must_use]
    pub fn div_currency(self, other: Self) -> Coefficient {
        let numerator = I256::from(self.0) * I256::from(MICRO);
        let ratio = numerator / I256::from(other.0);
        // convert back losslessly for values in Coefficient's [0,1] domain;
        // out-of-domain results are a Coefficient::new bounds error (III.11).
        let as_i128: i128 = ratio.try_into().expect("i256 intermediate must fit i128 for a [0,1] ratio");
        Coefficient::new(as_i128 as f64 / f64::from(MICRO as u32)).expect("out-of-[0,1] ratio: III.11 caller bug")
    }

    /// `Currency ÷ integer → Currency`, half-even.
    #[must_use]
    pub fn div_integer(self, divisor: i128) -> Self {
        Self(round_half_even_div(self.0, divisor))
    }
}

/// Half-even (banker's) rounding integer division — the `round_half_even`
/// kernel intrinsic (spec §6.2), pinned here for `Currency`'s own operators;
/// re-exported at crate root for BSL's numeric-annex use once Phase 2 wires
/// it as a callable intrinsic.
#[must_use]
pub fn round_half_even_div(numerator: i128, denominator: i128) -> i128 {
    let q = numerator / denominator;
    let r = numerator % denominator;
    let twice_r = r.checked_mul(2).expect("round_half_even_div: 2*remainder overflow");
    match twice_r.abs().cmp(&denominator.abs()) {
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Greater => q + numerator.signum() * denominator.signum(),
        std::cmp::Ordering::Equal => {
            if q % 2 == 0 { q } else { q + numerator.signum() * denominator.signum() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_overflow_is_loud_not_wrapping() {
        let max = Currency::from_micro_units(i128::MAX);
        let one = Currency::from_micro_units(1);
        assert_eq!(max.checked_add(one), Err(CurrencyOverflow { op: "Currency + Currency" }));
    }

    #[test]
    fn round_half_even_div_ties_to_even() {
        assert_eq!(round_half_even_div(5, 2), 2); // 2.5 -> 2 (even)
        assert_eq!(round_half_even_div(7, 2), 4); // 3.5 -> 4 (even)
        assert_eq!(round_half_even_div(-5, 2), -2);
    }

    #[test]
    fn div_integer_matches_round_half_even() {
        let c = Currency::from_micro_units(5);
        assert_eq!(c.div_integer(2), Currency::from_micro_units(2));
    }
}
```
Run red-then-green: write these tests first with a `todo!()` body for
`mul_coefficient`/`div_currency`/`round_half_even_div`, confirm `cargo test -p
babylon-kernel currency --locked` panics/fails, then paste the real bodies in and
re-run to green.

- [ ] **Step 5: Cross-check against `docs/reference/determinism-contract.rst`'s
  Currency chapter** (Phase 0 Task 12). If that chapter does not yet pin the exact
  "represent `Coefficient` as an integer numerator over `1_000_000` for the multiply"
  algorithm this step uses, this is a genuine gap in the reference the earlier phase
  didn't anticipate — draft a short addendum there (same review bar as the rest of
  III.12(a): normative table + one worked example) rather than silently diverging
  code from spec. Do not skip this cross-check.

- [ ] **Step 6: Full crate gate + commit + PR**

```bash
cd rust
cargo test -p babylon-kernel --locked
cargo clippy -p babylon-kernel --all-targets --locked -- -D warnings -D clippy::pedantic
cd ..
git add rust/crates/babylon-kernel docs/reference/determinism-contract.rst
git commit -m "$(cat <<'EOF'
feat(kernel): grid-quantized scalars + Currency i128 checked arithmetic (P27 §6.1)

Ports babylon.kernel.math.quantize byte-for-byte (cross-language
conformance vector) for the bounded float sorts, and implements
Currency as i128 micro-units with the four spec-pinned operators
(checked add/sub, half-even mul-by-Coefficient, i256-intermediate
div-by-Currency, half-even div-by-integer). No unsafe, no wrapping.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
EOF
)"
git log --oneline -1
gh pr create --base dev --title "feat(kernel): scalars + Currency i128 (P27 §6.1)" --body "..."
```

---

### Task 4: Sim clock + deterministic per-tick correlation id (§6.5)

Replaces the Python `uuid4()`-per-tick correlation id, which spec §6.5 calls "log-only
today; the replacement is strictly better" — a deterministic counter.

**Files:**
- Create: `rust/crates/babylon-kernel/src/clock.rs`
- Modify: `rust/crates/babylon-kernel/src/lib.rs`

**Interfaces:**
- Produces: `pub struct SimClock { session_id: SessionId, tick: u64 }`,
  `SimClock::advance(&mut self) -> u64`, `SimClock::correlation_id(&self) -> String`
  (a deterministic `format!("{session_id}-{tick:010}")`, replacing the log-only
  `uuid4()`).

- [ ] **Step 1: Branch; write the failing tests**

```bash
git checkout dev && git pull && git checkout -b feature/p27-sim-clock
```

```rust
// rust/crates/babylon-kernel/src/clock.rs
//! Deterministic sim clock + per-tick correlation id (spec §6.5 — replaces
//! Python's `uuid4()` per-tick id, which was log-only and non-deterministic
//! by construction; this replacement is strictly better for the same job).

/// Opaque session identifier — a validated non-empty string, not a raw
/// `String`, so an empty session id is a construction-time error (III.11).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptySessionId;

impl SessionId {
    pub fn new(id: impl Into<String>) -> Result<Self, EmptySessionId> {
        let id = id.into();
        if id.is_empty() {
            return Err(EmptySessionId);
        }
        Ok(Self(id))
    }
}

/// The tick clock: monotonic, no wall-clock reads, no randomness.
#[derive(Debug, Clone)]
pub struct SimClock {
    session_id: SessionId,
    tick: u64,
}

impl SimClock {
    #[must_use]
    pub fn new(session_id: SessionId) -> Self {
        Self { session_id, tick: 0 }
    }

    /// Advance one tick; returns the new tick number.
    pub fn advance(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Deterministic per-tick correlation id — a pure function of
    /// `(session_id, tick)`, never a UUID.
    #[must_use]
    pub fn correlation_id(&self) -> String {
        format!("{}-{:010}", self.session_id.0, self.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_id_is_a_pure_function_of_session_and_tick() {
        let clock_a = SimClock { session_id: SessionId::new("abc").unwrap(), tick: 3 };
        let clock_b = SimClock { session_id: SessionId::new("abc").unwrap(), tick: 3 };
        assert_eq!(clock_a.correlation_id(), clock_b.correlation_id());
    }

    #[test]
    fn advance_is_monotonic_and_never_resets() {
        let mut clock = SimClock::new(SessionId::new("s").unwrap());
        assert_eq!(clock.advance(), 1);
        assert_eq!(clock.advance(), 2);
        assert_eq!(clock.tick(), 2);
    }

    #[test]
    fn empty_session_id_is_a_loud_construction_error() {
        assert_eq!(SessionId::new(""), Err(EmptySessionId));
    }
}
```

- [ ] **Step 2: Run red (fields are private in the test's struct literal — adjust
  visibility or add a `#[cfg(test)] pub(crate)` constructor if the literal doesn't
  compile) then green**

```bash
cd rust && cargo test -p babylon-kernel clock --locked && cd ..
```

- [ ] **Step 3: Gate + commit + PR** (slug `sim-clock`).

---

### Task 5: RNG service — pinned algorithm + seeding derivation (§9, R8)

R8 rules RNG **streams change at cutover** — no bit-exact CPython-MT19937
requirement. This task picks and pins a concrete algorithm (an engineering call, not
an amendment — R8 already authorizes stream divergence) and mirrors today's
`resolve_rng` seeding *structure* (`sha256(session_id ‖ tick ‖ salt)`, salt
`0xBA1AC1A`) without reproducing MT19937's bit stream.

**Files:**
- Create: `rust/crates/babylon-kernel/src/rng.rs`
- Modify: `rust/crates/babylon-kernel/Cargo.toml` (RNG crate dependency)
- Modify: `docs/reference/determinism-contract.rst` (RNG chapter, if Phase 0 Task 12
  left the algorithm choice as "TBD Phase 1" — confirm and fill in)

**Interfaces:**
- Produces: `pub fn seed_for(session_id: &SessionId, tick: u64) -> [u8; 32]` (the
  seeding derivation) and `pub struct KernelRng(ChaCha8Rng)` wrapping the pinned
  generator with `next_u64`/`next_f64`-shaped accessors sufficient for the intrinsics
  Phase 2 will call.

- [ ] **Step 1: Branch; pin the algorithm with the reasoning inline**

```bash
git checkout dev && git pull && git checkout -b feature/p27-rng-service
```

```toml
# rust/crates/babylon-kernel/Cargo.toml — add
[dependencies]
rand_chacha = "0.3"   # verify version at implementation time
rand_core = "0.6"     # verify version at implementation time
```

```rust
// rust/crates/babylon-kernel/src/rng.rs
//! The kernel RNG service (spec §9, R8): one pinned algorithm, seeded per
//! `(session_id, tick, salt)`.
//!
//! **Algorithm choice (Phase-1 engineering call, not amendment-gated — R8
//! already authorizes the stream divergence from Python's MT19937):**
//! `ChaCha8Rng` (`rand_chacha`). Rationale: (1) it takes an exact 32-byte
//! seed, which is exactly a SHA-256 digest's width — the seeding derivation
//! below needs no truncation/expansion step, unlike a generator wanting a
//! `u64` or `[u8; 16]` seed; (2) it is a pure-Rust, no-`unsafe`,
//! platform-independent stream cipher construction with strong
//! statistical properties and no OS-entropy dependency (fully
//! deterministic from its seed, required for III.7); (3) 8 rounds is the
//! documented "fast, still no known practical distinguisher" configuration
//! — this is not a cryptographic-security use case, so `ChaCha8` is
//! preferred over `ChaCha20` purely for speed with no correctness cost.
//! This choice is pinned in `docs/reference/determinism-contract.rst`'s RNG
//! chapter (Phase 0 Task 12) — if that chapter still says "TBD Phase 1"
//! when this task starts, fill in this paragraph there verbatim as the
//! ratified text.
use crate::clock::SessionId;
use rand_chacha::ChaCha8Rng;
use rand_core::SeedableRng;
use sha2::{Digest, Sha256};

/// Mirrors `kernel/system_base.py::_SYSTEM_RNG_SEED_SALT` structurally (same
/// salt constant, same mixing shape: `session_id ‖ tick ‖ salt`) — NOT the
/// same stream, per R8.
pub const SEED_SALT: u64 = 0x0BA1_AC1A;

/// Derive the 32-byte `ChaCha8Rng` seed for `(session_id, tick)`.
#[must_use]
pub fn seed_for(session_id: &SessionId, tick: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    hasher.update(tick.to_le_bytes());
    hasher.update(SEED_SALT.to_le_bytes());
    hasher.finalize().into()
}

/// The kernel's one pinned RNG. Constructed only via `for_tick` — there is
/// no `KernelRng::from_entropy()`; every stream is a pure function of
/// `(session_id, tick)` (III.7).
pub struct KernelRng(ChaCha8Rng);

impl KernelRng {
    #[must_use]
    pub fn for_tick(session_id: &SessionId, tick: u64) -> Self {
        Self(ChaCha8Rng::from_seed(seed_for(session_id, tick)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::SessionId;
    use rand_core::RngCore;

    #[test]
    fn same_session_and_tick_reproduce_the_same_stream() {
        let sid = SessionId::new("s1").unwrap();
        let mut a = KernelRng::for_tick(&sid, 7);
        let mut b = KernelRng::for_tick(&sid, 7);
        assert_eq!(a.0.next_u64(), b.0.next_u64());
    }

    #[test]
    fn different_ticks_diverge() {
        let sid = SessionId::new("s1").unwrap();
        let mut a = KernelRng::for_tick(&sid, 7);
        let mut b = KernelRng::for_tick(&sid, 8);
        assert_ne!(a.0.next_u64(), b.0.next_u64());
    }
}
```
`SessionId::as_bytes` needs adding to Task 4's `SessionId` (a one-line
`pub fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }`); do that as part of this
task's Step 1 with its own tiny red/green cycle, not silently.

- [ ] **Step 2: Run to green; write the within-implementation replay conformance
  vector** — a pinned `(session_id, tick) -> first 4 u64s` table in
  `docs/reference/determinism-contract.rst`'s RNG chapter, generated once and then
  treated as a byte-pin (any future `cargo test` divergence is a determinism
  regression, not a "the RNG got better" excuse).

```bash
cd rust && cargo test -p babylon-kernel rng --locked && cd ..
```

- [ ] **Step 3: Gate + commit + PR** (slug `rng-service`). PR body states plainly:
  "streams differ from Python by design (R8); this is the pinned Rust-side
  replacement, not a port."

---

### Task 6: Event-bus port (§6.5) — ordering guarantees, generic over topic

Full byte-for-byte behavioral parity with `kernel/event_bus.py` (288 lines):
registration-order dispatch, append-before-emit, stable-sorted interceptor chain.
Phase 1 ports the **generic ordering machinery** only; the 100-value `EventType`
domain enum is a `babylon-domain`/`babylon-engine` concern (Phase 2/3) — this crate's
bus is generic over any `Ord`-free, insertion-order-preserving topic type so it
doesn't need to wait on that enum's port.

**Files:**
- Read: `src/babylon/kernel/event_bus.py` (288 lines, the port target)
- Create: `rust/crates/babylon-kernel/src/event_bus.rs`
- Modify: `rust/crates/babylon-kernel/src/lib.rs`

**Interfaces:**
- Produces: `pub struct EventBus<T>` generic over topic type `T: Clone`, with
  `subscribe(&mut self, topic: T, handler: Box<dyn FnMut(&Event<T>)>)` (registration
  order preserved), `emit(&mut self, event: Event<T>)` (append-before-dispatch: the
  event is pushed to the bus's own log before any handler runs, mirroring the Python
  ordering guarantee), and an ordered interceptor chain (`add_interceptor` /
  stable-sort by declared priority, ties broken by registration order).

- [ ] **Step 1: Read the Python source end-to-end first (F1 discipline)**

```bash
git checkout dev && git pull && git checkout -b feature/p27-event-bus
wc -l src/babylon/kernel/event_bus.py
sed -n '1,288p' src/babylon/kernel/event_bus.py
```
Extract the exact three guarantees this port must preserve before writing a single
Rust line: (1) handlers for one topic fire in subscription order; (2) the event is
appended to the bus's internal log **before** any handler executes (so a handler that
inspects the log sees itself already recorded); (3) interceptors run in a stable sort
by priority, registration order as the tiebreak.

- [ ] **Step 2: Write the failing ordering tests**

```rust
// rust/crates/babylon-kernel/src/event_bus.rs
//! Deterministic event bus (ports `kernel/event_bus.py`'s ordering
//! guarantees — spec §6.5 — generic over the topic/payload type; the
//! 100-value `EventType` domain enum lands with `babylon-domain`/
//! `babylon-engine` in Phase 2/3, not here).
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Event<T> {
    pub topic: T,
    pub tick: u64,
}

pub struct EventBus<T: Clone> {
    log: Vec<Event<T>>,
    handlers: Vec<(T, Rc<RefCell<dyn FnMut(&Event<T>)>>)>,
}

impl<T: Clone + PartialEq> EventBus<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { log: Vec::new(), handlers: Vec::new() }
    }

    /// Registration-order dispatch: later `subscribe` calls for the same
    /// topic fire strictly after earlier ones.
    pub fn subscribe(&mut self, topic: T, handler: Rc<RefCell<dyn FnMut(&Event<T>)>>) {
        self.handlers.push((topic, handler));
    }

    /// Append-before-dispatch: `event` is in `self.log` before any handler
    /// for it runs.
    pub fn emit(&mut self, event: Event<T>) {
        self.log.push(event.clone());
        for (topic, handler) in &self.handlers {
            if *topic == event.topic {
                handler.borrow_mut()(&event);
            }
        }
    }

    #[must_use]
    pub fn log(&self) -> &[Event<T>] {
        &self.log
    }
}

impl<T: Clone + PartialEq> Default for EventBus<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handlers_fire_in_registration_order() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let order = Rc::new(RefCell::new(Vec::<u32>::new()));
        let o1 = Rc::clone(&order);
        let o2 = Rc::clone(&order);
        bus.subscribe("t", Rc::new(RefCell::new(move |_: &Event<&str>| o1.borrow_mut().push(1))));
        bus.subscribe("t", Rc::new(RefCell::new(move |_: &Event<&str>| o2.borrow_mut().push(2))));
        bus.emit(Event { topic: "t", tick: 0 });
        assert_eq!(*order.borrow(), vec![1, 2]);
    }

    #[test]
    fn event_is_appended_before_handlers_run() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let seen_log_len_at_dispatch = Rc::new(RefCell::new(0usize));
        let seen = Rc::clone(&seen_log_len_at_dispatch);
        // NOTE: this closure cannot read `bus.log()` (borrow checker — bus is
        // already mutably borrowed by emit); the real assertion is written
        // against a handle captured before emit, or by checking bus.log()
        // length immediately after emit() returns equals 1 unconditionally.
        // Replace this test with a design that actually observes ordering,
        // e.g. a handler that asserts on a passed-in `&EventBus` snapshot if
        // the port needs that guarantee testable in Rust's aliasing rules —
        // resolve during implementation, the GUARANTEE is what's pinned,
        // not this sketch's exact mechanism.
        let _ = seen;
        bus.emit(Event { topic: "t", tick: 0 });
        assert_eq!(bus.log().len(), 1);
    }
}
```
The second test as sketched is a placeholder acknowledging a real Rust
aliasing wrinkle (a handler cannot easily observe "the bus's own log" while the bus
is mutably borrowed for dispatch, unlike Python's freely-aliased `self`) — the
implementer must resolve this properly (e.g. pass the already-appended log length
into the handler callback signature, or restructure to snapshot-then-dispatch) before
this task can close; do not ship the placeholder assertion as the final test.

- [ ] **Step 3: Add the interceptor chain (stable sort by priority, registration
  tiebreak) with its own failing test**, then implement.

- [ ] **Step 4: Run to green; gate + commit + PR** (slug `event-bus-port`).

---

### Task 7: `ContentDigest` — the `defines_hash` half (kernel side)

`ContentDigest { defines_hash, rules_hash }` per spec §7. `rules_hash` needs the
canonical BSL AST serialization, which doesn't exist until Task 12 (babylon-bsl's
reader + serializer); this task builds the `defines_hash` half and the
`ContentDigest` container shape, with `rules_hash` left as a typed placeholder Task 12
fills in.

**Files:**
- Create: `rust/crates/babylon-kernel/src/content_digest.rs`
- Modify: `rust/crates/babylon-kernel/src/lib.rs`

**Interfaces:**
- Produces: `pub struct ContentDigest { pub defines_hash: [u8; 32], pub rules_hash:
  Option<[u8; 32]> }` (an `Option` until Task 12 makes it mandatory — a deliberate,
  documented interim shape, not a permanent honest-null), and
  `defines_hash_of(canonical_json: &str) -> [u8; 32]` matching the Python
  `canonical_defines_hash` byte layout exactly (Phase 0 Task 1's fix — sorted-keys,
  pinned separators, full 64-hex/32-byte SHA-256, no stringly fallback).

- [ ] **Step 1: Branch; write the cross-language conformance test**

```bash
git checkout dev && git pull && git checkout -b feature/p27-content-digest
```

```rust
// rust/crates/babylon-kernel/src/content_digest.rs
//! `ContentDigest` (spec §7): the canonical `{defines_hash, rules_hash}`
//! pair. This module owns `defines_hash`; `rules_hash` is wired once
//! `babylon-bsl`'s canonical AST serializer exists (Task 12 of this plan).
use sha2::{Digest, Sha256};

/// `ContentDigest.rules_hash` is `None` until Task 12 lands the canonical
/// BSL AST serializer — an explicit interim state, not a silent default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDigest {
    pub defines_hash: [u8; 32],
    pub rules_hash: Option<[u8; 32]>,
}

/// SHA-256 over the caller-supplied canonical JSON string. The canonical
/// form itself (sorted keys, `(",", ":")` separators, `ensure_ascii`) is the
/// Python side's job (`babylon.config.defines.canonical_defines_hash`,
/// Phase 0 Task 1) — this function trusts its input is already canonical
/// and only does the hashing, so the SAME canonicalization bug class Phase
/// 0 fixed cannot reappear split across two languages.
#[must_use]
pub fn defines_hash_of(canonical_json: &str) -> [u8; 32] {
    Sha256::digest(canonical_json.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-language conformance vector: the canonical JSON string and its
    /// expected hash, both taken from a real
    /// `canonical_defines_hash(GameDefines())` run (Phase 0 Task 1) — paste
    /// the real values at implementation time, do not invent placeholder
    /// hex.
    #[test]
    fn matches_the_python_canonical_defines_hash() {
        let canonical_json = r#"PASTE_REAL_CANONICAL_JSON_HERE"#;
        let expected_hex = "PASTE_REAL_64_HEX_HASH_HERE";
        let got = defines_hash_of(canonical_json);
        let got_hex: String = got.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(got_hex, expected_hex);
    }
}
```

- [ ] **Step 2: Generate the real conformance vector from Python and paste it in**

```bash
uv run python -c "
from babylon.config.defines import GameDefines, canonical_defines_hash
import json
d = GameDefines()
payload = d.model_dump(mode='json')
canonical = json.dumps(payload, sort_keys=True, separators=(',', ':'), ensure_ascii=True)
print(canonical)
print(canonical_defines_hash(d))
"
```
Paste the two printed lines into the test's placeholders verbatim. This is Task 1's
authorized bug fix's Rust-side conformance check — if Phase 0's `defines_hash` PR
hasn't merged yet when this task runs, STOP and escalate (this task is blocked on
that PR, not just "nice to have it").

- [ ] **Step 3: Run to green; gate + commit + PR** (slug `content-digest-defines-hash`).

---

### Task 8: **Director gate** — sigmoid: polynomial approximation vs. pinned deterministic libm

> **REFRAMED by ADR172 ruling (5) + ADR173 ruling (1), 2026-07-29:** no
> functional form may be imposed — the survival logistic does NOT port as a
> kernel intrinsic (its emergence formulation replaces it, Task 17 note). This
> task's analysis is therefore scoped to the transcendentals the closed algebra
> STILL needs after the proscription audit (`reports/
> p27-proscription-audit-2026-07-29.md`) — e.g. exp/log where entropy and
> metabolic math genuinely require them — and must open by enumerating that
> surviving set with citations before comparing poly vs. libm for it.

Spec §13 open ruling 2, explicitly scoped to Phase 1: "Sigmoid/transcendental
intrinsics: polynomial approximation vs pinned deterministic libm." This ruling gates
every transcendental kernel intrinsic Phase 2 will implement (sigmoid, exp, log, tanh,
sqrt, entropy) — get it early so Phase 2 isn't blocked waiting on it.

**Files:**
- Create: `ai/_inbox/sigmoid-ruling-p27.md`

**Interfaces:**
- Produces: a ratification-ready analysis with a recommended default, presented for
  Director sign-off — **not** a Phase-1 code deliverable; no intrinsic is implemented
  by this task.

- [ ] **Step 1: Gather the concrete options with tradeoffs, no hand-waving**

```bash
git checkout dev && git pull && git checkout -b docs/p27-sigmoid-ruling
rg -n "sigmoid|expit" src/babylon/formulas/ -l
rg -n "def sigmoid" -A15 src/babylon/formulas/*.py
```
Read every current Python sigmoid call site (Survival, ReserveArmy, TickDynamics per
spec §6.4's float-hazard inventory) to know what precision the game actually needs at
those call sites (e.g. is the sigmoid argument ever near saturation where polynomial
approximation error compounds visibly in a 520-tick campaign?).

- [ ] **Step 2: Write the two-option analysis**

  - **Option A — a fixed-degree polynomial (e.g. minimax or Padé) approximation.**
    Pros: bit-identical across every platform/toolchain by construction (pure `+`/`×`
    on `f64`, no libm call at all — the strongest form of III.7 determinism, and
    trivially portable to a future non-Rust BSL host, e.g. WASM). Cons: bounded
    accuracy (state the actual max absolute error over `[-10, 10]` for a chosen
    degree, computed, not assumed); every additional transcendental (exp, log, tanh,
    sqrt, entropy) needs its own hand-derived polynomial and its own error bound —
    more one-off math to get right and review.
  - **Option B — a pinned deterministic libm** (e.g. vendor a specific
    `libm`-equivalent pure-Rust crate such as the `libm` crate, which reimplements
    the C `libm` functions in portable Rust with no OS/platform dependency — verify
    at implementation time that it is bit-reproducible across the pinned toolchain's
    supported targets, since "system libm" would NOT be — that variant is out of
    scope by construction). Pros: full IEEE-754-adjacent precision, one crate covers
    every transcendental named in spec §5 uniformly, no per-function derivation
    burden. Cons: an external dependency in the determinism-critical path (mitigated
    if the crate is itself deterministic pure-Rust, but that claim needs verifying,
    not assuming — cite the crate's own test suite/CI matrix as evidence in the PR).
  - **Recommended default:** state one, with the one-paragraph reasoning (this
    plan's authors lean towards Option B — a vetted deterministic pure-Rust `libm`
    crate — on the grounds that per-function polynomial derivation is exactly the
    kind of one-off math the F1–F4 failure-class review exists to catch, and a single
    well-tested crate is a smaller trusted surface than N hand-derived polynomials —
    but this is explicitly the Director's call, not a default to self-merge).

- [ ] **Step 3: Commit + PR titled `docs(kernel): sigmoid/transcendental intrinsic
  ruling (P27 §13 item 2) — DIRECTOR GATE`. Do NOT self-merge.** State plainly in the
  PR body: "Phase 2's numeric-intrinsic work (the RUST_INTRINSIC and HYBRID systems'
  transcendental cores) is blocked on this ruling."

---

### Task 9: `babylon-bsl` reader — the s-expression parser

**Files:**
- Create: `rust/crates/babylon-bsl/src/reader.rs`
- Modify: `rust/crates/babylon-bsl/src/lib.rs`

**Interfaces:**
- Produces: `pub enum SExpr { Atom(String), List(Vec<SExpr>) }` and `pub fn
  read(source: &str) -> Result<SExpr, ReadError>` — the untyped s-expression tree the
  typechecker (Task 10) consumes; this task does not know about BSL's specific forms
  (conditions/effects/folds), only generic Lisp syntax (parens, atoms, string
  literals, keyword-colon tokens like `:material-basis`).

- [ ] **Step 1: Branch; write the failing parser tests**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-reader
```

```rust
// rust/crates/babylon-bsl/src/reader.rs
//! The BSL reader: a hand-written recursive-descent s-expression parser
//! (no external parser-combinator dependency — the grammar is small and a
//! hand-written parser keeps the dependency graph minimal, consistent with
//! this crate's "no unsafe, small trusted surface" bar). Untyped: produces
//! [`SExpr`] only; [`crate::typecheck`] (Task 10) assigns BSL-specific
//! meaning to the shapes.

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadError {
    pub message: String,
    pub position: usize,
}

/// Parse one top-level `SExpr` from `source`. A rule file transcribes as
/// one or more top-level forms — callers loop `read` over remaining input;
/// this function returns exactly one form and where parsing stopped.
pub fn read(source: &str) -> Result<(SExpr, usize), ReadError> {
    let mut chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    skip_whitespace(&chars, &mut pos);
    let expr = read_expr(&mut chars, &mut pos)?;
    Ok((expr, pos))
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && (chars[*pos].is_whitespace()) {
        *pos += 1;
    }
}

fn read_expr(chars: &mut Vec<char>, pos: &mut usize) -> Result<SExpr, ReadError> {
    skip_whitespace(chars, pos);
    if *pos >= chars.len() {
        return Err(ReadError { message: "unexpected end of input".into(), position: *pos });
    }
    match chars[*pos] {
        '(' => {
            *pos += 1;
            let mut items = Vec::new();
            loop {
                skip_whitespace(chars, pos);
                if *pos >= chars.len() {
                    return Err(ReadError { message: "unterminated list".into(), position: *pos });
                }
                if chars[*pos] == ')' {
                    *pos += 1;
                    return Ok(SExpr::List(items));
                }
                items.push(read_expr(chars, pos)?);
            }
        }
        ')' => Err(ReadError { message: "unexpected ')'".into(), position: *pos }),
        '"' => read_string_atom(chars, pos),
        _ => read_bare_atom(chars, pos),
    }
}

fn read_string_atom(chars: &[char], pos: &mut usize) -> Result<SExpr, ReadError> {
    let start = *pos;
    *pos += 1; // opening quote
    let mut s = String::new();
    while *pos < chars.len() && chars[*pos] != '"' {
        s.push(chars[*pos]);
        *pos += 1;
    }
    if *pos >= chars.len() {
        return Err(ReadError { message: "unterminated string".into(), position: start });
    }
    *pos += 1; // closing quote
    Ok(SExpr::Atom(format!("\"{s}\"")))
}

fn read_bare_atom(chars: &[char], pos: &mut usize) -> Result<SExpr, ReadError> {
    let start = *pos;
    while *pos < chars.len() && !chars[*pos].is_whitespace() && chars[*pos] != '(' && chars[*pos] != ')' {
        *pos += 1;
    }
    Ok(SExpr::Atom(chars[start..*pos].iter().collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_flat_list() {
        let (expr, _) = read("(add 1 2)").unwrap();
        assert_eq!(
            expr,
            SExpr::List(vec![
                SExpr::Atom("add".into()),
                SExpr::Atom("1".into()),
                SExpr::Atom("2".into()),
            ])
        );
    }

    #[test]
    fn parses_nested_lists() {
        let (expr, _) = read("(fold (node social_class) (sum wealth))").unwrap();
        assert!(matches!(expr, SExpr::List(items) if items.len() == 3));
    }

    #[test]
    fn parses_keyword_atoms() {
        let (expr, _) = read("(:material-basis \"exploitation of labor\")").unwrap();
        assert!(matches!(&expr, SExpr::List(items) if items.len() == 2 && items[0] == SExpr::Atom(":material-basis".into())));
    }

    #[test]
    fn unterminated_list_is_a_loud_error_not_a_panic() {
        assert!(read("(add 1 2").is_err());
    }

    #[test]
    fn unterminated_string_is_a_loud_error() {
        assert!(read("(:material-basis \"unterminated").is_err());
    }
}
```

- [ ] **Step 2: Run red, then implement to green**

```bash
cd rust && cargo test -p babylon-bsl reader --locked && cd ..
```

- [ ] **Step 3: Clippy pedantic pass** (this hand-rolled parser is exactly the kind of
  code pedantic complains about — indexing, `as` casts; fix every lint or add a
  narrowly-scoped `#[allow(clippy::...)]` with a one-line reason, per the "explicit
  exemption with a reason" house rule, never a blanket allow).

- [ ] **Step 4: Gate + commit + PR** (slug `bsl-reader`).

---

### Task 10: `babylon-bsl` typechecker — types, closed enums, intensive/extensive kind

**Files:**
- Create: `rust/crates/babylon-bsl/src/types.rs` (the BSL type universe)
- Create: `rust/crates/babylon-bsl/src/typecheck.rs`
- Create: `rust/crates/babylon-bsl/src/exemptions.rs` (`EXTENSIVE_INTENSIVE_EXEMPTIONS`)
- Modify: `rust/crates/babylon-bsl/src/lib.rs`

**Interfaces:**
- Produces: `pub enum BslType` (kernel scalars + closed enums + bool + typed
  node/edge-set refs), `pub fn typecheck(expr: &SExpr, env: &TypeEnv) -> Result<BslType,
  TypeError>`, and the intensive/extensive-kind propagation rule: a fold aggregating an
  intensive-kinded field without an explicit weight term is a `TypeError`, not a
  runtime surprise.

- [ ] **Step 1: Branch; define the type universe with a failing "unweighted
  aggregation of an intensive field is a type error" test first (this is the load-
  bearing law from spec §5 Types — write it before anything else compiles)**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-typechecker
```

```rust
// rust/crates/babylon-bsl/src/types.rs
//! The BSL type universe (spec §5 Types): kernel scalars, closed enums,
//! booleans, and typed node/edge-set references. Intensivity is a
//! per-field DECLARATION on model fields (`:kind intensive|extensive`), not
//! a property of the scalar type itself — `FieldKind` travels alongside a
//! field's `BslType`, and the typechecker (not the type) enforces the
//! unweighted-aggregation-is-an-error rule.

#[derive(Debug, Clone, PartialEq)]
pub enum BslType {
    Probability,
    Intensity,
    Coefficient,
    Currency,
    Bool,
    Enum(&'static str), // the closed enum's name, e.g. "DoctrineTag"
    NodeSet(&'static str), // typed node-set reference, e.g. NodeType name
    EdgeSet(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Intensive,
    Extensive,
}

/// A model field's declared type + kind — the typechecker's environment
/// entry for anything a fold can read.
#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub ty: BslType,
    pub kind: FieldKind,
}
```

```rust
// rust/crates/babylon-bsl/src/typecheck.rs
//! The BSL typechecker (spec §5 Types, §6.3 "the extensive/intensive
//! lexicon becomes real types"). Load-bearing law: unweighted aggregation
//! of an intensive-kinded field is a `TypeError`; weighted aggregation is
//! legal (M10 — the spec's own correction of an earlier too-strong claim).
use crate::reader::SExpr;
use crate::types::{BslType, FieldDecl, FieldKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub message: String,
}

pub struct TypeEnv {
    pub fields: HashMap<String, FieldDecl>,
}

/// Typecheck a `(sum <field>)`-shaped aggregation form. This is the
/// narrowest possible slice of the full typechecker (folds, guards,
/// intrinsic calls land in later tasks) — enough to pin the intensive law
/// red-then-green before the rest of the surface grows around it.
pub fn typecheck_aggregation(expr: &SExpr, env: &TypeEnv) -> Result<BslType, TypeError> {
    let SExpr::List(items) = expr else {
        return Err(TypeError { message: "aggregation must be a list form".into() });
    };
    let [SExpr::Atom(op), SExpr::Atom(field_name), rest @ ..] = items.as_slice() else {
        return Err(TypeError { message: "aggregation form must be (op field [:weight w])".into() });
    };
    let field = env
        .fields
        .get(field_name)
        .ok_or_else(|| TypeError { message: format!("unknown field: {field_name}") })?;
    let has_weight = rest.iter().any(|e| matches!(e, SExpr::Atom(a) if a == ":weight"));
    if field.kind == FieldKind::Intensive && !has_weight && op != "count" {
        return Err(TypeError {
            message: format!(
                "unweighted aggregation ({op}) of intensive field '{field_name}': \
                 add an explicit :weight term (spec §5 Types)"
            ),
        });
    }
    Ok(field.ty.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;

    fn env_with_intensive_wealth_share() -> TypeEnv {
        let mut fields = HashMap::new();
        fields.insert(
            "wealth_share".to_string(),
            FieldDecl { ty: BslType::Coefficient, kind: FieldKind::Intensive },
        );
        TypeEnv { fields }
    }

    #[test]
    fn unweighted_average_of_intensive_field_is_a_type_error() {
        let env = env_with_intensive_wealth_share();
        let (expr, _) = read("(avg wealth_share)").unwrap();
        let result = typecheck_aggregation(&expr, &env);
        assert!(result.is_err(), "unweighted intensive aggregation must be rejected");
    }

    #[test]
    fn weighted_average_of_intensive_field_is_legal() {
        let env = env_with_intensive_wealth_share();
        let (expr, _) = read("(avg wealth_share :weight population)").unwrap();
        assert!(typecheck_aggregation(&expr, &env).is_ok());
    }

    #[test]
    fn count_never_needs_a_weight_even_for_intensive_fields() {
        let env = env_with_intensive_wealth_share();
        let (expr, _) = read("(count wealth_share)").unwrap();
        assert!(typecheck_aggregation(&expr, &env).is_ok());
    }
}
```

- [ ] **Step 2: Run red, implement, run green**

```bash
cd rust && cargo test -p babylon-bsl typecheck --locked && cd ..
```

- [ ] **Step 3: The `EXTENSIVE_INTENSIVE_EXEMPTIONS` ledger** (spec §5: "Exemptions
  live in a declared `EXTENSIVE_INTENSIVE_EXEMPTIONS` ledger with a mandatory reason
  string; adding a row takes the same sign-off as a sentinel exemption") — mirror the
  shape of Python's `SentinelExemption` (`src/babylon/sentinels/exemptions.py` /
  `vocabulary/registry.py`'s `ATTRIBUTE_EXEMPTIONS`):

```rust
// rust/crates/babylon-bsl/src/exemptions.rs
//! Exemptions from the unweighted-intensive-aggregation rule (spec §5),
//! mirroring the shape of Python's `SentinelExemption`
//! (`babylon.sentinels.exemptions`) — same governance bar: every row needs
//! a reason, an owner, and a date. This ledger starts EMPTY in Phase 1 (no
//! BSL content exists yet to need an exemption); Phase 2's transcription
//! work is expected to populate it, never this task.

#[derive(Debug, Clone)]
pub struct IntensiveAggregationExemption {
    pub field_name: &'static str,
    pub reason: &'static str,
    pub owner: &'static str,
    pub date: &'static str,
}

pub const EXTENSIVE_INTENSIVE_EXEMPTIONS: &[IntensiveAggregationExemption] = &[];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_exemption_row_carries_a_non_empty_reason() {
        for exemption in EXTENSIVE_INTENSIVE_EXEMPTIONS {
            assert!(!exemption.reason.is_empty());
        }
    }
}
```

- [ ] **Step 4: Gate + commit + PR** (slug `bsl-typechecker`).

---

### Task 11: The graph trait boundary — `babylon-graph` (native-hyperedge shape, Amendment D)

Per this plan's task assignment: plan the trait boundary **explicitly**. The typed
structural verbs (Task 16) need *something* to typecheck graph mutation against.
**Amendment D ruled 2026-07-29: NATIVE HYPEREDGE** (Amendment AE clause (vi);
`ai/_inbox/amendment-d-analysis-p27.md` §9, PR #353), so the trait is no longer written
blind to the data shape:

- **Hyperedges are first-class in the exposed model and type system.** A hyperedge is a
  typed object with its own identity and a member list — not a clique expansion of
  pairwise edges (VIII.9 preserved by construction), and not *exposed* as a bipartite
  incidence encoding.
- **The dyadic edge API remains alongside it, unchanged.** II.9's morphism layer stays
  strictly dyadic; D-2 rules that type-level separation inside one substrate satisfies
  II.7's separation clause — strictly stronger than the pre-v2 "two libraries" reading.
  One substrate, typed homes.
- **Levi/incidence remains a permitted *internal storage strategy*.** It is the standard
  realization of native hyperedges in one typed substrate and is what `hypergraph-rs`
  implements (`NodeKind::{Agent, Hyperedge}` + `MembershipEdge<M>`); a concrete
  implementation may store hyperedges that way, but no caller may observe it. Nothing in
  the trait below exposes an incidence node or an incidence edge.
- Still deliberately **not** decided here: the concrete storage type (`StableDiGraph`
  bipartite vs. anything else), adjacency iteration order at the storage level, and the
  closed `NodeType`/`EdgeType`/`HyperedgeType` enums (those are `babylon-domain`,
  Phase 2/3). This task still ships a trait plus a placeholder implementation, both
  sufficient for Tasks 16/17 to compile — what changed is that the trait's *surface* is
  now ruled, so downstream code no longer typechecks against a shape that might move.

Simplicial closure is **not** the membership substrate (D-6): a simplicial view may
exist later only as a derived construct carrying its own III.10 justification, and no
part of this task builds one.

**Files:**
- Create: `rust/crates/babylon-graph/Cargo.toml`, `rust/crates/babylon-graph/src/lib.rs`
- Create: `rust/crates/babylon-graph/src/substrate.rs` (the trait)
- Create: `rust/crates/babylon-graph/src/placeholder.rs` (an in-memory `HashMap`-backed
  toy implementation, explicitly NOT the production shape)
- Modify: `rust/Cargo.toml` (add crate to workspace members)
- Modify: `.mise.toml` (`rust:check` per-crate lines)

**Interfaces:**
- Produces: `pub trait GraphSubstrate` with the typed-verb surface — the dyadic half
  (`add_node`, `remove_node`, `add_edge`, `remove_edge`, `update_node`) plus the
  hyperedge half Amendment D rules first-class (`add_hyperedge`, `remove_hyperedge`,
  `members_of`, `hyperedges_of`) — generic over `NodeType`/`EdgeType`/`HyperedgeType`
  markers, with NO commitment to adjacency representation or storage backend (a Levi
  bipartite store is permitted and unobservable). `PlaceholderGraph` implements the
  trait with `HashMap`s, gated behind a doc comment making clear it is a compile-target
  for Tasks 16/17, never a production candidate.

- [ ] **Step 1: Branch; scaffold the crate**

```bash
git checkout dev && git pull && git checkout -b feature/p27-graph-trait-boundary
```

```toml
# rust/crates/babylon-graph/Cargo.toml
[package]
name = "babylon-graph"
description = "Graph substrate trait boundary (Program 27, native-hyperedge — see src/substrate.rs)"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[dependencies]
babylon-kernel = { path = "../babylon-kernel" }

[dev-dependencies]
pretty_assertions = "1"
```

```rust
// rust/crates/babylon-graph/src/lib.rs
//! The graph substrate crate (spec §6, crate table). **Amendment D ruled
//! NATIVE HYPEREDGE** (2026-07-29; Amendment AE clause (vi),
//! `ai/_inbox/amendment-d-analysis-p27.md` §9): hyperedges are first-class
//! objects in this crate's exposed model and type system — membership is one
//! typed hyperedge, never a clique expansion, and never *exposed* as a
//! bipartite incidence encoding. Levi/incidence is a permitted INTERNAL
//! storage strategy (what `hypergraph-rs` implements); nothing in the exposed
//! API reveals it. The strictly dyadic morphism API (II.9) lives alongside it
//! in the same trait, separated by type (D-2).
//!
//! This crate exposes ONLY the [`substrate::GraphSubstrate`] trait plus a
//! [`placeholder::PlaceholderGraph`] toy implementation sufficient to let
//! downstream crates (`babylon-bsl`'s typed structural verbs, the conformance
//! corpus) compile and typecheck against a real trait object today. The
//! concrete production storage type is Phase 2 work. Do not build production
//! logic against `PlaceholderGraph` — it is a compile-target, not a
//! foundation.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod placeholder;
pub mod substrate;
```

- [ ] **Step 2: Write the trait with failing tests against the placeholder**

```rust
// rust/crates/babylon-graph/src/substrate.rs
//! The `GraphSubstrate` trait: the typed-verb surface BSL's structural
//! effects compile against (`docs/reference/bsl-language.rst` §2.6 queries,
//! §2.8 verbs), independent of the underlying storage.
//!
//! **Two typed halves, one substrate (Amendment D, sub-ruling D-2).** The
//! dyadic half (`add_edge`/`remove_edge`) is II.9's strictly dyadic morphism
//! layer. The hyperedge half (`add_hyperedge`/`remove_hyperedge`/
//! `members_of`/`hyperedges_of`) is Amendment D's first-class membership
//! layer. A dyadic caller cannot be handed a hyperedge and vice versa —
//! II.7's "MUST remain separate" is enforced by the type system rather than
//! by two libraries.
//!
//! **Silent on representation, loud on shape.** No adjacency iteration order
//! and no storage type is exposed; a Levi/incidence bipartite store is
//! permitted and unobservable. What IS exposed, because the ruling fixes it:
//! a hyperedge has an identity and a member list, and there is no method
//! anywhere that expands a member list into pairwise edges (VIII.9).

/// Opaque node identity — a newtype so no caller depends on it being an
/// integer index vs. a UUID vs. anything else the concrete shape picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// Opaque hyperedge identity. **Distinct from `NodeId` on purpose**: a
/// hyperedge is a first-class object, not a node with a member set stashed in
/// its attributes (the shape D-4 explicitly declines to ratify for
/// `ECONOMIC_SECTOR`). Two id types is what makes the dyadic/hyperedge
/// separation type-level rather than conventional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HyperedgeId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphError {
    pub message: String,
}

/// The typed structural-verb surface a `GraphSubstrate` implementation
/// provides. `node_type`/`edge_type`/`hyperedge_type` are `&'static str` here
/// (the closed `NodeType`/`EdgeType`/`HyperedgeType` enums are a
/// `babylon-domain` concern, Phase 2/3; this trait is domain-agnostic on
/// purpose so it compiles before those enums port).
pub trait GraphSubstrate {
    // ---- dyadic half (II.9 morphism layer) ----
    fn add_node(&mut self, node_type: &'static str) -> Result<NodeId, GraphError>;
    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError>;
    fn add_edge(&mut self, edge_type: &'static str, from: NodeId, to: NodeId) -> Result<(), GraphError>;
    fn remove_edge(&mut self, edge_type: &'static str, from: NodeId, to: NodeId) -> Result<(), GraphError>;
    /// Update a single attribute on a node under the I.15 edge-mode state
    /// machine's constraints — this trait method does NOT itself enforce
    /// I.15 (that is a `babylon-domain` law over the concrete shape); it is
    /// the mechanical write point I.15's checker wraps.
    fn update_node(&mut self, id: NodeId, attribute: &'static str, value: f64) -> Result<(), GraphError>;
    fn node_exists(&self, id: NodeId) -> bool;

    // ---- hyperedge half (Amendment D: first-class membership) ----
    /// Mint one typed hyperedge over `members`. The member list is a SET:
    /// a repeated `NodeId`, an unknown `NodeId`, or an empty list is a loud
    /// error (BSL `E-EVAL-031`), never deduplicated or ignored. Cost is
    /// `members.len()` incidences — never `C(n,2)` edges.
    fn add_hyperedge(
        &mut self,
        hyperedge_type: &'static str,
        members: &[NodeId],
    ) -> Result<HyperedgeId, GraphError>;
    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError>;
    /// Members of one hyperedge, in **ascending `NodeId` order** — declared
    /// member order is never observable (BSL §2.6 draft ruling D25).
    fn members_of(&self, id: HyperedgeId) -> Result<Vec<NodeId>, GraphError>;
    /// The hyperedges of the given type a node belongs to, in ascending
    /// `HyperedgeId` order.
    fn hyperedges_of(&self, node: NodeId, hyperedge_type: &'static str) -> Vec<HyperedgeId>;
    fn hyperedge_exists(&self, id: HyperedgeId) -> bool;
}
```

**Note for the implementer.** `members_of` returns an owned `Vec` rather than an
iterator deliberately: BSL materializes a query in sort order before the fold body runs
(language reference §4.4), so the trait's contract is "a sorted snapshot", and an
iterator would leak the storage's own ordering into the signature — exactly the thing
the ruling says must stay internal. If profiling later justifies a borrowed form, it is
a Phase-2 change with the sort obligation restated, not a Phase-1 optimization.

```rust
// rust/crates/babylon-graph/src/placeholder.rs
//! `PlaceholderGraph`: an in-memory `HashMap`-backed toy [`GraphSubstrate`]
//! implementation. **This is NOT the production graph storage** — it exists
//! solely so Tasks 16/17 of the Phase-1 plan (BSL's typed structural verbs,
//! the conformance corpus) have something real to typecheck and run against
//! before the concrete Phase-2 storage lands. It DOES honor the Amendment D
//! shape the trait fixes (hyperedges are their own objects with their own id
//! space, members are a sorted set, no pairwise expansion anywhere), because
//! that shape is ruled, not provisional. Deleting this module and swapping in
//! the production storage is expected, low-risk churn — nothing outside this
//! crate and its direct test dependents should assume its internals.
use crate::substrate::{GraphError, GraphSubstrate, HyperedgeId, NodeId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PlaceholderGraph {
    nodes: HashMap<NodeId, &'static str>,
    attributes: HashMap<(NodeId, &'static str), f64>,
    /// Hyperedge id -> (type, sorted member list). Stored as ONE record per
    /// hyperedge — the toy analogue of a first-class object. A production
    /// store may instead keep incidence edges (Levi); callers cannot tell.
    hyperedges: HashMap<HyperedgeId, (&'static str, Vec<NodeId>)>,
    next_id: u64,
    next_hyperedge_id: u64,
}

impl PlaceholderGraph {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl GraphSubstrate for PlaceholderGraph {
    fn add_node(&mut self, node_type: &'static str) -> Result<NodeId, GraphError> {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id, node_type);
        Ok(id)
    }

    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        self.nodes
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| GraphError { message: format!("no such node: {id:?}") })
    }

    fn add_edge(&mut self, _edge_type: &'static str, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        if !self.node_exists(from) || !self.node_exists(to) {
            return Err(GraphError { message: "edge endpoint does not exist".into() });
        }
        Ok(()) // placeholder: no edge storage at all, deliberately (see module doc)
    }

    fn remove_edge(&mut self, _edge_type: &'static str, _from: NodeId, _to: NodeId) -> Result<(), GraphError> {
        Ok(())
    }

    fn update_node(&mut self, id: NodeId, attribute: &'static str, value: f64) -> Result<(), GraphError> {
        if !self.node_exists(id) {
            return Err(GraphError { message: format!("no such node: {id:?}") });
        }
        self.attributes.insert((id, attribute), value);
        Ok(())
    }

    fn node_exists(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    fn add_hyperedge(
        &mut self,
        hyperedge_type: &'static str,
        members: &[NodeId],
    ) -> Result<HyperedgeId, GraphError> {
        if members.is_empty() {
            return Err(GraphError { message: "hyperedge must have at least one member".into() });
        }
        let mut sorted: Vec<NodeId> = members.to_vec();
        sorted.sort_unstable_by_key(|n| n.0);
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(GraphError { message: "duplicate member in hyperedge".into() });
        }
        if let Some(missing) = sorted.iter().find(|n| !self.node_exists(**n)) {
            return Err(GraphError { message: format!("no such member node: {missing:?}") });
        }
        let id = HyperedgeId(self.next_hyperedge_id);
        self.next_hyperedge_id += 1;
        self.hyperedges.insert(id, (hyperedge_type, sorted));
        Ok(id)
    }

    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError> {
        self.hyperedges
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| GraphError { message: format!("no such hyperedge: {id:?}") })
    }

    fn members_of(&self, id: HyperedgeId) -> Result<Vec<NodeId>, GraphError> {
        self.hyperedges
            .get(&id)
            .map(|(_, members)| members.clone()) // already sorted at insert
            .ok_or_else(|| GraphError { message: format!("no such hyperedge: {id:?}") })
    }

    fn hyperedges_of(&self, node: NodeId, hyperedge_type: &'static str) -> Vec<HyperedgeId> {
        let mut found: Vec<HyperedgeId> = self
            .hyperedges
            .iter()
            .filter(|(_, (ty, members))| *ty == hyperedge_type && members.contains(&node))
            .map(|(id, _)| *id)
            .collect();
        found.sort_unstable_by_key(|h| h.0);
        found
    }

    fn hyperedge_exists(&self, id: HyperedgeId) -> bool {
        self.hyperedges.contains_key(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_then_update_then_read_back() {
        let mut g = PlaceholderGraph::new();
        let n = g.add_node("social_class").unwrap();
        g.update_node(n, "wealth", 42.0).unwrap();
        assert_eq!(g.attributes.get(&(n, "wealth")), Some(&42.0));
    }

    #[test]
    fn edge_to_nonexistent_node_is_a_loud_error() {
        let mut g = PlaceholderGraph::new();
        let n = g.add_node("territory").unwrap();
        assert!(g.add_edge("adjacency", n, NodeId(9999)).is_err());
    }

    #[test]
    fn hyperedge_members_come_back_sorted_not_in_declared_order() {
        // Amendment D / BSL D25: declared member order is unobservable.
        let mut g = PlaceholderGraph::new();
        let a = g.add_node("social_class").unwrap();
        let b = g.add_node("social_class").unwrap();
        let c = g.add_node("social_class").unwrap();
        let h = g.add_hyperedge("economic_sector", &[c, a, b]).unwrap();
        assert_eq!(g.members_of(h).unwrap(), vec![a, b, c]);
    }

    #[test]
    fn duplicate_member_is_a_loud_error() {
        let mut g = PlaceholderGraph::new();
        let a = g.add_node("social_class").unwrap();
        assert!(g.add_hyperedge("economic_sector", &[a, a]).is_err());
    }

    #[test]
    fn a_hyperedge_mints_no_pairwise_edges() {
        // VIII.9 by construction: n members cost one object, not C(n,2) edges.
        let mut g = PlaceholderGraph::new();
        let a = g.add_node("social_class").unwrap();
        let b = g.add_node("social_class").unwrap();
        let h = g.add_hyperedge("economic_sector", &[a, b]).unwrap();
        assert_eq!(g.hyperedges_of(a, "economic_sector"), vec![h]);
        // the dyadic half is untouched by minting a hyperedge
        assert!(g.hyperedges.len() == 1);
    }
}
```

- [ ] **Step 3: Wire into the workspace + mise, same shape as Task 1**

```toml
# rust/Cargo.toml
[workspace]
members = [
    "crates/babylon-kernel",
    "crates/babylon-bsl",
    "crates/babylon-graph",
    "crates/babylon-md",
    "crates/babylon-tui",
    "crates/babylon-tui-python",
]
```
Append `cargo clippy -p babylon-graph ... -D clippy::pedantic` / `cargo test -p
babylon-graph --locked` lines to `mise run rust:check`, same as Task 1.

- [ ] **Step 4: Verify the layering law** — `babylon-bsl` may reference
  `babylon-graph`'s trait (Task 13 adds this dependency, not this task); confirm this
  task does NOT add a `babylon-graph` → `babylon-bsl` edge in either direction yet:

```bash
cd rust
cargo tree -p babylon-graph | rg babylon-bsl   # expect: nothing
cargo build --workspace --locked
cargo clippy -p babylon-graph --all-targets --locked -- -D warnings -D clippy::pedantic
cd ..
```

- [ ] **Step 5: Commit + PR** — title: `feat(graph): trait boundary — native-hyperedge
  surface (P27, Amendment D)`. Body states: "The trait's shape follows the Director's
  2026-07-29 NATIVE HYPEREDGE ruling (Amendment AE clause (vi)): first-class typed
  hyperedges alongside the dyadic morphism API, Levi/incidence permitted as internal
  storage only, no clique expansion expressible. This PR commits no concrete storage
  type — `PlaceholderGraph` is a compile-target for Tasks 16/17, explicitly not
  production; the production store is Phase 2." Self-mergeable: it implements a ruling,
  it does not make one.

---

### Task 12: Canonical BSL AST serialization → `rules_hash` (closes `ContentDigest`)

**Files:**
- Create: `rust/crates/babylon-bsl/src/canonical_ast.rs`
- Modify: `rust/crates/babylon-kernel/src/content_digest.rs` (make `rules_hash`
  mandatory; this is the task that gets to do that, per Task 7's documented interim
  state)
- Modify: `docs/reference/determinism-contract.rst` (fill in the canonical AST
  serialization worked example if Phase 0 Task 12 left it as a byte-layout table
  without one)

**Interfaces:**
- Produces: `pub fn canonical_bytes(expr: &SExpr) -> Vec<u8>` — a
  whitespace/comment-insensitive canonical serialization (spec §7: "so a rule edit is
  declared input drift while a formatting edit is nothing"), and
  `rules_hash_of(rules: &[SExpr]) -> [u8; 32]`.

- [ ] **Step 1: Branch; write the failing "two differently-formatted-but-semantically-
  identical rules hash the same" test — this is the property the whole feature exists
  for, pin it first**

```bash
git checkout dev && git pull && git checkout -b feature/p27-canonical-ast-serialization
```

```rust
// rust/crates/babylon-bsl/src/canonical_ast.rs
//! Canonical, whitespace/comment-insensitive BSL AST serialization (spec
//! §7): the byte layout `rules_hash` hashes. A rule edit is declared input
//! drift; a formatting-only edit (reindentation, comment change) is not —
//! this module is what makes that distinction real rather than aspirational.
use crate::reader::SExpr;
use sha2::{Digest, Sha256};

/// Canonical byte form: parenthesized, single-space-separated, atoms
/// written verbatim (already comment-free — the reader, Task 9, does not
/// preserve comments in the parsed `SExpr` at all, so "comment-insensitive"
/// is true by construction, not by a stripping step here).
#[must_use]
pub fn canonical_bytes(expr: &SExpr) -> Vec<u8> {
    let mut out = Vec::new();
    write_canonical(expr, &mut out);
    out
}

fn write_canonical(expr: &SExpr, out: &mut Vec<u8>) {
    match expr {
        SExpr::Atom(a) => out.extend_from_slice(a.as_bytes()),
        SExpr::List(items) => {
            out.push(b'(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                write_canonical(item, out);
            }
            out.push(b')');
        }
    }
}

/// `ContentDigest.rules_hash` over an ordered set of top-level rule forms.
/// Order matters (insertion-order-as-structure, per the spec's adopted
/// Haskell-draft ruling) — this is NOT a sorted/order-independent hash.
#[must_use]
pub fn rules_hash_of(rules: &[SExpr]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for rule in rules {
        hasher.update(canonical_bytes(rule));
        hasher.update([0u8]); // separator, so (a)(b) != (a b) at the top level
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;

    #[test]
    fn differently_formatted_semantically_identical_rules_hash_the_same() {
        let (a, _) = read("(add\n  1\n  2)").unwrap();
        let (b, _) = read("(add 1 2)").unwrap();
        assert_eq!(rules_hash_of(&[a]), rules_hash_of(&[b]));
    }

    #[test]
    fn a_genuine_content_edit_changes_the_hash() {
        let (a, _) = read("(add 1 2)").unwrap();
        let (b, _) = read("(add 1 3)").unwrap();
        assert_ne!(rules_hash_of(&[a]), rules_hash_of(&[b]));
    }

    #[test]
    fn rule_order_is_load_bearing() {
        let (a, _) = read("(rule-a)").unwrap();
        let (b, _) = read("(rule-b)").unwrap();
        assert_ne!(rules_hash_of(&[a.clone(), b.clone()]), rules_hash_of(&[b, a]));
    }
}
```

- [ ] **Step 2: Run red, implement, run green**

```bash
cd rust && cargo test -p babylon-bsl canonical_ast --locked && cd ..
```

- [ ] **Step 3: Close out `ContentDigest`** — flip `rules_hash` from `Option<[u8;
  32]>` to `[u8; 32]` in `babylon-kernel`'s `content_digest.rs`, add a
  `babylon-bsl`-side constructor `ContentDigest::new(defines_hash: [u8; 32], rules:
  &[SExpr]) -> ContentDigest` (this necessarily lives in `babylon-bsl`, not
  `babylon-kernel`, since it's the first thing in the crate DAG that knows about
  `SExpr` — update the layering-law note in `babylon-kernel`'s doc comment to point
  there). Update Task 7's test to exercise the now-mandatory field.

- [ ] **Step 4: Gate + commit + PR** (slug `canonical-ast-rules-hash`).

---

### Task 13: Load-time bound checker — declared cardinality ceilings + the fuel cost model

**Files:**
- Create: `rust/crates/babylon-bsl/src/fuel.rs` (the per-AST-node cost model)
- Create: `rust/crates/babylon-bsl/src/bound_checker.rs`
- Modify: `rust/crates/babylon-bsl/Cargo.toml` (dependency on `babylon-graph`'s trait,
  for the fold-target cardinality declarations)

**Interfaces:**
- Produces: `pub fn static_fuel_bound(expr: &SExpr, ceilings: &CardinalityCeilings) ->
  u64` (the worst-case bound: sum over folds of declared-ceiling × per-node AST cost)
  and `pub fn check_bound(expr: &SExpr, ceilings: &CardinalityCeilings, budget: u64) ->
  Result<(), BoundExceeded>` — a rule whose bound exceeds its declared budget is
  rejected **at load time** (III.11), making the Power-of-10 Rule 2 claim a static
  property (spec §5 Totality).

- [ ] **Step 1: Branch; pin the fuel cost constants from the III.12(a) reference**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-bound-checker
rg -n "literal 0|variable-ref 1|arithmetic op 1|fold 2" docs/reference/determinism-contract.rst
```
These constants are pinned by Phase 0 Task 12's fuel chapter (literal: 0,
variable-ref: 1, arithmetic op: 1, comparison: 1, boolean op: 1, intrinsic call: 5 +
callee cost, fold: 2 + ceiling×body) — **read the actual chapter, do not
re-invent the numbers here**; if any constant in this task's code disagrees with the
reference doc, the reference doc wins and this task's code is wrong, not the other
way around (revising the constants requires the vector re-bless the reference chapter
itself specifies).

```rust
// rust/crates/babylon-bsl/src/fuel.rs
//! The per-AST-node fuel cost model (spec §5 Totality;
//! `docs/reference/determinism-contract.rst`'s fuel chapter, Phase 0 Task
//! 12, is the normative source for every constant below — this module
//! transcribes it, it does not originate it).

/// Mirrors the reference doc's cost table exactly. Changing a constant here
/// requires the conformance-vector re-bless the reference chapter mandates
/// — never edit silently.
pub mod cost {
    pub const LITERAL: u64 = 0;
    pub const VARIABLE_REF: u64 = 1;
    pub const ARITHMETIC_OP: u64 = 1;
    pub const COMPARISON: u64 = 1;
    pub const BOOLEAN_OP: u64 = 1;
    pub const INTRINSIC_CALL_BASE: u64 = 5;
    pub const FOLD_BASE: u64 = 2;
}

/// Declared per-NodeType/per-EdgeType/per-HyperedgeType cardinality ceilings
/// from a scenario manifest (spec §5: "declared against declared cardinality
/// ceilings, not the runtime graph"). Phase 1 takes this as an opaque lookup;
/// the scenario-manifest format itself is a Phase 2 content concern.
///
/// **Two axes, since Amendment D** (language reference §3.7): a hyperedge type
/// declares both how many hyperedges may exist (`ceilings`) and how many
/// members any one of them may carry (`max_members`). A fold over `members-of`
/// bounds against the second; without it there is no static bound at all.
pub struct CardinalityCeilings {
    ceilings: std::collections::HashMap<&'static str, u64>,
    max_members: std::collections::HashMap<&'static str, u64>,
}

impl CardinalityCeilings {
    #[must_use]
    pub fn new(
        ceilings: std::collections::HashMap<&'static str, u64>,
        max_members: std::collections::HashMap<&'static str, u64>,
    ) -> Self {
        Self { ceilings, max_members }
    }

    #[must_use]
    pub fn get(&self, graph_element_type: &str) -> Option<u64> {
        self.ceilings.get(graph_element_type).copied()
    }

    /// The declared `:max-members` of a hyperedge type. `None` for a
    /// node/edge type — and a `None` here on a `members-of` fold is a
    /// load error (`E-LOAD-042`), never a silent zero.
    #[must_use]
    pub fn max_members(&self, hyperedge_type: &str) -> Option<u64> {
        self.max_members.get(hyperedge_type).copied()
    }
}
```

- [ ] **Step 2: Write the failing static-bound test** — a fold over a declared-ceiling
  node type must bound to `FOLD_BASE + ceiling * body_cost`, and a rule whose bound
  exceeds its declared per-rule budget is rejected before any evaluation:

```rust
// rust/crates/babylon-bsl/src/bound_checker.rs
//! Load-time static fuel bound checking (spec §5 Totality): "a rule whose
//! bound exceeds its budget is rejected at content load" — this is the
//! machinery that makes that claim true.
use crate::fuel::{cost, CardinalityCeilings};
use crate::reader::SExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct BoundExceeded {
    pub computed_bound: u64,
    pub declared_budget: u64,
}

/// Compute the worst-case static fuel bound for `expr` against `ceilings`.
/// This is intentionally a small, total, non-recursive-in-the-dangerous-
/// sense function: `expr` is already a finite tree (the reader, Task 9,
/// cannot produce a cyclic structure), so structural recursion here
/// terminates by construction — no separate loop-bound proof is needed for
/// THIS function; it computes a bound, it does not enforce one at runtime
/// (that's `crate::evaluator`, Task 14).
#[must_use]
pub fn static_fuel_bound(expr: &SExpr, ceilings: &CardinalityCeilings) -> u64 {
    match expr {
        SExpr::Atom(a) if a.parse::<f64>().is_ok() => cost::LITERAL,
        SExpr::Atom(_) => cost::VARIABLE_REF,
        SExpr::List(items) => match items.first() {
            Some(SExpr::Atom(op)) if op == "fold" => {
                let target_type = fold_target_type(items);
                let ceiling = ceilings.get(&target_type).unwrap_or(0);
                let body_cost: u64 = items[2..].iter().map(|e| static_fuel_bound(e, ceilings)).sum();
                cost::FOLD_BASE + ceiling.saturating_mul(body_cost)
            }
            Some(SExpr::Atom(op)) if is_intrinsic_call(op) => {
                cost::INTRINSIC_CALL_BASE
                    + items[1..].iter().map(|e| static_fuel_bound(e, ceilings)).sum::<u64>()
            }
            Some(SExpr::Atom(op)) if is_comparison(op) => {
                cost::COMPARISON + items[1..].iter().map(|e| static_fuel_bound(e, ceilings)).sum::<u64>()
            }
            Some(SExpr::Atom(op)) if is_boolean(op) => {
                cost::BOOLEAN_OP + items[1..].iter().map(|e| static_fuel_bound(e, ceilings)).sum::<u64>()
            }
            _ => cost::ARITHMETIC_OP + items.iter().map(|e| static_fuel_bound(e, ceilings)).sum::<u64>(),
        },
    }
}

fn fold_target_type(_items: &[SExpr]) -> String {
    // TODO(implementer): extract the fold's target node/edge-type atom from
    // its declared shape once Task 2's grammar pins the exact fold syntax
    // (e.g. `(fold (node social_class) body...)`); placeholder returns an
    // empty string, which `ceilings.get` correctly treats as "no ceiling
    // declared", itself a load-time error the real implementation must
    // raise rather than silently defaulting to 0 (would UNDER-count the
    // bound, the opposite of loud failure) — resolve before this task closes.
    //
    // Amendment D note: the resolver must ALSO dispatch on the query head,
    // because the two ceiling axes are not interchangeable (language
    // reference §3.7) — `nodes`/`edges`/`neighbors`/`hyperedges`/
    // `hyperedges-of` bound against `ceilings.get(..)`, while `members-of`
    // bounds against `ceilings.max_members(..)`. Using the wrong axis
    // silently mis-bounds every membership fold.
    String::new()
}

fn is_intrinsic_call(op: &str) -> bool {
    op.starts_with("intrinsic:")
}
fn is_comparison(op: &str) -> bool {
    matches!(op, "=" | "<" | ">" | "<=" | ">=" | "!=")
}
fn is_boolean(op: &str) -> bool {
    matches!(op, "and" | "or" | "not")
}

/// Reject at load time if the computed bound exceeds the declared budget
/// (III.11).
pub fn check_bound(
    expr: &SExpr,
    ceilings: &CardinalityCeilings,
    declared_budget: u64,
) -> Result<(), BoundExceeded> {
    let computed = static_fuel_bound(expr, ceilings);
    if computed > declared_budget {
        return Err(BoundExceeded { computed_bound: computed, declared_budget });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;
    use std::collections::HashMap;

    #[test]
    fn a_rule_within_budget_is_accepted() {
        let (expr, _) = read("(+ 1 2)").unwrap();
        let ceilings = CardinalityCeilings::new(HashMap::new(), HashMap::new());
        assert!(check_bound(&expr, &ceilings, 100).is_ok());
    }

    #[test]
    fn a_rule_over_budget_is_rejected_at_load_not_at_eval() {
        let (expr, _) = read("(+ 1 2)").unwrap();
        let ceilings = CardinalityCeilings::new(HashMap::new(), HashMap::new());
        let result = check_bound(&expr, &ceilings, 0);
        assert!(result.is_err());
    }
}
```
The `fold_target_type` placeholder is a **known gap this task must close before
finishing** — it depends on Task 2's grammar pinning the exact fold syntax shape;
resolve it as part of this task's implementation, not left as a TODO in the merged
PR.

- [ ] **Step 3: Run red, resolve the fold-target-type gap, implement, run green**

```bash
cd rust && cargo test -p babylon-bsl bound_checker --locked && cd ..
```

- [ ] **Step 4: Gate + commit + PR** (slug `bsl-bound-checker`).

---

### Task 14: The fuel evaluator (runtime backstop)

**Files:**
- Create: `rust/crates/babylon-bsl/src/evaluator.rs`
- Create: `rust/crates/babylon-bsl/src/intrinsic_host.rs` (the `IntrinsicHost` trait
  extension point — numeric intrinsics themselves are Phase 2's `babylon-domain`)

**Interfaces:**
- Produces: `pub fn evaluate(expr: &SExpr, env: &EvalEnv, host: &dyn IntrinsicHost,
  fuel: &mut u64) -> Result<Value, EvalError>` — strict left-to-right evaluation,
  IEEE-754 basic ops + fixed-point integer arithmetic only (no transcendentals as BSL
  primitives — every named transcendental crosses through `IntrinsicHost`, which Phase
  1 defines as a trait with zero real implementations; Phase 2 populates it once
  Task 8's sigmoid ruling lands).

- [ ] **Step 1: Branch; define `Value` and the `IntrinsicHost` seam first**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-fuel-evaluator
```

```rust
// rust/crates/babylon-bsl/src/intrinsic_host.rs
//! The named-intrinsic call boundary (spec §5 Determinism: "Transcendentals
//! ... are never BSL primitives — they are named kernel intrinsics"). Phase
//! 1 defines the trait only; `babylon-domain`'s intrinsic table (Phase 2)
//! is the first real implementation, gated on Task 8's sigmoid ruling.
use crate::evaluator::{EvalError, Value};

pub trait IntrinsicHost {
    /// Dispatch a named intrinsic call by name + positional args. An
    /// unknown intrinsic name is a load-time error (the typechecker, Task
    /// 10, should have already rejected it against the registered table —
    /// this is the evaluator's defense-in-depth check, not the primary
    /// gate).
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, EvalError>;
}

/// A host with no registered intrinsics at all — every call fails loud.
/// Used by Phase-1 tests that only exercise arithmetic/comparison/fold
/// forms, which never call an intrinsic.
pub struct EmptyIntrinsicHost;

impl IntrinsicHost for EmptyIntrinsicHost {
    fn call(&self, name: &str, _args: &[Value]) -> Result<Value, EvalError> {
        Err(EvalError { message: format!("no intrinsic registered: {name}") })
    }
}
```

```rust
// rust/crates/babylon-bsl/src/evaluator.rs
//! The fuel-metered BSL evaluator (spec §5 Determinism/Totality): strict
//! left-to-right, IEEE-754 basic ops + fixed-point integer only, fuel
//! decremented per evaluation step as the RUNTIME backstop to Task 13's
//! static bound (spec: "the runtime fuel meter remains as the III.11
//! backstop" — this module exists even though the static check should
//! already have rejected any rule that could exhaust it, because the
//! static bound's own soundness is exactly what M1/M2's adversarial
//! finding worried about).
use crate::intrinsic_host::IntrinsicHost;
use crate::reader::SExpr;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
    pub message: String,
}

pub struct EvalEnv {
    pub bindings: HashMap<String, Value>,
}

/// Evaluate `expr`, decrementing `*fuel` by each step's cost (Task 13's
/// `fuel::cost` table). Fuel exhaustion is a loud runtime error — it should
/// be unreachable if Task 13's static bound was computed correctly for
/// this rule against its declared budget, and reaching it anyway is itself
/// diagnostic information (a static-bound soundness bug), not merely "the
/// rule was too expensive."
pub fn evaluate(
    expr: &SExpr,
    env: &EvalEnv,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, crate::fuel::cost::VARIABLE_REF)?; // minimum charge per node visited
    match expr {
        SExpr::Atom(a) => {
            if let Ok(n) = a.parse::<f64>() {
                return Ok(Value::Number(n));
            }
            env.bindings
                .get(a)
                .cloned()
                .ok_or_else(|| EvalError { message: format!("unbound variable: {a} (load-time error, spec §5 Bindings)") })
        }
        SExpr::List(items) => evaluate_list(items, env, host, fuel),
    }
}

fn evaluate_list(
    items: &[SExpr],
    env: &EvalEnv,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    let Some(SExpr::Atom(op)) = items.first() else {
        return Err(EvalError { message: "empty or malformed form".into() });
    };
    // Strict left-to-right: every arg is evaluated in source order before
    // the operator applies, no short-circuiting except `and`/`or` (which
    // the spec's grammar explicitly permits as guards — implement their
    // short-circuit explicitly, don't accidentally get it from Rust's `&&`).
    match op.as_str() {
        "+" | "-" | "*" | "/" => {
            charge(fuel, crate::fuel::cost::ARITHMETIC_OP)?;
            let mut args = Vec::with_capacity(items.len() - 1);
            for item in &items[1..] {
                args.push(as_number(evaluate(item, env, host, fuel)?)?);
            }
            arithmetic(op, &args)
        }
        "=" | "<" | ">" | "<=" | ">=" | "!=" => {
            charge(fuel, crate::fuel::cost::COMPARISON)?;
            let a = as_number(evaluate(&items[1], env, host, fuel)?)?;
            let b = as_number(evaluate(&items[2], env, host, fuel)?)?;
            Ok(Value::Bool(compare(op, a, b)))
        }
        "and" => {
            charge(fuel, crate::fuel::cost::BOOLEAN_OP)?;
            for item in &items[1..] {
                if !as_bool(evaluate(item, env, host, fuel)?)? {
                    return Ok(Value::Bool(false)); // deliberate short-circuit
                }
            }
            Ok(Value::Bool(true))
        }
        name if name.starts_with("intrinsic:") => {
            charge(fuel, crate::fuel::cost::INTRINSIC_CALL_BASE)?;
            let mut args = Vec::with_capacity(items.len() - 1);
            for item in &items[1..] {
                args.push(evaluate(item, env, host, fuel)?);
            }
            host.call(&name["intrinsic:".len()..], &args)
        }
        other => Err(EvalError { message: format!("unknown form: {other}") }),
    }
}

fn charge(fuel: &mut u64, amount: u64) -> Result<(), EvalError> {
    *fuel = fuel
        .checked_sub(amount)
        .ok_or_else(|| EvalError { message: "fuel exhausted at runtime — static bound was unsound, escalate".into() })?;
    Ok(())
}

fn as_number(v: Value) -> Result<f64, EvalError> {
    match v {
        Value::Number(n) => Ok(n),
        Value::Bool(_) => Err(EvalError { message: "expected Number, got Bool".into() }),
    }
}
fn as_bool(v: Value) -> Result<bool, EvalError> {
    match v {
        Value::Bool(b) => Ok(b),
        Value::Number(_) => Err(EvalError { message: "expected Bool, got Number".into() }),
    }
}

fn arithmetic(op: &str, args: &[f64]) -> Result<Value, EvalError> {
    let result = match op {
        "+" => args.iter().sum(),
        "-" if args.len() == 1 => -args[0],
        "-" => args[1..].iter().fold(args[0], |acc, x| acc - x),
        "*" => args.iter().product(),
        "/" => args[1..].iter().fold(args[0], |acc, x| acc / x),
        _ => unreachable!("guarded by caller match arm"),
    };
    Ok(Value::Number(result))
}

fn compare(op: &str, a: f64, b: f64) -> bool {
    match op {
        "=" => (a - b).abs() < f64::EPSILON,
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        ">=" => a >= b,
        "!=" => (a - b).abs() >= f64::EPSILON,
        _ => unreachable!("guarded by caller match arm"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intrinsic_host::EmptyIntrinsicHost;
    use crate::reader::read;

    fn empty_env() -> EvalEnv {
        EvalEnv { bindings: HashMap::new() }
    }

    #[test]
    fn evaluates_simple_arithmetic() {
        let (expr, _) = read("(+ 1 2 3)").unwrap();
        let mut fuel = 100;
        let result = evaluate(&expr, &empty_env(), &EmptyIntrinsicHost, &mut fuel).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn unbound_variable_is_a_load_time_shaped_error_not_a_default() {
        let (expr, _) = read("(+ 1 undeclared_var)").unwrap();
        let mut fuel = 100;
        let result = evaluate(&expr, &empty_env(), &EmptyIntrinsicHost, &mut fuel);
        assert!(result.is_err());
    }

    #[test]
    fn fuel_exhaustion_is_loud() {
        let (expr, _) = read("(+ 1 2 3)").unwrap();
        let mut fuel = 1; // deliberately too little
        let result = evaluate(&expr, &empty_env(), &EmptyIntrinsicHost, &mut fuel);
        assert!(result.is_err());
    }

    #[test]
    fn and_short_circuits_left_to_right() {
        let (expr, _) = read("(and (< 1 0) undeclared_var)").unwrap();
        // If `and` evaluated the second arg it would error on the unbound
        // variable; short-circuiting on the first false means it returns
        // Bool(false) cleanly instead.
        let mut fuel = 100;
        let result = evaluate(&expr, &empty_env(), &EmptyIntrinsicHost, &mut fuel).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
```

- [ ] **Step 2: Run red, implement, run green**

```bash
cd rust && cargo test -p babylon-bsl evaluator --locked && cd ..
```

- [ ] **Step 3: Gate + commit + PR** (slug `bsl-fuel-evaluator`).

---

### Task 15: Bindings + `:material-basis` — the load-time error surface (M3, "Bindings not honest-null")

**Files:**
- Modify: `rust/crates/babylon-bsl/src/typecheck.rs` (binding declarations,
  `:optional`/`:default`)
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs` (unbound-declared-binding is
  load-time, not eval-time, once the typechecker owns it — this task moves the
  "unbound variable" case from Task 14's evaluator-level error to a typecheck-level
  rejection for any binding that is NOT declared `:optional`)
- Create: `rust/crates/babylon-bsl/src/material_basis.rs`
- Create: `rust/crates/babylon-bsl/src/default_lint.rs` (the migration-corpus
  `:default` allowlist)

**Interfaces:**
- Produces: `pub fn check_bindings(expr: &SExpr, declared: &[BindingDecl]) ->
  Result<(), Vec<BindingError>>` (unbound non-optional binding = load-time error);
  `pub fn check_material_basis(expr: &SExpr) -> Result<(), MaterialBasisError>`
  (presence + non-emptiness only, per M3's scoping); `pub const
  DEFAULT_ALLOWLIST: &[&str]` (the migration corpus's permitted `:default 0` sites,
  starts empty — Task 17 populates it).

- [ ] **Step 1: Branch; write the failing tests for each of the three rules**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-bindings-material-basis
```

```rust
// rust/crates/babylon-bsl/src/material_basis.rs
//! `:material-basis` presence-at-parse-time check (spec §5, adversarial
//! finding M3 — scoped honestly: the parser enforces presence and
//! non-emptiness ONLY; the semantic III.8 obligation — does the named
//! material process actually ground the construct? — stays with Director
//! review, never automated here).
use crate::reader::SExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBasisError {
    pub message: String,
}

/// A rule form (a top-level `SExpr::List`) must contain a
/// `(:material-basis "<non-empty string>")` pair among its top-level
/// elements.
pub fn check_material_basis(rule: &SExpr) -> Result<(), MaterialBasisError> {
    let SExpr::List(items) = rule else {
        return Err(MaterialBasisError { message: "rule must be a list form".into() });
    };
    for pair in items.windows(2) {
        if let [SExpr::Atom(key), SExpr::Atom(value)] = pair {
            if key == ":material-basis" {
                let text = value.trim_matches('"');
                if text.is_empty() {
                    return Err(MaterialBasisError { message: "material-basis string is empty".into() });
                }
                return Ok(());
            }
        }
    }
    Err(MaterialBasisError { message: "rule is missing a mandatory :material-basis field".into() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;

    #[test]
    fn rejects_a_missing_material_basis() {
        let (rule, _) = read("(rule (effect (+ 1 1)))").unwrap();
        assert!(check_material_basis(&rule).is_err());
    }

    #[test]
    fn rejects_an_empty_material_basis() {
        let (rule, _) = read("(rule (:material-basis \"\") (effect (+ 1 1)))").unwrap();
        assert!(check_material_basis(&rule).is_err());
    }

    #[test]
    fn accepts_a_non_empty_material_basis() {
        let (rule, _) = read("(rule (:material-basis \"wage relation\") (effect (+ 1 1)))").unwrap();
        assert!(check_material_basis(&rule).is_ok());
    }
}
```

- [ ] **Step 2: Bindings — declared/`:optional`/`:default`, unbound = load-time error**

```rust
// (append to rust/crates/babylon-bsl/src/typecheck.rs)

/// A rule's declared binding: the variable name it reads, and whether it
/// is required or `:optional` with a literal default (spec §5 "Bindings,
/// not honest-null" — a plain declared binding that is unbound is a
/// LOAD-TIME error; only an `:optional` binding may be absent at
/// evaluation).
#[derive(Debug, Clone, PartialEq)]
pub struct BindingDecl {
    pub name: String,
    pub default: Option<f64>, // Some(_) iff declared :optional with a literal default
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingError {
    pub name: String,
    pub message: String,
}

/// Check that every variable `expr` reads is either bound in `declared`
/// (required or optional) — a variable referenced but not declared at all
/// is ALSO a load-time error (spec: "A rule declares the variables it
/// reads"), distinct from "declared but required-and-unbound-at-eval",
/// which Task 14's evaluator still guards as defense-in-depth.
pub fn check_bindings(expr: &SExpr, declared: &[BindingDecl]) -> Result<(), Vec<BindingError>> {
    let mut errors = Vec::new();
    collect_free_variables(expr, declared, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_free_variables(expr: &SExpr, declared: &[BindingDecl], errors: &mut Vec<BindingError>) {
    match expr {
        SExpr::Atom(a) if a.parse::<f64>().is_err() && !a.starts_with(':') => {
            if !declared.iter().any(|d| d.name == *a) {
                errors.push(BindingError {
                    name: a.clone(),
                    message: format!("undeclared binding: {a} (spec §5 Bindings — declare it, or mark :optional)"),
                });
            }
        }
        SExpr::List(items) => {
            for item in items {
                collect_free_variables(item, declared, errors);
            }
        }
        SExpr::Atom(_) => {}
    }
}

#[cfg(test)]
mod binding_tests {
    use super::*;
    use crate::reader::read;

    #[test]
    fn undeclared_binding_is_a_load_time_error() {
        let (expr, _) = read("(+ x 1)").unwrap();
        assert!(check_bindings(&expr, &[]).is_err());
    }

    #[test]
    fn declared_binding_passes() {
        let (expr, _) = read("(+ x 1)").unwrap();
        let declared = vec![BindingDecl { name: "x".into(), default: None }];
        assert!(check_bindings(&expr, &declared).is_ok());
    }

    #[test]
    fn optional_binding_with_default_passes_declaration_check() {
        let (expr, _) = read("(+ x 1)").unwrap();
        let declared = vec![BindingDecl { name: "x".into(), default: Some(0.0) }];
        assert!(check_bindings(&expr, &declared).is_ok());
    }
}
```

- [ ] **Step 3: The `:default` migration-corpus allowlist lint**

```rust
// rust/crates/babylon-bsl/src/default_lint.rs
//! Lint forbidding new `:default` declarations outside the migration
//! corpus's enumerated, Director-approved allowlist (spec §5: "a lint
//! forbids new `:default` declarations outside that set without Director
//! sign-off"). Starts EMPTY — Task 17 (conformance corpus transcription)
//! is the only task expected to populate it, one row per trap-DSL
//! pinned-absent-reads-as-0 site it transcribes.

/// Each entry names the exact rule (by its transcribed source file +
/// binding name) permitted a `:default`, plus who approved it and why —
/// same governance bar as `EXTENSIVE_INTENSIVE_EXEMPTIONS` (Task 10) and
/// Python's `SentinelExemption`.
#[derive(Debug, Clone)]
pub struct DefaultAllowlistEntry {
    pub rule_file: &'static str,
    pub binding_name: &'static str,
    pub reason: &'static str,
    pub owner: &'static str,
}

pub const DEFAULT_ALLOWLIST: &[DefaultAllowlistEntry] = &[];

/// Check whether `(rule_file, binding_name)` is on the allowlist — callers
/// (the content-loading pipeline, Phase 2/3) reject any `:default`
/// declaration not found here.
#[must_use]
pub fn is_allowed(rule_file: &str, binding_name: &str) -> bool {
    DEFAULT_ALLOWLIST
        .iter()
        .any(|e| e.rule_file == rule_file && e.binding_name == binding_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_undeclared_default_is_rejected() {
        assert!(!is_allowed("some/rule.bsl", "unlisted_binding"));
    }

    #[test]
    fn every_allowlist_row_carries_a_reason_and_owner() {
        for entry in DEFAULT_ALLOWLIST {
            assert!(!entry.reason.is_empty());
            assert!(!entry.owner.is_empty());
        }
    }
}
```

- [ ] **Step 4: Run all three red-then-green; gate + commit + PR** (slug
  `bsl-bindings-material-basis`).

---

### Task 16: Typed structural verb algebra + modding ordering anchors

**Files:**
- Modify: `rust/crates/babylon-bsl/Cargo.toml` (add `babylon-graph` dependency — the
  crate-DAG edge this task deliberately adds, per Task 11's note)
- Create: `rust/crates/babylon-bsl/src/structural_verbs.rs`
- Create: `rust/crates/babylon-bsl/src/mod_anchors.rs`

**Interfaces:**
- Produces the **seven** typed verbs of `docs/reference/bsl-language.rst` §2.8 as
  revised by the Amendment D ruling — the five dyadic ones `(add-node <type> <id>)`,
  `(remove-node <ref>)`, `(add-edge <type> <from> <to> :strength <e>)`, `(remove-edge
  <type> <from> <to>)`, `(update-node <ref> <attr> <value>)`, plus the two hyperedge
  ones `(add-hyperedge <type> <id> (members <ref>+) <field-init>*)` and
  `(remove-hyperedge <ref>)` — all typechecking against
  `babylon_graph::substrate::GraphSubstrate` and executing by calling it (against
  `PlaceholderGraph` in Phase 1 tests; the production store swaps in at the Phase 1/2
  boundary). There is deliberately **no** `add-member`/`remove-member`/`update-hyperedge`
  verb: membership change is whole-hyperedge replacement (`remove-hyperedge` then
  `add-hyperedge` in one effect list), per the §2.8 draft ruling. Also produces
  `(anchor :before <system-name>)` / `(anchor :after <system-name>)` declarations that
  typecheck as content but whose RESOLUTION into a total order is explicitly deferred
  to `babylon-engine`'s anchor-based registry (Phase 3 — this task only validates the
  declaration shape, it does not resolve an order).

- [ ] **Step 1: Branch; add the dependency, write the failing structural-verb test**

```bash
git checkout dev && git pull && git checkout -b feature/p27-bsl-structural-verbs
```

```toml
# rust/crates/babylon-bsl/Cargo.toml — add
[dependencies]
babylon-graph = { path = "../babylon-graph" }
```

```rust
// rust/crates/babylon-bsl/src/structural_verbs.rs
//! The typed structural verb algebra (spec §5 Expressible: "effects ...
//! plus typed structural verbs: add/remove node/edge and update-node under
//! the I.15 edge-mode state machine", plus the two hyperedge verbs Amendment
//! D's NATIVE HYPEREDGE ruling adds — language reference §2.8). Executes
//! against any [`babylon_graph::substrate::GraphSubstrate`] — in Phase 1 that
//! means `babylon_graph::placeholder::PlaceholderGraph` only; the production
//! store is Phase 2 (Task 11).
//!
//! **No clique expansion exists in this module** and none may be added: a
//! member list is handed to `GraphSubstrate::add_hyperedge` whole. That is
//! Anti-Pattern VIII.9 enforced where the verbs live.
//!
//! **I.15 note:** this module calls `GraphSubstrate::update_node`
//! mechanically; it does NOT itself enforce the I.15 edge-mode state
//! machine's transition law (EXTRACTIVE cannot jump to SOLIDARISTIC) — that
//! check belongs to `babylon-domain` once the concrete edge-mode model
//! exists (Phase 2), same layering reason `babylon-bsl` doesn't know about
//! `EdgeType` as a closed enum yet.
use babylon_graph::substrate::{GraphError, GraphSubstrate, HyperedgeId, NodeId};
use crate::reader::SExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct VerbError {
    pub message: String,
}

impl From<GraphError> for VerbError {
    fn from(e: GraphError) -> Self {
        Self { message: e.message }
    }
}

/// Execute one structural-verb form against `graph`. Strict left-to-right
/// evaluation of arguments, same discipline as `evaluator::evaluate`
/// (Task 14) — structural verbs are effects, not a separate evaluation
/// order.
pub fn execute_structural_verb(expr: &SExpr, graph: &mut dyn GraphSubstrate) -> Result<(), VerbError> {
    let SExpr::List(items) = expr else {
        return Err(VerbError { message: "structural verb must be a list form".into() });
    };
    let Some(SExpr::Atom(op)) = items.first() else {
        return Err(VerbError { message: "empty structural verb form".into() });
    };
    match op.as_str() {
        "add-node" => {
            let node_type = atom_str(&items[1])?;
            graph.add_node(leak_str(node_type))?;
            Ok(())
        }
        "remove-node" => {
            let id = node_id_from_atom(&items[1])?;
            graph.remove_node(id)?;
            Ok(())
        }
        "add-edge" => {
            let edge_type = atom_str(&items[1])?;
            let from = node_id_from_atom(&items[2])?;
            let to = node_id_from_atom(&items[3])?;
            graph.add_edge(leak_str(edge_type), from, to)?;
            Ok(())
        }
        "update-node" => {
            let id = node_id_from_atom(&items[1])?;
            let attr = atom_str(&items[2])?;
            let value: f64 = atom_str(&items[3])?
                .parse()
                .map_err(|_| VerbError { message: "update-node value must be numeric".into() })?;
            graph.update_node(id, leak_str(attr), value)?;
            Ok(())
        }
        // ---- Amendment D: the two hyperedge verbs ----
        // NOTE: like the `add-node` arm above, this sketch elides the
        // grammar's explicit-id and <field-init>* operands (§2.8) — they land
        // with the reader's typed AST, not with this shape-level dispatcher.
        "add-hyperedge" => {
            let hyperedge_type = atom_str(&items[1])?;
            let members = members_from_form(&items[2])?;
            graph.add_hyperedge(leak_str(hyperedge_type), &members)?;
            Ok(())
        }
        "remove-hyperedge" => {
            let id = hyperedge_id_from_atom(&items[1])?;
            graph.remove_hyperedge(id)?;
            Ok(())
        }
        other => Err(VerbError { message: format!("unknown structural verb: {other}") }),
    }
}

/// Parse the `(members <ref>+)` sub-form. The member list is passed to the
/// substrate WHOLE — there is no path here that turns it into pairwise edges
/// (VIII.9), and no path that reorders it meaningfully either: the substrate
/// sorts, because declared member order is unobservable (BSL D25).
fn members_from_form(expr: &SExpr) -> Result<Vec<NodeId>, VerbError> {
    let SExpr::List(items) = expr else {
        return Err(VerbError { message: "expected a (members ...) form".into() });
    };
    let Some(SExpr::Atom(head)) = items.first() else {
        return Err(VerbError { message: "empty members form".into() });
    };
    if head != "members" {
        return Err(VerbError { message: format!("expected members form, got {head}") });
    }
    if items.len() < 2 {
        return Err(VerbError { message: "a hyperedge needs at least one member".into() });
    }
    items[1..].iter().map(node_id_from_atom).collect()
}

fn atom_str(expr: &SExpr) -> Result<&str, VerbError> {
    match expr {
        SExpr::Atom(a) => Ok(a.as_str()),
        SExpr::List(_) => Err(VerbError { message: "expected atom, got list".into() }),
    }
}

fn node_id_from_atom(expr: &SExpr) -> Result<NodeId, VerbError> {
    let s = atom_str(expr)?;
    s.parse::<u64>()
        .map(NodeId)
        .map_err(|_| VerbError { message: format!("expected a NodeId, got: {s}") })
}

fn hyperedge_id_from_atom(expr: &SExpr) -> Result<HyperedgeId, VerbError> {
    let s = atom_str(expr)?;
    s.parse::<u64>()
        .map(HyperedgeId)
        .map_err(|_| VerbError { message: format!("expected a HyperedgeId, got: {s}") })
}

/// NOTE (implementer TODO, resolve before this task closes): leaking a
/// `String` to `&'static str` per call is a memory-growth smell the trait's
/// `&'static str` node/edge-type signature (Task 11) forces on any caller
/// that only has a runtime string. The real fix is either (a) changing
/// `GraphSubstrate`'s signature to take `&str` with a non-`'static` bound
/// once Task 11's placeholder gets revisited, or (b) interning type names
/// once at content-load time into a `&'static` table (mirroring how the
/// closed `NodeType`/`EdgeType` enums will work in Phase 2 anyway, which
/// ARE `&'static` by construction). Do not ship `Box::leak` as the Phase-2
/// answer — it is acceptable ONLY as a Phase-1 placeholder scoped to tests
/// against `PlaceholderGraph`, and this comment is the marker to fix it
/// when the real enum lands.
fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;
    use babylon_graph::placeholder::PlaceholderGraph;

    #[test]
    fn add_node_then_update_node_round_trips() {
        let mut graph = PlaceholderGraph::new();
        let (add, _) = read("(add-node social_class)").unwrap();
        execute_structural_verb(&add, &mut graph).unwrap();
        let (update, _) = read("(update-node 0 wealth 42.0)").unwrap();
        execute_structural_verb(&update, &mut graph).unwrap();
    }

    #[test]
    fn update_node_on_nonexistent_id_is_a_loud_error() {
        let mut graph = PlaceholderGraph::new();
        let (update, _) = read("(update-node 999 wealth 1.0)").unwrap();
        assert!(execute_structural_verb(&update, &mut graph).is_err());
    }

    #[test]
    fn add_hyperedge_takes_the_member_list_whole() {
        let mut graph = PlaceholderGraph::new();
        for _ in 0..3 {
            let (add, _) = read("(add-node social_class)").unwrap();
            execute_structural_verb(&add, &mut graph).unwrap();
        }
        let (add_h, _) = read("(add-hyperedge economic_sector (members 2 0 1))").unwrap();
        execute_structural_verb(&add_h, &mut graph).unwrap();
        assert_eq!(
            graph.members_of(HyperedgeId(0)).unwrap(),
            vec![NodeId(0), NodeId(1), NodeId(2)] // sorted, not as declared
        );
    }

    #[test]
    fn a_zero_member_hyperedge_is_a_loud_error() {
        let mut graph = PlaceholderGraph::new();
        let (add_h, _) = read("(add-hyperedge economic_sector (members))").unwrap();
        assert!(execute_structural_verb(&add_h, &mut graph).is_err());
    }

    #[test]
    fn membership_change_is_remove_then_add() {
        // §2.8 draft ruling: no add-member/remove-member verb exists.
        let mut graph = PlaceholderGraph::new();
        for _ in 0..2 {
            let (add, _) = read("(add-node social_class)").unwrap();
            execute_structural_verb(&add, &mut graph).unwrap();
        }
        let (add_h, _) = read("(add-hyperedge economic_sector (members 0))").unwrap();
        execute_structural_verb(&add_h, &mut graph).unwrap();
        let (rm, _) = read("(remove-hyperedge 0)").unwrap();
        execute_structural_verb(&rm, &mut graph).unwrap();
        let (re_add, _) = read("(add-hyperedge economic_sector (members 0 1))").unwrap();
        execute_structural_verb(&re_add, &mut graph).unwrap();
        assert_eq!(graph.members_of(HyperedgeId(1)).unwrap().len(), 2);
        let (bad, _) = read("(add-member 1 1)").unwrap();
        assert!(execute_structural_verb(&bad, &mut graph).is_err()); // no such verb
    }
}
```

- [ ] **Step 2: Run red, implement, run green, resolve the `leak_str` TODO before
  closing** (either narrow the trait's lifetime bound in Task 11's crate — a small,
  isolated follow-up PR to `babylon-graph`, in-scope for this task since it's a direct
  consequence of exercising the trait for real — or interning; pick one, document the
  choice in the crate's doc comment, do not leave the leak as the final state).

```bash
cd rust && cargo test -p babylon-bsl structural_verbs --locked && cd ..
```

- [ ] **Step 3: Mod ordering anchors — declaration shape only**

```rust
// rust/crates/babylon-bsl/src/mod_anchors.rs
//! Mod ordering anchors (spec §5 Modding boundary: "Mods declare ordering
//! anchors (before/after a named system), never raw position floats; the
//! resolved total order goes inside the content hash"). This module
//! validates the DECLARATION shape only — `(anchor :before <name>)` /
//! `(anchor :after <name>)` — and does not resolve a total order; that is
//! `babylon-engine`'s anchor-based phase-indexed registry (Phase 3). A rule
//! whose anchor would interleave into the Material Base partition is
//! rejected at load, but which partition a named system belongs to is
//! itself `babylon-engine` state Phase 1 has no access to — so this
//! module's `check_anchor_declaration` validates SHAPE only (exactly one of
//! `:before`/`:after`, a non-empty system-name atom) and returns the parsed
//! anchor for the Phase-3 registry to resolve and partition-check.
use crate::reader::SExpr;

#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    Before(String),
    After(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnchorError {
    pub message: String,
}

pub fn check_anchor_declaration(expr: &SExpr) -> Result<Anchor, AnchorError> {
    let SExpr::List(items) = expr else {
        return Err(AnchorError { message: "anchor must be a list form".into() });
    };
    let [SExpr::Atom(head), SExpr::Atom(rel), SExpr::Atom(name)] = items.as_slice() else {
        return Err(AnchorError { message: "anchor must be (anchor :before|:after <name>)".into() });
    };
    if head != "anchor" {
        return Err(AnchorError { message: "not an anchor form".into() });
    }
    if name.is_empty() {
        return Err(AnchorError { message: "anchor target system name must not be empty".into() });
    }
    match rel.as_str() {
        ":before" => Ok(Anchor::Before(name.clone())),
        ":after" => Ok(Anchor::After(name.clone())),
        other => Err(AnchorError { message: format!("expected :before or :after, got {other}") }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;

    #[test]
    fn parses_a_before_anchor() {
        let (expr, _) = read("(anchor :before DoctrineSystem)").unwrap();
        assert_eq!(check_anchor_declaration(&expr).unwrap(), Anchor::Before("DoctrineSystem".into()));
    }

    #[test]
    fn rejects_a_raw_position_float_masquerading_as_an_anchor() {
        let (expr, _) = read("(anchor 14.7 DoctrineSystem)").unwrap();
        assert!(check_anchor_declaration(&expr).is_err());
    }
}
```

- [ ] **Step 4: Run red, implement, run green; gate + commit + PR** (slug
  `bsl-structural-verbs-and-anchors`). PR body notes explicitly: "anchor RESOLUTION
  into a total order, and the Material-Base-interleaving rejection, are Phase 3
  (babylon-engine) work — this PR validates declaration shape only."

---

### Task 17: Conformance corpus transcription (899 lines, 4 corrections) — the freeze-tag-parity milestone

> **ADR173 transcription rules (Director rulings 2026-07-29, added post-plan):**
> (a) **Outcome-equality assertions are excluded AS A CLASS** — any Python
> assertion of the form `outcome == GameOutcome.<terminal>` transcribes as the
> wiring contract (`!= IN_PROGRESS` + `== detector.recognized_pattern`), never
> as an outcome identity (emergent-endgames ruling; the two estate instances
> were repaired in PR #371). (b) **Survival-family vectors encode the EMERGENT
> formulation** — P(S|A) as the measure of class members whose wealth clears
> subsistence (curve = theorem from within-class wealth dispersion), NOT the
> frozen Python logistic; the frozen reference diverges by design on this
> family, so survival cross-checks compare against the emergent formulation's
> own vectors, not Python replay.

The big lift: transcribe the 271-line doctrine trap-condition corpus
(`tests/unit/domain/doctrine/test_mechanics.py`) and the 628-line event-evaluator
corpus (`tests/unit/engine/test_event_evaluator.py`) into BSL rule content + Rust
conformance tests, preserving every behavior **except** the four documented III.11
corrections (spec §5 "Grammar-superset honesty").

**Files:**
- Create: `rust/crates/babylon-bsl/tests/conformance/` (rule-file fixtures, one per
  transcribed Python test)
- Create: `rust/crates/babylon-bsl/tests/conformance_corpus.rs` (the Rust test harness)
- Modify: `rust/crates/babylon-bsl/src/default_lint.rs` (populate
  `DEFAULT_ALLOWLIST` with the exact trap-DSL pinned-absent-reads-as-0 sites this
  transcription finds)
- Create: `reports/p27-conformance-corpus-transcription.md` (the delta ledger)

**Interfaces:**
- Produces: a green `cargo test -p babylon-bsl --test conformance_corpus` suite that
  is the Rust-side analogue of the 899 Python lines, with the four corrections
  encoded as explicit "this now errors, and here is the Python line it replaces" test
  cases, not silently dropped.

- [ ] **Step 1: Read both Python corpora end-to-end first (F1 discipline — do not
  transcribe from memory or from the spec's one-line summary of each site)**

```bash
git checkout dev && git pull && git checkout -b feature/p27-conformance-corpus
wc -l tests/unit/domain/doctrine/test_mechanics.py tests/unit/engine/test_event_evaluator.py
sed -n '1,271p' tests/unit/domain/doctrine/test_mechanics.py
sed -n '1,628p' tests/unit/engine/test_event_evaluator.py
```

- [ ] **Step 2: Confirm the four correction sites against real line numbers** (the
  spec cites `event_evaluator.py:313/405/439/:103` — verify these still hold in this
  worktree's checkout, since the file may have drifted since spec-writing time):

```bash
sed -n '95,110p;300,320p;395,410p;430,445p' src/babylon/engine/event_evaluator.py
```
For each of the four (unknown graph metric → 0.0; unknown aggregation → False;
unknown comparison operator → False; empty precondition set → True), write down the
exact current line number and the exact current behavior — if a line number has
drifted, use the real one in the delta ledger, not the spec's possibly-stale citation.

- [ ] **Step 3: Transcribe ONE exemplar fully first — the empty-precondition-set
  correction (the simplest of the four, good for calibrating the harness shape)**

```
;; rust/crates/babylon-bsl/tests/conformance/empty_precondition_set.bsl
(rule
  (:material-basis "an event with no stated preconditions has no material trigger")
  (:name "empty_precondition_set_correction")
  (preconditions))
```

```rust
// rust/crates/babylon-bsl/tests/conformance_corpus.rs
//! The transcribed conformance corpus (spec §5, §8.1): 271 doctrine +
//! 628 event-evaluator Python test lines, ported with a documented
//! 4-point delta at exactly the sites spec §5 "Grammar-superset honesty"
//! names. Each correction test below cites the Python line it replaces
//! and the exact old vs. new behavior.
use babylon_bsl::reader::read;
use babylon_bsl::evaluator::{evaluate, EvalEnv};
use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
use std::collections::HashMap;

/// CORRECTION 1 of 4 (spec §5 M8): Python's `event_evaluator.py:103`
/// treats an empty precondition set as `True` (always passes) — silent
/// permissiveness. BSL rejects an empty `(preconditions)` form at
/// load/typecheck time: a rule author must write `(preconditions :always
/// true)` explicitly if that is really the intent, so the empty case can
/// never be an accident.
#[test]
fn empty_precondition_set_is_a_load_time_error_not_an_implicit_true() {
    let source = std::fs::read_to_string("tests/conformance/empty_precondition_set.bsl").unwrap();
    let (_rule, _) = read(&source).unwrap();
    // TODO(implementer): once the rule-loading pipeline (typecheck +
    // bound-check + material-basis check, composed) exists as a single
    // entry point, assert THAT rejects an empty (preconditions) body.
    // This placeholder pins the fixture and the intended assertion shape;
    // wire the real pipeline call before this task closes.
}
```
This exemplar is deliberately left with an explicit TODO marking the missing
composed "load a rule" entry point — **the implementer must build that thin
composition function** (typecheck → material-basis check → bound check, in that
order, matching spec §5/§9's stated rejection ordering) as part of closing this task,
since no earlier task in this plan created it (each earlier task tests its own layer
in isolation by design). This is the first task where the layers compose.

- [ ] **Step 4: Repeat Step 3's cycle for the remaining three corrections** (unknown
  graph metric, unknown aggregation, unknown comparison operator), each as its own
  `.bsl` fixture + test + delta-ledger row, one commit per correction.

- [ ] **Step 5: Transcribe the remaining conformance corpus systematically** — for
  every one of the 271 + 628 Python test lines that is NOT one of the four
  corrections, write the BSL-equivalent fixture + assertion preserving the Python
  behavior exactly. Batch by Python test function (one commit per source test
  function transcribed, `test(bsl-corpus): transcribe <python_test_name>`), not by
  line count — this keeps each commit reviewable and keeps the F3 idiom-mismatch
  review tractable per-function rather than as one giant diff.

- [ ] **Step 6: Populate `DEFAULT_ALLOWLIST`** (Task 15's lint) with the exact trap-DSL
  sites the transcription found using a pinned `:default 0`, per spec §5's "The
  migration corpus enumerates the exact rules permitted to carry `:default 0`."

- [ ] **Step 7: Write the delta ledger report**

`reports/p27-conformance-corpus-transcription.md`: a table with one row per Python
test function (source file:line range, BSL fixture path, pass/fail, and for the four
correction sites specifically: old behavior / new behavior / one-sentence
justification citing spec §5 M8).

- [ ] **Step 8: Full gate + commit + PR**

```bash
cd rust
cargo test -p babylon-bsl --locked
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cd ..
git add rust/crates/babylon-bsl/tests reports/p27-conformance-corpus-transcription.md \
  rust/crates/babylon-bsl/src/default_lint.rs
git commit -m "$(cat <<'EOF'
test(bsl-corpus): conformance corpus transcription complete (P27 §5/§8.1)

271 doctrine + 628 event-evaluator Python test lines transcribed to BSL
fixtures + Rust conformance tests, with the documented 4-point M8 delta
(unknown metric/aggregation/comparison, empty precondition set — all now
load-time or eval-time errors instead of silent degradation).

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
EOF
)"
git log --oneline -1
gh pr create --base dev --title "test(bsl-corpus): conformance corpus transcription (P27 Phase 1)" --body "..."
```

---

### Task 18: Phase-1 exit gate

Runs LAST: blocked on every prior task in this plan merging, and on Task 8's sigmoid
ruling being in hand (it does not need to be Phase-2-implemented yet, only ratified,
so Phase 2 isn't blocked the moment it starts).

**Files:**
- Create: `docs/reference/phase-1-exit-checklist.md`

**Interfaces:**
- Produces: the recorded exit state Phase 2 (Content & Intrinsics) starts from —
  what's green, what's deferred, what Phase 2's first task should read first.

- [ ] **Step 1: Verify every precondition**

```bash
cd rust
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p babylon-kernel --all-targets --locked -- -D warnings -D clippy::pedantic
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cargo clippy -p babylon-graph --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cd ..
gh pr list --state open --label director-gate   # Task 8's sigmoid ruling must be merged
```

- [ ] **Step 2: Write the exit checklist** — `docs/reference/phase-1-exit-checklist.md`
  states, in one table: what's DONE (BSL language spec, kernel scalars/Currency/sim
  clock/RNG/event-bus/ContentDigest, BSL reader/typechecker/bound-checker/evaluator,
  the graph trait boundary, the conformance corpus), what's DEFERRED to Phase 2/3 and
  why (numeric intrinsics — gated on the sigmoid ruling; the concrete graph *storage*
  behind the ruled native-hyperedge trait — Phase 2; the anchor total-order resolver —
  `babylon-engine`; the `EventType`/`NodeType`/`EdgeType`/`HyperedgeType` closed enums
  — `babylon-domain`), and the two
  known Phase-1-internal TODOs this plan flagged but deferred resolution of within
  their own tasks (Task 16's `leak_str` interning question if not fully resolved
  there, Task 17's rule-loading composition function if it needs further
  generalization for Phase 2's content pipeline).

- [ ] **Step 3: Commit + PR** (slug `phase-1-exit-checklist`). **Phase 1 is complete
  when this PR merges** — Phase 2 (Content & Intrinsics) may then begin with no
  outstanding constitutional gate: Amendment D ruled 2026-07-29, so a Phase-2 task
  that picks `babylon-graph`'s concrete storage is an engineering choice under a
  settled shape (native hyperedge; Levi/incidence permitted internally), not a
  Director gate. Flag per-system in the Phase-2 plan which systems need it (most of
  2a/2b do not; 2c's hybrid split likely does for some) — not decided here.

---

## Task dependency order

Tasks 1–2 are prerequisites for everything else (crate shells; the language spec other
tasks cite). Task 3 depends on 1. Tasks 4–7 depend on 3 (kernel scalars) but are
mutually independent — parallel-safe for read-only design work, **serialize the actual
`cargo build`/`test` runs** (machine-safety rule: single-flight cargo). Task 8 (Director
gate) has no code dependency and should start as early as possible in parallel with 3–7
so its ruling is ready before Phase 2 needs it — it blocks nothing in this plan itself
except entering it in the exit checklist (Task 18). Task 9 depends on 1–2. Task 10
depends on 9. Task 11 depends on 1 (kernel) only, independent of 9/10 — it can run in
parallel with the reader/typechecker work; its trait surface is now ruled (native
hyperedge) and its concrete storage is Phase 2, so neither blocks Phase 1's completion. Task 12 depends on 7 and 9. Task 13 depends on
10 and 11 (fold-target cardinality needs the graph trait's type vocabulary). Task 14
depends on 13. Task 15 depends on 10. Task 16 depends on 11 and 14. Task 17 depends on
everything 9–16 (it is the integration task exercising the whole composed pipeline).
Task 18 is last, gated additionally on Task 8's ruling being merged (not implemented —
that's Phase 2).
