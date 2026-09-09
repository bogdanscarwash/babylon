# Program 28 Kickoff Implementation Plan — Amendment AF + Ratatui Deletion + Bevy B0

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ratify Amendment AF (Bevy supersedes the Ratatui client), delete the Ratatui estate, and stand up the `babylon-client` Bevy scaffold (milestone B0) linked in-process to the Rust engine.

**Architecture:** Three sequential phases in three PRs. Phase A writes the constitutional act (docs-only, **Director-ratified — never self-merge**). Phase B executes the deletion ceremony the amendment authorizes (Rust crates, Python periphery, packaging, CI). Phase C adds the `babylon-client` Bevy crate to the `rust/` workspace, proving the engine seam by running one deterministic tick at startup and logging the byte-pinned state hash.

**Tech Stack:** Bevy 0.18 (resolves the spec §8 open question — current stable per docs.rs, feature-profile system), Rust workspace at `rust/` (toolchain 1.91.1, workspace floor 1.87), Python 3.12 + uv for the surviving periphery.

**Source spec:** `docs/superpowers/specs/2026-08-10-program-28-bevy-cutover-roadmap-design.md` (Director-approved, rulings R1–R10).

## Global Constraints

- Branch from `dev`; conventional commits; every commit ends with `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- **The Phase A PR is a director-gate: STOP after opening it. Only the Director merges a constitutional amendment.** Phases B and C are implementation PRs — the standing autonomy rulings license green-gate self-merge.
- Worktree execution recipe (scar class #2): symlink `.venv` from the main checkout, copy `data/` and `.env`, commit with `PYTHONPATH="$PWD/src"`. An intentional `uv.lock` change commits with `SKIP=worktree-contract` and WITHOUT `UV_FROZEN=1` (it conflicts with the `uv lock --check` hook).
- Gates: `mise run check` for every phase; `mise run rust:check` for any `rust/` change; Phase B additionally runs `mise run qa:regression` and `mise run qa:vault-regression-ci` (byte-identical — any drift is a STOP, not a ceremony).
- Vale: run `vale <file>` on every Markdown/prose doc touched; fix errors to 0.
- Palette canon (§9b, source of truth `src/babylon/render/tiers.py::TRUECOLOR_PALETTE`): FIELD `#1a0000`, TEXT/BONE `#e8e8e8`, CRIMSON `#dc143c`, GOLD `#ffd700`, DIM `#404040`, MUTED_DARK `#202020`, ROYAL `#4169e1`, GREEN_DARK `#228b22`.
- Constitution version: 3.0.0 → **3.1.0** (MINOR — registers an amendment; precedent: AC at v2.17.0 was MINOR). Recording ADR: **ADR186** (ADR185 is the last occupied number — verify before writing).
- No new mathematics anywhere in this plan (Amendment AE (ii) stands; spec §6).

## File Structure

| Phase | File | Action | Responsibility |
|---|---|---|---|
| A | `CONSTITUTION.md` | Edit | Amendment AF text + version-change block |
| A | `ai/decisions/ADR186_bevy_cutover_amendment_af.yaml` | Create | Recording ADR incl. sentinel disposition table |
| A | `ai/decisions/index.yaml` | Edit | ADR186 catalog entry |
| A | `CLAUDE.md`, `NORTH_STAR.md` | Edit | Constitutional sweep (client status, glyph floor) |
| B | `rust/Cargo.toml`, `rust/crates/babylon-{tui,tui-python,md}/`, `rust/pyproject.toml` | Delete/Edit | Rust estate deletion |
| B | `pyproject.toml`, `uv.lock`, `flake.nix`, `.mise.toml` | Edit | Wheel leaves packaging |
| B | `src/babylon/tui/`, `src/babylon/cli/play.py`, client-bound tests/sentinels | Delete/Edit | Python periphery deletion |
| B | `.github/workflows/ci.yml` | Edit | build-wheel leg removal |
| C | `rust/crates/babylon-tick/src/lib.rs` | Create | `run_once` seam shared by driver + client |
| C | `rust/crates/babylon-client/` | Create | Bevy B0 scaffold (main, palette, assets, tests) |
| C | `tests/unit/render/test_rust_theme_parity.py` | Re-create | Parity guard re-pointed at `palette.rs` |

---

## Phase A — Amendment AF (docs PR, director-gate)

### Task 1: Amendment AF text in CONSTITUTION.md

**Files:**
- Edit: `CONSTITUTION.md` (version-change block near line 24; Amendments section — insert after Amendment AE, which starts at line 658)

**Interfaces:**
- Produces: the ratified constitutional text every later task cites; clause numbers (i)–(viii) referenced by ADR186 and the deletion PR body.

- [ ] **Step 1: Add the version-change block entry** at the TOP of the version log (above the `2.18.0 → 3.0.0` entry, matching its exact format):

```text
================================================================================
Version Change: 3.0.0 → 3.1.0 (2026-08-10)
Bump Rationale: MINOR — Amendment AF (The Bevy Cutover) registered,
  Director-ratified (rulings R1–R10, 2026-08-10, in-session). Bevy replaces
  the Rust/Ratatui terminal client outright (Textual-deletion precedent);
  Amendment AC superseded in full and AE clause (xi) superseded; the ADR099
  glyph floor retires; the topology/hypergraph/Sankey visualization
  obligations transfer to Bevy scenes. The Ratatui estate is deleted by
  declared ceremony; the maturin wheel leaves the default dependency set.
  Recording ADR: ADR186. Design: docs/superpowers/specs/
  2026-08-10-program-28-bevy-cutover-roadmap-design.md.
```

- [ ] **Step 2: Insert the amendment paragraph** in the Amendments section, directly after the Amendment AE paragraph (after line 658), as one bold-headed paragraph in house style:

```markdown
**Amendment AF — The Bevy Cutover (Bevy Client, v1.0)** (ratified v3.1.0):
**Bevy replaces the Rust/Ratatui client outright** — Textual-deletion
precedent: no deprecation window, no dual-client period. Operative clauses:
**(i) the client** — the v1.0 client is a standalone **Bevy** (0.18 line)
executable crate `babylon-client` in the in-tree `rust/` cargo workspace;
engine crates link in-process; the shipped game is a **pure Rust binary**
with no PyO3 in the play path (Python remains the data-build pipeline and
the out-of-process AI observer, exactly where Amendment AE put it); v1.0
visual scope is the 2D county-map game with selective 3D moments (Patches
the tutorial guide; the topology view). **(ii) supersessions** — Amendment
AC is superseded **in full**; Amendment AE clause (xi) is superseded; the
ADR099 glyph floor and NORTH_STAR invariant 3's "fully playable glyph-only
over ssh" retire as binding requirements. The topology / hypergraph /
value-flow-Sankey visualization obligations **transfer to Bevy scenes** —
the obligations survive, the renderer changes. AC clause (ii)'s substance
carries forward verbatim: the client is a presentation-only viewport over
`observe()`-projection shapes (II.8/Amendment V holds), clients remain
disposable, and this amendment designates an implementation, not a
primitive. **(iii) deletion ceremony** — this amendment deletes the Ratatui
estate outright: workspace crates `babylon-tui`, `babylon-tui-python`, and
`babylon-md`, the maturin wheel packaging (`rust/pyproject.toml`), and the
Python client periphery that boots it; the wheel leaves the default
dependency set, so `uv sync` no longer requires cargo (which also removes
the #463 CI-timeout lane's trigger surface). **(iv) packaging** — `babylon
play` retires from the play path; the game launches as a normal binary; the
Python CLI survives only for the data-pipeline and observer periphery.
**(v) carried assets and contracts** — the §9b crimson/gold/near-black
palette (source of truth `render/tiers.py::TRUECOLOR_PALETTE`, with the
cross-language parity guard re-pointed at `babylon-client`), the Iosevka
type direction, the SFX suite (ADR152) and soundtrack (ADR153) porting to
Bevy's audio/asset system, and the ADR175 structured-JSONL engine log sink
all carry; the client log contract re-points from `rust-client.log` to the
Bevy client's log when its file sink lands (milestone B2). **(vi) gate
transfer** — AC (iii)'s tutorial-BDD parity condition transfers to the Bevy
client: the tutorial arc passing against the Bevy client is the
constitutional correctness/parity condition before v1.0 (the `TutorialStep`
script is the preserved content source; the harness is rebuilt Bevy-side),
and a Director eyes-on session in the Bevy client replaces the retired TUI
campaign gate (#262). **(vii) sentinel continuity (IX.5)** — client-bound
sentinel families receive declared dispositions recorded in ADR186: the
theme-parity guard PORTS to `babylon-client`; the raster-ban and
tutorial-pilot families RETIRE with the estate they guarded; tutorial
CONTENT checks are retained. No silent lapse. **(viii) hyperedge formalism
note** — implementing `HyperedgeSet` / `NodeRef` / `EdgeRef` /
`HyperedgeRef` in `BslType` mints no new mathematics: `bsl-language.rst`
§2.6 already specifies the element/result table, so it does not extend the
formalism surface under AE (ii); this recording closes the open question on
task #50, and the 2026-07-31 hyperedge-lane pause lifts (R5).
Windows-impact note (AA duty): Bevy/wgpu improves native-Windows
feasibility versus the terminal raster tiers; no new foreclosure entries.
Source: Director rulings R1–R10 (2026-08-10, in-session), design
`docs/superpowers/specs/2026-08-10-program-28-bevy-cutover-roadmap-design.md`;
recording ADR186. Director-ratified 2026-08-10.
```

- [ ] **Step 3: Update the header references to the amendment range.** Search `rg -n 'A–AE|A-AE' CONSTITUTION.md` and extend any "Amendments A–AE" range to "A–AF" (the doc header and any preamble line).

- [ ] **Step 4: Verify no stale count**: `rg -n 'AE\b' CONSTITUTION.md | head` — confirm no line claims AE is the latest amendment.

- [ ] **Step 5: Commit**

```bash
PYTHONPATH="$PWD/src" git commit -m "docs(constitution): Amendment AF — the Bevy Cutover (v3.1.0)" # + trailer
```

### Task 2: ADR186 + index entry

**Files:**
- Create: `ai/decisions/ADR186_bevy_cutover_amendment_af.yaml`
- Edit: `ai/decisions/index.yaml` (append after the ADR185 entry)

**Interfaces:**
- Consumes: Amendment AF clause numbers from Task 1.
- Produces: the sentinel disposition table Phase B cites when deleting sentinel families.

- [ ] **Step 1: Verify ADR186 is free**: `ls ai/decisions/ | rg 'ADR186'` — expect empty. If occupied, take the next free number and update every reference in Tasks 1–3.

- [ ] **Step 2: Write the ADR** following the ADR185 house format (`status` / `date` / `title` / `context` / `decision` / `consequences` keys). The `decision` block records rulings R1–R10 verbatim from the spec §2 table, the Amendment AF clause list, and this **sentinel disposition table** (IX.5 continuity — required by AF clause (vii)):

```yaml
    sentinel_dispositions:
      theme_parity_guard:            # tests/unit/render/test_rust_theme_parity.py
        disposition: PORTED
        target: rust/crates/babylon-client/src/palette.rs (re-pointed regex)
      raster_ban_family:             # tests/unit/sentinels/test_raster_bans.py
        disposition: RETIRED
        reason: guards the ratatui/hypergraph-rs dependency wall of AC (vi);
          the guarded estate is deleted by AF (iii)
      tutorial_pilot_family:         # tests/unit/tui/test_tutorial_pilot_rs.py + FFI/map smoke
        disposition: RETIRED
        reason: harness of the deleted client; the parity CONDITION transfers
          to the Bevy client per AF (vi), harness rebuilt at B2/B3
      tutorial_coverage_family:      # src/babylon/sentinels/tutorial_coverage/
        disposition: RETAINED_CONTENT_ONLY
        reason: content checks over the TutorialStep script survive; legs that
          import babylon_tui are stripped with the client
```

- [ ] **Step 3: Append the index.yaml entry** matching the ADR185 entry shape (`title` / `status: accepted` / `date: '2026-08-10'` / `file`).

- [ ] **Step 4: Commit** (`docs(adr): ADR186 — Amendment AF recording + sentinel dispositions`).

### Task 3: Constitutional sweep — CLAUDE.md + NORTH_STAR.md

**Files:**
- Edit: `CLAUDE.md` (Constitutional Compact amendment list; "Client status" paragraph; client-logs paragraph; the `uv sync`/cargo notes in the Architecture section)
- Edit: `NORTH_STAR.md` (invariant 3 at lines 237–238; the kitty-raster reference near line 140)

**Interfaces:**
- Consumes: Amendment AF text (Task 1).

- [ ] **Step 1: CLAUDE.md Constitutional Compact** — extend "Amendments A–AE" to A–AF and append a summary line in the existing per-amendment style:

```markdown
AF (2026-08-10, ADR186): **Bevy replaces the Ratatui client outright** — AC
superseded in full, AE (xi) superseded, glyph floor retired; the Ratatui
estate deleted by ceremony; `babylon-client` (Bevy) is the v1.0 client and
the game ships as a pure Rust binary; visualization obligations transfer to
Bevy scenes; hyperedge-lane pause lifted.
```

- [ ] **Step 2: CLAUDE.md client-status paragraph** — rewrite the "Client status" paragraph (currently ends "`babylon play` — the only terminal client since the M7 cutover…") to state: the Bevy client `rust/crates/babylon-client` is the v1.0 client (Amendment AF); the deletion ceremony removed the Ratatui client (2026-08-10); `uv sync` no longer needs cargo; keep the `observe()` durable-seam sentence unchanged. Update the client-logs sentence: `rust-client.log` retired; the Bevy client's file sink lands at B2.

- [ ] **Step 3: NORTH_STAR.md** — rewrite invariant 3 ("Text is the assertion medium; every raster has a text floor; the game is fully playable glyph-only over ssh") to record the AF supersession: III.12's text-assertion medium stays for engine/test assertions; Amendment AF retires the glyph-only-client rule. Fix the line-140 kitty-raster sentence the same way. Do NOT rewrite history elsewhere in the doc — mark supersession, don't erase.

- [ ] **Step 4: Vale both files' changed prose**: `vale CLAUDE.md NORTH_STAR.md` → 0 errors on the lines you touched (pre-existing warnings elsewhere are out of scope).

- [ ] **Step 5: Commit** (`docs(p28): constitutional sweep for Amendment AF — CLAUDE.md + NORTH_STAR`).

### Task 4: Open the AF PR — then STOP (director-gate)

- [ ] **Step 1:** `mise run check:quick` (no test leg needed — docs only) and push the branch.
- [ ] **Step 2:** Open the PR against `dev`, titled `docs(constitution): Amendment AF — the Bevy Cutover (v3.1.0)`, body listing the clause summary and linking the spec + PR #468.
- [ ] **Step 3: STOP.** Request Director ratification. **Never self-merge this PR.** Phases B and C wait until it merges.
- [ ] **Step 4 (post-ratification): board hygiene** — run exactly:

```bash
gh issue close 284 -c "Superseded by Amendment AF (ADR186): the kitty raster lane died with the Ratatui client; map rendering is a Bevy scene (Program 28, B1)."
gh issue close 262 -c "Superseded by Amendment AF (ADR186) clause (vi): the TUI campaign gate's client is deleted; a Director eyes-on session in the Bevy client replaces it before v1.0."
gh issue comment 291 -b "Rescoped by Amendment AF: the shipped game is a pure Rust binary (no wheel, no nix-bootstrap for the client). Shrink this train to plain game-executable distribution at plan time."
gh issue comment 292 -b "Rescoped by Amendment AF: see #291 — pure Rust binary distribution."
gh issue comment 293 -b "Rescoped by Amendment AF: see #291 — pure Rust binary distribution."
gh issue comment 282 -b "Re-scoped by Amendment AF + the P27 freeze: this is now the ENGINE-lane storage swap (babylon-graph consumes hypergraph-rs behind the trait seam, ADR179 T3). The Python XGI-surface framing is superseded."
```

---

## Phase B — The deletion ceremony (implementation PR; requires AF merged)

### Task 5: Delete the Rust Ratatui crates

**Files:**
- Delete: `rust/crates/babylon-tui/`, `rust/crates/babylon-tui-python/`, `rust/crates/babylon-md/`, `rust/pyproject.toml`, `rust/python/` (the wheel's Python shim dir — verify contents first)
- Edit: `rust/Cargo.toml` (members list), `rust/Cargo.lock` (regenerated), `.mise.toml` (`rust:check` lines 1640–1641; the comment near line 94)

**Interfaces:**
- Produces: a workspace of exactly `babylon-kernel`, `babylon-graph`, `babylon-bsl`, `babylon-tick`, `babylon-md`-free members that `rust:check --workspace` gates.

- [ ] **Step 1: Confirm `babylon-md` has no consumer outside the client**: `rg -l 'babylon-md' rust/crates/*/Cargo.toml` — expect only `babylon-tui`. If anything else consumes it, keep the crate and record why in the commit body.
- [ ] **Step 2: Verify `rust/python/` is wheel shim only** (`ls rust/python/`); delete it with the crates:

```bash
git rm -r rust/crates/babylon-tui rust/crates/babylon-tui-python rust/crates/babylon-md rust/pyproject.toml rust/python
```

- [ ] **Step 3:** Edit `rust/Cargo.toml` members down to the four engine crates. Run `cargo build --workspace --locked` in `rust/` — expect a lock error, then regenerate: `cargo build --workspace` (this lock update is the point) and check `git diff rust/Cargo.lock` only REMOVES packages.
- [ ] **Step 4:** Edit `.mise.toml`: delete the two `-p babylon-tui` legs from `rust:check` (lines 1640–1641) and rewrite the comment at ~line 94 (wheel build note) to state that Amendment AF removed the wheel.
- [ ] **Step 5:** `mise run rust:check` → green.
- [ ] **Step 6: Commit** (`refactor(rust)!: delete the Ratatui estate — Amendment AF (iii) deletion ceremony`).

### Task 6: The wheel leaves Python packaging

**Files:**
- Edit: `pyproject.toml` (dependency `"babylon-tui"` at ~line 90 with its comment block; `[tool.uv.sources]` entry at ~line 195; mypy override `"babylon_tui"` at ~line 448), `uv.lock` (re-resolve), `flake.nix` (uv2nix override block at lines ~91–115; smoke-import line 272), `tests/unit/cli/test_uv_migration.py` (whatever leg asserts the wheel/path-source — read it first)

**Interfaces:**
- Consumes: Task 5's deleted `rust/pyproject.toml` (the uv path source now dangles — this task removes the dangle).
- Produces: `uv sync --extra server --frozen` succeeding with NO cargo on PATH.

- [ ] **Step 1:** Remove the three `pyproject.toml` sites (dependency + comment block, uv source, mypy override; keep the `textual_image.*` line — separate retirement).
- [ ] **Step 2:** `uv lock` (worktree: no `UV_FROZEN`); check `git diff uv.lock` removes `babylon-tui` and adds nothing.
- [ ] **Step 3:** Edit `flake.nix`: delete the babylon-tui override block and the smoke-import line; replace the smoke import with `python -c 'import babylon'`.
- [ ] **Step 4:** Read `tests/unit/cli/test_uv_migration.py`, update the legs that reference the wheel/path source; run `mise run test:q -- tests/unit/cli/test_uv_migration.py` → green.
- [ ] **Step 5: Prove the criterion:** in a shell with cargo hidden (`PATH` stripped of `~/.cargo/bin` and the rustup shims): `uv sync --extra server --frozen` → succeeds.
- [ ] **Step 6: Commit** with `SKIP=worktree-contract` (intentional lock change) (`build(deps)!: babylon-tui wheel leaves the default set — uv sync without cargo (AF iii)`).

### Task 7: Delete the Python client periphery

**Files:**
- Delete: `src/babylon/tui/` (whole package), `src/babylon/cli/play.py`, `tests/unit/tui/`, `tests/unit/cli/test_play.py`, `tests/unit/sentinels/test_raster_bans.py`, `tests/unit/render/test_rust_theme_parity.py` (re-created in Phase C), `tests/unit/render/test_deps.py` (read first — delete only if wholly client-bound, else strip)
- Edit: `src/babylon/cli/__init__.py` (lines 91, 119, 151–154: play import, command registration, no-subcommand default → print help), `src/babylon/game/tutorial.py` (strip `babylon_tui` glue, KEEP the `TutorialStep` script), `tests/unit/game/test_tutorial.py` (drop pilot legs, keep content assertions), `src/babylon/sentinels/tutorial_coverage/` + `src/babylon/sentinels/_rust.py` (strip client legs per the ADR186 disposition table)

**Interfaces:**
- Consumes: ADR186 sentinel disposition table (Task 2) — cite it in the commit body for every deleted sentinel.
- Produces: `import babylon` and the full unit suite green with no `babylon_tui` reference anywhere in `src/` or `tests/`.

- [ ] **Step 1:** `git rm -r src/babylon/tui src/babylon/cli/play.py tests/unit/tui tests/unit/cli/test_play.py tests/unit/sentinels/test_raster_bans.py tests/unit/render/test_rust_theme_parity.py`
- [ ] **Step 2:** Edit `src/babylon/cli/__init__.py`: remove the play import + registration; the no-subcommand callback prints help (typer: `ctx.get_help()`) instead of launching play.
- [ ] **Step 3:** Edit `src/babylon/game/tutorial.py` + its test per the Interfaces note; run `mise run test:q -- tests/unit/game/test_tutorial.py` → green.
- [ ] **Step 4:** Edit the sentinel packages; run `mise run test:q -- tests/unit/sentinels` → green.
- [ ] **Step 5: Zero-reference proof:** `rg -l 'babylon_tui|babylon-tui' src tests` → empty.
- [ ] **Step 6: Commit** (`refactor(cli)!: delete the Python client periphery — AF (iii)/(iv); dispositions per ADR186`).

### Task 8: CI surgery — the wheel leg dies

**Files:**
- Edit: `.github/workflows/ci.yml` (the `build-wheel` job at lines ~37–80; every `needs: build-wheel` (lines ~86, ~212 — sweep them all); every `download-artifact`/`babylon-tui-wheel` step)

- [ ] **Step 1:** `rg -n 'build-wheel|babylon-tui|wheel' .github/workflows/ci.yml` — list every site.
- [ ] **Step 2:** Delete the `build-wheel` job; remove `build-wheel` from every `needs:`; delete the wheel download/install steps from dependent jobs.
- [ ] **Step 3:** Sweep the other workflows: `rg -ln 'babylon-tui|build-wheel' .github/workflows/` and fix any hit (nightly, frozen-engine weekly).
- [ ] **Step 4:** The pre-commit actionlint hook validates on commit; also run `mise run check:quick`.
- [ ] **Step 5: Commit** (`ci: remove the babylon-tui wheel leg — uv sync needs no cargo (AF iii)`).

### Task 9: Docs sweep, full gates, PR

- [ ] **Step 1:** `rg -ln 'babylon play|rust-client\.log|Ratatui|ratatui' docs/ CLAUDE.md README.md | head -30` — fix live-doc hits (how-to/reference pages that INSTRUCT the reader; leave historical ADRs/specs/reports untouched — immutability of history).
- [ ] **Step 2:** Full gates: `mise run check` && `mise run rust:check` && `mise run qa:regression` && `mise run qa:vault-regression-ci` — all byte-identical/green. Any golden drift = STOP (a client deletion must not move engine bytes).
- [ ] **Step 3:** Update `ai/state.yaml` (Ratatui deleted, AF executed).
- [ ] **Step 4:** Open the PR (`refactor!: the Ratatui deletion ceremony — Amendment AF (iii)`), body linking ADR186 + the AF PR. Self-merge on green.

---

## Phase C — B0: the `babylon-client` Bevy scaffold (implementation PR)

### Task 10: Extract the tick seam into a babylon-tick library

**Files:**
- Create: `rust/crates/babylon-tick/src/lib.rs`
- Edit: `rust/crates/babylon-tick/src/main.rs` (becomes a thin CLI over the lib), `rust/crates/babylon-tick/Cargo.toml` (add `[lib]` alongside the bin)

**Interfaces:**
- Produces: `babylon_tick::run_once(scenario_src: &str, rule_src: &str) -> Result<TickReport, String>` where `pub struct TickReport { pub before: [u8; 32], pub after: [u8; 32], pub fired: usize }`. Task 12 links against exactly this.

- [ ] **Step 1: Write the failing test** in `rust/crates/babylon-tick/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::run_once;
    const SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");
    const RULE: &str = include_str!("../content/rules/fundamental-theorem.bsl");

    #[test]
    fn run_once_is_deterministic() {
        let a = run_once(SCENARIO, RULE).expect("first run");
        let b = run_once(SCENARIO, RULE).expect("second run");
        assert_eq!(a.after, b.after);
        assert_ne!(a.before, a.after, "the rule must move state");
    }
}
```

- [ ] **Step 2:** `cargo test -p babylon-tick` in `rust/` → FAIL (`run_once` not defined).
- [ ] **Step 3:** Write `run_once` by MOVING the body of `main.rs`'s flow (scenario load → `load_rule` with `LoadContext` → pre-hash → `run_tick` → post-hash; follow `main.rs` exactly (the Slice 1 contract)). `main.rs` keeps only arg parsing, calling `run_once`, and printing. Also move the `hex(bytes: &[u8; 32]) -> String` formatter from `main.rs` into the lib and export it (`pub fn hex`) — Task 12's test uses it.
- [ ] **Step 4:** `cargo test -p babylon-tick` → PASS; `cargo run -p babylon-tick -- content/scenarios/two-classes.bscn content/rules/fundamental-theorem.bsl` prints the SAME before/after hashes as before the refactor (capture the pre-refactor output in Step 0 of your shell session and diff).
- [ ] **Step 5: Record the post-tick hash** printed by Step 4 — Task 12's golden pin.
- [ ] **Step 6: Commit** (`refactor(rust): extract babylon-tick run_once seam for the client (B0)`).

### Task 11: The babylon-client crate — window, palette, type direction

**Files:**
- Create: `rust/crates/babylon-client/Cargo.toml`, `src/main.rs`, `src/palette.rs`, `assets/fonts/` (Iosevka + OFL license), `tests/smoke.rs`
- Edit: `rust/Cargo.toml` (add member), `rust/Cargo.lock` (bevy tree), `rust/deny.toml` (license allowances as needed)
- Re-create: `tests/unit/render/test_rust_theme_parity.py` (Python side, per the ADR186 PORTED disposition)

**Interfaces:**
- Consumes: nothing from other tasks (pure scaffold).
- Produces: `babylon_client::palette` constants (names below) that the Python parity guard parses; the crate Task 12 extends.

- [ ] **Step 1:** Add `"crates/babylon-client"` to workspace members. Write `Cargo.toml`:

```toml
[package]
name = "babylon-client"
version.workspace = true
edition.workspace = true
license.workspace = true
# Bevy 0.18's MSRV may exceed the 1.87 workspace floor — if `cargo build`
# says so, set this crate's own `rust-version` to Bevy's floor (toolchain
# pin is 1.91.1, so any value ≤ 1.91 builds here).

[dependencies]
bevy = "0.18"
babylon-tick = { path = "../babylon-tick" }
```

- [ ] **Step 2:** Write `src/palette.rs` — one constant per line (the parity guard's regex depends on it), values from the §9b canon in Global Constraints:

```rust
//! KSBC role colors (DESIGN_BIBLE §9b). Source of truth:
//! `src/babylon/render/tiers.py::TRUECOLOR_PALETTE`; the parity guard
//! `tests/unit/render/test_rust_theme_parity.py` parses THIS file's
//! `Color::srgb_u8(r, g, b)` literals — keep each constant on one line.
use bevy::color::Color;

pub const FIELD: Color = Color::srgb_u8(26, 0, 0);
pub const BONE: Color = Color::srgb_u8(232, 232, 232);
pub const CRIMSON: Color = Color::srgb_u8(220, 20, 60);
pub const GOLD: Color = Color::srgb_u8(255, 215, 0);
pub const DIM: Color = Color::srgb_u8(64, 64, 64);
pub const MUTED_DARK: Color = Color::srgb_u8(32, 32, 32);
pub const ROYAL: Color = Color::srgb_u8(65, 105, 225);
pub const GREEN_DARK: Color = Color::srgb_u8(34, 139, 34);
```

- [ ] **Step 3:** Write `src/main.rs`:

```rust
mod palette;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Babylon — The Fall of America".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(palette::FIELD))
        .add_systems(Startup, (spawn_camera, spawn_title))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_title(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Text::new("BABYLON"),
        TextFont {
            font: assets.load("fonts/IosevkaTerm-Regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        TextColor(palette::GOLD),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
    ));
}
```

(If Bevy 0.18's UI API has drifted from this 0.15+-era shape, follow the current `docs.rs/bevy/0.18` text example — the deliverable is title text in GOLD at top-left, not these exact lines.)

- [ ] **Step 4: Font asset:** `fc-match -f '%{file}\n' 'Iosevka Term'` → copy the `.ttf` to `rust/crates/babylon-client/assets/fonts/IosevkaTerm-Regular.ttf` and add the SIL OFL 1.1 license text beside it as `OFL.txt` (Iosevka is OFL-licensed; the license text ships with the font). If `fc-match` finds nothing, STOP and ask the Director where her Iosevka files live.
- [ ] **Step 5: Write the smoke test** `tests/smoke.rs`:

```rust
use bevy::prelude::*;

#[test]
fn app_builds_and_updates_headless() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
}
```

- [ ] **Step 6:** `cargo test -p babylon-client` → PASS. `cargo deny check` in `rust/` — Bevy's wgpu tree brings licenses the current allow-list may lack (Zlib, BSL-1.0, Unicode-3.0, MPL-2.0 are the usual suspects): add each to `rust/deny.toml [licenses].allow` with a one-line comment, never blanket-disable the check.
- [ ] **Step 7: Re-create the Python parity guard** at `tests/unit/render/test_rust_theme_parity.py`: same shape as the deleted file (retrieve it: `git show HEAD~N:tests/unit/render/test_rust_theme_parity.py` from the Phase B PR, or from the `p27-python-freeze` tag), with the path re-pointed to `rust/crates/babylon-client/src/palette.rs` and the regex matching `Color::srgb_u8\((\d+), (\d+), (\d+)\)`. Map constant names → `RoleToken`s (FIELD→FIELD, BONE→TEXT, CRIMSON→ACCENT_CRIMSON, GOLD→ACCENT_GOLD; assert every parsed constant matches `TRUECOLOR_PALETTE` where a role exists). Run `mise run test:q -- tests/unit/render/test_rust_theme_parity.py` → PASS.
- [ ] **Step 8: Eyes-on:** `cargo run -p babylon-client` — a near-black (#1a0000) window titled "Babylon — The Fall of America" with gold "BABYLON" text opens. Screenshot for the PR body.
- [ ] **Step 9: Commit** (`feat(client): babylon-client Bevy scaffold — window, palette, Iosevka (B0)`).

### Task 12: The engine link — one tick at startup, hash logged

**Files:**
- Edit: `rust/crates/babylon-client/src/main.rs`, add `src/engine_link.rs`, add `tests/engine_link.rs`

**Interfaces:**
- Consumes: `babylon_tick::run_once` + `TickReport` (Task 10), the golden hash recorded in Task 10 Step 5.

- [ ] **Step 1: Write the failing test** `tests/engine_link.rs`:

```rust
#[test]
fn startup_tick_matches_the_pinned_hash() {
    let report = babylon_client_engine_link_probe().expect("tick");
    // The golden from `babylon-tick` on two-classes.bscn + fundamental-theorem.bsl
    // (Task 10 Step 5). If this moves, the ENGINE moved — investigate, never re-pin
    // without a declared ceremony.
    assert_eq!(
        hex(&report.after),
        "783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679",
    );
}
```

(The plan captured that golden by running `cargo run -p babylon-tick` on the two-classes/fundamental-theorem content at `dev` = `9020cc3a` — before hash `5a44ab0c…a205`, after hash as pinned. If Task 10 Step 5's capture differs, the engine moved between plan and implementation: pin what Step 5 measured and note the delta in the commit body. Export the probe as a `pub fn` from the crate's `engine_link` module; `hex` comes from `babylon_tick::hex`, exported in Task 10.)

- [ ] **Step 2:** `cargo test -p babylon-client` → FAIL (probe not defined).
- [ ] **Step 3:** Write `src/engine_link.rs`:

```rust
//! B0's proof that the client links the engine in-process: run the Slice 1
//! seam (scenario -> rule -> one tick -> state hash) at startup and log it.
use babylon_tick::{run_once, TickReport};

const SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/two-classes.bscn");
const RULE: &str =
    include_str!("../../babylon-tick/content/rules/fundamental-theorem.bsl");

pub fn engine_link_probe() -> Result<TickReport, String> {
    run_once(SCENARIO, RULE)
}
```

Wire a startup system in `main.rs` that calls it and logs `info!("engine link: post-tick state hash {}", …)`, and `panic!`s on `Err` — a client that opens with a dead engine link is the loud-failure case, not a warning.

- [ ] **Step 4:** `cargo test -p babylon-client` → PASS; `cargo run -p babylon-client` logs the hash on stdout.
- [ ] **Step 5: Commit** (`feat(client): engine link probe — deterministic tick + pinned hash at startup (B0)`).

### Task 13: Gates, state, PR

- [ ] **Step 1:** `mise run rust:check` (workspace-wide — picks up `babylon-client`) → green. Note the CI Rust Gate's compile time will grow with Bevy; its cargo cache keys on `rust/Cargo.lock`, which this PR changes once — later runs re-warm.
- [ ] **Step 2:** `mise run check` → green (the parity-guard test is the only Python change).
- [ ] **Step 3:** Update `ai/state.yaml` (B0 reached: window + palette + engine link) and the GitHub project board (Program 28 client lane: B0 done).
- [ ] **Step 4:** Open the PR (`feat(client): B0 — babylon-client Bevy scaffold with in-process engine link`), body with the screenshot and the pinned-hash note. Self-merge on green.

---

## Self-review notes

- **Spec coverage:** AF clauses (i)–(viii) cover spec §3 items 1–6 plus the gate-transfer and sentinel-continuity obligations §3 implies via AC (iii)/AE IX.5; Phase B = success criterion 2; Phase C = the B0 rung of criterion 3; board hygiene = spec §5. The Bevy version pin (spec §8 Q1) resolves to 0.18; the map-approach half of that question defers to the B1 plan by design. AF (vi) defines the eyes-on gate (spec §8 Q2). Transplant-vs-rewrite (spec §8 Q3): this plan deletes and retrieves from git history per-module when B1+ needs it; the palette PORTS now.
- **Out of scope (other lanes):** hyperedge query implementation (engine lane E1 — the AF (viii) recording merely unblocks it), BSL gap analysis (E2), storage swap (E3), ruling sessions (Director lane), #334 (data lane).
- **Known risk:** Bevy 0.18 API drift vs the sketches in Tasks 11–12 — the deliverable statements ("title text in GOLD", "hash logged at startup") govern over the exact lines; consult docs.rs/bevy/0.18 at implementation time.
