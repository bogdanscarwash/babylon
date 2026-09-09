# BSL Refactor Program — rev 1

**Provenance.** Director ruling 2026-08-18 (same day as the hygiene knock-out): begin the
refactor program NOW, overriding the prior "WS3-at-13/13" sequencing — the survey's own
`Timing: WS3` tags on themes 3(b)/6/7/8/9, and issue #502's post-port-refactor-program
sequencing constraint #2 ("draft the full charter … when the last system port lands"), both
assumed this work waits for Checkpoint A (all 13 Material Base systems ported). Director's
words: "we need to get this correct and we might as well do it now since there is work
ahead." **Corrected tally as of this draft (superseding this program's own dispatch brief,
which carried a stale count): 9/13 Material Base systems are landed on dev; 4 remain**
(ImperialRent, Community, TickDynamics — all with active worktrees at varying completion —
and ReserveArmy, held out per ADR210's own language, not yet an open worktree). Source: the
ranked survey synthesis (`bsl-refactor-synthesis.md`, 8 read-only surveyors, 9 themes,
evidence-cited) plus a live sizing sweep against `dev @ 7b8dbf98` done for this draft (six
parallel research passes + extensive direct verification, 2026-08-18). Sibling, not overlap:
Phase 0 (`2026-08-18-bsl-hygiene-knockout.md`, branch `feature/bsl-hygiene-knockout`,
worktree `wt-hygiene`) is EXECUTING — this program starts where it ends and depends on its
deliverables (the `bsl-lint` host, the two E-LOAD refusals, the fuel-report mode, the
lifecycle guard). **This program is NOT the #502 WS3 charter** — that remains a separate,
later, Director-run ceremony over the full D-record ledger, gated on "last system port
lands." This program is a targeted pull-forward of the specific items the survey found
mechanically ready today, named individually below, each tracing to its own evidence.

**A note on how this draft was produced, kept for the record (Immutability of History):**
multiple research passes dispatched for this draft were given narrow briefs but, inheriting
the full assignment context (including the "write the final output here" instruction), some
independently attempted the entire drafting task rather than staying scoped — one wrote a
full competing draft to this exact path; others surfaced late corrections after this author's
own draft was substantially complete. Three corrections landed late and are folded in here
rather than re-litigated: the Material Base tally above (9/13 + 4, not the dispatch brief's
stale "7 remain"); `EventType` has 100 members (AST-verified, matching CLAUDE.md, not the
survey's "98"); and — verified directly by this author moments before finalizing —
`scenario.rs`'s load-time `(hyperedge …)` handling hand-rolls its own `seeded_hyperedge_names:
HashSet<String>` (confirmed at `scenario.rs:586,669,1913,1929,1945`, with the code's own
comment naming it "a SEPARATE table from `named`"), genuinely distinct from
`structural_verbs.rs`'s RUNTIME `fresh_declared_name`/`declared_hyperedges` — so the
shared-helper-extraction opportunity in Phase 6 is real, not already discharged, and this
draft's Phase 6 section is written accordingly. The practical lesson, worth keeping distinct
from the plan's content: a `fork` dispatched for a narrow research question inherits the FULL
parent context, including an explicit "write the final output here" instruction — scoping a
fork's prompt is not enough on its own to prevent it from attempting the whole task, or from
several independent passes converging on the same file without coordination.

**Revision note (this pass).** An adversarial critique (`refactor-plan-critique-r1.md`, 1
Critical / 7 Important / 6 Minor) reviewed this draft against independent research and fresh
direct verification. VERDICT: NEEDS-REVISION, with this plan's core decisions AFFIRMED — the
Phase-0-only resumption gate, the phase ordering, and Phase 6's triple gate all survived
adversarial testing (the critique's own author disproved their own initial alternative for
Phase 6 — see §8). This revision applies every finding: C-1 (R4.1's registry redesigned around
evidence tiers), I-1 through I-7, and all six Minors — each is marked inline at its edit site.
No finding was rejected in this pass.

**The check-placement boundary (Director, 2026-08-18, binding on every task below):** semantic
coherence of a content set → IN-LANGUAGE load/type refusal (new E-LOAD/E-TYPE codes,
amendment-free loader hardening, strengthen-then-audit — landed content gets repaired, checks
never weakened); repo-relationship invariants (citations, numbering, cross-file duplication) →
`bsl-lint`; checks requiring NEW author declarations → amendment-class, Director-ruled, not
decided in this document. Every task below is classified against this boundary explicitly.

**The closed algebra stands (ADR172).** Everything planned here is amendment-free. `defevent`
and `bevy/dynamic_linking` appear ONLY in the QUEUED DIRECTOR RULINGS section at the end —
nowhere else in this plan builds either.

**Global constraints (same as Phase 0, restated because they bind here too).** Read-only
toward all in-flight train branches this program does not itself own. Pin law: all
pre-existing pins byte-identical after any task, or STOP. TDD: red → green per task.
Conventional commits; commit per unit. S-11 named refusals for every new error path. Every
new lint reuses babylon-bsl's own S-expression reader — never a second parser. New gates are
ADDITIVE legs; nothing existing is relaxed. ~100-line function guideline — single-function
widenings in this plan stay well under it; the `scenario.rs` split exists BECAUSE at least two
functions already violate it by closing-brace span (§8 corrects this draft's original
overstated count of four, critique M-2). Heavy builds stay single-flight, machine-wide — with 8
worktrees live simultaneously as of this draft, `rust:check`'s existing "one cargo target dir,
one file lock" discipline (`.mise.toml:1639`) is a cross-worktree constraint today, not merely
a per-worktree one, until Phase 1's build-environment task lands and makes moderate
concurrency safer.

---

## 1. The estate right now — why sequencing is the hard part, not the code

A snapshot at drafting time (`git worktree list`, 2026-08-18); it will be stale within hours
given the pace below. Treat it as the reasoning trail for §2's sequencing decisions, not as a
control surface anyone should poll.

Eight worktrees are live. Two are Phase 0 (`wt-hygiene` — W1's `bsl-lint` host and the
`TAG_*` distinctness test staged uncommitted, plus citation fixes to
`consciousness.bsl`/`solidarity.bsl`; `wt-cookbook` — W4 DONE, 2 commits, +207 lines to
`bsl-language.rst`'s new authoring-idioms section). Six are BSL-content or client trains this
program does not own but must sequence around:

| Worktree | Branch | State | Touches this program cares about |
|---|---|---|---|
| `wt-491` | `feature/491-rung-ladder` | ~21-26 commits (moving target), 31 files, **+8665/-327**, no PR posted | `typecheck.rs` (the E-TYPE-040 kind arm — `ExprKind`, `check_kind_mixing`), `scenario.rs` (+508 lines), `CanonicalState` tag `0x06` (kept, see below), ADR216 already accepted |
| `wt-652` | `feature/652-bsl-ls` | LSP train, span side-table (Task 1) + typed error identity (Task 2) committed | `scenario.rs` (+140 lines), new `error_identity.rs` (373 lines), `reader.rs` (+343 lines) — **already solved position-threading** via a side table, chosen over an `SExpr`/`Atom` struct-field retrofit after evaluating three candidates |
| `wt-community` | `feature/community-port-bsl` | Active; the `(hyperedge …)` top-form is already landed (commit `2e97733f`); later tasks (hyperedge attribute storage + a `CanonicalState` tag, then a `structural_verbs.rs` read/write surface) are HELD, not started | `scenario.rs` (+201-209 lines so far), a future `CanonicalState` tag claim (now `0x07`, see below), later `structural_verbs.rs` work |
| `wt-imperialrent` | `feature/imperialrent-port-bsl` | 11 commits, **+7283/-0** (all new files); **corrected per critique I-4: mid-Task-4 of PR A, NOT content-complete** — `r00`-`r04` implemented (5 of the 10 rules the branch's own plan specs; `r05`-`r09` entirely unwritten), Task 4's own closing step (TRPF-decay mutation world + inflow golden pins, then "Step 7: Open PR A") has no matching commit — latest commit is a review-round doc restatement | `imperial-rent.bsl` (834 lines, growing) + a conformance estate in progress — NOT ready to post yet |
| `wt-tickdynamics` | `feature/tickdynamics-port-bsl` | 4 commits, **plan/dossier docs only so far** (2897 lines, zero `.bsl` content committed) | early-stage; costs nothing to hold for any gate this program sets |
| `wt-b3` | `feature/b3-null-hypothesis-viewer` | 14+ commits, live uncommitted work continuing; "review round 1" is a recent commit | `loop_ui.rs` — **`payload_node_id` is already GONE on this branch**, replaced by `narration.rs`'s declared `subject_key` table (confirmed by direct read, see §6) |

**A live namespace collision, resolved by fence, not by the race.** `state_hash.rs`'s
`CanonicalState` tags run `0x01`-`0x05` on dev (the fifth minted by ADR198 R2). Both
`feature/491-rung-ladder` (Currency typed-attribute storage) and `feature/community-port-bsl`'s
held storage task targeted `0x06` off the same 5-tag base — the third realized instance of
survey theme 2's exact failure mode this month, after 9 historical ADR renumbers and 2
historical D99 collisions (the latter now mechanically fenced by `test_bsl_grammar_sync.py`).
**Resolution (controller fence):** #491 keeps `0x06`/layout-version-3 — already committed
in-tree, pins held, ruled merge priority; Community's held storage work moves to
`0x07`/version-4 with a re-derived elision proof and fresh measured pins. The instructive
nuance: Community's claim was DOC-level only (its plan file; zero code on that branch touches
`state_hash.rs`), so Phase 0's own `TAG_*` pairwise-distinctness test would have caught this
only AFTER both landed — a plan-level collision needs a human fence, a test over committed
code cannot see it coming a step earlier. That is the argument for Task R1.3 below: turn the
tag list into a single declared, position-derived array — a structural fix that closes the
CODE-level duplication class (plan-level collisions, like today's, remain a governance matter
the W1.4c check plus the fence protocol cover), rather than merely detecting it one commit later.

**What this means for sequencing:** `scenario.rs` is a three-way hot file this week (491, 652,
community). Splitting it now would triple the rebase cost across three independent trains
already deep in flight. And the position-threading question this program might otherwise have
had to design is not an open question at all — `feature/652-bsl-ls` has already built and
committed a span side-table, chosen deliberately over a struct-field retrofit. **Rule
position-threading OUT of this program's scope: it is in-flight and decided elsewhere, not
still to be designed here.**

---

## 2. Phase/lane map and the resumption-gate decision

**Resumption gate: Phase 0 (hygiene) landing on `dev` — nothing else.** Ports resume as soon
as Phase 0 merges, which given its current state (W4 done, W1/W2 in progress, W3/W5 small and
independent) should be imminent. The build-environment task, the conformance harness, the
kind-system gaps, the event-schema registry, theme-8's remainder, and the `scenario.rs` split
all run **alongside** port resumption on independent lanes — none of them gate a port train's
merge. Each instead carries a named audit-fallout obligation (§10) against content that lands
before it does. This directly implements the Director's "bias toward resuming ports early"
instruction while still protecting the pin law.

Reasoning against folding more into the gate:

- **Phase 0 as the gate: yes, and cheaply so.** W2's same-tick refusals are load-bearing
  correctness fixes (D116's premise is false today, not latently — 38 `:optional :default`
  bindings, 1 confirmed exposed). New multi-rule packs (TickDynamics' ~66-defconst transition
  engine, Community's hyperedge lane) should be written against the corrected loader from the
  start, not written blind and audited after. Phase 0 is nearly done regardless, so this gate
  costs little wall time.
- **The build-environment task: not in the gate.** It is a machine-safety fix (sccache, build
  profile, job cap, worktree cleanup) with zero correctness coupling to any port's content.
  The evidence for landing it FAST is strong (two swap exhaustions today across these same
  eight worktrees) — it should have priority — but as its own parallel lane, not a blocking
  gate, since gating ports on it would trade an acute problem for an unrelated delay.
- **The conformance harness: not in the gate.** The survey's own direction is explicit: land
  it "as a pure addition the next port branch picks up first, not a rewrite of the 26 existing
  files." Forcing six actively-changing test files to wait for a support module they are not
  required to adopt would slow Checkpoint A for no correctness benefit.
- **Kind-system gap (b): not in the gate, and it cannot be.** Gap (b) needs `expr_kind`/
  `field_kind`, which exist only on `feature/491-rung-ladder` — an 8665-line, unmerged,
  unposted branch this program does not own. Making gap (b) a gate would make port resumption
  hostage to a branch outside this program's control. It runs alongside, carrying the same
  audit-fallout obligation every other late-landing check carries; D184 already names its
  first expected finding, so the audit is not starting blind.

**The free mitigation the gate can still take (critique I-6, answering this plan's own §13
Q7).** Excluding gap (b) from the gate leaves real, quantifiable risk concentrated in exactly
one train — TickDynamics, whose ruled content shape (ADR210 D4-B/D5-C: populations, extensive,
against five class shares, intensive, over ~66 transition coefficients) is the densest
extensive/intensive conversion surface of any remaining pack, i.e. exactly gap (b)'s defect
class. Base rate on already-reviewed content (ADR216: 4 repairs found across 13 packs on the
kind arm's first sweep, ≈0.3/pack) suggests roughly one late-caught finding if TickDynamics
writes blind. **The mitigation costs nothing and needs no gate change:** `wt-491` already ships
`tools/kind_straddle_static_sweep.py` — runnable from that worktree against ANY pack today,
without #491 merging (it already validated D183 against all 13 landed packs). Rider, binding
on every resumed port train's own brief, not a task this program executes: **run the static
kind sweep from `wt-491` against your pack before posting your PR**, until R3.1/R3.2 land. This
covers gap (a)'s straddle/mixing class; it does NOT cover gap (b)'s store-boundary class (the
sweep checks expression shape, not write targets) — say so plainly in the rider text so a
train doesn't over-trust it. TickDynamics is named here as the concentrated risk specifically
so its brief carries the rider explicitly, not just by general policy. With the rider: keep the
gate exactly as decided above; do not grow it.

**Lane map** (each lane = its own worktree — never two implementers per worktree;
`scenario.rs` work is EXCLUSIVE to its lane and window; heavy builds stay single-flight per
the existing `rust:check` sequential-run-block convention — the build-environment task makes
that convention cheaper, it does not replace it):

```mermaid
flowchart LR
    P0["Phase 0 — hygiene\n(wt-hygiene + wt-cookbook,\nEXECUTING, not this program)"]
    R1["Phase 1 — build env\n+ CanonicalState tag array\n(own lane, starts now)"]
    R2["Phase 2 — conformance harness\nRust support/mod.rs\n+ Python _conformance_support.py\n(own lane, starts now)"]
    R3["Phase 3 — kind-system gaps a/b/c\n(own lane, GATED on #491 merge)"]
    R4["Phase 4 — event-schema registry\n(own lane, starts now;\nclient sub-task is verification-only,\nB3 already fixed it)"]
    R5["Phase 5 — theme-8 remainder lints\n(own lane, starts after Phase 0's\nbsl-lint host — already true in practice)"]
    R6["Phase 6 — scenario.rs split\n(own lane, GATED on\n491 + 652 + community ALL on dev)"]

    P0 -->|resumption gate| Ports["4 remaining port trains\nresume now, alongside R1-R6"]
    P0 -.->|precondition| R3
    Fork491["feature/491-rung-ladder\n(external, not owned here)"] -->|merge unblocks| R3
    Fork491 -->|one of three| R6
    Fork491 -.->|R1.3 also gated (I-1)| R1
    Fork652["feature/652-bsl-ls\n(external)"] -->|one of three| R6
    ForkComm["feature/community-port-bsl\n(external)"] -->|one of three| R6
    ForkB3["feature/b3-null-hypothesis-viewer\n(external)"] -->|merge triggers verification| R4v["R4's verification sub-task"]
```

---

## 3. Phase 1 — build environment + the namespace structural fix

**Governance:** tool (Director-approved infrastructure; the tag-array change is a structural
implementation detail of an already-hashed section, not a new author-facing declaration —
amendment-free). **Timing:** R1.1/R1.2 start immediately, no port dependency, lands first
opportunity given today's acute motivation (two swap exhaustions). **R1.3 is gated on #491's
merge as well** (critique I-1) — see its own task section below.

### Task R1.1 — sccache + build profile + job cap

**Files:** `flake.nix` (`devShells.default` packages + shellHook, mirroring the existing
`OMP_NUM_THREADS`-style export block); `.mise.toml` `[env]` (add `CARGO_BUILD_JOBS` next to
the existing BLAS/thread-pin vars — same rationale, oversubscription control); `rust/Cargo.toml`
(new `[profile.dev.package."*"]` block — the file today has exactly one profile section,
`[profile.release] overflow-checks = true`, so this is a pure addition, zero conflict).

Confirmed greenfield by direct read (2026-08-18): zero existing `RUSTC_WRAPPER`/`SCCACHE` hits
anywhere in the repo; `sccache` absent from PATH and from the flake's package list;
`/media/user/data/sccache` does not exist yet (target mount has 1.1T free); no `[profile.dev]`
section exists in `rust/Cargo.toml`; `CARGO_BUILD_JOBS` is absent from repo `.mise.toml`
(confirming it is currently only an interim user-global setting elsewhere).

- R1.1.1 Verification baseline (informational, not a gate): note current `rust/target/` size
  on one worktree before the change.
- R1.1.2 GREEN: add `sccache` to `devShells.default.packages`; set `RUSTC_WRAPPER=sccache`,
  `SCCACHE_DIR=/media/user/data/sccache`, `SCCACHE_CACHE_SIZE=25G` in the shellHook. Add
  `[profile.dev.package."*"] debug = false` (+ `split-debuginfo = "unpacked"`) to
  `rust/Cargo.toml`. Add `CARGO_BUILD_JOBS = "8"` to `.mise.toml`'s `[env]`. **(critique M-5)**
  Delete the interim `CARGO_BUILD_JOBS` entry from the user-global `~/.config/mise/config.toml`
  in the same commit — that file's own comment says "Revert: delete this entry" once this task
  lands; leaving both would leave a stale global entry and a now-wrong comment to confuse the
  next reader.
- R1.1.3 Verify: `mise run rust:check` green; confirm `qa:regression` + `qa:vault-regression-ci`
  byte-identical (a `[profile.dev]` change touches debug symbols only, never optimization
  level or codegen — zero pin risk expected, but prove it). Note the one-time fingerprint
  rebuild this absorbs; the cache fills from there. Confirm a second `rust:check` run shows a
  nonzero sccache hit rate.
- Commit(s): `chore(rust): sccache + profile.dev debug=false + CARGO_BUILD_JOBS=8 (Director-approved 2026-08-18)`.

### Task R1.2 — worktree-removal cargo-clean step

**Files:** new `mise` task (e.g. `wt:rust-clean`) — no existing ceremony to extend.
`tools/worktree_tool.py`'s `wt:done` task retires `.claude/worktrees/NAME`, hardcoded; every
live BSL-program worktree in this document lives under `/media/user/data/worktrees/wt-*`
instead, a convention `wt:done` never reaches. Confirmed: zero cargo/target-cleaning logic
anywhere in `worktree_tool.py`, and no doc under `docs/` or `ai/` governs
`/media/user/data/worktrees/` removal.

- R1.2.1 Propose `mise run wt:rust-clean -- <worktree-path>`: `cd <path>/rust && cargo clean`,
  with a guard refusing to run against the main checkout. Independent, zero pin risk.
- R1.2.2 See §13's unresolved-questions entry — the exact host for this (a new task vs. a
  documented manual step) needs a controller call, since no doc governs this convention today.
- Commit(s): `feat(tooling): wt:rust-clean — cargo-clean step for /media/user/data/worktrees/`.

### Task R1.3 — the CanonicalState tag structural fix

**Files:** `rust/crates/babylon-graph/src/state_hash.rs`.

**Timing, corrected (critique I-1): gated on #491's merge, not just Phase 0's W1.** #491
carries +341 lines to this exact file, including `TAG_CURRENCY_ATTRIBUTES = 0x06` as a sixth
independent const — and this program's own §1 fences #491 as merge-priority, keeping `0x06`.
Landing R1.3 first would force that priority merge to rebase 341 lines over a just-restructured
tag representation. This task now waits for BOTH Phase 0's W1 AND #491's merge, and builds the
array with **six** entries, `0x01..0x06`, not five.

**Motivation, restated and TEMPERED (critique I-7).** The `0x06` collision was resolved by a
human fence, not by any test, and Phase 0's own distinctness test is confirmed structurally
unable to prevent a plan-level collision (Community's claim never touched code). **Honest
claim, corrected from this draft's first pass:** a `const CANONICAL_SECTIONS: [(&str, u8); 6]`
with independently-chosen byte values does NOT make duplicates unrepresentable — two branches
could still both write `(_, 0x07)` into their own array entry. The by-construction property
requires tags DERIVED FROM POSITION (`tag = index + 1`, no byte value ever independently
chosen), a names-only array. Even then: the actual `0x06` incident was PLAN-DOCUMENT level, not
code — no code-structure fix reaches a collision that never touched code. **What this task
actually delivers: a code-level collision becomes a same-line merge conflict (git conflicts on
the same array line instead of two silently-coexisting consts), and a position-derived tag
makes code-level duplicate VALUES inexpressible; it does not and cannot reach a plan-level
collision before either side writes code.**

- R1.3.1 RED: a test asserting a single declared, ORDER-ONLY array of section names exists
  (`const CANONICAL_SECTION_NAMES: [&str; 6]`, or equivalent) with NO independently-chosen byte
  field, and that every tag used anywhere in the encoder is derived from that array's index
  (`index + 1` or equivalent), never a separately-declared literal — fails against both today's
  six-independent-consts shape (post-#491) and against a naively-generalized array that still
  carries independent byte values.
- R1.3.2 GREEN: the position-derived array (six entries, post-#491), re-deriving named accessor
  constants FROM the index via a `const fn` if call sites benefit from the symbolic name —
  every existing call site keeps working unchanged. Scope this to the accessor shape only — do
  NOT restructure the encoder's six section-writing call sites to loop over the array; they
  have heterogeneous per-section logic and forcing a loop is separate, larger, unscoped work.
- R1.3.3 Prove byte-identity: pure refactor of how six existing bytes get named/derived, zero
  behavior change — `qa:regression` + `qa:vault-regression-ci` byte-identical is the whole
  proof.
- Commit(s): `refactor(graph): CanonicalState tags position-derived from a declared array — closes code-level duplication by construction, plan-level needs a human fence (theme-2 incident #3, 2026-08-18, critique I-1/I-7)`.

**Gate (Phase 1 whole):** `mise run rust:check` green; `qa:regression` +
`qa:vault-regression-ci` byte-identical; no BSL content touched.

---

## 4. Phase 2 — the shared conformance harness

**Governance:** refactor-task (a pure addition; no author-facing surface, no amendment
question). **Timing:** starts immediately — independent of every active port train, since
nothing is required to adopt it.

Confirmed by direct read (2026-08-18): `rust/crates/babylon-tick/tests/` totals 12,158 lines
(grown from the survey's 10,509 — three port trains have added suites since). The
`run_once_into`/`CollectingSink`/`node_attribute` signature trio appears 315 times, confirming
any future signature change is a wide mechanical edit today without a shared module.
`attribute()`/`run()` redefinitions are NOT uniformly byte-identical across all files — at
least three distinct shapes exist (a `MemoryGraph`-keyed majority, a `HypergraphStore`-keyed
variant, and named alternates like `run_production()`), and the pin-table struct (`Expected`)
has at least one independently-renamed copy (`ExpectedTerritory`) beyond the two
byte-identical `lifecycle*` copies the survey named. **This matters for scope: a shared module
must accommodate genuine variants, not force one signature where call sites differ for real
reasons.** The Python frozen mirrors: exactly 25 files, 4,473 lines, `def main` in 25/25,
`def build_graph` in 19/25 — the survey's counts hold exactly; two files
(`solidarity_conformance.py`, `consciousness_ternary_conformance.py`) hand-roll a standalone
pipeline with no `ServiceContainer`/`BabylonGraph` at all and are out of scope for the shared
helper.

The template: `babylon-graph/src/conformance.rs` (read in full, 878 lines) is `pub fn
run_substrate_conformance<G, F>(make: F) where G: GraphSubstrate + CanonicalState, F: Fn() ->
G` dispatching to 21 small named invariant functions, each taking `&F` and calling `make()`
fresh so one failure never contaminates another's starting state. babylon-tick's need is
shaped differently — not "one generic runner over N invariants across substrate
implementations," but "many independent scenario-specific test files sharing utility
functions and a pin-table type" — so the borrowed pattern is the SPIRIT (small composable
named helpers, fresh state, no reparsing), not the literal dispatcher shape.

### Task R2.1 — `tests/support/mod.rs` (Rust)

**Files:** new `rust/crates/babylon-tick/tests/support/mod.rs` (standard Rust integration-test
convention — no new crate, no `Cargo.toml` change).

- R2.1.1 RED: a new, small conformance test file importing `support::{attribute, run_conformance,
  ExpectedField, assert_expected, assert_deterministic}` — fails to compile, `support` doesn't
  exist yet.
- R2.1.2 GREEN: `attribute<G: GraphSubstrate>(...)` generic over the substrate trait (subsuming
  the `MemoryGraph`/`HypergraphStore` split found above via `impl Into<NodeId>` where the
  wrapping differs); 2-3 `run_*` variants matching the shapes genuinely found, not one forced
  signature; a generic pin-table struct (`ExpectedField`) covering the fields actually shared
  across the `lifecycle*` pair and, where it diverges, `ExpectedTerritory`'s own shape (do not
  force a merge that papers over a real divergence); `assert_deterministic()` extracting the
  ~6-line double-run-compare-hash idiom appearing 19+ times. Float comparisons in the pin-table
  assertion need a stated tolerance policy with a written derivation (CLAUDE.md rule 4), not a
  bare epsilon.
- R2.1.3 **Do NOT sweep the 26+ existing files onto the new module in this task.** Land it as a
  pure addition; the next new conformance suite adopts it going forward. A mechanical migration
  sweep is a named, explicitly-deferred follow-up past Checkpoint A — several existing files are
  under active edit by concurrent port trains this week, and touching them now maximizes
  rebase conflict for a cosmetic win.
- Commit(s): `refactor(tick): tests/support/mod.rs — shared conformance helpers as a pure addition (theme 6)`.

### Task R2.2 — `_conformance_support.py` (Python frozen mirrors)

**Files:** new `rust/crates/babylon-tick/content/scenarios/_conformance_support.py`.

**Deadline, not urgency:** this must land before the Python-engine-freeze deletion ceremony —
after that ceremony the frozen mirrors become the surviving oracle and extracting shared
helpers against a dead reference is archaeology, not engineering. No freeze date is scheduled
yet. Recommend landing it inside this program's window rather than deferring indefinitely,
since the window is cheap and the deadline is real even if unscheduled.

- R2.2.1 RED: pick one `*_conformance.py` file (from the 23/25 that use
  `ServiceContainer`/`BabylonGraph`) as the first adopter — import from `_conformance_support`,
  fails, module doesn't exist.
- R2.2.2 GREEN: extract the confirmed-boilerplate `ServiceContainer.create()`/try-finally/
  print-header scaffold (verified genuinely shared, not carrying oracle-specific logic, by
  reading two representative files in full) into `run_tick_and_print(system_cls, subjects,
  fields_to_print)`-style helpers; the two standalone-pipeline files stay explicitly out of
  scope.
- R2.2.3 Same migration-scope decision as R2.1.3: pure addition, no mechanical sweep of the
  existing files in this task.
- Commit(s): `refactor(tick): _conformance_support.py — shared Python-mirror helpers ahead of the engine-freeze deadline (theme 6)`.

**Gate (Phase 2 whole):** `cargo test -p babylon-tick` unchanged for existing suites (nothing
migrated); the one adopter test in each language passes through the new shared module;
`qa:regression` byte-identical (test scaffolding only, never production content).

---

## 5. Phase 3 — kind-system completion

**Governance:** refactor-task, entirely within the existing closed kind vocabulary and the
existing E-TYPE-040 code (amendment-free) — per the boundary ruling, semantic coherence of
content belongs IN-LANGUAGE (`typecheck.rs`), never in `bsl-lint`. **Timing:** gated on
`feature/491-rung-ladder` merging — a precondition this program does not own or control.

**Precondition status.** #491 is not a small, clean, about-to-post PR — it is an
8665/-327-line branch across 31 files with no PR opened yet. It bundles the E-TYPE-040
kind-mixing arm (`check_kind_mixing` in `typecheck.rs`, walking `<arith>`/`if` via
`expr_kind`/`add_sub_kind`/`mul_div_kind`/`if_kind`/`fold_kind`, keyed off `field_kind()`
reading `FieldDecl.kind: FieldKind`), a brand-new content pack (`vitality-attrition.bsl`, 369
lines + a 944-line conformance suite), the Currency typed-attribute-storage half
(`CanonicalState` tag `0x06`, kept per §1), and `ADR216_kind_straddle_repair_ceremony.yaml`
(**already accepted** 2026-08-18) documenting four landed-content repairs the arm's first real
run found — including the `previous-wealth` defect the survey's headline example cites.
**Recommendation, not a task this program executes:** the controller should nudge #491 to post
its PR soon — content reads complete (an accepted ADR is a strong completeness signal), and
every day it stays unposted both grows review risk on an 8600+-line diff and blocks this
phase.

### Task R3.1 — gap (a): comparison kinds

**Files:** `typecheck.rs` (`check_one_comparison`, already walking every 3-item comparison
form with `env`/bindings via `classify()` — a DIFFERENT axis, comparable-scalar not
extensive/intensive, so this is composition onto an existing walker, not new machinery).

- R3.1.1 RED: two fixtures. (1) A comparison between an Extensive-kind field and an
  Intensive-kind field that should refuse under E-TYPE-040 but currently loads clean. (2)
  **(critique M-6)** a comparison where one operand is a `:optional :default` binding of
  mismatched kind — gap (a) composes with Phase 0's W2 same-tick-ordering machinery on exactly
  these bindings, and a fixture at the intersection is cheap insurance the two checks don't
  fight each other (e.g. refuse for the wrong reason, or one masking the other's finding).
- R3.1.2 GREEN: wire `field_kind`/`expr_kind` (both built by #491) into `check_one_comparison`'s
  existing walk — roughly 30-40 lines.
- R3.1.3 Audit (§10 — see the corrected report-vs-repair split there): load every landed-on-dev
  + in-flight content set under the widened checker; **189** comparison sites total (corrected
  per critique M-1; measured fresh 2026-08-18, a moving target as content lands — re-count at
  task time); triage findings per §10's protocol.
- Commit(s): `feat(bsl): kind-check comparisons — E-TYPE-040 gap (a) (theme 3)`.

### Task R3.2 — gap (b): store-boundary writes

**Files:** `typecheck.rs` (new static check, sibling to `check_kind_mixing` and
`check_no_arithmetic_on_enum_field` — the precedent this follows, ~35 lines, matching
`update-node`'s `[op, operand]` shape for `add`/`sub`/`scale` against `FieldKind`).

**Reclassified from the survey's framing, confirmed by direct read:** `numeric_write_value`
(`structural_verbs.rs`, confirmed reading `.ty` never `.kind`) is the RUNTIME evaluator
function — the wrong layer to patch. The correct fix is a STATIC sibling to
`check_kind_mixing` that compares each write-verb's RHS `expr_kind()` against its target
field's `field_kind()` at load time. Two complementary counts, both real, answering different
questions: at the Rust SOURCE level, `numeric_write_value` is invoked from 6 call sites across
4 write-verb shapes (`update-node`, `update-edge`, `add-node`, `add-edge`); at the BSL CONTENT
level, ~32 numeric-write sites use these shapes across territory/production/decomposition
against a wider ~95-site arithmetic-write surface the survey describes in aggregate. The
former sizes the code change; the latter sizes the audit.

- R3.2.1 RED: the exact D184 defect class as a fixture — a convex-combination expression typed
  Extensive end-to-end (legal on its own) stored into a field declared `probability
  intensive` — should refuse, currently loads clean.
- R3.2.2 GREEN: the new static check, roughly 150-250 lines (cheaper post-#491-merge than
  pre-merge, since the primitives it needs already exist there).
- R3.2.3 Audit (§10 — corrected protocol, see below): D184's own already-recorded consequence —
  `solidarity/p0-transmit`'s known-bad write, which is ALREADY LANDED ON DEV — is the FIRST
  expected finding, already named, already a repair candidate with no controller triage needed
  (D184 pre-adjudicated it — fix it in-task per §10's landed-on-dev repair path). Load every
  other landed-on-dev + in-flight content set under the new check; landed-on-dev findings
  triage per §10's repair path, in-flight findings on other trains' branches get REPORTED to
  the owning train per §10's read-only path, never repaired here.
- Commit(s): `feat(bsl): store-boundary kind check — E-TYPE-040 gap (b), discharges D184 (theme 3)`.

### Task R3.3 — gap (c): `:default` literal kinds

**Files:** `docs/reference/bsl-language.rst` (§3.5's default-binding rules — one sentence, or a
zero-diff confirmation if the existing prose already covers it by name).

Confirmed by direct read: no `:default`-kind handling exists anywhere in `typecheck.rs`;
literals carry no kind representation outside `ExprKind::Neutral`. The survey's suspicion
holds — this is documentation-only. Confirm at task time whether existing prose already states
"a `:default` literal is kind-neutral by the same rule as any other literal"; add the sentence
only if it's genuinely missing.

- Commit(s): `docs(reference): :default literals are kind-neutral by the literal rule — closes gap (c) (theme 3)`.

**Gate (Phase 3 whole):** `mise run rust:check` green including the two new checks;
`qa:regression` + `qa:vault-regression-ci` byte-identical after any audit-fallout repairs; the
audit table (§10) in the PR body, even if empty for a given sub-check.

---

## 6. Phase 4 — event-schema registry as data

**Governance:** tool (the registry itself) / refactor-task (Python `EVENT_BUILDERS` becoming a
consumer). **Boundary-ruling note:** moving `emit` payload enforcement INTO the grammar (a
`defevent`-style declared construct) crosses into amendment-class per the D101 precedent —
named in QUEUED DIRECTOR RULINGS, not built here. This phase builds data-plus-tooling only.
**Timing:** starts immediately for the registry/CI-check/`EVENT_BUILDERS` work; the client
sub-task is verification-only (see below — it is already done on another train).

**Confirmed facts (2026-08-18, corrects the survey where noted):**

- Python `EventType` has **100 members** (AST-parsed count — CLAUDE.md's "100" is right; the
  survey's "98" does not hold today, likely a regex undercount against multi-line entries).
- There is no Rust `enum EventType` at all, and none is imminent — Rust only has
  `EnumKind::EventType`, one variant of a small discriminator used to grammar-check `emit`'s
  operand against a RUNTIME-declared `ClosedVocabulary`; content declares only the specific
  EventType members it actually uses. The registry this phase builds is genuinely new on the
  Rust side, not an extension of anything partially there.
- `emit` payload-checking gap confirmed exactly: `grammar.rs` checks only that the operand is a
  legal EventType reference; `bound_checker.rs` only fuel/name-staticness; zero `"emit"` hits in
  `typecheck.rs` — payload KEYS are checked nowhere in Rust today.
- `control-ratio.bsl`'s two-shape CONTROL_RATIO_CRISIS payload confirmed exactly by direct
  read: the `enforcer-population > 0` branch emits 8 keys, the `= 0` branch emits 6 (both
  `actual-ratio` and `control-ratio` omitted) — deliberate, D-recorded, invisible to any
  consumer written against one branch.
- `EVENT_BUILDERS` (`src/babylon/engine/event_builders.py:782`) is Python's closest existing
  registry, and an existing `src/babylon/sentinels/fallback_coverage/` sentinel already checks
  ONE facet of it (bus-boundary coverage) — this phase's registry sits beside that precedent,
  not duplicating it.
- **Emit-site count in BSL content is a moving target** (several port trains add `emit` sites
  weekly) — do not hardcode a count anywhere in this task; re-count at execution time.

### Task R4.1 — the registry, tiered by evidence class (rescoped, critique C-1 CRITICAL)

**Files:** a new shared artifact — a Rust const table or a single TOML both toolchains parse
(implementer's call; TOML avoids hand-keeping two hardcoded tables in sync if Python needs to
read it directly).

**Redesigned from this draft's first pass, which was internally incoherent as specced.** A RED
demanding "100 verified entries" cannot be satisfied honestly: BSL content emits only 13
distinct EventTypes (15 emit sites — see the grep-undercount warning below), `EVENT_BUILDERS`
covers ~80 but is itself already known-narrower-than-its-source (the CONTROL_RATIO_CRISIS
9-vs-4 case this draft's own evidence paragraph documents), and the remaining ~20 EventTypes
have no builder and no emit site anywhere. As specced, an implementer would either transcribe
`EVENT_BUILDERS`'s known-incomplete entries as "the declared schema" — manufacturing a false
authority — or invent key lists for EventTypes with no observed emitter, documenting schemas
that exist nowhere. Both violate this estate's verifiability discipline. **The honest shape:
tier every entry by evidence class, and let a completeness sentinel say what's missing instead
of guessing it.**

- **Tier 1 — `verified-bsl`**: EventTypes with at least one observed `emit` site in
  `content/rules/*.bsl`. Row built directly from the content — the strongest evidence class.
- **Tier 2 — `verified-python-builder`**: EventTypes with an `EVENT_BUILDERS` entry but no BSL
  emit site yet. Row transcribes the builder's field set, EXPLICITLY MARKED as inheriting that
  source's own incompleteness — the registry does not claim these are complete schemas, only
  that this is what the builder currently declares.
- **Tier 3 — `no-known-emitter`**: every remaining `EventType` member. NO key list — a bare
  name in a completeness-sentinel list, honestly stating "declared in `events.py`, no emitter
  or builder found as of `<date>`."

- R4.1.1 RED: a test asserting the registry's per-tier COUNTS match a fresh measurement taken
  at authoring time (never hardcoded as "100 verified" — re-run the count, assert against the
  fresh number), and that every Tier 1 row is backed by at least one real citation (`file:line`)
  into `content/rules/*.bsl`. CONTROL_RATIO_CRISIS's Tier 1 entry lists the UNION of both
  observed branches' keys with the two branch-specific keys flagged optional, not required —
  the registry must describe reality (both legal shapes, D-recorded), never force a rename or a
  single shape onto content (port-AS-IS, ADR183 — BSL content payload keys are NEVER renamed by
  this task).
- R4.1.2 GREEN: build Tier 1 from a fresh, CAREFUL pass over every `emit` site — **not the
  obvious `\(emit ` grep** (critique M-3): `solidarity.bsl`'s two emits
  (`CONSCIOUSNESS_TRANSMISSION:254`, `MASS_AWAKENING:360`) use a multi-line `(emit\n
  EventType/…` form that grep undercounts on; confirmed true totals as of this draft are 15
  emit sites / 13 distinct EventTypes — verify fresh at task time, this is a moving target as
  ports land. Build Tier 2 from `EVENT_BUILDERS` for every EventType Tier 1 didn't already
  cover. Tier 3 is the arithmetic remainder against the fresh `events.py` count.
- Commit(s): `feat(bsl): event-schema registry as data, tiered by evidence class (theme 7, C-1 rescope)`.

### Task R4.2 — CI check over emit sites

**Files:** `bsl-lint` (the Phase 0 host) new subcommand. **Classified bsl-lint, not a load
refusal:** an `emit` site's key set matching its EventType's registry entry is
repo-relationship-shaped (comparing content against a declared data table), not a NEW author
declaration and not semantic-coherence-of-one-content-set — this differs from Phase 3's kind
checks, which ARE semantic coherence.

- R4.2.1 RED: a fixture `emit` site with an extra or missing key vs. its registry entry —
  should warn/fail the lint, currently silent.
- R4.2.2 GREEN: the check, reusing the reader per the standing constraint.
- Commit(s): `feat(lint): bsl-lint check for emit-payload key drift vs. the event-schema registry (theme 7)`.

### Task R4.3 — conformance tests generate expected payloads

**Files:** the ~35 Rust conformance tests hardcoding payload literals — **not a mechanical
sweep in this task**, same rationale as Phase 2. Land the GENERATOR (turning a registry entry
into an expected-payload assertion) as a pure addition to `tests/support/mod.rs` from Phase 2
— **explicit dependency: this task sequences after R2.1**, since it extends that module. New
conformance suites adopt it going forward; the existing ~35 stay as they are.

- Commit(s): `feat(tick): expected-payload generator from the event-schema registry (theme 7, rides tests/support)`.

### Task R4.4 — `EVENT_BUILDERS` sync: a ONE-WAY check now, repairs gated (critique C-1 + I-5)

**Files:** `src/babylon/engine/event_builders.py`.

**Redesigned: not a circular "sync."** The original framing risked exactly the false-authority
loop C-1 flags: "repair `EVENT_BUILDERS` to match the registry" is circular when the registry's
own Tier 2 rows were BUILT FROM `EVENT_BUILDERS` in the first place. The correct check is
ONE-WAY — **registry ⊇ builders** — every `EVENT_BUILDERS` entry's field set must be a subset
of its registry row (Tier 1 or Tier 2), never the reverse. This is satisfied by construction
for every Tier 2 row (built from the builder) and needs real verification only where Tier 1
(BSL-observed) and the builder disagree — the survey's own CONTROL_RATIO_CRISIS example.

- R4.4.1 RED + GREEN, lands now: the one-way sync TEST (modeled on
  `test_bsl_grammar_sync.py`'s D-register precedent, sitting beside
  `src/babylon/sentinels/fallback_coverage/`'s existing bus-boundary check) — asserts
  `EVENT_BUILDERS ⊆ registry` for every entry, never the reverse. The check normalizes names
  before comparing — Python builder fields are snake_case, Tier-1 BSL keys kebab-case; the
  test states its normalization rule (`_` ↔ `-`) explicitly so the first run cannot fail on
  spelling convention alone. Safe to land any time — it only reads both sides.
- R4.4.2 GREEN, GATED (critique I-5) — widening a builder to match Tier 1 reality (e.g.
  `control_ratio.py`'s builder from 4 fields toward its source's 9): **do not land this while
  any in-flight port train's Python conformance mirror consumes the changed builder** — the
  mirrors' shared print-boilerplate (Phase 2) includes an events-print block, so a widened
  builder can shift a mirror's printed output mid-port, which is ADR183 port-AS-IS territory,
  not a Phase 4 concern to force. Verify no in-flight mirror depends on the specific builder
  before repairing it, or defer the repair to post-Checkpoint-A entirely — controller's call at
  task time, not decided here.
- Commit(s): `feat(event_builders): one-way EVENT_BUILDERS ⊆ registry sync test (theme 7, C-1/I-5)`; repair commits, if any, land separately and only after the I-5 gate clears.

### Task R4.5 — client node-ref resolution: ALREADY LANDED on `wt-b3`, verification + a costed fast-follow

**Direct verification, 2026-08-18 — supersedes this program's original plan for this task.**
`feature/b3-null-hypothesis-viewer` has already replaced `payload_node_id` entirely:
`loop_ui.rs` no longer defines that function on that branch (confirmed — zero hits for `fn
payload_node_id`; the only remaining trace is a stale comment). In its place, `narration.rs`
declares a static `NARRATION_TABLE` (confirmed full 8 rows: `SUPERWAGE_CRISIS→receiver`,
`CLASS_DECOMPOSITION→source-class`, `CONTROL_RATIO_CRISIS→None`, `TERMINAL_DECISION→None`,
`LIFECYCLE_TRANSITION`/`LEGITIMATION_CRISIS`/`LEGITIMATION_RECOVERY→territory-id`,
`ENTITY_DEATH→entity-id`) — a DECLARED table, not a guessed match, per that file's own header
("never guessed"), deliberately scoped to "the two shipped stories," not an oversight. This
independently converges on the same fix this phase would have built, via an arguably better
shape — this program does not need to build a competing fix.

**The residual gap is now CONFIRMED, not hypothesized (critique Part 4, answering this draft's
own §13 Q3).** Cross-referencing the table against every landed BSL emit payload: `territory-id`
(`lifecycle.bsl:389,402,406`), `entity-id` (`vitality.bsl:97`), `receiver`
(`decomposition.bsl:263`), `source-class` (`decomposition.bsl:378`) all resolve. Three do NOT,
because their EventTypes have no `NARRATION_TABLE` row at all, not because of a matching gap:
`source-id`/`target-id` (`solidarity.bsl:256,257,362` — `CONSCIOUSNESS_TRANSMISSION` and
`MASS_AWAKENING`) and bare `territory` (`dispossession.bsl:402,413` — `VALUE_TRANSFER` and
`DISPOSSESSION_EVENT`). **The node-ref spelling census is SEVEN, not six** — `receiver` is a
real seventh spelling this program's early research (inherited from the survey) missed; R4.1's
registry and this task's audit both use the measured seven-spelling list.

- R4.5.1 At `wt-b3`'s merge time (not before — this program does not open a parallel task on a
  file that branch has substantial live, uncommitted work on): confirm `NARRATION_TABLE` is
  present, wired into the event-feed render path, unchanged from the 8 rows read above.
- R4.5.2 COSTED fast-follow (was uncosted, critique M-4): after B3 merges, add ~4 new
  `NARRATION_TABLE` rows (`CONSCIOUSNESS_TRANSMISSION`, `MASS_AWAKENING`, `VALUE_TRANSFER`,
  `DISPOSSESSION_EVENT`) plus their subject templates, respecting the parity-guard's
  one-row-per-line contract. Small, real, separately costed in §12.
- R4.5.3 Forward-looking recommendation, not a task: once R4.1's registry exists,
  `NARRATION_TABLE` is a natural future consumer of it (one declared source of truth instead of
  two independently-maintained tables) — named here so the connection isn't lost, not scheduled
  as work in this program.
- Commit(s): none from this program for R4.5.1 (verification only); R4.5.2's fast-follow rows commit as `feat(client): NARRATION_TABLE rows for the four subject-less post-B3 EventTypes (theme 7 fast-follow)`.

**Gate (Phase 4 whole):** `mise run rust:check` + the Python leg green; the new lint check
clean against current content (or its findings triaged per §10); `qa:regression` byte-identical
(this phase adds tooling and a Python builder sync, never renames a BSL content key, so zero
pin risk by construction — prove it anyway).

---

## 7. Phase 5 — theme-8 remainder (promotion-mismatch + empty-query visibility)

**Governance:** both are ADVISORY HEURISTICS against the boundary ruling, not hard invariants
(neither prevents a load; both make visible something the checker already half-knows), so both
are **bsl-lint warn tier**, not load refusals. **Timing:** depends on Phase 0's `bsl-lint` host
existing — already true in practice by the time this phase starts.

**Note on scope already closed by Phase 0:** theme 8's item (2), "one canonical authoring
idioms reference section," is `wt-cookbook`'s W4 — already done. This phase does not re-touch
it.

### Task R5.1 — promotion-mismatch lint

**Files:** `bsl-lint` new check.

**Resolved by the critique (§13 Q2), where this draft's first pass left it an open
investigation.** A per-node Int/Real classifier DOES exist, but only in the aggregation path —
`typecheck_aggregation` types `BslType::Int`/`Real`/… per-node (`typecheck.rs:207`,
`FoldOp::Count => Ok(BslType::Int)`; tests at `:655-684` assert the Int/Real/Intensity
distinctions). General arithmetic does NOT reuse this — it routes through
`score_class::classify`, which deliberately collapses `Int`/`Real`/`Currency`/`Coefficient`
into one `ScoreClass::Scalar` (`score_class.rs:92,164`) — the exact coarseness PR #493's own
commit message blamed for the near-miss this lint targets. **This task needs a small NEW
promotion-aware walker over arithmetic forms** (the rule: `Int op Int` stays `Int`; any genuine
binary64 operand promotes the whole form) — not a reuse of `score_class::classify`, but not
machinery invented from scratch either: the aggregation path's per-node typing is the pattern
to copy. R5.1.1's investigation step now collapses to confirming this framing rather than
discovering it from nothing; the 0.15 Mtok estimate holds.

- R5.1.1 Confirm the framing above against the live `typecheck_aggregation`/`score_class.rs`
  code at task time (cheap — the critique already did the discovery); do not re-derive from
  scratch.
- R5.1.2 RED: a fixture using `(- 1 0)` where `(- 1 0c)` was clearly intended (the exact PR
  #493 near-miss shape, which also saw a wrong Copilot-suggested `1c` fix land before the
  correct form was found) — should warn, currently silent.
- R5.1.3 GREEN: the new promotion-aware walker, per the framing above.
- Commit(s): `feat(lint): bsl-lint promotion-mismatch check for the Int/Real idiom (theme 8)`.

### Task R5.2 — empty-query visibility lint

**Files:** `rust/crates/bsl-lint/` new check, walking the reader's own output directly — like
Phase 0's citation check — **NOT `typecheck.rs`** (corrected per critique I-2).

**The mechanism correction.** This draft's first pass said the check "slots into
`check_one_selection`" (`typecheck.rs:460`) — but that function is load-REFUSAL machinery with
no warn channel, and the critique's own research confirms all 14 unguarded sites are ONE
already-D-recorded pattern (decomposition.bsl's single `carceral-register` INSTITUTION
carrier, D166) — a refusal there would break 14 documented-safe sites, contradicting this
check's own advisory-warn classification. An advisory check cannot live inside refusal
machinery without either becoming a refusal (wrong) or `typecheck.rs` growing new
non-refusing side-channel plumbing (unscoped). Fix: `bsl-lint` walks the S-expression reader's
output itself, exactly like the citation-drift check, entirely outside `typecheck.rs`.

**Precise measured inputs (critique Part 5):** 25 `select-max`/`select-min` sites total — 11
exists-guarded (production's 10 + territory's 1) and 14 carrier-assumed (all decomposition, the
D166 pattern above); `select-min` has ZERO landed uses as of this draft.

- R5.2.1 RED: a fixture `select-max` site with no guard and no singleton-carrier-node
  justification comment — should distinguish "checked-safe" (guarded, or the carrier-node
  pattern documented, like D166) from "assumed-safe" (neither) in its output; currently
  invisible.
- R5.2.2 GREEN: the check, roughly 40-60 lines, reusing the reader directly (per the standing
  constraint), never touching `typecheck.rs`.
- Commit(s): `feat(lint): bsl-lint checked-safe vs. assumed-safe visibility for select-max/select-min (theme 8, critique I-2 corrected)`.

**Gate (Phase 5 whole):** both new lint checks run clean (or warn, never fail the build — these
are advisory) against current content; `mise run rust:check` green.

---

## 8. Phase 6 — `scenario.rs` split (LAST)

**Governance:** refactor-task. **Timing:** GATED on `feature/491-rung-ladder` AND
`feature/652-bsl-ls` AND `feature/community-port-bsl` ALL merging to `dev`. This is the single
most collision-prone item in this program and is sequenced deliberately last among the
content-adjacent phases.

**Confirmed structure (direct read, 2026-08-18, corrected per critique M-2 — the original
174/140/118 figures were naive next-`fn`-gap measurements that count intervening doc comments
and structs, not the function bodies themselves):** `scenario.rs` is 4,044 lines (production
~1,848 / test 2,229). By closing-brace span, the actual measure: **two** functions exceed the
~100-line guideline — `load_scenario_inner` (155 lines; the file's own `#[allow]` comment names
this, the 7-arm dispatcher: `defenum`/`defvocabulary`/`deffield`/`defconst`/`node`/`edge`/
`edge-attr`) and `load_edge_attr` (101). `load_edge` (98) and `load_node` (89) do not, by this
measure. **The split is still justified** — 155 is real, and the file is 4,044 lines regardless
of exactly how many individual functions cross the guideline — but the evidence sentence
overstated it 4-for-2, corrected here. **The "about to take an 8th arm" is no longer future
tense** — `feature/community-port-bsl` has already landed `(hyperedge …)` as the 8th top-form,
ahead of this program.

`attribute_value` and `load_deffield` confirmed to independently restrict themselves to the
identical 7-of-10 `BslType` variants, held only by comment, not the compiler. `reader.rs`'s
`SExpr`/`Atom` confirmed to carry no span field — **and this is no longer an open design
question for this program**: `feature/652-bsl-ls` has already built and committed a span side
table, choosing it over a struct-field retrofit after evaluating three candidates.

**The shared-helper-extraction question, resolved by direct re-verification (correcting this
draft's own earlier assumption):** `structural_verbs.rs`'s `fresh_declared_name` genuinely
checks both `self.declared_nodes` and `self.declared_hyperedges`, but that is the RUNTIME
effect-execution layer (`add-node`/`add-hyperedge` EFFECTS during rule evaluation), not
`scenario.rs`'s LOAD-TIME declaration path. Direct read of `scenario.rs` on
`feature/community-port-bsl` confirms the load-time `(hyperedge …)` handler hand-rolls its
OWN, separate `seeded_hyperedge_names: HashSet<String>` (`scenario.rs:586,669,1913,1929,1945`)
— the code's own comment names it "a SEPARATE table from `named`" (the node-side load-time
check). **The shared-helper-extraction opportunity at load time is therefore real and not yet
discharged anywhere** — Task R6.2 below extracts a shared refusal/identity module covering
BOTH `scenario.rs`'s load-time declaration checks (nodes, edges, hyperedges) as one of its
concrete deliverables, not merely "a pattern to note."

### Task R6.1 — trigger condition (not a calendar date)

This program does not schedule a start date for Phase 6. Its start condition is: all three of
`feature/491-rung-ladder`, `feature/652-bsl-ls`, `feature/community-port-bsl` have merged to
`dev`. Re-verify Community's later (held) tasks' status before executing — this program's
evidence is current as of 2026-08-18 and this task may not execute for some time.

### Task R6.2 — the split

**Files:** `scenario.rs` → split by top-form (one module per major dispatch arm — declarations,
element-seeding, a shared refusal/identity module) — exact module boundaries decided at
execution time against whatever the three merged branches actually leave in the file, not
pre-specified here (pre-specifying boundaries against a file three other trains are actively
editing would be planning against a fixture).

- R6.2.1 The existing 2,229-line test suite is the safety net — no new test content required
  for a pure structural split; the bar is 100% pass, byte-identical behavior throughout.
- R6.2.2 GREEN: the split, building ON TOP of `feature/652-bsl-ls`'s span side table (already
  landed by the time this phase starts) rather than re-deriving position-threading. Extract the
  shared load-time refusal/identity module named above — a single duplicate-name check reused
  across the node, edge, and hyperedge declaration paths, replacing `seeded_hyperedge_names`'s
  hand-rolled set and its node-side counterpart with one function. Re-export the public surface
  unchanged (`load_scenario`, `LoadedScenario`, `ScenarioError`) so no downstream caller needs
  an import change.
- R6.2.3 Prove byte-identity: `qa:regression` + `qa:vault-regression-ci` byte-identical — a
  pure module reorganization must not move a single hash.
- Commit(s): one per module extracted, ending with `refactor(bsl): split scenario.rs by top-form + shared load-time refusal/identity toolkit (theme 9)`.

**Gate (Phase 6 whole):** `mise run rust:check` green; `qa:regression` +
`qa:vault-regression-ci` byte-identical; no function newly exceeds the ~100-line guideline
post-split.

---

## 9. Closure items (process, not new engineering)

### Task R7.1 — nudge #491 toward posting; ImperialRent needs a different nudge (corrected, critique I-4)

Not a task this program executes directly, but recorded as a recommendation. **#491:** reads
content-complete (§5's precondition status — an accepted ADR216 is a strong completeness
signal); recommend posting now. **ImperialRent — corrected from this draft's first pass, which
wrongly called it "content-complete, awaiting posting":** it is mid-Task-4 of its own PR A,
5 of 10 specced rules written, with Task 4's closing step (the TRPF-decay mutation world +
inflow golden pins) unrun as of this revision. The accurate ask is **"finish Task 4's closing
step, then post PR A"** — not "post now." A merge-order table that called this train
content-complete would have invited closing it at PR A with half its rules unwritten.

### Task R7.2 — sweep-class incidentals

**Evidence.** A stale method-count claim in a `lib.rs` doc comment (survey theme 9's closing
note); `lifecycle.bsl:1070`'s stale self-citation, already seeded as Phase 0's citation
sentinel's known-true-positive.

- R7.2.1 Fix the stale count in the `lib.rs` doc comment.
- R7.2.2 Confirm (don't fix — that's Phase 0's task) that Phase 0's citation check carries an
  allowlist entry for `lifecycle.bsl:1070`; if it doesn't yet, flag it back to the hygiene lane
  rather than duplicating the check here.
- Commit(s): `docs(bsl): fix stale method-count comment (sweep-class incidental)`.

---

## 10. Audit-fallout protocol (applies to every phase that adds a new refusal or check)

Mirrors Phase 0's own precedent (repair content, never weaken the arm) — **corrected per
critique I-3, which caught a real contradiction: this program's own Global Constraints declare
every in-flight train branch read-only, but the first pass's step 3 directed repairing findings
"in-task" regardless of whether the content lived on dev or on someone else's branch.** Fixed:

1. The new check/refusal lands (Phase 3's gaps a/b are the primary generators here — see
   below for which phases don't generate one).
2. Load every landed-on-`dev` content set AND every in-flight (other-train-branch) content set
   under it — the audit's SCOPE is still everything; only the disposition below now depends on
   WHERE the content lives.
3. **Landed on `dev`, mechanical + pin-safe** (a rename, a citation fix, a trivial guard, a
   repair matching an already-adjudicated D-record like D184) → repair in-task; prove pins
   byte-identical.
4. **Landed on `dev`, semantic** (a genuine transcription ambiguity, a case the D-register
   hasn't ruled on) → STOP, full inventory table in the task/PR report, escalate to
   controller/Director triage. Never weaken the check to make a finding disappear.
5. **In-flight, on another train's branch — ANY finding, mechanical or semantic** → this
   program's read-only constraint governs: **REPORT to the owning train** (an issue comment or
   lane message naming the exact site and violation), recorded in this task's own audit table
   as `reported-not-repaired`. Never edit another train's branch, even for an
   obviously-mechanical fix — the owning train repairs its own content, informed by this
   report.
6. Record the audit table in the landing commit/PR body — even an EMPTY table, so a reviewer
   sees the audit ran rather than was skipped. The table's disposition column now has three
   values, not two: `repaired`, `stopped-for-triage`, `reported-not-repaired`.

**Which phases actually generate an audit obligation:** unchanged from this draft's first
pass — Phase 3 (gaps a and b) is the primary case; Phase 4's registry/CI-check does not
generate one (built to describe existing reality, including the deliberate
CONTROL_RATIO_CRISIS two-shape case, never to force a rename, per ADR183's port-AS-IS
constraint — a lint finding there is advisory); Phase 6 (the split) is a pure refactor with no
new semantic check — its only obligation is the mechanical byte-identity proof. Phases 1, 2, 5
touch no BSL content semantics at all.

---

## 11. PR structure and merge order

One PR per phase-task-group, following the house PR-A/PR-B handoff convention (ADR211/212/213
precedent) where a phase naturally splits that way.

| # | PR | Phase | Gated on | Notes |
|---|---|---|---|---|
| 1 | `chore(rust): sccache + profile.dev + CARGO_BUILD_JOBS` | R1.1 | nothing | first to land, acute motivation |
| 2 | `feat(tooling): wt:rust-clean` | R1.2 | nothing | independent, small |
| 3 | `refactor(graph): CanonicalState tags position-derived array` | R1.3 | **Phase 0's W1 AND #491 merged** (critique I-1) | fast-follow, same file, six entries |
| 4 | `refactor(tick): tests/support/mod.rs` | R2.1 | nothing | pure addition |
| 5 | `refactor(tick): _conformance_support.py` | R2.2 | nothing | pure addition, deadline-bound |
| 6 | `feat(bsl): kind gaps a+b+c + audit` | R3.1-R3.3 | **#491 merged** | recommend nudging #491 to post first |
| 7 | `feat(bsl): event-schema registry + CI check + EVENT_BUILDERS sync` | R4.1-R4.4 | nothing | |
| 8 | `feat(tick): expected-payload generator` | R4.3 | PR #4 (R2.1) merged | extends tests/support |
| 9 | — verification note only, no PR from this program | R4.5 | `wt-b3` merged | fix already exists there |
| 10 | `feat(lint): promotion-mismatch + empty-query checks` | R5.1-R5.2 | Phase 0's W1 merged | |
| 11 | `refactor(bsl): scenario.rs split` | R6.1-R6.2 | **491 + 652 + community all merged** | last, exclusive lane |

**External, not this program's PRs, but named because they gate it:**

- `feature/491-rung-ladder` — recommend posting now; gates PR #6 and is one of three gates on
  PR #11.
- `feature/imperialrent-port-bsl` — mid-PR-A (Task 4's closing TRPF-decay step is unrun and
  held; 5 of 10 specced rules landed): post PR A only after Task 4 closes it (§1's corrected
  row and R7.1 govern); it does not gate anything in THIS program directly.
- `feature/652-bsl-ls`, `feature/community-port-bsl` — their own trains; gate PR #11 only.
- `feature/b3-null-hypothesis-viewer` — its own train; triggers R4.5's verification, not a
  gate on any of this program's own PRs.
- `feature/tickdynamics-port-bsl` — early-stage; does not gate anything in this program.

---

## 12. Estimate (Mtok, house convention — see `2026-08-17-576-intrinsic-host.md` §8)

| Phase | Task | Mtok |
|---|---|---|
| 1 | R1.1 sccache/profile/jobs (incl. M-5 global-entry retirement) | 0.08 |
| 1 | R1.2 wt-rust-clean | 0.05 |
| 1 | R1.3 tag array (position-derived, six entries, gated on #491 — I-1/I-7) | 0.12 |
| 2 | R2.1 tests/support/mod.rs | 0.30 |
| 2 | R2.2 _conformance_support.py | 0.20 |
| 3 | R3.1 gap (a), incl. M-6 fixture | 0.15 |
| 3 | R3.2 gap (b) | 0.30 |
| 3 | R3.3 gap (c) | 0.02 |
| 3 | audit (both gaps) — **a bet, not a measurement; a >3-finding audit blows this number** | 0.15 |
| 4 | R4.1 registry, tiered (C-1 rescope) | 0.25 |
| 4 | R4.2 CI check | 0.10 |
| 4 | R4.3 payload generator | 0.10 |
| 4 | R4.4 EVENT_BUILDERS one-way sync test only (I-5 split; repairs deferred, uncosted here) | 0.08 |
| 4 | R4.5 verification | 0.02 |
| 4 | R4.5 fast-follow — 4 NARRATION_TABLE rows (M-4, was uncosted) | 0.08 |
| 5 | R5.1 promotion-mismatch (Q2 answered — investigation shortened) | 0.15 |
| 5 | R5.2 empty-query visibility (I-2 — bsl-lint, not typecheck) | 0.10 |
| 6 | R6.2 scenario.rs split + load-time helper extraction (re-centered per critique: 652's comparable Task 1 cost 0.50 for one side table/one file; this is bigger) | 0.75 |
| **Total** | | **~3.00** |

**Re-priced from rev 1's ~2.62** per the adversarial critique's Part 6 sanity check (C-1's
honest registry pricing, R4.4's repair-half deferral, R6.2's re-centering, and the smaller
I-1/M-4/M-5 adjustments) — landing at ~3.00, inside the critique's own ~2.9-3.2 estimate. Still
comparable in scale to the intrinsic-host train (~2.15 Mtok, similarly multi-part, smaller
scope). Revise at each phase's closeout per the same convention; Phase 6's estimate still
carries the widest error bars since its scope depends on what three other trains leave behind,
and the audit line (Phase 3) is explicitly a bet, not a measurement.

---

## 13. Unresolved questions (not decided in this document)

1. **R1.2's exact shape** — new `mise` task vs. documented manual step for worktree
   cargo-cleaning. Controller sign-off needed since it touches `worktree_tool.py`'s territory
   even though it doesn't edit that file.
2. ~~R5.1's premise~~ **RESOLVED (critique Q2):** yes, but only in the aggregation path
   (`typecheck_aggregation`, `typecheck.rs:207,655-684`); general arithmetic collapses through
   `score_class::classify` into `ScoreClass::Scalar`. R5.1 needs a small new promotion-aware
   walker, not a reuse — see its rewritten section.
3. ~~R4.5's residual coverage~~ **RESOLVED (critique Q3):** confirmed NOT fully closed —
   `source-id`/`target-id`/bare `territory` do not resolve (4 landed emitting EventTypes have
   no `NARRATION_TABLE` row); the spelling census is SEVEN, not six. See R4.5's rewritten
   section and its now-costed fast-follow (R4.5.2).
4. **Posting timing for #491 and ImperialRent** — recommended, not commanded; outside this
   program's authority. ImperialRent's specific ask is corrected in R7.1 (critique I-4): finish
   Task 4's closing step, then post — not "post now."
5. **Next-free ADR/E-LOAD/E-TYPE/CanonicalState-tag numbers at each task's actual execution
   time** — deliberately NOT hardcoded anywhere in this plan, given the live `0x06` incident.
   Every task must re-check against Phase 0's namespace-unique check (once landed) at execution
   time, not against numbers current as of this draft.
6. **Exact module boundaries for the `scenario.rs` split (R6.2)** — deliberately left
   undecided, to be set against whatever 491/652/community actually leave in the file, not
   against today's shape.
7. ~~Is the resumption gate too small?~~ **RESOLVED (critique Q7, and I-6's rider):** no — keep
   the gate exactly as decided (Phase 0 only); the risk is concentrated in one train
   (TickDynamics) and the free kind-sweep rider (§2) mitigates the gap-(a) class at zero gate
   cost. Gap (b)'s store-boundary class is NOT covered by the rider — that residual is
   accepted, not mitigated, and stays the honest remaining bet behind the gate decision.

---

## 14. DIRECTOR RULINGS — both RULED at the popup sitting of 2026-08-18 (~19:30 EDT)

- **`defevent` — RULED: AMENDMENT PATH APPROVED** (Director chose over the deferred-to-WS1
  recommendation). Disposition: a new gated task **R4.6** — draft the constitutional
  amendment (next free letter) + its ADR for a grammar-level declared event-schema top-form,
  GATED on R4.1's registry landing (the registry's three tiers are the amendment's evidence
  base: the schema data must exist and prove itself before the grammar enshrines it).
  Drafting is workforce work; RATIFICATION is the Director's, at a dedicated sitting with the
  D101 precedent in the papers. Phase 4's data/tooling remains exactly as specced — the
  registry is now also the amendment's substrate, which raises the bar on its tier honesty
  (C-1's redesign), not its scope.
- **`bevy/dynamic_linking` — RULED: APPROVED WITH FENCE.** Disposition: folded into Phase 1
  as **R1.4** — an OPT-IN dev profile (mise task/alias enabling the feature for local
  babylon-client iteration), plus the fence as CODE: a CI-side guard asserting the feature
  never appears in any CI, release, or pin-ceremony build (a workflow-visible check, not a
  comment). Pins are runtime hashes and unaffected either way; the fence exists so the .so
  never rides a shipped or gate-evidentiary binary.
- **Out of this program's scope, tracked elsewhere:** the `Real × Ratio` operator gap (survey
  watch-only; already named as a #502/WS3-adjacent item, `territory.bsl:33-36`); the
  emergence-audit sentinel for logistic-shaped subexpressions (already filed as its own issue,
  #659 — not duplicated here); the full #502 WS3 charter ceremony (still gated on "last system
  port lands," per #502's own sequencing constraint — this program pulls forward specific
  ready items, it does not substitute for that ceremony).
