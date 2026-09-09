# Implementation plan — issue #576, the intrinsic-host train

**Status:** ready to execute, with **one charter item STOPPED** (see §0.1).
**Authority:** ADR208 R29/C-04 (train proceeds, no Director ruling needed for the *train*),
`reports/port-estate-survey-2026-08-12.md` §4.4/§4.5 (gating analysis), ADR176 r20/r21,
ADR188 Rows 2/6/7/11, ADR202 R7/R8/R9, `docs/reference/bsl-language.rst` §3.10 + D69,
`docs/reference/determinism-contract.rst` (RNG chapter + float/tolerance policy).
**Author:** planning pass, 2026-08-17. Every claim below was read out of the tree; line
numbers are from `dev` at plan time.

---

## 0. Scope verdict — read this before writing any code

The charter (`gh issue view 576`) names three items: **RNG binding**, **`exp`/`log` dispatch**,
**`sqrt` dispatch**. Two of the three do not survive contact with the ratified record.

### 0.1 `sqrt` — HARD STOP. Do not implement.

The survey calls `sqrt` "a one-line `DECLARABLE_INTRINSICS` amendment"
(`reports/port-estate-survey-2026-08-12.md:137, 241, 265`). **That is stale as to authority.**
`ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml:49-53`, ratified 2026-08-10:

> Row 6  sqrt — ELIMINATED (preferred branch ratified): platform fit re-derives as a measure
> (the share of a class's interest dimensions a platform satisfies); no norm, no rider. The
> fallback rider is NOT taken. A silent switch to squared magnitudes remains forbidden.

Three corroborations:

1. The elimination is **mechanically pinned in-tree**, in the conformance suite itself —
   `rust/crates/babylon-bsl/tests/r9_chapters.rs:2594` asserts `check_intrinsic_cap("sqrt")`
   **fails**, alongside `tanh`/`entropy`/`renormalize`/`abs`/`trunc`. Landing `sqrt` means
   *deleting an assertion a Director ruling put there*.
2. The claimed consumer is **exactly the site Row 6 names**. The survey attributes `sqrt` to
   Allegiance @17.42; the actual call sites are `src/babylon/formulas/politics.py:145`
   (`platform_vector`'s L2 norm) and `:227-228` (`interest_fit`'s cosine norms), reached from
   `allegiance.py:399, 440, 504` via `interest_fit`. That IS "platform fit."
3. ADR188's own consequences make the re-derivation a **port-time design obligation** on the
   consuming pack ("the platform-fit consumer"), per ADR183's repair-at-the-port doctrine.

**Action (Task 0.1, no code):** post a comment on #576 removing `sqrt` from scope, citing the
three items above; open a director-gate issue *only if* the Allegiance port later argues the
measure re-derivation is infeasible. Do **not** add `sqrt` to `DECLARABLE_INTRINSICS`,
`kernel_signature`, or `clippy.toml`'s allow-list. `sqrt` stays in the outside-the-cap roster.

> Note for the record: IEEE-754 *does* specify `sqrt` exactly (correctly rounded), so the
> determinism argument that gates `exp`/`log` would not have gated `sqrt` — the `floor`
> precedent (§0.4) would have applied. The blocker is **doctrinal, not numerical**, and that
> distinction must be stated when the STOP is recorded, so nobody re-litigates it on
> determinism grounds.

### 0.2 RNG — proceeds, but **not** in the shape the survey describes

The survey says the gap is "a `DECLARABLE_INTRINSICS` name plus a `KernelIntrinsicHost::call`
arm" (`survey:130, 260-261`). The name and the arm are necessary but **not sufficient**, and the
signature the language register specifies is **not declarable today**:

- `docs/reference/bsl-language.rst:3482-3500` (D69) fixes the carrier key as
  `(session, tick, domain, stable_key)` where `domain` is "a closed-vocabulary **enum operand**"
  and `stable_key` "derives from the identities of the call's **reference operands**".
- `<intrinsic-decl>`'s `:params` vocabulary is `<intrinsic-type-name> ::= <type-name> | "real"`
  (`bsl-language.rst:1239`), and `parse_type_name`
  (`rust/crates/babylon-bsl/src/declarations.rs:650-686`) admits exactly eight names:
  `int bool currency probability intensity coefficient real enum` — with `enum` **refused
  outright** at that position (`declarations.rs:675-681`) because `<intrinsic-decl>` has no
  `:enum-type` companion slot, and with **no row at all** for a node/edge reference
  (`BslType::NodeSet`/`EdgeSet` exist but are unreachable from `parse_type_name`).
- Therefore D69's literal signature cannot be declared without widening the
  `<intrinsic-decl>` grammar — which changes §5.6 canonical-AST bytes and the `rules_hash`.
  **That widening is out of scope for this train.**

Resolution: land a **context-keyed** signature that preserves every property D69 makes
load-bearing while spelling only `int` in `:params`. Full design in §3; the superseding
D-record is Task 6.2.

### 0.3 `exp`/`log` — proceeds, unblocked, but the gating count is overstated

Mechanically the cheapest item: **both are already in the cap and already have kernel
signatures.**
`declarations.rs:110` — `DECLARABLE_INTRINSICS = ["exp", "log", "floor"]`;
`declarations.rs:837-846` — `kernel_signature("exp" | "log") = ((vec![Real]), Real)`.
The only missing pieces are (a) the `KernelIntrinsicHost::call` arm
(`intrinsic_host.rs:59-70` serves `floor` alone) and (b) the ADR176 r21 libm crossing.

**Honest consumer re-count** (the survey's "5 systems: Survival, Consciousness, Community,
MarketScissors, ImperialRent" predates or omits ADR202):

| Site | Frozen source | Post-ADR202 disposition | Live `exp`/`log` need? |
|---|---|---|---|
| Survival @15.0 P(S\|A) | `formulas/survival_calculus.py:43,90` | ADR188 Row 7 + ADR173 — re-derive as measure | **No** (retired) |
| Consciousness @17.0 Gaussian | `formulas/sustained_exploitation.py:198` | **ADR202 R7 REVERSED Row 7 for this site** — not transcribed | **No** (retired) |
| MarketScissors @17.8 tanh | `formulas/market.py:97-107` | **ADR202 R8** — per-county ensemble, tanh replaced | **No** (retired) |
| Contradiction @18.0 financialization | `engine/systems/contradiction.py:455` `math.exp(clamped)` | **ADR202 R9 UPHELD verbatim** (cap retired) | **`exp` — YES** |
| MarketScissors @17.8 monetary anchor | `domain/economics/monetary/anchor.py:89` `math.log(ratio)` | survey row 105: "`log` (declarable, needs a tolerance derivation)" | **`log` — YES** |
| Community @6.0 Shannon entropy | `formulas/consciousness_routing.py:45,470` (`_LOG3`, `p*log(p)`) | survey row 80: "`math.log` ×2 — **a measure**, not an ADR188 Row-7 stipulated form" | **`log` — YES** |
| ImperialRent | — | **claim unverified**: `rg 'math\.(exp\|log\|sqrt)' src/babylon/engine/systems/phi_distribution.py` returns nothing | **Unknown** |

So: **`log` has two doctrinally-clean, ready consumers** (Community entropy, MarketScissors
anchor); **`exp` has exactly one** (Contradiction @18.0 — itself blocked on D35/D65 edge storage,
survey C-4). Land both anyway — they are one code change and `exp` without `log` is not a
smaller train — but **record the corrected count** so #576's closeout does not claim eight
unblocked systems. The RNG half is what actually unblocks three (Doctrine, Struggle, OODA).

### 0.4 The `floor` precedent this train copies

`floor` entered the cap by ADR188 Row 2 / D97, *separately* from the transcendental cap, with an
explicit written disposition of the libm-golden-vector consequence: "there is no libm crossing
here to pin a golden vector against, so that half of the consequence is declined rather than
owed" (`bsl-language.rst:3385-3400`; mirrored in `intrinsic_host.rs:8-20`). `rng-draw` enters the
same way — by its own authority (ADR188 Row 11 + §3.10's RNG convention + D69), not as a
transcendental-cap widening. `exp`/`log`, being genuine transcendentals, **owe** the full
consequence: libm crossing + per-intrinsic golden vectors + a written tolerance derivation.

---

## 1. Global constraints

Non-negotiable for every task below.

1. **Constitution III.7 / III.11.** Every tick deterministic. No wall-clock, no OS entropy, no
   `uuid4`, no `from_entropy`. A missing precondition **fails loud**; it never degrades to a
   default. (`babylon-kernel/src/rng.rs:66-68` already builds this posture: there is
   deliberately no `from_entropy()` and no tick-global constructor.)
2. **`seed_for` is FROZEN.** `babylon-kernel/src/rng.rs:53-63`'s derivation and its byte-pinned
   conformance vector (`rng.rs:181-200`, mirrored in `determinism-contract.rst:1091-1104`) are
   **not touched by this train**. New keying composes a `stable_key` *string* and calls
   `KernelRng::for_carrier` unchanged. Any diff to `seed_for` or that vector is a red STOP.
3. **No new mathematics.** AE re-opened the formalism surface for BSL as
   *rules-as-content expressing the closed algebra*, minting no new mathematics. This train adds
   a **crossing mechanism** and a **seam**, not a construct. No imposed functional forms
   (ADR172 r5 / ADR173 / the standing 2026-07-29 ruling).
4. **No invented E-codes.** The E-code sequence is contiguous per decade block and is
   maintained *in prose* at `bsl-language.rst:3701-3714`; there is **no doc↔enum sync test**
   (verified: `docs/reference/error-codes.rst` contains zero `E-EVAL` hits). D105
   (`bsl-language.rst:5843-5853`) records the discipline: take **whatever is next free at
   landing time**, never a number baked into a draft. Highest today: `E-EVAL-042`
   (`EvalCode::EnumWriteShapeViolation`, `evaluator.rs:199-218`) ⇒ **expected next free
   `E-EVAL-043`, re-verify before hard-coding.**
5. **TDD, red phase mandatory.** Every task below is written failing-test-first. Use
   `#[test]` in the crate under change; `#[pytest.mark.red_phase]` has no Rust analogue, so a
   red step is a commit whose test fails and whose message says so, immediately followed by the
   green commit. Commit per unit of work via `mise run commit -- "type(scope): msg"`.
6. **Power-of-10 rules.** Explicit typing, no catch-alls, ≤~100 lines per function, all loops
   statically bounded, smallest possible scope. `rust:check` runs `-D warnings -D
   clippy::cognitive_complexity` workspace-wide and `-D clippy::pedantic` on
   `babylon-kernel`/`babylon-bsl`/`babylon-graph` — the pedantic legs will demand
   `#[must_use]`, `# Errors` rustdoc sections, and explicit cast lints on everything new.
7. **RST docstrings on every public item** (`cargo doc -D warnings` is in the gate).
8. **No Python-engine change.** `src/babylon/` is the frozen reference. `qa:regression` and
   `qa:vault-regression-ci` must stay byte-identical *because nothing Python moved* — if either
   drifts, something leaked, STOP.
9. **Machine safety.** Run heavy commands uncapped but **single-flight**. Never fan out pytest
   or `cargo test` across parallel agents. `rust:check` is one cargo target dir with one file
   lock; the mise task is deliberately a sequential run-block.
10. **`rust/Cargo.lock`:** after adding a dependency, refresh with `mise run rust:lock-refresh`
    (builds *without* `--locked`, preserving existing pins), then let `rust:check`'s `--locked`
    legs be the gate. Commit the lock diff in the same PR. Never commit a worktree-modified
    `uv.lock` (unrelated, but the same reflex applies).

---

## 2. The determinism policy — stated explicitly, as required

### 2.1 What is ruled, and by whom

ADR176 ruling 21 (`ai/decisions/ADR176_director_rulings_batch_gds_dispositions.yaml:89-90`),
ratified 2026-07-29:

> (21) P27 Task 8: transcendentals cross via a **PINNED SOFT-FLOAT LIBM crate** with
> **golden vectors per intrinsic**.

Reaffirmed by ADR188's decision paragraph (`ADR188...yaml:31-33`): "the r21 mechanism (pinned
soft-float libm, per-intrinsic golden vectors) governs how the two members cross."

**The mechanism is therefore RULED, not open.** `bsl-language.rst` §4.3's sentence that the
polynomial-vs-libm choice "is an open Phase-1 Director ruling (design §13 item 2) and is
deliberately not decided here" is **STALE** and is repaired by this train (Task 1.5).
The `ai/bsl-architecture-standard.md` OQ-1d row ("OPEN — gates Task 18", line ~1099) closes with
it. What remains as workforce work is exactly two things: **which crate**, and **the written
tolerance derivation**.

> This corrects a suggestion in this train's own tasking note ("e.g. Rust std libm on the
> pinned toolchain = deterministic per-build"). **Rust std is not an option.** `f64::exp` /
> `f64::ln` route to the *platform* libm (glibc vs musl vs Apple's), which is the exact
> non-reproducibility `determinism-contract.rst:53-66` names. Per-build determinism is a weaker
> claim than the ruling requires and would silently make the tick hash platform-dependent.

### 2.2 The chosen policy — headline

> **`exp` and `log` cross via the `libm` crate, version-pinned at `0.2.16` with
> `default-features = false`, promoted to a direct dependency of `babylon-kernel` and wrapped in
> `babylon_kernel::transcendental`. `f64::exp` / `f64::ln` / `f64::log*` / `f64::powf` /
> `f64::tanh` are BANNED at and below the intrinsic seam by a `clippy.toml`
> `disallowed-methods` row. Per-intrinsic golden vectors pin the exact `u64` bit patterns.
> Consequence: the Rust engine's tick hash is byte-identical across OS, libc and CPU
> architecture — a STRONGER claim than Constitution III.12 corollary (b), which continues to
> govern comparisons against the FROZEN PYTHON engine (glibc) and nothing else.**

### 2.3 Why `libm 0.2.16` satisfies "pinned soft-float", verified in the vendored source

Read at `~/.cargo/registry/src/index.crates.io-*/libm-0.2.16/` (already in `rust/Cargo.lock:3297-3299`
as a transitive Bevy/glam/naga dependency — so the source, checksum and license are already
present; it is **not** currently a direct dependency of any Babylon crate, so `cargo tree -p
babylon-bsl` shows nothing today):

- **It is the pure-Rust MUSL libm port.** `#![no_std]`, no C, no platform libm.
- **License `MIT`** (`libm-0.2.16/Cargo.toml:36`) — already on `rust/deny.toml`'s allowlist
  (`deny.toml:45`). No new license exception, no new source (crates.io only).
- **Feature surface** (`libm-0.2.16/Cargo.toml:39-49`):
  `arch = []`, `default = ["arch"]`, `force-soft-floats = []`, `unstable*`.
  `default-features = false` drops `arch`.
- **`log` has NO architecture dispatch at all** — `rg 'select_implementation' src/math/log.rs`
  returns nothing; it is unconditionally the soft-float implementation.
- **`exp`'s only dispatch is unreachable on every target Babylon ships.**
  `src/math/exp.rs:86-90`:
  ```rust
  select_implementation! { name: x87_exp, use_arch_required: x86_no_sse, args: x, }
  ```
  `use_arch_required` deliberately ignores the feature flag
  (`src/math/support/macros.rs:48-50, 74`), but the predicate is `x86_no_sse` — 32-bit x86
  without SSE, i.e. legacy x87. Unreachable on `x86_64` (SSE2 is baseline) and on `aarch64`.
  On both of Babylon's targets, `libm::exp` takes the generic soft-float path.
- Therefore: `libm::exp` and `libm::log` at `default-features = false` are **bit-identical
  across `x86_64` and `aarch64`**, by inspection of the dispatch predicates, and the golden
  vectors turn that inspection into an executable guard.

### 2.4 The tolerance derivation (the artifact ADR176 r21 owes)

Written as a new subsection of `docs/reference/determinism-contract.rst`
("Transcendental crossing — `exp`/`log`"), Task 1.5. Its content:

1. **Within the Rust engine: tolerance is ZERO.** One pinned crate, one pinned version, one
   soft-float code path, arch dispatch proven unreachable. Comparisons are `assert_eq!` on
   `f64::to_bits()`, never `abs(a-b) < eps`. Any drift is a red gate: a `libm` bump, a feature
   flip, or an accidental `f64::exp` all fail the golden vectors.
2. **Against the frozen Python engine: tolerance is the III.12 corollary-(b) regime.**
   CPython's `math.exp`/`math.log` call glibc; glibc and MUSL disagree in the last 1–2 ULPs
   (`determinism-contract.rst:53-66`). Derivation: the crossing error is bounded by
   `2 ulp(result)` per call — glibc documents ≤1 ulp for `exp`/`log`, MUSL's libm targets
   ≤1 ulp, so the pairwise difference is ≤2 ulp; for `f64` that is a **relative** bound of
   `2 × 2⁻⁵²  ≈ 4.44e-16`. Composed through the one live `exp` site
   (`exp(clamp(log(ratio)))` — Contradiction @18.0's financialization index, ADR202 R9), two
   crossings give a relative bound of ~`8.9e-16`, seven orders of magnitude *inside* the
   `qa:regression` checkpoint tolerance of `1e-5`
   (`determinism-contract.rst:493-547`, regime 1). **Therefore no existing gate needs its
   tolerance widened** — a fact worth stating, because the alternative (widening a gate) would
   have been a ceremony.
3. **Standing obligation on port trains:** a dual-implementation oracle that compares a BSL
   pack's output against the frozen Python engine through an `exp`/`log` site **must** use
   regime-1 tolerance, never byte equality. This is the gotcha that will bite the first
   consuming pack; it is stated here so it bites the doc instead.

### 2.5 The RNG's own determinism (already ruled, already landed, unchanged)

`babylon-kernel/src/rng.rs` is complete and pinned: `ChaCha8Rng`, per-carrier counter-based
streams, `seed_for` as the exact byte layout, salt `0x0BA1_AC1A` mirroring
`_SYSTEM_RNG_SEED_SALT`, `next_f64` as "top 53 bits × 2⁻⁵³ — no libm, no rounding-mode
dependence." Zero references to it exist anywhere under `rust/crates/babylon-bsl/`
(verified: `rg -i 'rng|random' src/*.rs` returns nothing). **That absence is the entire gap.**

R8 (`determinism-contract.rst:1110-1133`) is the other half: the Rust streams **diverge from
Python's MT19937 by design**, so stochastic baselines re-bless at cutover under
**ensemble-envelope comparison, never byte replay**. Any port consuming `rng-draw` inherits
that; this train does not need an envelope of its own, because no BSL content calls `rng-draw`
when it lands.

---

## 3. The RNG binding design

### 3.1 Requirements harvested from the frozen estate

Every RNG draw in the eight-system reach is one of two shapes:

| Consumer | Site | Shape | Draws per subject per tick |
|---|---|---|---|
| Struggle @16.0 | `engine/systems/struggle.py:343` | `rng.random() < spark_probability` | 1 per class |
| Doctrine @14.7 | `engine/systems/doctrine.py:537` | `roll = rng.random() if needs_roll else 0.0` | 1 per org, **behind a guard** |
| OODA @14.0 | `ooda/state_ai/decision.py:553` | `combined = … + rng.uniform(0.0, 0.01)` | **N per org** (one per scored candidate) |
| (FascistFaction) | `reactionary.py:264` | `rng.random() < p_defect` | superseded — ADR202 R6 rules a full measure at port |
| (Electoral) | `electoral.py:842` | `rng.random() < 0.5` tiebreak | Slice-4 blocked; same shape |

Consequences:

- **One primitive suffices**: a uniform on `[0, 1)`. `rng.uniform(0, 0.01)` is
  `(mul (rng-draw 0) 0.01)`. `rng.gauss` (`domain/bifurcation/consciousness.py:207,214`) and
  `rng.shuffle` (`resilience.py:230`) are **outside** the eight systems and outside this train.
- **More than one draw per subject is required** (OODA's per-candidate tiebreak), so a
  zero-operand signature is insufficient.
- **Doctrine's `needs_roll` guard is the exact motivating case for D69's keyed design**: with a
  streamed RNG, one org skipping its roll would shift every later org's draw. Cite
  `doctrine.py:527-537` in the conformance vector's comment — it is a real, in-tree instance of
  the failure mode the design prevents, not a hypothetical.

### 3.2 The chosen shape

```
(intrinsic rng-draw :params (int) :returns real :cost 12)
```

- **Name `rng-draw`.** Verified free: not in `RESERVED_FORM_TAGS`
  (`declarations.rs:32-82`, 49 names), not in `PROHIBITED_INTRINSIC_NAMES`
  (`declarations.rs:116`).
- **`:params (int)`** — the **draw slot**: an operand that discriminates independent draws
  inside one `(rule, subject, element-chain, tick)`. Declarable today; needs no type-name
  widening.
- **`:returns real`** — the §3.3 unbounded binary64 intermediate, exactly
  `KernelRng::next_f64()`'s value on `[0, 1)`. Not `probability`: `IntrinsicTypeName::Real`'s
  whole point is the intrinsic-position intermediate
  (`declarations.rs:778-798`), and a store into a probability field runs the store's own range
  check (`E-EVAL-020`) rather than pretending the intrinsic returns a bounded type.
- **`:cost 12`** — content-declared; `fuel.rs` hard-codes no per-intrinsic cost. The charge is
  `INTRINSIC_CALL_BASE(5) + declared + Σargs` (`fuel.rs:29`, `evaluator.rs:1486`,
  `bound_checker.rs:287-289`). 12 is proposed as "more than an arithmetic node, less than a
  fold" — a SHA-256 plus a ChaCha8 block. Record the number as a D-record row so revising it
  is a declared vector re-bless (`fuel.rs:6-10`'s own convention).

### 3.3 The carrier key

```
session      := the host's construction-time SessionId          (never an operand — D69)
tick         := the host's construction-time tick               (never an operand — D69)
domain       := the firing rule's own id string                 (kernel-derived, closed)
stable_key   := framed( subject_content_id
                      , element_content_id … outermost→innermost
                      , slot )
```

then, with **`seed_for` untouched**:

```rust
KernelRng::for_carrier(session, tick, domain, &stable_key).next_f64()
```

— one draw at stream index 0, per key. **The host holds no state.**

`framed(...)` is a new, small, `babylon-bsl`-local composition that mirrors `seed_for`'s own
length-prefix discipline so it is injective by construction: each segment is emitted as
`<decimal-len> ":" <segment>` and segments are joined by `"|"`. Two different chains cannot
render to one string, and the test for that is a conformance row, not a comment.

**Why `domain` = the rule id rather than D69's enum operand.** The enum operand is
undeclarable (§0.2) without a §5.6-CAS-touching grammar widening. The rule id preserves every
property D69 asserts and strengthens one:

| D69 property | Enum operand | Rule id | Verdict |
|---|---|---|---|
| `session`/`tick` never operands | ✓ | ✓ | equal |
| Content cannot mint a new stream | ✓ (amendment-gated member) | ✓✓ — content cannot *name* a stream at all; a new stream requires a new rule, which is already declared, hash-covered content | **stronger** |
| Key is stable across runs | ✓ | ✓ (rule ids are content bytes) | equal |
| Key is independent of insertion history | ✓ | ✓ | equal |
| **A draw is a pure function of its key, not a stream position** | ✓ | ✓ | equal — the load-bearing clause, preserved verbatim |

### 3.4 `stable_key` must be a CONTENT id, not a `NodeId` handle — and this is the one real
### plumbing task

`babylon_graph::substrate::NodeId` is `pub struct NodeId(pub u64)`
(`babylon-graph/src/substrate.rs:33`) — an opaque handle minted by
`add_node(&mut self, node_type: &str) -> Result<NodeId, _>` (`substrate.rs:80`). The substrate
exposes **no stable string identity**: `node_attribute` returns `f64` only, and there is no
`stable_id`/`content_id` accessor anywhere in `substrate.rs`.

Keying on the handle would be **replay-deterministic but insertion-history-dependent** — adding
a node to a scenario shifts every later handle, which is precisely the butterfly the ADR176 r20
rider exists to prevent ("adding a single carrier shifts every later draw that tick … LOD
refinement becomes a butterfly generator", `rng.rs:19-28`) and precisely what D69's
"independent of insertion history" forbids.

**The fix is cheap and needs no substrate or hash change.** `babylon-bsl/src/scenario.rs:336`
**already builds** the content-id map during hydration:

```rust
let mut named: HashMap<String, NodeId> = HashMap::new();
```

It is function-local and discarded. Task 3 retains the inverse on `LoadedScenario` and threads
it through `PreparedRules` to the tick seam. Zero `babylon-graph` change, zero canonical-state
change, zero `graph_content_hash` movement.

> **Escalation note, not a blocker:** a reviewer may prefer the stable identity live *in* the
> substrate. That is a **Program 29 substrate-widening item with a Constitution III.7 hash
> question** (does the canonical state cover a node's content id?) and must not be improvised
> here. The retained-map approach is deliberately the non-hash-touching one, and the D-record
> (Task 6.4) says so.

### 3.5 How context reaches the host — the seam change

`IntrinsicHost::call(&self, name: &str, args: &[Value])` (`intrinsic_host.rs:27-35`) receives
neither context nor `&mut self`. Three options were weighed:

- **(a) Interior mutability** (`RefCell<KernelRng>` on the host). **Rejected.** It models a
  *stream*, which D69 forbids; and `KernelIntrinsicHost` is constructed fresh **inside the
  per-rule loop** at both production sites (`babylon-tick/src/lib.rs:377`,
  `babylon-tick/src/session.rs:85`), so any state would silently reset per rule — a latent
  correctness trap with no compiler help.
- **(b) `&mut dyn IntrinsicHost`.** **Rejected.** `host` is threaded as a plain parameter
  through ~30 signatures across `evaluator.rs` and `structural_verbs.rs`, frequently with two
  sibling `evaluate(...)` calls sharing one borrow (`evaluator.rs:626-627`). Cascading, and it
  buys the wrong thing (mutation).
- **(c) Context on `EvalEnv` + one trait parameter.** **Chosen.** `EvalEnv<'a>`
  (`evaluator.rs:268-299`) is *already* threaded everywhere, *already* rebuilt per subject
  (`tick.rs::bind_subject`), and *already* carries the element stack
  (`pub elements: Vec<(Option<String>, Element)>`) that `it`/`:as` resolve through. The draw
  key's every non-operand component is available exactly where it is needed, and
  `eval_intrinsic` already holds `env`.

Concretely:

```rust
/// The non-operand half of a draw key (D69: session and tick are
/// kernel-supplied and are never operands).
pub struct DrawContext<'a> {
    pub session: &'a SessionId,
    pub tick: u64,
    pub domain: &'a str,            // the firing rule's id
    pub subject: &'a str,           // the subject's CONTENT id
}

// EvalEnv gains:
pub draw_context: Option<&'a DrawContext<'a>>,

// The trait gains one parameter (2 production impls + 2 test doubles):
fn call(&self, name: &str, args: &[Value], ctx: IntrinsicCallCtx<'_>)
    -> Result<Value, EvalError>;
```

`IntrinsicCallCtx` carries `Option<&DrawContext>` plus the resolved element-content-id chain.
`None` (a pure-expression caller — `:expr` binding resolution, the arithmetic conformance
vectors, `EmptyIntrinsicHost` paths) makes `rng-draw` **fail loud**, never return `0.0`. Every
other intrinsic ignores the parameter, so `floor`/`exp`/`log` are untouched by it.

`SessionId` provenance: `TickSession::new` gains a `SessionId` parameter (its `SimClock`
counterpart already pairs session+tick, `babylon-kernel/src/clock.rs:47-98`). `run_once` /
`run_once_into` — one-shot conformance/CLI drivers pinned at tick 1
(`babylon-tick/src/lib.rs:382-385`) — pass a **fixed literal** `SessionId::new("run-once")`.
A campaign's session id must be **deterministic** (III.7: no UUID, no wall-clock); the natural
choice is the `ContentDigest` hex or the scenario id, and picking it is a small recorded
decision (Task 6.5), not an open blocker.

### 3.6 Error classes minted (the sentinel-every-error-class rule)

| Class | Code | Trigger |
|---|---|---|
| `log` of a non-positive argument | **new**, next free (expect `E-EVAL-043` — re-verify) | `(log 0)`, `(log -1)` |
| `exp` overflowing to non-finite | **existing** `EvalCode::NonFinite` = `E-EVAL-014` | `(exp 1e10)` |
| non-`Real`-lane argument to any of the three | **uncoded** `EvalError::plain` | `(exp 5)` — a bare `Int` literal; mirrors `eval_floor`'s precedent (`intrinsic_host.rs:101-112, 120-125`) and the no-coercions rule §3.1 |
| wrong arity | **uncoded** `EvalError::plain` | same precedent — a load-time gate's defense-in-depth |
| `rng-draw` with no `DrawContext` | **uncoded** `EvalError::plain` | a driver that never supplied a session/tick — III.11 loud failure |
| `rng-draw` slot operand not an `Int` | **uncoded** `EvalError::plain` | `:params (int)` is checked by `kernel_signature`; the host is defense-in-depth |
| call to `rng-draw` before its cap row | **existing** `E-LOAD-021` | `bound_checker.rs:279-290` |

**Only ONE new code is minted.** Reusing `E-EVAL-014` for `exp` overflow is deliberate: §4.3's
non-finite law already owns that failure, and minting a second code for it would break the
"no invented codes" convention.

**Mutation validation per class** (the standing rule): for each row above, after the green
commit, hand-mutate the guard (flip the comparison, delete the branch, swap `<=` for `<`) and
confirm at least one test fails. Record the mutation table in the PR body. The `floor` estate
sets the precedent — `intrinsic_host.rs:171-179, 243-252` document exactly this, including a
verifier's finding that a `2_000_000.0` mutated ceiling survived until a real-magnitude
boundary row was added. **Choose boundary values at real magnitude**, not toy ones.

---

## 4. Tasks

Each task: **RED step → GREEN step → gate**. Do not proceed past a red gate.

### Task 0 — Governance & scope closure (no code) — ~0.05 Mtok

- **0.1** Comment on #576: `sqrt` **removed from scope**, citing ADR188 Row 6,
  `r9_chapters.rs:2594`, and `politics.py:145,227-228` as the named site. State explicitly that
  the blocker is doctrinal, not numerical.
- **0.2** Comment on #576: the corrected exp/log consumer table from §0.3, and the corrected
  gating claim (RNG unblocks 3; `log` has 2 ready consumers; `exp` has 1, itself blocked on
  D35/D65). File the ImperialRent `exp`/`log` claim as **unverified** for the survey's errata.
- **0.3** Comment on #576: the RNG signature correction from §0.2/§3.2, so the shape is recorded
  before it is built.
- **Gate:** three comments posted; #576's body scope no longer reads as three-for-three.

### Task 1 — The libm crossing — ~0.35 Mtok

- **1.1 RED.** New `rust/crates/babylon-kernel/tests/transcendental_goldens.rs`. Assert
  `babylon_kernel::transcendental::exp(x).to_bits()` and `::ln(x).to_bits()` equal pinned
  `u64` constants over a deliberate roster: `exp` at `{0.0, -0.0, 1.0, -1.0, 0.5, 709.0,
  709.782712893384, -745.0, 1e-300}`; `ln` at `{1.0, 2.0, 0.5, f64::MIN_POSITIVE, 3.0,
  1e300, 1.0000000000000002}`. Leave the constants as `0x0` placeholders → **the test fails,
  and so does the build** (module absent). Commit red.
- **1.2 GREEN.** `babylon-kernel/Cargo.toml`: add
  `libm = { version = "0.2.16", default-features = false }`. `mise run rust:lock-refresh`.
  New `babylon-kernel/src/transcendental.rs`: two `#[must_use]` wrappers with `# Errors`-free
  infallible signatures (`pub fn exp(x: f64) -> f64 { libm::exp(x) }`, `pub fn ln(x: f64) ->
  f64 { libm::log(x) }`), and a module doc recording §2.3's verified dispatch analysis
  (log: no dispatch; exp: `use_arch_required: x86_no_sse`, unreachable on x86_64/aarch64) with
  the exact source coordinates. Re-export from `lib.rs`.
- **1.3 GREEN.** Fill the golden constants from the first green run. **Pin thereafter** — the
  `rng.rs:191-192` precedent ("filled from the first green run … and byte-pinned thereafter"),
  with the same comment convention: any later divergence is a determinism regression, never
  "the math got better."
- **1.4 GREEN — the sentinel.** `rust/clippy.toml`: add
  ```toml
  disallowed-methods = [
    { path = "f64::exp",  reason = "platform libm — use babylon_kernel::transcendental::exp (ADR176 r21)" },
    { path = "f64::ln",   reason = "platform libm — use babylon_kernel::transcendental::ln (ADR176 r21)" },
    { path = "f64::log",  reason = "…" },
    { path = "f64::log2", reason = "…" },
    { path = "f64::log10",reason = "…" },
    { path = "f64::exp2", reason = "…" },
    { path = "f64::exp_m1", reason = "…" },
    { path = "f64::ln_1p",  reason = "…" },
    { path = "f64::powf", reason = "…" },
    { path = "f64::tanh", reason = "…" },
    { path = "f64::sqrt", reason = "no sqrt intrinsic — ADR188 Row 6 ELIMINATED it" },
  ]
  ```
  `-D warnings` is already workspace-wide, so these become hard errors.
  **Verify no existing call site trips them** — if `babylon-client`'s Bevy code needs `sqrt`,
  scope the row with an `#[allow]` carrying a cited reason, never by dropping the row.
  Then **mutation-validate the sentinel**: temporarily write `x.exp()` in
  `transcendental.rs`, confirm clippy fails, revert.
- **1.5 GREEN — docs.** New `determinism-contract.rst` subsection "Transcendental crossing —
  `exp`/`log`" carrying §2.2's headline, §2.3's verification, and §2.4's three-part tolerance
  derivation verbatim. Repair `bsl-language.rst` §4.3's stale "open Phase-1 Director ruling"
  sentence to cite ADR176 r21 + ADR188 as the closing authority. Repair
  `ai/bsl-architecture-standard.md`'s OQ-1d row to CLOSED, with a pointer. (Its §7.4
  register snapshot is separately stale at D28 — leave it; correcting the whole snapshot is not
  this train's job, but add a one-line "see bsl-language.rst for the live register" if the row
  is edited anyway.)
- **Gate:** `mise run rust:check` green; goldens pass; the mutation table for 1.4 recorded;
  `vale` clean on the two edited RST files.

### Task 2 — `exp`/`log` dispatch — ~0.3 Mtok

- **2.1 RED.** In `intrinsic_host.rs`'s test module:
  `KernelIntrinsicHost.call("exp", &[Value::Real(0.0)], ctx)` must be
  `Ok(Value::Real(1.0))`; `("log", [Real(1.0)])` must be `Ok(Real(0.0))`; `("log", [Real(0.0)])`
  and `("log", [Real(-1.0)])` must carry the **new** code; `("exp", [Real(1e10)])` must carry
  `EvalCode::NonFinite`; `("exp", [Int(5)])` and `("exp", [])` must be `Err` (uncoded).
  All fail today (`intrinsic_host.rs:63-68`). Commit red.
- **2.2 GREEN.** Verify the next free `E-EVAL` number against `bsl-language.rst:3701-3714` and
  the D105 row; mint one `EvalCode` variant (`TranscendentalOutOfDomain`) + its `spec_code()`
  arm (`evaluator.rs:118-220`).
- **2.3 GREEN.** `eval_exp`, `eval_log` in `intrinsic_host.rs`, each modelled line-for-line on
  `eval_floor`: destructure `[Value::Real(x)] = args` (**no `real_lane` promotion** — the
  intrinsic boundary does not coerce, `intrinsic_host.rs:101-112`), reject non-finite input,
  reject `log`'s `x <= 0.0` with the new code (and `-0.0` explicitly: `-0.0 <= 0.0` is true, so
  it rejects — the mirror of `floor`'s negative-zero row), compute via
  `babylon_kernel::transcendental::*`, reject a non-finite **result** with `E-EVAL-014`. Wire
  both arms into `KernelIntrinsicHost::call`; update its `other =>` message and the module doc
  (which currently declares `{exp, log}` "Phase 2 … future work" — that sentence retires here).
- **2.4 GREEN.** Update the "an undeclared name fails loud" test
  (`intrinsic_host.rs:269-277`) — it currently asserts `exp` fails. `round-half-even` stays
  failing (ADR188 Row 3 is ratified but unlanded; `declarations.rs:742-746` says so).
- **2.5 GREEN — docs.** Create the **first real rows of the normative intrinsic table** in
  `bsl-language.rst` §3.10 (ADR188's consequences owe it and it does not exist yet — verified:
  no populated per-intrinsic list-table anywhere). Proposed shape, following the §3.2
  Currency-table convention:
  ```rst
  .. list-table::
     :header-rows: 1
     :widths: 14 16 12 8 50
     * - Name
       - ``:params``
       - ``:returns``
       - ``:cost``
       - Crossing, domain, and authority
     * - ``floor``
       - ``(real)``
       - ``int``
       - 5
       - IEEE-754 ``roundToIntegralTowardNegative``; domain ``[0, ∞)``; ADR188 Row 2 / D97.
         No libm crossing, so no golden vector (consequence declined, not omitted).
     * - ``exp``
       - ``(real)``
       - ``real``
       - 10
       - ``libm 0.2.16`` soft-float, ``default-features = false``; non-finite result is
         ``E-EVAL-014``; ADR176 r21 + ADR188 cap. Golden vectors: DXXX.
     * - ``log``
       - ``(real)``
       - ``real``
       - 10
       - As ``exp``; natural log; domain ``(0, ∞)``, non-positive is ``E-EVAL-0NN``.
     * - ``rng-draw``
       - ``(int)``
       - ``real``
       - 12
       - Kernel seam, not a transcendental: ``KernelRng::for_carrier(…).next_f64()`` on
         ``[0, 1)``; key per DXXX; ADR188 Row 11. No libm crossing, no golden vector.
  ```
  **`:cost` provenance.** The only intrinsic declaration in shipped content today is
  `rust/crates/babylon-tick/content/rules/territory.bsl:67` —
  `(intrinsic floor :params (real) :returns int :cost 5)` — which is also the proof that the
  whole intrinsic path is production-wired end to end, not merely unit-tested. `floor`'s row
  therefore reads **5**, quoted from content. The `exp`/`log`/`rng-draw` numbers (10/10/12) are
  **proposals for the first declaring pack**, not kernel constants: `fuel.rs` hard-codes no
  per-intrinsic cost, and the table cell must say "author-declared; the first pack sets it,
  pinned by vector thereafter" rather than imply the kernel fixes it.
  Extend the §4.6 contiguity paragraph's `E-EVAL` range.
- **Gate:** `rust:check` green; mutation table for each new error class; `vale` clean.
- **Sequencing note:** Tasks 1–2 are **independent of Tasks 3–5** and ship as PR A.

### Task 3 — Content-stable node identity to the tick seam — ~0.25 Mtok

- **3.1 RED.** In `babylon-bsl/src/scenario.rs`'s tests: load a two-node scenario and assert
  `loaded.node_content_ids.get(&node_id) == Some("<declared id>")`. Field absent → fails.
  Second red row: hydrate scenario `S` and scenario `S′` (= `S` with one node **inserted
  before** the others) and assert the *shared* nodes' content ids are unchanged even though
  their `NodeId` handles moved — the grain-invariance guard. Commit red.
- **3.2 GREEN.** Add `pub node_content_ids: HashMap<NodeId, String>` to `LoadedScenario`
  (`scenario.rs:224-244`), populated by inverting the existing local `named` map
  (`scenario.rs:336`). Assert injectivity at construction — two content ids mapping to one
  `NodeId` is a loud hydration bug, not a silent overwrite.
- **3.3 GREEN.** Thread it onto `PreparedRules` (`babylon-tick/src/lib.rs:93`) and hold it on
  `TickSession`. No `babylon-graph` change; no canonical-state change.
- **Gate:** `rust:check` green. **`fundamental_theorem_tick.rs`'s state hash must be
  byte-identical** — this task adds a side table and touches no graph write path. If the hash
  moves, STOP: something reached the substrate.

### Task 4 — The `DrawContext` seam — ~0.35 Mtok

- **4.1 RED.** A test asserting that a host call for `rng-draw` with `ctx` carrying no
  `DrawContext` returns `Err` with a message naming the missing session/tick. Fails to compile
  (no `IntrinsicCallCtx`). Commit red.
- **4.2 GREEN.** Add `DrawContext<'a>` and `IntrinsicCallCtx<'a>` to `intrinsic_host.rs`; add
  the third parameter to `IntrinsicHost::call`; update `EmptyIntrinsicHost`,
  `KernelIntrinsicHost`, and the two in-test doubles `Doubler` (`evaluator.rs:2257`) and
  `RogueIntrinsicHost` (`structural_verbs.rs:2589`).
- **4.3 GREEN.** Add `pub draw_context: Option<&'a DrawContext<'a>>` to `EvalEnv`
  (`evaluator.rs:268-299`). `eval_intrinsic` (`evaluator.rs:1473-1492`) builds the
  `IntrinsicCallCtx` from `env.draw_context` + `env.elements` (mapping each
  `Element::Node(id)` through the Task-3 map; `Element::Edge` renders as its two endpoints'
  content ids, framed) and passes it to `host.call`. **This is the only `evaluator.rs` call
  site that changes** — `host` stays `&dyn`, immutable, threaded exactly as today.
- **4.4 GREEN.** `run_tick` / `collect_pass` (`tick.rs:536-560, 615-660`) construct one
  `DrawContext` per subject inside the existing subject loop: `domain` = the rule id from
  `loaded`, `subject` = the Task-3 content id for the current `NodeId`. `run_tick` gains a
  `session: &SessionId` parameter (it already carries `tick: i64` and already wears
  `#[allow(clippy::too_many_arguments)]`).
- **4.5 GREEN.** `TickSession::new` gains a `SessionId`; `run_once` / `run_once_into` pass
  `SessionId::new("run-once")` (a fixed literal, documented as the conformance-driver session).
  Update `babylon-client`'s engine link and `babylon-tick/src/main.rs`.
- **Gate:** `rust:check` green; **`fundamental_theorem_tick.rs`'s hash byte-identical** (no
  content calls `rng-draw` yet, so no draw can enter any hash — if the hash moves, the seam
  leaked into a write path). Refactor-only PR: **no new intrinsic lands here.**

### Task 5 — The `rng-draw` intrinsic — ~0.5 Mtok

- **5.1 RED — the conformance family.** New `mod c14_rng_draw` in
  `rust/crates/babylon-bsl/tests/r9_chapters.rs` (chapters run C1→C13 today; C14 is next), plus
  a `tests/conformance/rng_keyed_draw.bsl` fixture pair. Rows, each failing:
  1. `check_intrinsic_cap("rng-draw")` is `Ok(())`, and `DECLARABLE_INTRINSICS` is the
     four-name set. **`sqrt` stays in the outside roster** (`r9_chapters.rs:2594` keeps it).
  2. `kernel_signature("rng-draw") == Some((vec![Scalar(Int)], Real))`; a declaration with any
     other `:params`/`:returns` is `E-LOAD-020`.
  3. **Same key ⇒ equal draws** (§6.2 family 22's own words): two rules whose ids are equal
     is impossible, so the row is *two evaluations of the same rule at the same tick over the
     same subject*, asserting bit-equality.
  4. **A skipped draw shifts nothing** — the D69/§6.2 row. Two rules identical except for a
     guard that suppresses subject A's draw; subject B's draw is bit-identical across both.
     Comment must cite `src/babylon/engine/systems/doctrine.py:527-537` as the real
     `needs_roll` instance this prevents.
  5. Different **slot** ⇒ different draw (`(rng-draw 0)` vs `(rng-draw 1)`).
  6. Different **subject** ⇒ different draw; different **element** in a fold ⇒ different draw.
  7. Different **tick** ⇒ different draw; different **session** ⇒ different draw.
  8. Result is in `[0.0, 1.0)` over ≥1000 draws, and is an exact multiple of `2⁻⁵³`
     (`rng.rs:88-95`'s guarantee, asserted here so a future `next_f64` change is caught at the
     BSL boundary too).
  9. Key-framing **injectivity**: chains `("ab","c")` and `("a","bc")` render to different
     `stable_key`s — the mirror of `rng.rs:138-142`.
  10. `(rng-draw 0)` with no `DrawContext` is a loud `Err`, never `0.0`.
  11. `(rng-draw 0)` **before** the cap row is `E-LOAD-021` at the bound checker.
  12. `(rng-draw 0.5)` / `(rng-draw)` / `(rng-draw 0 1)` are `Err`.
  13. **`seed_for`'s pinned vector is unchanged** — re-assert `rng.rs:193-198`'s four `u64`s
      from the BSL side, so this train cannot silently re-derive the kernel seed.
  Commit red.
- **5.2 GREEN.** `DECLARABLE_INTRINSICS` += `"rng-draw"` with a comment carrying the authority
  chain (ADR188 Row 11 + §3.10's RNG convention + D69 + the new D-record), phrased on the
  `floor` model — `check_intrinsic_cap`'s message (`declarations.rs:739-746`) must be updated
  to name the RNG row's own separate authority, so "adding a name is a Director ruling" is not
  contradicted by an unexplained fourth entry. Add the `kernel_signature` arm. Add the
  `KernelIntrinsicHost::call` arm + `eval_rng_draw` + the `framed()` key composer.
- **5.3 GREEN.** Update the `c13_intrinsic_cap::exp_log_and_floor_are_declarable` test's
  expected constant (rename it) and the `intrinsic_host.rs` "undeclared name" test.
- **5.4 GREEN — docs & registers.** Extend §6.2 family 22 (which already reserves the two RNG
  rows — `bsl-language.rst:4390-4394`) with rows 5–13 above; add the intrinsic-table row from
  Task 2.5.
- **Gate:** `rust:check` green; **13-row mutation table** in the PR body (per the
  sentinel-every-error-class rule); `fundamental_theorem_tick.rs` hash byte-identical (no
  shipped content calls `rng-draw`).

### Task 6 — Registers, D-records, ADR — ~0.25 Mtok

Highest D-number today is **D155** (`bsl-language.rst:7071`, Train B item 6 / #591). Verify at
landing (Train B item 4/5 may land rows first) and allocate contiguously. Register table format
is three columns — `#` / `Section` / `Ruling` (`bsl-language.rst:4691-4699`).

- **6.1** `D<n>` — §4.3, §3.10: the transcendental crossing. `libm 0.2.16`,
  `default-features = false`, the verified dispatch analysis, golden vectors as the guard,
  zero-tolerance within Rust, III.12 corollary (b) against the frozen engine only.
- **6.2** `D<n+1>` — §3.10: **`rng-draw`'s signature, superseding D69's operand-shape half and
  preserving its purity clause verbatim.** Must state (i) the enum/reference-operand shape is
  undeclarable because `<intrinsic-decl>`'s `:params` vocabulary admits neither, (ii) closing
  that would touch §5.6 CAS bytes and is deliberately not done here, (iii) `domain` = rule id
  is *stronger* on the content-cannot-mint-a-stream axis, (iv) D69's "a draw is a pure function
  of its key, not a stream position" survives unamended.
- **6.3** `D<n+2>` — §3.10: the `stable_key` byte derivation (the `framed()` layout, the
  element chain's outermost→innermost order, the slot's position) and its injectivity argument.
  This is a **language-agnostic-to-the-byte contract** — write the layout out, do not let one
  implementation imply it.
- **6.4** `D<n+3>` — §3.10: the content-id-vs-handle disposition. Why the retained hydration map
  and not a substrate widening; the Program 29 escalation path if a reviewer prefers the latter;
  the explicit statement that `graph_content_hash` is untouched.
- **6.5** `D<n+4>` — §3.10: session-id provenance. Deterministic only; `"run-once"` for the
  one-shot drivers; a campaign's id is the `ContentDigest` hex or the scenario id, minted by the
  client, **never** a UUID or a wall-clock read.
- **6.6** `D<n+5>` — §2.7/§3.10: the **name-level-only nature of the `sigmoid` gate.**
  `exp` + arithmetic can express `1/(1+exp(-x))` and `tanh` out of two permitted intrinsics
  (`survey:305` names exactly this hazard). `PROHIBITED_INTRINSIC_NAMES` cannot see it.
  Record that gate 2 stays Director review (D71's own position) and **recommend** an
  emergence-audit sentinel that pattern-matches logistic-shaped subexpressions in loaded
  content. **Do not build that sentinel in this train** — it is its own issue, and the
  recommendation must say so.
- **6.7** ADR `ADR2NN_intrinsic_host_train.yaml` + `ai/decisions/index.yaml` row: transcribes
  §0's scope verdict (the `sqrt` STOP, the RNG shape correction, the consumer re-count), §2's
  determinism policy, and §3's design with its rejected alternatives. `supersedes: []`;
  `related:` ADR176, ADR188, ADR202, ADR208.
- **6.8** `ai/state.yaml` closing entry. `reports/port-estate-survey-2026-08-12.md` errata:
  the `sqrt` row, the exp/log count, the RNG-shape row, the unverified ImperialRent claim.
- **Gate:** `vale` clean on every edited Markdown/RST; `mise run check:quick`; ADR present.

### Task 7 — Closeout — ~0.1 Mtok

- Post #576's evidence comment: the three PR links, the mutation tables, the golden-vector
  values, the corrected gating claim, the `sqrt` STOP and where it went.
- Revise the token estimate against actual (charter says ~2 Mtok / 1 window; this plan sums to
  ~2.15 Mtok, which is a confirmation, not a revision).
- Close #576. Open the two follow-ups named above: the emergence-audit sentinel (6.6) and, only
  if a reviewer asks for it, the substrate stable-identity widening (6.4 → Program 29).

---

## 5. Gates

### Per-commit (every task)
```bash
mise run rust:check      # fmt --check; clippy -D warnings -D cognitive_complexity workspace-wide;
                         # cargo test --workspace; pedantic legs on kernel/bsl/graph; cargo doc -D warnings
```
Run it **single-flight**. Nothing else runs concurrently against `rust/target`.

### Per-PR
```bash
mise run check                     # Python lane: lint + format + typecheck + test:unit
mise run qa:regression             # must be BYTE-IDENTICAL — no Python engine change
mise run qa:vault-regression-ci    # separate estate; must be byte-identical for the same reason
```
Plus, from `rust/`:
- `cargo test -p babylon-tick --locked` — `fundamental_theorem_tick.rs`'s state hash
  **byte-identical in every PR**. This is the train's real determinism gate: no shipped BSL
  content calls the new intrinsics, so **any** hash movement means something leaked into a write
  path. Treat movement as a STOP, never as expected drift.
- `cargo deny check` if `deny.toml`/`Cargo.lock` moved (Task 1.2). `libm` is MIT and crates.io —
  expected clean, but run it rather than assume.

### Ceremonies
**None expected.** This train writes no `tests/baselines/**` file, so no `§6.5
Baselines: blessed(...)` trailer is owed. If any baseline moves, **STOP** — it means Task 3 or 4
reached the substrate, and the correct response is to fix the leak, not to bless the drift.

### Merge protocol (ADR181)
Per PR: verify every check completed and `headRefOid == headSha`; **harvest the Copilot review**
(wait for it; reply to or fix every inline comment — zero unaddressed is a precondition); merge
with `mise run pr:merge -- N`, the only sanctioned path. Branch from `dev`.

---

## 6. PR structure

Three PRs, stacked but independently green.

| PR | Tasks | Branch | Scope | Why separable |
|---|---|---|---|---|
| **A** | 0, 1, 2 | `feature/intrinsic-host-transcendentals` | The libm crossing, `exp`/`log` dispatch, the `disallowed-methods` sentinel, the intrinsic table's first rows, the §4.3/OQ-1d doc repair, one new `E-EVAL` code | Touches no seam. Reviewable as "a crossing mechanism + two match arms." Delivers `log`'s two ready consumers on its own. |
| **B** | 3, 4 | `feature/intrinsic-host-draw-seam` | Content-id retention, `DrawContext`, the `IntrinsicHost::call` third parameter, `SessionId` threading | **Refactor-only, no new intrinsic.** Its acceptance test is "the hash did not move and `rng-draw` fails loud." Keeping it separate means the trait-signature churn is reviewed without the RNG semantics on top. |
| **C** | 5, 6, 7 | `feature/intrinsic-host-rng-draw` | `rng-draw`, the C14 conformance family, the five D-records, the ADR, closeout | The semantic payload, reviewed against a seam that already landed green. |

Do **not** use `--delete-branch` on A or B while C is stacked (#193: it closes rather than
merges the child). `pr:merge` refuses it by construction.

Commit-message scopes: `feat(kernel)`, `feat(bsl)`, `feat(tick)`, `docs(bsl)`, `docs(determinism)`,
`chore(rust)`. Conventional commits, `Co-Authored-By` trailer on every one.

---

## 7. Blockers and escalations — stated, not planned around

| # | Item | Severity | Disposition |
|---|---|---|---|
| **B1** | **`sqrt` is eliminated by ADR188 Row 6**, ratified, and pinned in-tree at `r9_chapters.rs:2594`. Its only claimed consumer is the exact site Row 6 names. | **HARD STOP on a charter item** | Removed from scope (Task 0.1). Reopening it is a new Director ruling superseding Row 6 — not an implementation PR. The Allegiance port owes a measure re-derivation. |
| **B2** | **D69's RNG signature is undeclarable.** `<intrinsic-decl>`'s `:params` vocabulary admits neither an enum-ref nor a node-ref (`declarations.rs:650-686`); widening it changes §5.6 CAS bytes. | Design blocker, **resolved in-plan** | Context-keyed shape (§3.2) + a superseding D-record (Task 6.2). No CAS change. |
| **B3** | **The substrate has no stable node identity.** `NodeId(pub u64)` is an insertion-order handle; `GraphSubstrate` exposes no content id. Keying on it would violate D69's "independent of insertion history" and ADR176 r20's grain-invariance rationale. | Real, **resolved in-plan** | Retain hydration's existing `HashMap<String, NodeId>` (`scenario.rs:336`) inverted onto `LoadedScenario` (Task 3). Zero hash change. **Escalation path if a reviewer wants it in the substrate: Program 29 + a III.7 hash ADR.** |
| **B4** | **`exp`'s only live consumer is blocked elsewhere.** After ADR202 R7/R8, the one surviving verbatim `exp` site is Contradiction @18.0's financialization index (R9), and Contradiction is blocked on D35/D65 edge storage (survey C-4). | Scoping honesty | Land `exp` anyway (one code change with `log`), but #576's closeout must **not** claim eight unblocked systems. `log`'s two consumers (Community entropy, MarketScissors anchor) are genuinely ready; the RNG unblocks three. |
| **B5** | **`exp` + arithmetic can express the prohibited `sigmoid`/`tanh`** out of two permitted intrinsics (`survey:305`). `PROHIBITED_INTRINSIC_NAMES` is a name-level gate only. | Doctrinal hazard, **not** a code blocker | Recorded as D-record 6.6; gate 2 stays Director review (D71's own position). Recommend an emergence-audit sentinel as its own issue. **Do not build it here** and do not pretend the mechanical gate covers it. |
| **B6** | **`bsl-language.rst` §4.3's "open Phase-1 Director ruling"** on polynomial-vs-libm contradicts ADR176 r21, which ruled the mechanism. | Doc staleness | Repaired in Task 1.5. Flag it in the PR body so a reviewer who remembers the open question sees why it closed. |
| **B7** | **The §6.1 `(vector ...)` conformance harness does not exist.** The normative grammar is at `bsl-language.rst:4121-4198`; the actual `tests/conformance/*.bsl` files are bare `(rule …)` forms with every assertion hand-written in Rust, and nothing anywhere parses a `(vector …)` form. | Pre-existing gap | Follow existing practice: bare fixture + Rust asserts in `r9_chapters.rs::c14_rng_draw`. **Do not invent the harness in this train.** |
| **B8** | **No doc↔enum E-code sync test exists** (`docs/reference/error-codes.rst` has zero `E-EVAL` hits; the sequence is maintained in prose at `bsl-language.rst:3701-3714`). | Pre-existing gap | Re-verify the next free number by hand before minting (Task 2.2), and update the contiguity paragraph in the same commit. Filing the sync sentinel is out of scope; note it in the ADR's consequences. |
| **B9** | **Session-id provenance is unspecified** for a campaign. III.7 forbids a UUID or a wall-clock read. | Small open decision | Recorded as D-record 6.5: deterministic only, `"run-once"` for one-shot drivers, `ContentDigest` hex or scenario id for a campaign, minted client-side. If the Director wants player-visible session identity to work differently, that is her call — the plan does not foreclose it. |
| **B10** | **`ai/bsl-architecture-standard.md` §7.4's register snapshot is stale at D28** while the live register runs to D155. | Pre-existing | Out of scope. Do not cite §7.4 as current. Add a one-line pointer if the OQ-1d row is edited anyway (Task 1.5). |

**Nothing in this list stops the train.** B1 removes one charter item; B2/B3 are resolved by
design inside the plan; the rest are recorded facts a reviewer needs.

---

## 8. Estimate

| Task | Mtok |
|---|---|
| 0 governance | 0.05 |
| 1 libm crossing | 0.35 |
| 2 exp/log dispatch | 0.30 |
| 3 stable node identity | 0.25 |
| 4 DrawContext seam | 0.35 |
| 5 rng-draw + C14 family | 0.50 |
| 6 registers/D-records/ADR | 0.25 |
| 7 closeout | 0.10 |
| **Total** | **~2.15** |

Consistent with the charter's "~2 Mtok / 1 window @1M". Revise at closeout per #255 conventions.
