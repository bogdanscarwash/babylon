# The Raster Cutover — Rust/Ratatui Archive Client (Design)

**Date:** 2026-07-26 · **Rev 2:** 2026-07-27 (BD interview rulings BD-1…BD-10 + 5-lane ecosystem
research folded in; factual corrections recorded in §13) · **Status:** Approved design ·
**Scope:** full replacement of the Textual ArchiveApp with a Rust/Ratatui client consumed as a
Python extension module, **including the chartered 3D visualization lane**.

## 1. Purpose

Replace the Textual terminal client (`src/babylon/tui/`, "The Archive", **7,866 LOC across 31
files**; total surface with tests + goldens ≈ 22.5k LOC) with a Rust client built on **Ratatui
0.30**, imported into Python as a maturin/PyO3 extension and launched from the existing
composition root (`babylon play`). The end state: a coherent, live-data terminal client with a
playable tutorial touching every player option, geographic maps with lens filters, a topology
page with real rasterization (**3D hypergraph rendering via hypergraph-rs — release-blocking,
BD-4**), a contradiction-field 3D surface (**release-blocking, BD-4**), a "stock market"
economics dashboard suite, and the wiki view — with hover popups and full keyboard+mouse
navigation. The existing test suite stays green throughout; the Textual lane is retired in the
declared M7 cutover ceremony **inside the v1.0.0 release** (BD-5) after parity is proven.

## 2. Binding context (what this design must honor)

- **CONSTITUTION.md II.8 / Amendment V:** a client is a presentation-only viewport over the
  `observe()` projection contract; shapes are JSON-serializable; transport is independent
  (in-process call, HTTP, or file materialization all legal); clients are disposable. A Rust client
  is legal by construction.
- **II.5 / R4:** no LLM in the input path; verbs enter only via the deterministic verb registry
  (nine Article-V verbs).
- **II.11:** cross-subsystem reads only through declared SQL views — never raw tables.
- **II.6:** no DB I/O during tick; the read side is post-commit.
- **III.7:** determinism — the CLI re-exec seals `PYTHONHASHSEED=0` + thread pins before any client
  starts; the tick hash chain must not diverge.
- **III.11:** honest absence — absence renders loud, never fabricated.
- **III.12/III.13:** behavioral contracts (baselines, golden vault) are the durable spec;
  materializations are regenerable, never authoritative.
- **Amendment D caution:** hyperedge rendering is read-only; no affordances implying hyperedge
  mutation.
- **Amendment AA disclosure duty:** this ADR/amendment records a one-line Windows-impact note
  (see §10).
- **`ai/_inbox/post-v1.0.0/rasterizer.md` (pre-blessed port analysis):** "A ratatui client is
  correct iff the tutorial suite passes against it and the playthrough transcripts match." The
  vault bake stays Python-side; the Rust client reads baked markdown. Textual snapshot SVGs die
  with the client ("regenerate freely, never ceremony", ADR099).
- **Import-linter contracts** (`pyproject.toml`): `babylon.tui` must not import
  `babylon.engine`/`babylon.persistence`/django. The composition root (`cli/play.py`) is the only
  place all layers meet.
- **2026-07-22 extraction ruling — superseded FOR CLIENT CRATES by BD-9 (2026-07-27):** generic
  Rust libraries still live outside the babylon tree (hypergraph-rs stays a sibling with its own
  remote), but **client crates live in-tree** at `rust/` — the client is coupled to the FFI seam
  and the tutorial gate, and the sibling model's costs (CI stub theater, non-atomic seam commits
  recreating the deleted Py↔TS drift class, worktree lockfile gotchas, player-closure complexity)
  fall exactly on the artifact that must ship in the default install.

## 3. Decisions

### Owner-approved 2026-07-26 (D2 superseded 2026-07-27)

| # | Question | Decision |
|---|----------|----------|
| D1 | Coexistence vs. replacement | **Full replacement with a constitutional amendment.** The Ratatui client becomes the designated terminal client; the Textual lane (app + **221 Pilot `run_test` sites / 574 test functions** + ~27 SVG snapshot goldens + Textual deps) is deleted in a declared cutover ceremony after parity. |
| D2 | Crate location | ~~Sibling repo `../babylon-tui-rs`~~ **SUPERSEDED by BD-9: in-tree cargo workspace `rust/`** (crates `babylon-tui`, `babylon-tui-python`), rust-toolchain pin 1.91.1 (matches hypergraph-rs), maturin pyproject at `rust/pyproject.toml`, own `mise run rust:check` gate. Consumed via `babylon-tui = { path = "rust" }` uv source. |
| D3 | FFI runtime shape | **Rust owns the loop.** Python builds `GameSession` as today, then calls `babylon_tui.run(host, config_json)`; Rust runs its native ratatui event loop, pulls frozen view-models across the FFI as JSON, and calls back into Python for `advance_tick()` / `issue_verb()` / view pulls. Python remains the single writer. |

### BD interview rulings 2026-07-27 (the charter; recorded in Amendment AC + ADR150)

| # | Ruling |
|---|--------|
| BD-1 | The Rust/Ratatui client **IS v1.0's client** — superseding ruling over the 2026-07-23 critical path; DoD/T7 renumber around it. |
| BD-2 | 3D visualization chartered, **all four targets**: topology hypergraph, contradiction-field surface, choropleth extrusion, trend ridgelines. Premise corrected in-session: Ratatui has no built-in 3D — 3D = hypergraph-rs `cells3d`/`raster` → Canvas cell-grid blit, + ratatui-image kitty true-pixel tier. |
| BD-3 | 3D is a **v1.0 blocker**; the raster lane moves earlier (old M5 → post-M2, now milestone M4); `cells3d` API de-risk + a walking-skeleton blit land at M0. |
| BD-4 | Release-blocking 3D: **topology hypergraph + field surface**. Extrusion + ridgelines best-effort (slip to v1.1 only if they would delay). |
| BD-5 | v1.0 ships **WITH the full M7 Textual deletion** (the one-way door is inside the release — reaffirmed knowingly). Consequence: the client cannot stay an opt-in dependency group at release; the maturin wheel joins the default install / T7 uv2nix player closure. |
| BD-6 | Sequencing: **finish P25 first** (#259 → #260 → merge PR #261's 59 commits), THEN M0. |
| BD-7 | The shared 3D rasterizer lives in **hypergraph-rs's `raster` feature** — this ruling un-pauses that lane for feature work (generic core is already shipped; see §3.1). |
| BD-8 | **BD Gate 3 (#262) runs ON the Rust client at M3** — a combined content+client gate. Mitigant for late content problems: informal Textual playtests remain free anytime pre-M7. |
| BD-9 | Client home: **in-tree `rust/`** (see D2). Supersedes the 2026-07-22 extraction ruling for client crates only; generic libs stay siblings. |
| BD-10 | hypergraph-rs gains a **git remote**; the client consumes `raster`/`cells3d` as a **rev-pinned cargo git-dependency**; CI authenticates via a read-only deploy key (nightly babylon-data rebuild-verify precedent). |

### 3.1 Fact base (design-time verification + 2026-07-27 ecosystem research)

Ratatui has **no built-in 3D rasterizer** — and the 5-lane ecosystem research (wf_7e5cfea4-256)
found **no maintained, z-buffered 3D-to-terminal-cell-grid rasterizer anywhere in the Rust
ecosystem**: every prior artifact (ratatui-wireframe, tui-globe, ratatui's own volatility-surface
example) is draw-order painter's-algorithm only, and the one crate that promised more
(ratatui-plt) was archived pre-0.1. Meanwhile hypergraph-rs's raster core (Phase R DONE,
101 golden tests) is a **generic, z-buffered, deterministic** pipeline — `Vertex3`/`Face`/
`Strut`/`Node3`/`SceneGraph3D`, `Camera` + `project_wh`, stable `depth_sort`, z-buffered
`SubBuffer` with Bresenham lines / edge-function triangle fill, `rasterize() → CellGrid`
(braille/ANSI text) and `render_pixels() → PixelBuffer` (RGB8, kitty-tier ready) — that already
accepts arbitrary primitives, not just graph nodes. **We already own the strongest 3D terminal
rasterizer in the ecosystem.** The four targets are each just a different way of producing
Faces/Struts/Node3s (§7.1). This matches the rasterizer.md vision: "graph layout *and*
rasterization in one deterministic Rust pass."

## 4. Governance artifacts (M0 deliverables)

- **Amendment AC — "The Raster Cutover"** (lettering confirmed: AB remains held by the
  Material-Triad draft): designates the Rust/Ratatui client as the canonical v1.0 terminal
  Archive client; charters the 3D lane (BD-2/3/4); retires the Textual implementation via the
  declared ceremony inside v1.0 (BD-5); states the parity gate (tutorial-BDD suite + transcripts)
  as the blocking condition; carries the AA one-line Windows-impact note.
- **ADR150** — the recording ADR (**not ADR139/140** — those numbers sit inside the T1.2/T4
  controller-allocated lane blocks and are claimed by issues #259/#260; 150 is the next free slot
  ≥ the allocated blocks, per the controller-allocation protocol in `ai/decisions/index.yaml`):
  context, decisions D1–D3 + rulings BD-1…BD-10, the FFI contract, milestone plan, risks, the
  cutover ceremony definition, the ecosystem adoptions, and the Windows-impact line.
- The `tutorial_coverage` sentinel is re-pointed: every Rust binding must be exercised by a
  `TutorialStep` or carry a cited exemption (same law, new client).
- No engine/projection/game code changes → `mise run check` and `qa:regression` remain
  byte-identical for the entire program. Deletions land only in the M7 ceremony commit.

## 5. Repository layout (in-tree `rust/`, BD-9)

```
babylon/rust/                       # cargo workspace, THIS repo
├── Cargo.toml                      # workspace, resolver 2, rust-version = "1.85"
├── rust-toolchain.toml             # channel = "1.91.1", components rustfmt, clippy
├── pyproject.toml                  # maturin backend; [tool.maturin] module-name = "babylon_tui._core",
│                                   #   manifest-path = "crates/babylon-tui-python/Cargo.toml", python-source = "python"
├── python/babylon_tui/             # thin shell: loud-failure `from babylon_tui import _core`,
│                                   #   re-exports, hand-written _core.pyi, py.typed
└── crates/
    ├── babylon-tui/                # pure-Rust core: app state, views, markdown/wikilink rendering,
    │                               #   JSON view-model types (serde), layout registry, reducers,
    │                               #   scene builders (§7.1). Headless-testable; no pyo3.
    ├── babylon-md/                 # FORK of tui-markdown v0.3.9 (MIT/Apache preserved): markdown →
    │                               #   ratatui Text. Two surgical patches: pass through pulldown-cmark
    │                               #   Options (ENABLE_WIKILINKS) + stop discarding link metadata.
    └── babylon-tui-python/         # cdylib, pyo3 0.29 extension-module — THIN FFI shell only:
                                    #   run(host, config_json), host-call bridges, GIL discipline.
```

Core dependencies (research-verified against ratatui 0.30, 2026-07-27):

- `ratatui = "0.30"` (+ its `crossterm` re-export — never a direct crossterm dep), `serde`,
  `serde_json`, `pyo3 = "0.29"` (binding crate only), `insta` (dev).
- **`hypergraph-rs = { git = "<remote>", rev = "<pin>", features = ["cells3d"] }`** (BD-10) — the
  3D rasterizer. ratatui stays OUT of hypergraph-rs's dependency graph (its own architectural
  ruling); the `Cell{ch,fg,bg} → ratatui::Cell` adapter and the `Rgb → bytes` pixel bridge
  (hypergraph-rs open item D-T3) live in `babylon-tui`.
- **`pulldown-cmark = "0.13"`** — has **native `Options::ENABLE_WIKILINKS`** (`[[Target]]`,
  `[[Target|Label]]` → `LinkType::WikiLink`), eliminating the custom `[[...]]` scanning pass and
  the wikilinks.py private-API hostage class entirely.
- **`ratatui-image = "11"`** (official ratatui org, kitty/sixel/iTerm2/halfblocks) — the optional
  true-pixel tier, fed by `render_pixels()`. Two open items gate final adoption (§11).
- **`tui-popup` / `tui-scrollview` / `tui-tree-widget`** (official ratatui org / high-adoption,
  0.30-compatible) — popups, scrollable panes, tree nav.
- Optional polish: `tachyonfx` (ratatui org; deterministic — effects advance on caller-supplied
  `Duration`, so golden tests feed a fixed synthetic delta). Explicitly skipped: ratatui-markdown
  (0.29 pin + unresolved license discrepancy), plotters-ratatui-backend (0.29 pin, stale,
  redundant vs. our rasterizer), hyperrat/OSC-8 links (ratatui#1227 unresolved; hit-registry +
  mouse instead), textplots (subsumed by native Canvas).

babylon-side wiring:

- `pyproject.toml`: `babylon-tui = { path = "rust" }` in `[tool.uv.sources]`; during development a
  `[dependency-groups] tui` group keeps the build opt-in — **but per BD-5 this flips at M7**: the
  wheel enters the default dependency set and the T7 uv2nix player closure (see risk R6).
- **No CI stub needed for the client** (in-tree — always present). CI gains a Rust build leg only
  at the M7 flip; until then bare `uv sync` and CI never build it. `tools/ci_hypergraph_stub.sh`
  is untouched (hypergraph-rs is consumed by the CLIENT's Cargo.toml as a git-dep, not by uv).
- hypergraph-rs CI access: read-only deploy key for the private remote (BD-10; nightly
  babylon-data precedent).
- `uv lock` re-resolution on the dev box; `flake.nix` unchanged (rustc/cargo already in the
  default devshell); the uv2nix `projectSrc` fileset gains `rust/` at the M7 packaging flip.
- Dev loop: `uv sync --group tui` builds once; after Rust edits, `uvx maturin develop` inside
  `rust/` (uv does not rebuild path-source native modules on edit). In-tree home means worktrees
  need no sibling symlinks for the client (the old `../babylon-tui-rs` gotcha is gone); the
  git-dep means cargo needs no local hypergraph-rs checkout either.
- Windows-impact note (Amendment AA duty): crossterm is the cross-platform backend and cargo/
  maturin support native Windows, so the Rust client does not foreclose lane 2; kitty/TGP raster
  is absent from Windows Terminal, so the glyph floor (ADR099) — which the CellGrid text tier IS —
  remains the portability insurance. Unchanged.

## 6. The FFI seam — `src/babylon/tui/` shrinks to a host adapter

The Python package keeps its name (so `cli/play.py` wiring, the import-linter contract, and the
WO-37 structural-typing trick survive), but all Textual widgets are replaced by one module:

```python
# src/babylon/tui/host.py — the only Python surface the Rust client ever sees.
# Structural mirror of CampaignHandle + PacedDriverHandle (tui/app.py today).
# Every pull returns a JSON STRING of a frozen view-model (II.8 shapes); no objects cross.
class RustClientHost:
    # --- lobby ---
    def lobby_catalog_json(self) -> str: ...          # CampaignCatalog rows
    # --- wiki / pages ---
    def read_page(self, subject: str) -> str | None: ...        # baked vault markdown; None = redlink
    def known_subjects_json(self) -> str: ...                   # frozenset[str] -> sorted list
    def backlinks_json(self, subject: str) -> str: ...
    # --- views ---
    def dashboard_view_json(self) -> str: ...           # EconomyView
    def subject_view_json(self, subject: str) -> str: ...       # ProjectionRecord (tagged union)
    def trend_json(self, last_n: int) -> str: ...               # v_national_trend rows (NationalTrendView)
    def choropleth_json(self, tier: str, lens: str) -> str: ... # ChoroplethCell[] (value|tension|fog)
    def topology_json(self, kind: str, focus: str | None) -> str: ...  # paoh|levi|incidence payloads
    def field_state_json(self) -> str: ...              # contradiction-field dossier (T3 projection) — 3D surface input
    def endgame_status_json(self) -> str: ...           # EndgameStatus (5 terminal-outcome axes)
    def verb_plate_view_json(self) -> str: ...          # VerbPlateView (9 Article-V verbs, OODA-gated)
    def tutorial_state_json(self) -> str: ...           # current step id, overlay_text, completion flags
    def watchlist_json(self) -> str: ...
    # --- verbs (Rust -> Python) ---
    def advance_tick(self) -> str: ...                  # TickOutcome JSON (tick, paused, chronicle)
    def issue_verb(self, action_id: str, target_id: str | None,
                   target_community: str | None) -> str: ...    # remaining actions / outcome JSON
    def pin_watchlist(self, subject: str, pinned: bool) -> None: ...
    def save_nav_state(self, nav_json: str) -> None: ...        # jumplist/current-view persistence (BabylonMetaStore)
```

Entry point: `babylon_tui.run(host, config_json) -> None`, called from
`cli/play.py::run()` exactly where `ArchiveApp(...).run()` is called today. `config_json` carries:
campaign id, vault root, render tier (`glyph|pixel`, from the existing `--render` option), tutorial
enabled (tri-state resolved), narrator enabled, and the defines-hash/engine-version strings the
lobby displays today.

**Seam gap ledger** (the four methods the rev-1 doc left unassigned, now homed):

- `chronicle_salience` display rules (dedupe, volume floor, autopause tiers) ship **pre-computed
  by the host** inside `TickOutcome.chronicle` — Rust renders, never ranks.
- Statblock/narrative fenced directives: the vault bake already resolves directive payloads into
  the page markdown; `babylon-md` renders them as styled fenced blocks (data-driven — the fence
  info string selects the style; no directive logic in Rust).
- `NavPersistence`: `save_nav_state()` (above) persists via `BabylonMetaStore` on exit, mirroring
  the `pin_watchlist` write path; restored through `config_json` at launch.
- `backlinks_json`: `GameSession` has no counterpart today — the M1 host task **builds** the
  vault backlink-index read path (a real work item, not a delegation).

Rules of the seam:

- **JSON strings only across the FFI** (serde on the Rust side into typed view-models). The frozen
  Pydantic shapes are the contract; `model_dump_json()` on the Python side.
- **Callbacks only on the event-loop thread; GIL held** for every `host.*` call (pyo3
  `Python::with_gil`). No Rust worker threads touch Python.
- **Pull-on-render, not push:** each frame pulls only the visible view's data; `trend_json` pulls
  once per tick commit, not per frame.
- Determinism sealing (`PYTHONHASHSEED=0`, BLAS/rayon pins) happens in the CLI re-exec before Rust
  starts; Rust sets no Python env.
- The engine is never re-entered during a tick: `advance_tick()` runs the paced driver to the next
  pause/commit, then the read side unlocks (II.6).

## 7. Views (all live-data, all navigable, hover popups everywhere)

Global chrome: top bar with view tabs (Lobby/Wiki/Map/Topology/Market + page pane), chronicle
rail (right), watchlist rail, verb plate (bottom), fuzzy palette (`/`), tick controls
(`t` = step, `r` = run, `a` = acknowledge), jumplist (`Ctrl-O`/`Ctrl-I`, `[`/`]`), peek (`K` or
hover). Victoria-3 nested-view doctrine (R7); mouse + keyboard first-class (R3); honest absence
(III.11).

| View | Content & data source | Ratatui machinery |
|---|---|---|
| **Lobby** | Campaign catalog (BabylonMetaStore rows via host), new/resume | `List`, `Block`, `Clear` modal |
| **Wiki** | Baked vault markdown per known entity; `[[target\|alias]]` wikilinks → `babylon://` targets; redlinks; backlinks; concept cards; narrator blocks | `babylon-md` (tui-markdown fork) + pulldown-cmark native `ENABLE_WIKILINKS`; `tui-scrollview` pages; link hit-registry for hover/click |
| **Map** | Live choropleth: county/state tiers from `v_hex_state_asof` (never raw `dynamic_hex_state` — spec-089 sparse-delta discipline); lenses **value \| tension \| fog**; zoom/pan; county WKT polygons from reference data via host. **3D modes:** contradiction-field surface (release-blocking, BD-4) over the map from `field_state_json`; choropleth extrusion (best-effort) | `Canvas` `HalfBlock`/`Braille` for 2D; §7.1 scene builders → CellGrid blit for 3D; lens/mode switcher keybindings |
| **Topology** | PAOH bars, Levi ego-tree, incidence matrix (existing `projection/topology/{paoh,levi,incidence}.py` payloads via host); **hypergraph 3D render (release-blocking, BD-4)** — §7.1 builder → hypergraph-rs raster → cell grid; read-only (Amendment D) | `Canvas` custom `Shape`s for 2D; CellGrid blit for 3D; 2D views remain the glyph floor |
| **Stock Market** | `v_national_trend` series: imperial rent (+Δ), price⟷value scissors (price_log/fictitious_log + deltas), cumulative market corrections, total c/v/s, exploitation rate, profit rate; endgame axis bars (5 terminal outcomes). **3D mode:** trend ridgelines (best-effort) | `Chart` (Braille line datasets), `Sparkline` strip, `BarChart`, `Gauge` for O=C/B overshoot — all ratatui built-ins (research: they fully cover the 2D dashboard; candlestick crates exist if ever wanted) |
| **Tutorial overlay** | `WAYNE_OPENING_ARC` steps verbatim; predicates polled via `tutorial_state_json()`; `escape` dismiss | `Clear` + `tui-popup`; predicate evaluation stays Python-side (`game/tutorial_runtime.py`) |
| **Hover popups** | Peek plates (depth 0–3, port of `peek.py` semantics) on mouse hover over any link/entity glyph; `K` keyboard equivalent | crossterm SGR mouse capture (`EnableMouseCapture`, motion events); per-frame retained layout-rect registry for hit-testing; `Clear` overlay |

### 7.1 The 3D lane (BD-2/3/4/7/10)

One shared pipeline, four thin scene builders. hypergraph-rs provides the whole rasterization
core unchanged (§3.1); each target is a `SceneGraph3D` builder in `babylon-tui` (following the
existing instrument-builder pattern but **generic** — NOT the DeckWorld-bound builders, and NOT
an extension of the Babylon-economics ingest AttrContract):

| Target | Blocking? | Builder shape (research-verified template) |
|---|---|---|
| **Topology hypergraph** | **v1.0 (BD-4)** | nodes as `Node3` + hulls/edges as `Face`/`Strut` — `cylinder.rs`'s node/hull/strut assembly minus the DeckWorld/spectral coupling; positions from the topology payload (layout stays Python/rustworkx-side) |
| **Contradiction-field surface** | **v1.0 (BD-4)** | `terrain.rs`'s IDW quad-grid → Faces loop is already parameterized by generic `(x, y, scalar)` triples — closest to a direct lift; fed by `field_state_json` |
| **Choropleth extrusion** | best-effort | `sankey.rs`'s ground-polygon + riser-Strut pattern, PLUS new work in hypergraph-rs: real `h3o::CellIndex::boundary()` corner extraction (today's `ground_hex` is a synthetic regular hexagon) and a generalized fan-triangulate-any-polygon fn |
| **Trend ridgelines** | best-effort | stacked offset scalar curves as quad-strips (~100–150 LOC on the same Face/heat/depth-sort machinery; no crate anywhere implements ridgelines — confirmed custom) |

Render tiers: **glyph floor** = `rasterize() → CellGrid → to_text/to_ansi → Canvas blit`
(deterministic, byte-golden-testable, the ADR099 insurance that also ports to Windows);
**pixel tier** = `render_pixels() → PixelBuffer → [D-T3 bridge] → ratatui-image` (kitty TGP,
gated on §11 opens). Camera interaction (rotate/zoom) is client state; every `(scene, camera,
cols, rows) → frame` is a pure function — 3D frames golden-test exactly like 2D ones.

M0 de-risk (BD-3): the walking skeleton renders one hypergraph-rs fixture scene through the
git-dep → CellGrid → Canvas blit → TestBackend golden, proving the whole chain (remote, rev-pin,
Cell adapter, blit, snapshot) before any view work builds on it.

## 8. Testing strategy

- **Rust core (headless):** unit tests for markdown/wikilink rendering, JSON view-model decoding,
  reducers, layout registry, hit-testing, scene builders. `insta` snapshot tests over ratatui's
  `TestBackend` buffer → terminal-frame goldens (text-native, diffable — the rasterizer.md
  text-as-assertion-medium doctrine; research confirms TestBackend+insta is the ecosystem-standard,
  risk-free pairing). 3D frames golden-test as text via the CellGrid glyph tier. Regenerate
  freely; never ceremony.
- **Tutorial-BDD harness (the constitutional rewrite test):** ported from
  `tests/unit/tui/test_tutorial_pilot.py` (**1,172 LOC — the real, live parity gate**; rev-1's
  cite of `tui/shell/bdd/harness.py` was wrong — that 53-LOC module is dead scaffolding); drives
  the Rust client headless (TestBackend + scripted input events) against a real `GameSession` over
  `WayneCountyScenario` through the actual PyO3 boundary. Parity condition (explicit): every
  `TutorialStep` predicate in the WAYNE arc evaluates complete in order under scripted input, and
  the recorded transcript (step sequence + per-step rendered frame captured from `TestBackend`)
  matches step-for-step — frame text is golden-tested per step, not byte-compared against the old
  Textual SVGs (those die with the client per ADR099 doctrine). **BD Gate 3 runs here (BD-8).**
- **Python FFI contract tests (new lane `tests/unit/tui/`):** host adapter JSON shape per method
  (round-trip through `model_dump_json` → serde decode on a Rust test harness, or golden JSON
  files), verb round-trip (`issue_verb` decrements/actions), fog gating through the seam,
  `cli/play` wiring (mirrors today's `test_play.py`), `tutorial_runtime` predicate tests
  (unchanged).
- **Untouched lanes:** engine, projection, game, archive-integration goldens (vault byte-gate),
  `qa:regression` 6-scenario byte-identity, `check:gate-coverage` — all stay green; no engine-side
  change exists to move them.
- **Cutover ceremony (M7):** one declared commit — `test(cutover): retire Textual Archive lane` —
  deleting the Textual test files, `__snapshots__/`, Textual/textual-image/textual-plotext deps,
  and re-pointing the tutorial_coverage sentinel. Subject-to and body per the repo's ceremony
  discipline; baselines (`tests/baselines/**`) are NOT touched (no baseline ceremony needed).

## 9. Milestones (each leaves `mise run check` green; reordered per BD-3)

- **M0 — Foundations.** Amendment AC + ADR150 landed (this charter); in-tree `rust/` workspace;
  rust-toolchain + mise gate green; uv path-source + dev-time opt-in group + `uv lock`;
  hypergraph-rs remote + rev-pinned git-dep + deploy-key CI access (BD-10);
  `babylon_tui.run(host, config)` renders a hello-frame and exits clean; FFI round-trip test
  green; **cells3d walking-skeleton blit golden (BD-3, §7.1)**.
- **M1 — Read-only Archive.** Lobby → wiki view: `babylon-md` fork + native wikilinks + redlinks,
  jumplist, backlinks (host builds the read path — §6 gap ledger), peek plates, palette,
  watchlist read. Snapshot goldens for each entity-kind page.
- **M2 — Playable.** Tick controls wired to the paced driver, chronicle rail (host-computed
  salience), verb plate + `issue_verb` round-trip, watchlist pin writes, endgame HUD bars,
  nav-state persistence.
- **M3 — Tutorial gate.** Tutorial overlay + headless BDD harness; WAYNE arc passes against the
  Rust client; transcripts recorded. **← parity proven here; BD Gate 3 runs here (BD-8); cutover
  unblocked.**
- **M4 — The 3D lane (release-blocking half).** Generic scene builders + CellGrid/Canvas blit
  hardened; **topology hypergraph 3D + contradiction-field surface** (BD-4); camera controls;
  glyph-floor goldens; pixel tier via ratatui-image where §11 opens resolve.
- **M5 — Maps.** Choropleth (county/state), three lenses, zoom/pan, hover peek on regions;
  **extrusion mode best-effort** (needs the h3o boundary work in hypergraph-rs — slips to v1.1
  before it delays v1.0).
- **M6 — Stock Market.** Trend dashboards from `v_national_trend`; scissors chart; corrections
  counter; c/v/s + rates; endgame gauges; **ridgelines best-effort**.
- **M7 — Cutover ceremony (inside v1.0, BD-5).** Declared commit deleting the Textual lane + deps;
  sentinel re-pointed; **packaging flip: wheel into the default dependency set, CI Rust build leg,
  uv2nix closure gains `rust/`** (risk R6); `AGENTS.md`/`CLAUDE.md`, `ai/architecture.yaml`,
  `ai/state.yaml`, `ai/tooling.yaml` updated; docs revised; `mise run check` + `qa:regression`
  green post-delete.

Ordering rationale: the constitutional correctness gate (M3) stays before the visual-heavy views;
the 3D lane jumps the queue to M4 because it is now release-blocking (BD-3/4) — its riskiest
unknowns (git-dep chain, Cell adapter, blit) are burned down even earlier, at M0, by the walking
skeleton. Program sequencing: **P25 lands first (BD-6); M0 does not start before it merges.**

## 10. Risks & mitigations

- **R1 — Markdown/wiki parity** (was the heaviest workstream; now materially smaller). The
  markdown-it-py fenced-directive pipeline dies with Textual. Mitigation: **fork tui-markdown
  v0.3.9** (`babylon-md`) — headings/lists/tables/footnotes/GFM-alerts/syntect-highlighted fences
  already solved, actively maintained by a ratatui core-team member, MIT/Apache, ~4k LOC; native
  pulldown-cmark `ENABLE_WIKILINKS` replaces the custom inline pass. Genuinely new: the link
  hit-registry (ratatui `Span` carries no metadata slot — accumulate line/column ranges during
  render) and the two fork patches. Snapshot goldens per entity kind pin parity.
- **R2 — Hover hit-testing needs a retained layout registry.** Ratatui is immediate-mode; nothing
  retains rects, and there is no built-in mouse hit-testing. Mitigation: the core crate keeps a
  per-frame `LayoutRegistry` (widget id → rect → entity target); mouse motion events hit-test
  against it; keyboard `K` uses the same registry via focus. Standard pattern, but budget the
  fiddliness.
- **R3 — 3D lane scope** (rev-1's "cells3d maturity" risk is RETIRED — the research verified the
  core is done, generic, z-buffered, golden-tested). Remaining real work: four new builders, the
  `h3o` boundary extraction in hypergraph-rs (extrusion only), the D-T3 Rgb→bytes bridge, the
  Cell adapter. Mitigation: BD-4's blocking/best-effort split caps the v1.0 surface at the two
  builders with the cheapest templates (surface ≈ terrain.rs lift; hypergraph ≈ cylinder.rs minus
  DeckWorld); the M0 walking skeleton proves the chain before view work stacks on it.
- **R4 — GIL/event-loop integration bugs** (callbacks off-thread, re-entrancy during tick).
  Mitigation: single-threaded event loop discipline in the binding crate; `advance_tick` is the
  only mutating call and is strictly serialized; FFI contract tests include a tick-under-render
  interleaving test.
- **R5 — Scope.** This is a multi-week program. Mitigation: milestone gates; M3 is the go/no-go;
  the Textual client remains the playable client until M7.
- **R6 — Packaging (NEW; previously uncosted).** BD-5 means the maturin wheel must ship in the
  default install and the T7 uv2nix player closure — a real workstream: CI Rust build leg,
  uv2nix + maturin composition, wheel caching, closure-size audit. Mitigation: costed as explicit
  M7 tasks in the plan; the dev-time opt-in group defers the cost without hiding it; T7-beta
  builds strictly post-cutover (2026-07-23 ruling), so the closure work lands exactly once.
- **R7 — Cross-repo coordination via git-dep (NEW).** The client pins hypergraph-rs by rev;
  builder/API work happens in a sibling with its own history. Mitigation: rev bumps are explicit
  in-tree commits (reviewable, bisectable — this is the point of the pin); the deploy-key CI
  pattern is already proven by the nightly babylon-data verify; hypergraph-rs work items are
  tracked in the program issue so the seam never drifts silently.

## 11. Open questions

- ~~Exact `cells3d`/`raster` API surface~~ — **ANSWERED** by the 2026-07-27 research (§3.1/§7.1);
  the API is read, verified, and templated.
- **ratatui-image probe-once constructor**: does v11 expose construct-from-recorded-capability
  instead of runtime `from_query_stdio()` probing? ADR097 D4's probe-once rule requires it (or a
  thin wrapper). hypergraph-rs's own assessment tracks this as D-T7. Resolve at M4 kickoff.
- **kitty/TGP under tmux/SSH**: verify ratatui-image behavior before making the pixel tier
  default-on anywhere; the glyph floor is the fallback regardless.
- Stock-market dual-axis charts: ratatui `Chart` is one axis pair per widget — default to stacked
  single-axis (terminal-idiomatic), revisit in M6.

## 12. References

- `CONSTITUTION.md` (v2.17.0; II.5/II.6/II.8/II.11/III.7/III.11/III.12/III.13; Amendments D, V,
  AA, **AC**)
- `ai/decisions/ADR150_raster_cutover.yaml` — the recording ADR
- `ai/_inbox/post-v1.0.0/rasterizer.md` — the pre-blessed port analysis (correctness gate quote)
- `ai/_inbox/tui/20260719archiveinterfacedesign.md` — R1–R8, S1–S11 (the Archive charter)
- `ai/_inbox/tui/20260719archivestackresearch.md` — stack research (fenced-directive finding)
- `src/babylon/tui/app.py` — `CampaignHandle`/`PacedDriverHandle` Protocols (the seam being mirrored)
- `src/babylon/cli/play.py` — composition root (`run()`, `_load_campaign`, `_driver_factory`)
- `src/babylon/game/session.py` — `GameSession` (host-side composer)
- `src/babylon/projection/` — view-models, `registry.py` declared views, `topology/`,
  field-state dossier (T3), `epistemic_search.py`
- `src/babylon/game/tutorial.py`, `game/tutorial_runtime.py`,
  `tests/unit/tui/test_tutorial_pilot.py` — tutorial-is-BDD machinery (the rewrite test)
- `/home/user/projects/game/hypergraph-rs` — `raster`/`cells3d`/`raster-png` (the shared 3D
  rasterizer, BD-7/BD-10); its `plans/RATATUI-ASSESSMENT.md` (D-T3 pixel bridge, D-T7 probe-once)
- 2026-07-27 ecosystem research (workflow wf_7e5cfea4-256, 5 lanes, crates verified via the
  crates.io JSON API): tui-markdown v0.3.9, pulldown-cmark 0.13 wikilinks, ratatui-image v11.0.6,
  tui-widgets family, tachyonfx, insta; negative results — no z-buffered terminal rasterizer, no
  ridgeline crate, no n-ary hypergraph layout crate, no geographic choropleth crate

## 13. Rev-2 correction log (2026-07-27)

Recorded per the verification doctrine — rev 1 shipped with these factual errors, caught by the
2026-07-27 audit and corrected above: `src/babylon/tui/` is **7,866 LOC / 31 files** (was
"~4.5k"); the Pilot estate is **221 `run_test` sites / 574 test functions** (was "~100 tests");
the parity harness is **`test_tutorial_pilot.py` (1,172 LOC)** — `tui/shell/bdd/harness.py`
(53 LOC) is dead scaffolding (was cited as the harness); **ADR139/140 were already claimed** by
issues #259/#260 inside controller-allocated lane blocks (rev 1 assumed 139); the rev-1 structure
had the `tui` dependency group permanently opt-in while M7 flipped the default client to rust —
an uninstallable default on the documented path (resolved by BD-5's packaging flip, risk R6).
