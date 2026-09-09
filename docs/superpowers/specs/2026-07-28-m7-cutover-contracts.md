# M7 — Cutover Ceremony: Execution Contracts (2026-07-28)

Milestone kickoff pin for the raster-cutover plan's Tasks 44–47
(`docs/superpowers/plans/2026-07-26-ratatui-client.md` lines 658–663), expanded to
bite-sized contracts per the plan's own deferred-granularity rule. Branch
`feature/ratatui-m7` off dev `0f660f42` (post-M6, PR #339). One PR, auto-merge on
green, per the queue-autonomy ruling. GitHub train issue: #333.

## 0. Governing rulings (why this milestone is shaped this way)

- **BD-5** (design §3): v1.0 ships WITH the full Textual deletion — the one-way door
  is inside the release; the wheel therefore joins the default install + the T7
  uv2nix player closure. **Risk R6** (design §10) costs the packaging work here.
- **Parity evidence tier (Director ruling, 2026-07-28):** M7 proceeds on
  harness-tier evidence — the M3 parity harness (24-step WAYNE arc on the Rust
  client, two-run byte-identical transcript, committed golden
  `tests/unit/tui/transcripts/wayne_opening_arc.json`). **BD Gate 3 (#262) — the
  Director's own in-game content+client ceremony — remains open and does NOT block
  the cutover** (recorded ruling; ADR150's M3 note stays historically accurate).
  The ceremony commit body cites the harness evidence and names Gate 3's status
  honestly — it does not claim Gate 3 ran.
- **Ceremony discipline** (design §8): the deletion is ONE declared commit
  `test(cutover)!: retire Textual Archive lane`. `tests/baselines/**` untouched —
  no baseline ceremony. Preparatory refactors land BEFORE the ceremony commit so
  the ceremony itself is purely deletion + re-pointing.
- **T7 coordination** (2026-07-23 ruling): T7-beta (embedded Postgres manager)
  builds strictly post-cutover; Task 44 lands the closure work exactly once.

## 1. Execution order

44 (packaging flip) → 45 (default-client flip) → **46-pre (decoupling refactor —
NEW, forced by recon)** → 46 (the ceremony commit) → 47 (program close).

Rationale for 46-pre: recon (4-scout sweep, wf_086f7d53-e95) found the wholesale
deletion as chartered would BREAK the Rust path — six couplings, §4 below.

## 2. Task 44 — the packaging flip

Commit: `build(tui): rust client in the default install`.

**2.1 pyproject.toml.** Move `"babylon-tui"` from `[dependency-groups] tui` into
`[project] dependencies`; delete the now-empty `tui` group (and its long comment —
fold the durable parts into the `[tool.uv.sources]` comment). The
`[tool.uv.sources] babylon-tui = { path = "rust" }` row is unchanged. Reword the
`[tool.mypy.overrides]` `babylon_tui` comment (currently "CI never builds the Rust
extension before the M7 packaging flip") — the override itself stays (tolerates
absence on machines that haven't synced).

**2.2 uv.lock re-resolution — worktree hazard.** Re-resolution reads metadata from
the `../hypergraph-rs` path source, which in a worktree resolves to
`.claude/worktrees/hypergraph-rs` — symlink it first
(`ln -s /home/user/projects/game/hypergraph-rs .claude/worktrees/hypergraph-rs`).
Run `uv lock` from the worktree root; **diff-audit the lock**: the only acceptable
change is `babylon-tui` moving from the `tui` group requires-dist into the base
`requires-dist` (+ the group table row dropping). Any wholesale re-resolution drift
= STOP and investigate (nightly-lock-staleness scar, PR #306).

**2.3 Consequence to own honestly:** `babylon-tui` is a `directory` source with no
wheels stanza — every `uv sync` (bare included) now builds the cdylib via maturin,
which needs cargo/rustc AND network access to the `hypergraph-rs` git dep (the
wheel unconditionally carries the raster feature). Every CI job that syncs needs
the toolchain (2.4); the flake sandbox needs vendored crates (2.5); a
toolchain-less user gets a hard sync failure — which is BD-5's declared cost, and
`babylon doctor` / the install docs point at the Nix closure as the no-toolchain
path.

**2.4 CI.** Two changes to `.github/workflows/ci.yml` + the composite:

- `.github/actions/bootstrap-python/action.yml` gains a Rust toolchain step
  (rustup honoring `rust/rust-toolchain.toml` — channel 1.91.1) + caching:
  `~/.cargo/registry` + `rust/target` keyed on `hashFiles('rust/Cargo.lock')`.
  Every job that syncs (fast-gate, test-unit, qa-regression, security) builds the
  wheel through uv; the cache makes it incremental (this IS R6's "wheel caching").
- A new `rust-gate` job runs `mise run rust:check` (already CI-parity by its own
  description; cargo builds stay single-flight within the job per machine-safety).
- Effect on the Python suite: the three `pytest.importorskip("babylon_tui")` gates
  (`test_rust_client_ffi.py`, `test_tutorial_pilot_rs.py`, `test_map_smoke_rs.py`)
  stop skipping in CI and run for real — the parity harness finally executes on
  every PR. Their `reason=` strings ("opt-in tui group not installed") are
  restated as absence-honesty ("babylon_tui extension not built").

**2.5 flake.nix — the closure gains the wheel.** `babylon-tui` enters
`workspace.deps.default`, so `babylonEnv` must build it inside the Nix sandbox
(no network): add a maturin/cargo overlay to the `pythonSet` composition (the
`buildFixupOverlay` precedent) — `rustPlatform.importCargoLock` over
`rust/Cargo.lock` with an `outputHashes` entry for the `hypergraph-rs` git dep
(rev `0c95db0663737b492af27f85e70b223833a18c2e`), cargo + rustc + maturin as
native build inputs for the `babylon-tui` package derivation. Declare `./rust`
in `projectSrc` by intent (today it is included by omission), excluding
`rust/target`. `checks.smoke` additionally imports `babylon_tui` — the closure
provably carries the wheel. **Closure-size audit:** `nix path-info -rS` (or
`du` of the closure) before/after, recorded in the commit body + the ADR150 M7
note. Box-safety: nix builds eat the root disk (P26 scar) — check `df` before,
one build, no speculative rebuilds.

**2.6 Guard-message updates.** `src/babylon/cli/play.py:709-717`: the
`--client rust` ImportError→RuntimeError guard STAYS (honest absence for a
broken/missing extension) but its message drops `--group tui` (gone) — new text
points at `uv sync` / `uvx maturin develop` in `rust/` after Rust edits.
`tests/unit/cli/test_play.py::test_rust_without_extension_raises_actionable_runtime_error`
match string updates with it. CLI help text at `play.py:852-858` drops "opt-in via
`uv sync --group tui`" (Task 45 rewrites the rest of that string anyway).

**2.7 Docs touched here:** CLAUDE.md client-status line (the
"`uv sync --group tui`" incantation) — final wording lands at Task 46/47;
minimally, nothing may instruct a now-nonexistent group.

## 3. Task 45 — flip the default client

**SUPERSEDED MID-EXECUTION (Director ruling, 2026-07-28, this session):
"delete the textual code outright."** The plan's "textual remains available
one release behind a deprecation warning" is void — no deprecation window.
Revised Task 45: `feat(cli)!: rust client is the only terminal client` —
`ClientKind`, `--client`, and `run()`'s Textual composition path are DELETED;
`run()` delegates straight to `_run_rust_client`. The textual composition
tests' wiring pins (driver_factory identity, one-runtime-one-catalog loader
partial, watchlist/nav persistence sharing, narrator threading) are re-homed
as rust-lane pins against `RustClientHost` — coverage moves, it does not
drop. §3.1–3.4 below are retained as the record of what was superseded.

Original commit plan: `feat(cli): rust client is the default`.

**3.1 The default lives in THREE places — all flip together:**
`run()`'s kwarg (`play.py:774`), `play()`'s `typer.Option` (`play.py:853`), and —
implicitly — `cli/__init__.py:154` (the no-subcommand path calls `play_cmd.run()`
with zero args, inheriting `run()`'s default).

**3.2 Deprecation warning** in the textual branch at the `run()` dispatch point
(`play.py:791` fall-through), canonical pattern
(`domain/economics/melt/class_position.py:368-374`):
`warnings.warn("--client textual is deprecated; the Rust client is the default. "
"The Textual lane is removed in the next release.", DeprecationWarning,
stacklevel=2)`.

**3.3 filterwarnings gotcha:** `pyproject.toml` sets
`error::DeprecationWarning:babylon.*` — every test that still exercises the
textual branch must wrap in `pytest.warns(DeprecationWarning)`. Test flips:

- `tests/unit/cli/test_app.py` lines 50/62/74/86 — omitted `--client` now asserts
  `ClientKind.RUST`.
- `tests/unit/cli/test_play.py` bare `run()` sites (112/142/163/178/204) — decide
  per test: those pinning textual-path composition pass
  `client=ClientKind.TEXTUAL` explicitly inside `pytest.warns`; they die wholesale
  at Task 46 anyway (they build `ArchiveApp`), so minimal surgery.
- `test_textual_default_still_boots_archive_app` (`test_play.py:560`) — the name
  IS the old default's assertion: replaced by `test_rust_default_...` (rust lane
  boots from a bare `run()`) + an explicit deprecated-textual test with
  `pytest.warns`.

**3.4 Docstrings/help:** `ClientKind` class docstring (105-114), `run()`'s param
doc (787-789), the `--client` help text (855-858) — all state rust default +
textual's one-release deprecation window.

## 4. Task 46-pre — decouple the Rust seam from Textual (NEW)

Commit: `refactor(tui): textual-free rust seam`. Pure refactor; zero behavior
change; all gates green with Textual still installed. Forced by recon: the traced
runtime chain `host.py → campaign_menu.py / shell/backlinks.py → wikilinks.py →
theme.py → textual` means `from babylon.tui.host import RustClientHost` (the line
`cli/play.py:730` executes) CANNOT survive dep removal without these splits:

| # | Module | Extract (textual-free survivor) | Leaves behind (dies at 46) | Consumer forcing it |
|---|--------|--------------------------------|---------------------------|---------------------|
| 1 | `app.py` | `CampaignHandle` Protocol (+ companion protocols it defines, e.g. `PacedDriverHandle` — verify exact set at execution) → new `src/babylon/tui/contract.py` | all of `ArchiveApp` | **runtime** imports in every keep-test (`test_host_contract.py`, `test_rust_host_m2-m6`, `test_tutorial_pilot_rs.py`); TYPE_CHECKING in `host.py`, `game/session.py`, `game/tutorial_runtime.py`, `game/tutorial.py`, `cli/play.py` |
| 2 | `campaign_menu.py` | keep `CampaignMenu`/`CampaignCatalog`/`CampaignSummary`/`LobbyRow`/`InMemoryCampaignCatalog`/`operation_codename` in place; move `LobbyScreen` → new textual-only `lobby_screen.py` (app-side; deleted at 46) | `LobbyScreen` | **runtime** `host.py:70` |
| 3 | `wikilinks.py` | `WIKILINK_RE` → new `wikilink_grammar.py` (the canonical grammar; `wikilinks.py` re-imports until 46) | Textual render machinery (whole file at 46) | **runtime** `shell/backlinks.py:11` ← `host.py:84` |
| 4 | `theme.py` | plain hex constants stay; move `KSBC = Theme(...)` → textual-only module (with `lobby_screen.py` or into `app.py`; dies at 46) | the Textual `Theme` object | **runtime transitive** via `wikilinks.py:49` (post-split: `wikilink_grammar` must not import theme; re-check the residual chain) |
| 5 | `tutorial_overlay.py` | `TutorialProgress`/`TutorialStepView` Protocols → `contract.py` (same home as #1) | `TutorialOverlay` widget | TYPE_CHECKING `host.py:105`, `cli/play.py:102` |
| 6 | `directives.py` | `StatblockRow`/`StatblockProvider` aliases → `src/babylon/tui/statblocks.py` | `MarkdownFence` dispatch | TYPE_CHECKING `projection/organization.py:95`, `projection/institution.py:79` |

Also split `tests/unit/tui/test_campaign_menu.py` along the same line (logic tests
keep; `LobbyScreen` Pilot tests move to a delete-bound file), and re-point
`test_wikilinks.py` at `wikilink_grammar`. Acceptance: `rg` proves
`babylon.tui.host`'s transitive import closure contains zero `textual` imports
(the decisive check the ceremony relies on); `mise run check` green.

## 5. Task 46 — the ceremony commit

Single declared commit: `test(cutover)!: retire Textual Archive lane`.

**5.1 Delete — production** (file-level list, not directory-level; `src/babylon/tui/`
is NOT monolithic): `app.py`, `dispatch.py`, `map_room.py`, `palette.py`,
`peek_overlay.py`, `verb_plate.py`, `wikilinks.py`, `tutorial_overlay.py`,
`directives.py`, the 46-pre strays (`lobby_screen.py`, the KSBC module),
`shell/app_shell.py`, `shell/views/` (all 4), `shell/bdd/` (both),
`topology/matrix.py`, `topology/egotree.py` (+ `topology/__init__.py`), and —
**outside the plan's stated scope, named here by intent** —
`src/babylon/render/widgets/palette_plate.py` (orphaned ADR097 seed widget, zero
src importers). `verb_plate.py`/`dispatch.py` verified deletable (only importer:
`app.py` + own tests; `host.py.verb_plate_view_json` calls the session directly).

**5.2 Keep — production**: `host.py`, `contract.py`, `chronicle.py`,
`chronicle_salience.py`, `nav.py`, `router.py`, `peek.py`, `watchlist.py`,
`trade_dossier.py` (runtime dep of `game/session.py:147` — BOTH clients),
`campaign_menu.py` (post-split), `wikilink_grammar.py`, `theme.py` (post-split),
`statblocks.py`, `shell/backlinks.py`. `render/capability.py`'s lazy
`textual_image` import is try/except-guarded — stays as an honest-absence path
(one-line note in the commit body).

**5.3 Delete — tests**: all 14 `test_app_*.py`; `test_tutorial_pilot.py` (the OLD
Textual parity gate); `test_directives*.py`, `test_egotree_directive.py`,
`test_map_room_directive.py`, `test_matrix_directive.py`, `test_palette.py`,
`test_peek_overlay.py`, `test_verb_plate.py`, `test_nav_shell.py`,
`test_tutorial_overlay.py`, `test_trade_reachability.py`,
`test_t3_live_reachability.py`, `test_snapshot.py` + `snapshot_app.py`; the whole
`shell/` test tree except `shell/test_backlinks.py`; the whole `snapshots/` tree
(+ 24 `.raw` goldens); `tests/unit/tui/conftest.py` (truecolor fixture — snapshot
lane only) + `shell/conftest.py` + `snapshots/conftest.py`. Outside `tests/unit/tui/`:
`tests/unit/render/test_deps.py` (asserts textual imports — replace with a dual
asserting `babylon_tui` imports + the deps are GONE), `test_palette_plate.py` (+
its golden), the `snap_compare` leg of
`tests/integration/archive/test_county_e2e.py` (+ its golden + `e2e_snapshot_app.py`
— keep any non-snapshot assertions), and audit `tests/unit/game/test_tutorial.py`'s
direct textual import (recon found no production reason).

**5.4 Deps**: remove `textual`, `textual-image`, `textual-plotext` from
`[project] dependencies` (today they are UNCONDITIONAL defaults — recon);
`pytest-textual-snapshot` + `syrupy` from the dev group; the whole
`[tool.uv] override-dependencies` block (existed solely for the
snapshot-plugin/syrupy conflict; syrupy has exactly one consumer, dying here).
`rich` STAYS (10 independent importers). `uv lock` re-run under the §2.2 hazard
procedure. Per-package check for `dulwich`/`jinja2`/`pillow`/`markdown-it-py` etc.
before touching anything else on the keel comment block — they serve the vault
pipeline, not Textual; expected outcome: untouched.

**5.5 The tutorial_coverage sentinel — re-architecture, not a path swap.**
`declared_bindings()` (`sentinels/_ast.py:1666`) reads the Textual
`class X: BINDINGS=[Binding(...)]` idiom; post-delete it would return ZERO and the
gate goes **vacuously green (dark), not red** — the exact sentinel-goes-dark class
the standing rule exists for. Contract:

- New extractor reading the RUST single source of truth for player-facing keys:
  text-parse `rust/crates/babylon-tui/src/views/keybar.rs` (the Wave-1
  "one source of truth" hint/help tables) → `(surface, key)` pairs — precedent
  for Rust-source-as-text checks: `tests/unit/render/test_rust_theme_parity.py`.
- Anchor grammar in `game/tutorial.py` rewritten client-neutral:
  `binding:<Surface>:<key>` naming keybar surfaces, replacing the 20
  `binding:ArchiveApp|LobbyScreen|BriefingScreen:*` literals (which would
  otherwise name deleted classes — fiction, III.11). If anchors flow into
  `tutorial_state_json`/the transcript, regenerate via `BABYLON_REGEN_TRANSCRIPT=1`
  (byproduct, no ceremony — M3 doctrine).
- **Non-vacuity guard (the dual):** the extractor yielding zero (or below a
  measured floor) options is itself a RED sentinel failure — mutation-validated
  per the standing sentinel-every-error-class rule.
- Both checks (`covered_or_exempted` + `exemption_still_real`) and the exemption
  registry re-keyed to the new grammar; `tutorial_coverage/__init__.py` docstring
  updated; `check:tutorial-coverage` mise task description survives as-is.

**5.6 Other sentinels**: `seam_algebra`'s `_ARCHIVE_SEVERITY_PATH` hardcodes
`src/babylon/tui/chronicle_salience.py` — a KEEP file; verify the gate still
resolves post-delete (it fails loud on a missing file, but prove it green).
`check:vocabulary`, `lint:imports` contracts unaffected (no textual estate in
either); re-run the full sentinel family.

**5.7 Docs (living refs only; dated specs/plans + state.yaml history are
append-only per Immutability-of-History):** CLAUDE.md client-status paragraph
(rewrite: Rust client IS the terminal client, default install; Textual GONE)
+ the Amendment AC summary line; NORTH_STAR.md line ~156 mermaid node
`Textual Archive` → the Rust/Ratatui Archive; `ai/architecture.yaml` — add the
missing terminal-client section (`src/babylon/tui/` survivors + `rust/`
workspace), fix `presentation_layer.status` (still claims the React web app is
"LIVE — THE frontend", contradicting Amendment V) and `directory_map` (omits
`src/babylon/tui/` and `rust/` entirely); `ai/tooling.yaml` — document
`rust:check`/`rust:format` (missing today).

**5.8 Commit body**: parity evidence (ADR150 M3 STATUS NOTE citation: harness
green, 24-step arc, two-run byte-identity, committed golden; Gate 3 #262 open =
the Director's own stop, ruled non-blocking); re-counted deletion stats measured
AT EXECUTION (plan's "221 run_test / 574 test fns / ~27 SVG" is stale by a full
milestone — recon measured ~230 `run_test` sites / 750 test fns / 27 `.raw`
goldens (SVG content, `.raw` extension)); the §5.4 dep removals; gates run.

**5.9 Gates post-delete**: `mise run check` + `mise run rust:check` +
`qa:regression` (byte-identical — no engine change exists to move it) +
`qa:vault-regression-ci` (separate estate) + the sentinel family. No
`tests/baselines/**` changes.

## 6. Task 47 — program close

Commit: `docs(state): raster cutover M7 — program close`.
`ai/decisions/ADR150_raster_cutover.yaml` `status: accepted → implemented` **and
the `ai/decisions/index.yaml` row in lockstep** (both fields exist independently —
recon); append the M7 STATUS NOTE in the M5/M6 house shape (dated, branch-named,
headline, evidence: deletion counts, closure-size audit, gates); `ai/state.yaml`
bump + M7 note; plan Tasks 44-47 checkboxes ticked + plan/design marked done (the
repo has no `done/` directory convention — status markers in-file); hypergraph-rs
rev pin recorded at its final value (currently
`0c95db0663737b492af27f85e70b223833a18c2e`); CLAUDE.md final client-status wording
if any residue remains. Close #333 with evidence; board → Done.

## 7. Deviations ledger (append at execution; M5/M6 discipline)

| # | Contract said | Reality | Disposition |
|---|---------------|---------|-------------|
| 1 | §3: default flips, textual stays one release behind a DeprecationWarning | Director ruled mid-execution (2026-07-28): "delete the textual code outright" | Task 45 became `feat(cli)!: rust client is the only terminal client` — ClientKind/`--client`/textual path deleted; wiring pins re-homed on the rust lane; the warning I had just written was removed unshipped |
| 2 | §5.8: plan's stale counts (221 run_test / 574 fns / ~27 SVG) to be re-measured | Measured at execution: **231 `.run_test(` sites / 429 deleted test functions / 27 `.raw` goldens (SVG content) / 143 files / −22,701 LOC**. The 429-vs-750 gap vs recon: recon counted ALL tui test fns; ~336 keep-bound fns survive (Rust-host suites + shared-logic tests) plus re-homed pins | recorded here + in the ceremony commit body |
| 3 | §5.3 named the delete-bound test files | Two dispositions not in the contract: `test_paoh.py`'s round-trip leg dropped (its `parse_paoh_body` died with `directives.py`; NOTHING parses `{paoh}` fence bodies Python-side post-cutover — the Rust client renders topology from `topology_json` envelopes, and a directive fence renders as a headered text block; the serialization stays pinned by the format tests). `test_search_resolver.py` REWRITTEN, not deleted: the epistemic reach∪intel contract survives as direct resolver verdicts (the markdown-it plugin leg died with `wikilinks.py`; the render half is babylon-md's, pinned by its Rust tests + the parity harness) | both recorded in the commit body |
| 4 | §5.5 sentinel re-architecture | Landed as its OWN commit BEFORE the ceremony (8d906ce5) so the gate never has a dark window — proven green against the still-present Textual estate first. The non-vacuity floor is 40 (62 measured) | sequencing improvement, not a gap |
| 5 | Immutability-of-History boundary | `game/tutorial.py`/`tutorial_runtime.py`/`session.py` docstrings: PRESENT-TENSE grounding claims naming deleted classes were rewritten (fiction about the present, III.11); PAST-TENSE narrative ("at authoring time there was no…", "That theory held only until…") kept verbatim | the boundary applied: history immutable, present honest |
| 6 | §5.4 dep removal expected clean | The new deps dual FIRED ON ITS MAIDEN RUN: `mutmut` (dev group) transitively pins `textual`, so the retired stack survived the removal. Per the mutation-testing-is-local-only ruling, `mutmut` moved to its own opt-in `mutation` group (`uv sync --group mutation`) — CI and the default dev sync never carry it; the local sentinel-mutation-validation workflow is intact | the gate class worked exactly as designed — transitive reappearance is a caught regression, not harmless extra |
| 7 | §5.3 test dispositions | `test_play.py`'s `_patched_composition_root` fixture still monkeypatched `"babylon.tui.app.ArchiveApp"` BY STRING (pytest imports the module to resolve the attr — 13 collection-independent errors invisible to the scoped runs); the ArchiveApp/CampaignMenu fakes and `_captured` died with the fixture legs | the string-path monkeypatch is a hidden import edge `rg`-for-imports sweeps cannot see — recorded as a scar class |
| 8 | §5.5 said the sentinel re-architecture SUPERSEDES the Textual reader | The old reader itself — `declared_bindings()` in `sentinels/_ast.py` — was not in any deletion inventory: full-gate run caught its live test reading the deleted `tui/app.py`, plus stale `binding:LobbyScreen:n`/`binding:ArchiveApp:t` literals in the anchors test and a false present-tense claim in `TestVerbTargetingStep`'s docstring (the keybar parser DOES see the composite `F1-F9` row the old AST reader couldn't) | helper + its 5 tests retired in the ceremony; anchor literals re-keyed to the M7 anchors; docstring rewritten to the honest present (III.11); `_rust.py`'s history note de-xref'd to a plain literal |
| 9 | §2 costed the packaging flip in closure bytes, not CI minutes | PR #340's maiden run: the COLD venv cache (new key: rust/ sources in the hash) made `bootstrap-python` compile the wheel from a cold cargo registry — 14m38s of bootstrap before the ~10min unit shard, cancelled at test-unit's 20-minute `timeout-minutes`. Every OTHER job passed, including the new rust-gate (2m51s) — so the sibling jobs' saved caches make re-runs warm | `timeout-minutes: 30` on test-unit with the cold-cache arithmetic in a comment; the ceiling still kills a real hang loudly (the CI header's own rule) |
