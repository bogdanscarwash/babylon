# Ratatui Client (The Raster Cutover) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Rev 2 (2026-07-27):** rewritten for the BD interview rulings BD-1…BD-10 and the 5-lane
> ecosystem research (see design §3/§3.1/§13). Headline deltas vs rev 1: client home is **in-tree
> `rust/`** (was sibling repo); the 3D lane is **release-blocking and moves to M4** (was
> feature-flagged M5), seeded by an M0 walking skeleton; hypergraph-rs is consumed as a
> **rev-pinned cargo git-dependency**; the wiki stack is a **fork of tui-markdown v0.3.9** with
> pulldown-cmark's **native `ENABLE_WIKILINKS`** (no custom `[[...]]` pass); governance is
> **ADR150** (139/140 were lane-allocated); M7 gains the **packaging flip** (BD-5) resolving
> rev 1's opt-in-vs-default contradiction; the four orphan host methods are assigned (design §6
> gap ledger). Task count 1–47.

**Goal:** Replace the Textual ArchiveApp with a Rust/Ratatui terminal client (in-tree `rust/`
workspace, PyO3 extension) that passes the tutorial-BDD suite and ships lobby/wiki/map/topology/
stock-market views — including the release-blocking 3D lane (topology hypergraph +
contradiction-field surface) — with hover popups and live data.

**Architecture:** Rust owns the terminal event loop; Python (`GameSession`) remains the single writer and serves frozen view-models as JSON strings across a thin PyO3 seam (`RustClientHost`). Full design: `docs/superpowers/specs/2026-07-26-ratatui-client-design.md` (approved 2026-07-26; rev 2 2026-07-27).

**Tech Stack:** Rust 1.91.1 (MSRV 1.85), ratatui 0.30 (+ re-exported crossterm), serde/serde_json, pulldown-cmark 0.13 (native wikilinks), `babylon-md` (in-tree fork of tui-markdown v0.3.9), hypergraph-rs `raster`/`cells3d` as rev-pinned git-dep, ratatui-image v11 (pixel tier, gated), tui-popup/tui-scrollview/tui-tree-widget, pyo3 0.29, maturin ≥1.14,<2, insta; Python 3.12, uv path-source (`rust/`) + dev-time opt-in group, pytest.

**Sequencing (BD-6):** P25 lands first (#259 → #260 → merge PR #261). M0 does not start before it merges.

## Global Constraints

Every task implicitly includes ALL of the following (from the approved spec, rev 2):

- **JSON strings only across the FFI** — `model_dump_json()` on Python, serde on Rust; no Python objects cross except the single `host` handle.
- **Callbacks only on the event-loop thread with the GIL held** (`Python::with_gil`); no Rust worker thread ever touches Python.
- **Determinism sealing happens in the CLI re-exec before Rust starts** (`PYTHONHASHSEED=0`, BLAS/rayon pins, `src/babylon/cli/__init__.py`); Rust sets no Python env.
- **No engine/projection/game code changes** anywhere in this program (the declared exceptions: the `field_state_json`/backlink-index host read paths in `src/babylon/tui/`, which is client territory) — `mise run check` and `mise run qa:regression` (byte-identical) must stay green after every task. `tests/baselines/**` are NEVER touched (no baseline ceremony applies).
- **Honest absence (III.11):** missing data renders as a loud absence state, never a fabricated default.
- **No LLM in the input path (II.5/R4):** verbs enter only via `host.issue_verb(...)`.
- **Read-only hyperedge rendering (Amendment D):** no UI affordance implying hyperedge mutation.
- **Import-linter:** `babylon.tui` must not import `babylon.engine`, `babylon.persistence`, or django (`mise run lint:imports`).
- **Rust pins:** `rust/rust-toolchain.toml` channel `1.91.1`; workspace `rust-version = "1.85"`; `pyo3 = "0.29"`; `ratatui = "0.30"` (default features = crossterm backend — ALWAYS use `ratatui::crossterm` re-export, never a direct crossterm dep, to avoid version skew).
- **hypergraph-rs is a REV-PINNED git-dependency** (BD-10): `hypergraph-rs = { git = "<remote>", rev = "<sha>", default-features = false, features = ["cells3d"] }`. Rev bumps are explicit, reviewable in-tree commits — never `branch =`. ratatui stays OUT of hypergraph-rs's dependency graph (its own ruling); the `Cell → ratatui::Cell` adapter and the D-T3 `Rgb → bytes` bridge live in `babylon-tui`.
- **The `tui` dependency group is opt-in DURING DEVELOPMENT ONLY** — bare `uv sync` and CI never build Rust before M7. At M7 the wheel enters the default dependency set + the T7 uv2nix player closure (BD-5; Task 44). Rev 1's permanent-opt-in wording is superseded.
- **Commit discipline:** everything client-side is single-repo now — `mise run commit -- "type(scope): msg"`, conventional commits, `Co-Authored-By` trailer; never to `main`/`dev` directly — branch `feature/raster-cutover-*`. hypergraph-rs work items (h3o boundary, D-T3) commit in that repo on its own discipline, consumed here via rev bumps.
- **No `test_` prefix in production code; RST docstrings on Python public API; MyPy strict.**
- **Worktree note:** the uv path source `rust/` is in-repo — no sibling symlink is needed for the client (rev 1's `../babylon-tui-rs` gotcha is retired). The `../hypergraph-rs` uv-level note applies only to the unrelated Python `hypergraph` group; cargo fetches the git-dep itself.
- **Tutorial-is-BDD is the constitutional correctness gate:** M3 must pass before the M7 cutover; **BD Gate 3 runs at M3 on the Rust client (BD-8)**; until M7 the Textual client remains the playable default.

## Program Map (milestones → plans)

| Milestone | Deliverable | Gate | Plan granularity |
|---|---|---|---|
| **M0 Foundations** | In-tree `rust/` workspace, uv wiring, `run(host, config)` hello-frame, FFI round-trip, hypergraph-rs remote + git-dep + **walking-skeleton blit golden (BD-3)** | `mise run check` green; FFI + skeleton tests green | **Fully planned below (Tasks 1–10)** |
| **M1 Read-only Archive** | Lobby → wiki view (`babylon-md` fork + native wikilinks), jumplist, peek, palette, watchlist read, backlink read path | Page snapshot goldens per entity kind | **Fully planned below (Tasks 11–20)** |
| **M2 Playable** | Tick controls, chronicle, verb plate, `issue_verb`, endgame HUD, nav persistence | Verb round-trip test | Task-level (Tasks 21–26) |
| **M3 Tutorial gate** | Tutorial overlay + headless BDD harness; WAYNE arc passes | **PARITY GATE + BD GATE 3 — cutover unblocked** | Task-level (Tasks 27–29) |
| **M4 Topology + 3D lane** | PAOH/ego-tree/incidence 2D + **hypergraph 3D + field surface (release-blocking, BD-4)**; pixel tier gated | Glyph-floor frame goldens | Task-level (Tasks 30–36) |
| **M5 Maps** | Choropleth, lenses value/tension/fog, zoom/pan; **extrusion best-effort** | Canvas snapshot goldens | Task-level (Tasks 37–40) |
| **M6 Stock Market** | `v_national_trend` dashboards, scissors chart, endgame gauges; **ridgelines best-effort** | Trend goldens | Task-level (Tasks 41–43) |
| **M7 Cutover ceremony** | **Packaging flip (BD-5)**; Textual lane + deps deleted; docs updated | `mise run check` + `qa:regression` green post-delete | Task-level (Tasks 44–47) |

M2–M7 are planned at task level (files, exact interfaces, test names/assertions, acceptance
commands, commits). **At each milestone kickoff, expand its tasks into bite-sized red/green steps
via writing-plans before implementing** — interfaces below are the contract those expansions must
honor. M0/M1 need no expansion; they are executable as written.

---

# M0 — Foundations

## File structure (M0)

```
babylon/                                    # THIS repo (branch feature/raster-cutover-m0)
├── rust/                                   # NEW in-tree cargo workspace (BD-9)
│   ├── Cargo.toml                          # workspace
│   ├── rust-toolchain.toml                 # 1.91.1 + rustfmt + clippy
│   ├── pyproject.toml                      # maturin target for the uv path-source
│   ├── python/babylon_tui/{__init__.py,_core.pyi,py.typed}
│   └── crates/
│       ├── babylon-tui/                    # core: config, app, headless run, raster blit
│       │   ├── Cargo.toml
│       │   └── src/{lib.rs,config.rs,app.rs,raster_bridge.rs}
│       └── babylon-tui-python/             # cdylib FFI shell
│           ├── Cargo.toml
│           └── src/lib.rs
├── .mise.toml / mise config                # rust:check task added (cwd rust/)
├── pyproject.toml                          # [tool.uv.sources] babylon-tui = { path = "rust" } + [dependency-groups] tui
├── uv.lock                                 # regenerated on dev box
├── src/babylon/tui/host.py                 # RustClientHost (M0 surface: lobby)
├── tests/unit/tui/test_rust_client_ffi.py  # FFI round-trip contract test
└── src/babylon/cli/play.py                 # --client textual|rust option (default textual)

hypergraph-rs (sibling repo)                # gains: git remote + read-only deploy key (BD-10)
```

### Task 1: In-tree workspace scaffold

**Files:**
- Create: `rust/Cargo.toml`, `rust/rust-toolchain.toml`, `rust/.gitignore`
- Modify: the repo mise config (add `rust:check`)

**Interfaces:**
- Produces: cargo workspace at `rust/` with members `crates/babylon-tui`, `crates/babylon-tui-python`; a babylon `mise run rust:check` task (`cd rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo doc --no-deps`), task shape mirrored from `/home/user/projects/game/hypergraph-rs/.mise.toml`.

- [ ] **Step 1: Write `rust/Cargo.toml`**

```toml
[workspace]
members = ["crates/babylon-tui", "crates/babylon-tui-python"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "AGPL-3.0-or-later"   # in-tree = babylon's own license, not the sibling BSD
```

- [ ] **Step 2: Write `rust/rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.91.1"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 3: Write `rust/.gitignore` (`/target`, `/dist`, `*.whl`) and add the `rust:check` mise task to the repo config (dir-scoped to `rust/`).**

- [ ] **Step 4: Commit: `mise run commit -- "chore(rust): in-tree client workspace scaffold"`.**

### Task 2: Core crate — config + headless hello app (TDD)

**Files:**
- Create: `rust/crates/babylon-tui/Cargo.toml`, `src/lib.rs`, `src/config.rs`, `src/app.rs`, `tests/hello_frame.rs`

**Interfaces:**
- Produces (relied on by Tasks 3, 5–6, and all M1+ tasks):
  - `babylon_tui::config::AppConfig` — `pub struct AppConfig { pub campaign_id: String, pub campaign_name: String, pub render_tier: RenderTier, pub tutorial_enabled: bool, pub narrator_enabled: bool, pub headless: bool }`; `RenderTier { Glyph, Pixel }`; `impl AppConfig { pub fn from_json(s: &str) -> Result<Self, ConfigError> }`.
  - `babylon_tui::app::App<H: Host>` — `pub fn new(cfg: AppConfig, host: H) -> Self`; `pub fn render_frame<B: ratatui::backend::Backend>(&self, terminal: &mut Terminal<B>) -> std::io::Result<()>`.
  - `babylon_tui::host::Host` (trait) — M0 surface: `fn lobby_catalog_json(&self) -> String`.

- [ ] **Step 1: Write `rust/crates/babylon-tui/Cargo.toml`**

```toml
[package]
name = "babylon-tui"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
ratatui = "0.30"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
insta = { version = "1", features = ["filters"] }
```

- [ ] **Step 2: Write the failing config test (`src/config.rs` test module)**

```rust
#[test]
fn parses_minimal_config() {
    let cfg = AppConfig::from_json(
        r#"{"campaign_id":"c1","campaign_name":"Wayne","render_tier":"glyph",
            "tutorial_enabled":true,"narrator_enabled":false}"#,
    )
    .unwrap();
    assert_eq!(cfg.campaign_name, "Wayne");
    assert_eq!(cfg.render_tier, RenderTier::Glyph);
    assert!(!cfg.headless); // default false
}

#[test]
fn rejects_bad_tier() {
    assert!(AppConfig::from_json(
        r#"{"campaign_id":"c1","campaign_name":"W","render_tier":"3d",
            "tutorial_enabled":false,"narrator_enabled":false}"#
    )
    .is_err());
}
```

- [ ] **Step 3: `cargo test -p babylon-tui` → FAIL (no such item). Implement `src/config.rs` (serde derive, `#[serde(default)]` on `headless`, `#[serde(rename_all = "lowercase")]` on `RenderTier`) and `src/lib.rs` (`pub mod app; pub mod config; pub mod host;`) + `src/host.rs` (`pub trait Host { fn lobby_catalog_json(&self) -> String; }`). `cargo test` → PASS.**

- [ ] **Step 4: Write the failing hello-frame snapshot test (`tests/hello_frame.rs`)**

```rust
use babylon_tui::{app::App, config::AppConfig, host::Host};
use ratatui::{backend::TestBackend, Terminal};

struct FakeHost;
impl Host for FakeHost {
    fn lobby_catalog_json(&self) -> String {
        r#"[{"campaign_id":"c1","name":"Wayne County","tick":0}]"#.to_string()
    }
}

#[test]
fn hello_frame_shows_campaign() {
    let cfg = AppConfig::from_json(
        r#"{"campaign_id":"c1","campaign_name":"Wayne County","render_tier":"glyph",
            "tutorial_enabled":false,"narrator_enabled":false,"headless":true}"#,
    )
    .unwrap();
    let app = App::new(cfg, FakeHost);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    app.render_frame(&mut terminal).unwrap();
    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!(format!("{:?}", buffer));
}
```

- [ ] **Step 5: `cargo test -p babylon-tui --test hello_frame` → FAIL. Implement `src/app.rs`: parse `host.lobby_catalog_json()` (serde_json into `Vec<LobbyRow>`, honest-absence paragraph when empty), render a `Block::bordered().title("The Archive — <campaign_name>")` + the campaign list via `ratatui::widgets::List`. `cargo insta review` (or `INSTA_UPDATE=always cargo test` once) to bless, then `cargo test` → PASS.**

- [ ] **Step 6: `mise run rust:check` green. Commit: `mise run commit -- "feat(rust): core config + headless hello-frame with snapshot"`.**

### Task 3: PyO3 binding crate — `run(host, config_json)` with headless mode

**Files:**
- Create: `rust/crates/babylon-tui-python/Cargo.toml`, `src/lib.rs`
- Create: `rust/python/babylon_tui/__init__.py`, `_core.pyi`, `py.typed`
- Create: `rust/pyproject.toml` (maturin target)

**Interfaces:**
- Produces (relied on by Tasks 6–7 and ALL later milestones):
  - Python: `babylon_tui.run(host: Any, config_json: str) -> str` — runs the client; **returns a JSON transcript string** `{"frames": [<buffer text>...], "host_calls": [<method name>...]}`. In `headless: true` configs it renders one frame to `TestBackend` and returns immediately (no terminal I/O) — this is the CI-testable path.
  - Rust side: `PyHost` implements `babylon_tui::host::Host` by calling `host.<method>()` via `Python::with_gil` and recording each call name in a `Vec<String>` that becomes `host_calls`.

- [ ] **Step 1: Write `rust/crates/babylon-tui-python/Cargo.toml`**

```toml
[package]
name = "babylon-tui-python"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
name = "_core"
crate-type = ["cdylib"]

[dependencies]
babylon-tui = { path = "../babylon-tui" }
pyo3 = { version = "0.29", features = ["extension-module"] }
serde_json = "1"
```

- [ ] **Step 2: Write the failing Python-side round-trip test FIRST (red phase): `tests/unit/tui/test_rust_client_ffi.py`**

```python
"""FFI contract: babylon_tui.run drives a headless frame and records host calls."""

from __future__ import annotations

import json

import babylon_tui


class _FakeHost:
    def lobby_catalog_json(self) -> str:
        return json.dumps([{"campaign_id": "c1", "name": "Wayne County", "tick": 0}])


def test_run_headless_renders_and_records_calls() -> None:
    transcript = json.loads(
        babylon_tui.run(
            _FakeHost(),
            json.dumps(
                {
                    "campaign_id": "c1",
                    "campaign_name": "Wayne County",
                    "render_tier": "glyph",
                    "tutorial_enabled": False,
                    "narrator_enabled": False,
                    "headless": True,
                }
            ),
        )
    )
    assert "lobby_catalog_json" in transcript["host_calls"]
    assert "Wayne County" in transcript["frames"][0]
```

- [ ] **Step 3: Run (`mise run test:q -- tests/unit/tui/test_rust_client_ffi.py`) → FAIL (`ModuleNotFoundError: babylon_tui`).**

- [ ] **Step 4: Implement `rust/crates/babylon-tui-python/src/lib.rs`**

```rust
use babylon_tui::{app::App, config::AppConfig, host::Host};
use pyo3::prelude::*;
use ratatui::{backend::TestBackend, Terminal};

struct PyHost {
    obj: Py<PyAny>,
    calls: std::cell::RefCell<Vec<String>>,
}

impl Host for PyHost {
    fn lobby_catalog_json(&self) -> String {
        self.calls.borrow_mut().push("lobby_catalog_json".into());
        Python::with_gil(|py| {
            self.obj
                .call_method0(py, "lobby_catalog_json")
                .and_then(|v| v.extract::<String>(py))
                .unwrap_or_else(|_| "[]".into()) // honest absence: empty catalog
        })
    }
}

#[pyfunction]
fn run(py: Python<'_>, host: Py<PyAny>, config_json: &str) -> PyResult<String> {
    let cfg = AppConfig::from_json(config_json)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let py_host = PyHost { obj: host, calls: vec![].into() };
    let app = App::new(cfg.clone(), py_host);
    py.allow_threads(|| {
        if cfg.headless {
            let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
            app.render_frame(&mut t).unwrap();
            let frame = format!("{:?}", t.backend().buffer());
            let calls = app.host_calls();
            Ok(serde_json::json!({"frames": [frame], "host_calls": calls}).to_string())
        } else {
            babylon_tui::app::run_interactive(app) // M0: same frame + quit on q/Esc
                .map(|calls| serde_json::json!({"frames": [], "host_calls": calls}).to_string())
                .map_err(|e| e.to_string())
        }
    })
    .map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)
}
```

(`App` gains `fn host_calls(&self) -> Vec<String>` in the core crate; `run_interactive` does crossterm init → same render → poll for `q`/`Esc` → restore → return calls. Add both with their own red/green steps. **GIL note:** every `host.*` call re-acquires via `Python::with_gil` inside the loop thread — the `allow_threads` wrapper releases only between callbacks.)

- [ ] **Step 5: Write `rust/pyproject.toml` and the Python shell**

```toml
[build-system]
requires = ["maturin>=1.14,<2.0"]
build-backend = "maturin"

[project]
name = "babylon-tui"
dynamic = ["version"]
dependencies = []

[tool.maturin]
manifest-path = "crates/babylon-tui-python/Cargo.toml"
module-name = "babylon_tui._core"
python-source = "python"
strip = true
```

```python
# rust/python/babylon_tui/__init__.py
"""Babylon Archive client (Rust/Ratatui). Loud-failure import doctrine."""
from babylon_tui import _core as _core

run = _core.run

__all__ = ["run"]
```

`_core.pyi`: `def run(host: object, config_json: str) -> str: ...`; empty `py.typed`.

- [ ] **Step 6: `uvx maturin develop` in `rust/` (with babylon's venv active, after Task 4 wiring) → build green. Re-run the Step-2 pytest → PASS. `mise run rust:check` green. Commit: `mise run commit -- "feat(rust): run(host, config_json) FFI with headless transcript"`.**

### Task 4: babylon uv wiring (in-tree path source; NO CI stub)

**Files:**
- Modify: `pyproject.toml` (`[tool.uv.sources]`, `[dependency-groups]`)
- Modify: `uv.lock` (via `uv lock` on dev box)

**Interfaces:**
- Produces: `uv sync --group tui` builds the extension into the venv; bare `uv sync` and CI never build it (until the M7 flip, Task 44).

- [ ] **Step 1: Add to babylon `pyproject.toml`**

```toml
# In [tool.uv.sources]:
babylon-tui = { path = "rust" }

# In [dependency-groups] (opt-in DURING DEVELOPMENT — flips to a default
# dependency at M7 per BD-5/Task 44; enable with `uv sync --group tui`;
# after Rust edits run `uvx maturin develop` in rust/):
tui = [
    "babylon-tui",
]
```

- [ ] **Step 2: `uv lock` then `uv sync --group tui && (cd rust && uvx maturin develop)`; verify `python -c "import babylon_tui; print(babylon_tui.run)"` in the venv. NOTE: no CI stub work — the in-tree path source is always present in every checkout (rev 1's `ci_hypergraph_stub.sh` generalization is retired unbuilt; the script is untouched).**

- [ ] **Step 3: Run the red test from Task 3 Step 2 → now PASS. `mise run check` → green.**

- [ ] **Step 4: Commit: `mise run commit -- "build(tui): consume in-tree rust/ client as opt-in uv path source"`.**

### Task 5: hypergraph-rs remote + rev-pinned git-dep + walking-skeleton blit (BD-3/BD-10)

**Files:**
- hypergraph-rs repo: add the git remote; register a read-only deploy key (nightly babylon-data precedent) — owner/ops half is a one-time ceremony, record the remote URL in the ADR150 file.
- Modify: `rust/crates/babylon-tui/Cargo.toml` (git-dep behind a `raster` feature)
- Create: `rust/crates/babylon-tui/src/raster_bridge.rs`, `tests/raster_skeleton.rs`

**Interfaces:**
- Produces (the entire M4 lane stands on this):
  - `raster_bridge::blit(grid: &hypergraph_rs::raster::CellGrid, ctx: &mut ratatui canvas/buffer target)` — the `Cell{ch,fg,bg} → ratatui::Cell` adapter (isomorphic per hypergraph-rs's own RATATUI-ASSESSMENT).
  - A passing end-to-end golden: fixture `SceneGraph3D` (reuse a `tests/fixtures/scene3d/*.json` shape) → `rasterize(scene, camera, cols, rows)` → `blit` → `TestBackend` → insta snapshot.

- [ ] **Step 1: Cargo.toml — `[features] raster = ["dep:hypergraph-rs"]`; `hypergraph-rs = { git = "<remote>", rev = "<pin>", default-features = false, features = ["cells3d"], optional = true }`. `cargo build --features raster` fetches and compiles.**
- [ ] **Step 2: Red — `tests/raster_skeleton.rs` (feature-gated): load the fixture scene JSON, rasterize at 80×24, blit, snapshot. FAIL (no bridge).**
- [ ] **Step 3: Implement `raster_bridge.rs` (pure, deterministic — no clock, no rand). Bless the snapshot. `mise run rust:check` green (with and without `--features raster`).**
- [ ] **Step 4: Commit: `mise run commit -- "feat(rust): hypergraph-rs raster git-dep + walking-skeleton blit golden"`. This is the BD-3 de-risk: remote, rev-pin, adapter, blit, and golden all proven before any view work.**

### Task 6: `RustClientHost` M0 surface + contract test

**Files:**
- Create: `src/babylon/tui/host.py`
- Test: `tests/unit/tui/test_host_contract.py`

**Interfaces:**
- Consumes: `GameSession`-shaped objects and the lobby catalog (`BabylonMetaStore`), exactly as `cli/play.py` already composes them.
- Produces (THE seam; grows per milestone — signatures binding for M1–M6):
  - `RustClientHost(catalog: CampaignCatalog, *, defines_hash: str, engine_version: str)`
  - `lobby_catalog_json() -> str` (M0, real)
  - `bind_session(session: GameSession, driver: PacedDriverHandle) -> None` (M0, real)
  - M1+: `read_page(subject) -> str | None`, `known_subjects_json() -> str`, `backlinks_json(subject) -> str` (host BUILDS the vault backlink-index read path — design §6 gap ledger), `subject_view_json(subject) -> str`
  - M2+: `advance_tick() -> str`, `issue_verb(action_id, target_id=None, target_community=None) -> str`, `verb_plate_view_json() -> str`, `endgame_status_json() -> str`, `pin_watchlist(subject, pinned) -> None`, `watchlist_json() -> str`, `save_nav_state(nav_json) -> None`
  - M3: `tutorial_state_json() -> str`
  - M4: `topology_json(kind, focus=None) -> str`, `field_state_json() -> str`
  - M5: `choropleth_json(tier, lens) -> str`
  - M6: `trend_json(last_n) -> str`, `dashboard_view_json() -> str`

- [ ] **Step 1: Failing test `tests/unit/tui/test_host_contract.py` — construct `RustClientHost` over `InMemoryCampaignCatalog` (import from `babylon.tui.campaign_menu`), assert `json.loads(host.lobby_catalog_json())` round-trips the catalog rows and `defines_hash` appears. Run → FAIL.**

- [ ] **Step 2: Implement `src/babylon/tui/host.py` — thin adapter; RST docstrings; MyPy-strict; every `*_json` method returns `model_dump_json()` or `json.dumps` of primitives; honest absence = `None`/empty list, never fabricated rows. Run → PASS. `mise run check` green.**

- [ ] **Step 3: Commit: `mise run commit -- "feat(tui): RustClientHost seam (M0 lobby surface)"`.**

### Task 7: `cli/play.py --client` wiring

**Files:**
- Modify: `src/babylon/cli/play.py` (`run()` signature + ArchiveApp/Rust branch)
- Test: `tests/unit/cli/test_play.py` (extend existing patterns)

**Interfaces:**
- Produces: `babylon play --client textual|rust` (default `textual` until M7). The rust branch: build catalog + `RustClientHost`, `host.bind_session(...)` inside `campaign_loader`, then `babylon_tui.run(host, config_json)` where `ArchiveApp(...).run()` is called today. Import of `babylon_tui` is lazy (inside the branch) so the opt-in group stays opt-in.

- [ ] **Step 1: Failing test — `--client rust` without the extension installed raises a loud, actionable `RuntimeError` (red).**
- [ ] **Step 2: Implement the option + lazy import + branch (green). `--client textual` path byte-identical to today.**
- [ ] **Step 3: `mise run check` + `mise run qa:regression` green (no engine change; prove it).**
- [ ] **Step 4: Commit: `mise run commit -- "feat(cli): --client rust lane behind opt-in group"`.**

### Task 8: Governance — LANDED PRE-M0

Amendment AC (CONSTITUTION.md v2.17.0) + ADR150 + `index.yaml` were landed by the 2026-07-27
charter revision (the commit series carrying this plan rev). At M0 kickoff: verify they merged,
flip ADR150's `status` note if the BD amends anything, and cite ADR150 in the M0 PR body. No
drafting work remains here.

### Tasks 9–10: M0 close-out

- [ ] **Task 9:** `mise run rust:check` + `mise run check` + `mise run qa:regression` all green; record outputs in the PR description. Update `ai/state.yaml` (raster cutover M0 done); one-line `AGENTS.md`/`CLAUDE.md` note that the Rust client lane exists behind `--client rust` + `--group tui`. Commit: `docs(state): raster cutover M0`.
- [ ] **Task 10:** Manual smoke: `babylon play --client rust` (real terminal, Postgres up via the normal runtime) → hello-frame renders the real lobby catalog; `q` quits clean. Record the transcript JSON in the milestone notes.

---

# M1 — Read-only Archive

## File structure (M1)

```
rust/crates/
├── babylon-md/            # FORK of tui-markdown v0.3.9 (MIT/Apache-2.0 headers preserved)
│   └── src/...            # two surgical patches, see Task 11
└── babylon-tui/src/
    ├── wiki_render.rs     # babylon-md integration + wikilink → hit-registry wiring
    ├── router.rs          # babylon:// URI parsing (port of tui/router.py semantics)
    ├── layout_registry.rs # per-frame widget id -> Rect -> entity target (hover/peek foundation)
    ├── views/{lobby.rs,wiki.rs,palette.rs,watchlist.rs,peek.rs}
    └── app.rs             # gains: view stack, jumplist, key/mouse routing
```

### Task 11: Vendor `babylon-md` (tui-markdown fork) + the two patches (TDD)

**Files:** Create `rust/crates/babylon-md/` (forked from tui-markdown v0.3.9 — MIT OR Apache-2.0, license files + attribution preserved; add to workspace members), `rust/crates/babylon-tui/src/wiki_render.rs`, `tests/wiki_render.rs`

**Interfaces:** Produces `pub fn render_page(src: &str, width: u16, known: &BTreeSet<String>) -> (ratatui::text::Text<'static>, Vec<LinkSpan>)`. The fork carries exactly two patches (keep the diff minimal — upstream is actively maintained and we may rebase): (1) **Options passthrough** — expose `pulldown_cmark::Options` on the parser-construction call so `ENABLE_WIKILINKS` can be set (upstream hardcodes its option set); (2) **link metadata retention** — `link.rs` currently renders the destination as trailing literal text and discards `LinkType`; patch it to emit link start/end + `LinkType::WikiLink`/dest through a callback the caller uses to build `LinkSpan`s (ratatui `Span` has no metadata slot — the side-channel IS the design). Everything else (headings, lists, box-drawn tables, blockquotes, GFM alerts, syntect-highlighted fences via the bundled ansi-to-tui pipeline) is inherited working. Fenced directive blocks render as styled fences (fence-only rule; payloads pre-resolved by the vault bake).

- [x] Steps: vendor the crate + license/NOTICE → failing tests (heading style, fence block, `[[Target]]` → LinkSpan with `exists=false` when unknown, `[[Target|Label]]` pothole, empty→honest-absence) → apply the two patches → insta snapshots per fixture in `tests/fixtures/markdown/*.md` → `mise run rust:check` → commit `feat(rust): babylon-md fork with wikilink passthrough`.

### Task 12: Wikilink semantics parity (TDD)

**Files:** `rust/crates/babylon-tui/tests/wikilinks.rs`

**Interfaces:** Native `ENABLE_WIKILINKS` (pulldown-cmark 0.13) replaces rev 1's custom inline pass — there is NO scanning code to write. This task ports the behavior TABLE from `tests/unit/tui/test_wikilinks.py` as contract tests over `render_page`: bare `[[t]]`, aliased `[[t|Label]]`, unknown target → redlink style + `exists=false`, `babylon://` target normalization (via Task 13's router). Document the two upstream parser quirks for content authors: pipe cannot be backslash-escaped inside a wikilink; empty `[[]]` renders literal.

- [x] Steps: red (ported table) → green (wiring only, no parser code) → commit `test(rust): wikilink semantics parity table`.

### Task 13: Router (TDD)

**Files:** Create `rust/crates/babylon-tui/src/router.rs`, `tests/router.rs`

**Interfaces:** Produces `pub enum BabylonTarget { Entity(String), Kind { kind: String, id: String }, Redlink(String) }` and `pub fn parse_babylon_uri(uri: &str) -> Result<BabylonTarget, RouterError>` — `babylon://<kind>/<id>`, `babylon://<id>`, `babylon://redlink/<id>`. Port the case table from `tests/unit/tui/test_router.py` verbatim.

- [x] Steps: red (ported cases) → green → commit `feat(rust): babylon:// router`.

### Task 14: Layout registry + hover/peek foundation (TDD)

**Files:** Create `rust/crates/babylon-tui/src/layout_registry.rs`, `src/views/peek.rs`, `tests/layout_registry.rs`

**Interfaces:** Produces `pub struct LayoutRegistry { rects: Vec<(WidgetId, Rect, Option<String>)> }` with `pub fn register(&mut self, id: WidgetId, area: Rect, entity: Option<String>)` and `pub fn hit(&self, col: u16, row: u16) -> Option<&(WidgetId, Rect, Option<String>)>`; `App::handle_mouse(MouseEvent)` → on `Moved`, `hit()` → set `peek_target: Option<String>`; `views/peek.rs::render_peek(frame, area, subject_view_json, depth)` rendering depths 0–3 (port depth semantics from `src/babylon/tui/peek.py`). Mouse, not OSC 8 (ratatui#1227 unresolved — design R1).

- [x] Steps: failing hit-test unit tests (nested rects → innermost wins; miss → None) → implement → commit `feat(rust): layout registry + hover peek foundation`.

### Task 15: WikiView (TDD)

**Files:** Create `rust/crates/babylon-tui/src/views/wiki.rs`, `tests/wiki_view.rs`

**Interfaces:** Consumes Tasks 11–14 + host `read_page`/`known_subjects_json`. Produces `pub struct WikiView { current: Option<String>, jumplist: Vec<String>, jumplist_idx: usize }` with `pub fn open(&mut self, target: &BabylonTarget, host: &dyn Host)` — redlink target renders the honest-absence page; `Ctrl-O`/`Ctrl-I` and `[`/`]` traverse the jumplist. Rendering pipeline: `read_page` → `render_page` (babylon-md) → link spans registered into `LayoutRegistry` (hover/click navigates; hover peeks); long pages scroll via `tui-scrollview`.

- [x] Steps: failing snapshot tests (entity page with links; redlink page; jumplist back/forward across 3 pages) → implement → commit `feat(rust): wiki view with jumplist navigation`.

### Task 16: LobbyView (TDD)

**Files:** Create `rust/crates/babylon-tui/src/views/lobby.rs`, `tests/lobby_view.rs`

**Interfaces:** Consumes `host.lobby_catalog_json()`. Produces `pub struct LobbyView { rows: Vec<LobbyRow>, selected: usize }`; `Enter` on a row → `AppEvent::LoadCampaign(Uuid)`; `n` → `AppEvent::NewCampaign`. Empty catalog → honest-absence body. Mirrors `campaign_menu.py` display fields (name, tick, engine version, defines hash).

- [x] Steps: failing snapshot tests (populated, empty, selection move) → implement → commit `feat(rust): lobby view`.

### Task 17: Host M1 surface (Python, TDD) — includes the backlink-index build

**Files:** Modify `src/babylon/tui/host.py`; Test `tests/unit/tui/test_host_contract.py` (extend)

**Interfaces:** Implements `read_page(subject) -> str | None` (delegates to the vault read path — read-only, never bakes), `known_subjects_json() -> str` (sorted list from `known_subjects()`), `backlinks_json(subject) -> str` — **`GameSession` has no counterpart today (design §6 gap ledger): build the vault backlink-index read path here**, `subject_view_json(subject) -> str` (`GameSession.subject_view` → `model_dump_json`).

- [x] Steps: failing contract tests (round-trip a `CountyView` fixture through `subject_view_json` and assert the JSON decodes with the expected tag; unknown subject → `None`; backlinks over a 3-page fixture vault) → implement → `mise run check` → commit `feat(tui): host M1 read surface + backlink index`.

### Task 18: Palette + watchlist read (TDD)

**Files:** Create `rust/crates/babylon-tui/src/views/palette.rs`, `src/views/watchlist.rs`, tests

**Interfaces:** Palette: `/` opens fuzzy input over `known_subjects_json`; `Enter` → `open()`. Watchlist rail: renders `watchlist_json()` rows; click/`p` navigation. (Pin WRITES land in M2 Task 25.)

- [x] Steps: failing snapshot tests → implement → commit `feat(rust): palette + watchlist read view`.

### Task 19: App shell integration — view stack + key/mouse routing (TDD)

**Files:** Modify `rust/crates/babylon-tui/src/app.rs`, `rust/crates/babylon-tui-python/src/lib.rs`; tests

**Interfaces:** `App` gains `view_stack: Vec<View>` (Lobby root), global bindings (tab/`1`–`5` view switch scaffold, `q` back/quit, `K` peek, `/` palette), crossterm mouse capture enabled in `run_interactive`, `handle_mouse` wired to `LayoutRegistry`. Headless mode gains scripted-input replay: `config_json` accepts `"script": [{"key": "..."}, {"mouse": [c, r]}...]` so integration tests drive flows without a terminal (this is the BDD harness foundation M3 builds on).

- [x] Steps: failing scripted-input integration test (lobby → open campaign page → back → quit; assert transcript frame count + host call order) → implement → `mise run rust:check` + `mise run check` → commit `feat(rust): app shell — view stack, global keys, mouse, scripted-input headless`.

### Task 20: M1 close-out

- [x] Both gates green; manual smoke `babylon play --client rust` browses the REAL vault for a live campaign; `ai/state.yaml` updated; commit `docs(state): raster cutover M1`.

---


**M1 CLOSED 2026-07-27** (branch `feature/ratatui-m1`). Recorded deviations:
`tui-scrollview` was NOT adopted — WikiView wraps with its own span-preserving
word wrap because the layout registry needs the exact display cells each
wikilink occupies and ratatui-side wrapping discards that mapping; scrolling is
`Paragraph::scroll` clamped to content. Additions beyond the task list, from
the adversarial verify panel: `Host::load_campaign` (the production
composition-root verb — plan Task 7's `bind_session` had no caller),
loud-failure FFI (a raising host panics across the seam, III.11), keyboard
link cursor `n`/`p`/`Enter` feeding peek, the backlinks footer as the Task 17
seam's consumer, lobby `codename` display (spec-116), theme.rs + the
cross-language palette guard, and loud parse-failure states in
lobby/palette/watchlist. Real-vault smoke: campaign minted via the real menu,
Enter → briefing rendered from the live Postgres vault, exact seam call order
pinned. TTY smoke remains owner-side (as M0 Task 10).

# M2 — Playable (task-level; expand at kickoff)

- [x] **Task 21 — Tick controls.** Rust: `t`/`r`/`a` bindings → `host.advance_tick()` (step), run-until-paused loop flag, acknowledge. Python: `advance_tick()` delegates to the paced driver → `TickOutcome` JSON (`tick`, `paused`, `chronicle: [...]`). Tests: Rust scripted-input tick test with a fake host; Python contract test over `tests/unit/game/test_pacing.py` fixture shapes. Commit `feat(tui): tick controls`.
- [x] **Task 22 — Chronicle rail.** Renders `TickOutcome.chronicle` (salience/dedupe/volume-floor/autopause rules from `tui/chronicle_salience.py` ship as DATA the host pre-computes — Rust renders, never ranks; design §6 gap ledger). Snapshot goldens. Commit `feat(rust): chronicle rail`.
- [x] **Task 23 — Verb plate + issue_verb.** Renders `verb_plate_view_json()` (9 Article-V verbs, OODA-gated disabled states); F1–F9 + click dispatch → `host.issue_verb(action_id, target_id, target_community)` → re-render. Tests: round-trip contract test (verb decrements remaining actions, mirrors `tests/integration/archive/test_verb_resolution.py` shapes at unit level); scripted-input verb dispatch test. Commit `feat(tui): verb plate + dispatch`.
- [x] **Task 24 — Endgame HUD.** 5 terminal-outcome axis bars from `endgame_status_json()` (`Gauge`/`BarChart`); honest absence pre-game. Commit `feat(rust): endgame HUD`.
- [x] **Task 25 — Watchlist writes + nav persistence.** `p` pin/unpin → `pin_watchlist(subject, pinned)` → `BabylonMetaStore` persistence; rail re-renders. `save_nav_state(nav_json)` on exit + restore via `config_json` at launch (design §6 gap ledger — the NavPersistence home). Contract test round-trips both. Commit `feat(tui): watchlist pin writes + nav persistence`.
- [x] **Task 26 — M2 close-out.** Gates green; manual smoke plays 5 real ticks with one real verb; `ai/state.yaml`. Commit `docs(state): raster cutover M2`.

**M2 CLOSED 2026-07-27** (branch `feature/ratatui-m2`; contracts:
`docs/superpowers/specs/2026-07-27-m2-seam-contracts.md`). Recorded
deviations, all scout/panel-driven: the Task-23 test asserts the two REAL
`test_verb_resolution.py` behaviors, never "decrements remaining actions"
(the action-point budget is dormant — no production caller);
`pin_watchlist`/tick verbs return ok-envelopes (the sketch's `-> None`
would crash on the player-reachable capacity ValueError); nav restore
rides a post-bind `nav_state_json()` pull (config_json predates
selection) and Back-to-lobby is the sole save point; salience rides a
host-owned accumulator (`chronicle_rail_json` — rules span ticks); NO
"quiet" row kind (chronicle_stream never emits an empty bulletin); `P`
pins (lowercase `p` = wiki link cursor; rail-`p` toggles the highlighted
row). Additions from the 43-finding Opus verify panel: visible focus
(`Tab` cycle, ● marker, focus-gated highlights), Esc = rail defocus
(never campaign teardown), honest-unreadable pacing/outcome states
(never "campaign ended" from a parse failure), HUD 8-cell gauges (five
fit 80 cols), verb-plate preview+warnings render + CLICK dispatch,
highlight preservation across refreshes, nav dedupe/cap/ack-check.
Real-vault smoke: 5 real engine ticks + a real verb + pin/unpin
persistence across sessions, through the production composition root.
TTY smoke remains owner-side.

# M3 — Tutorial gate (task-level; expand at kickoff) — **PARITY GATE + BD GATE 3 (BD-8)**

**M3 EXECUTED 2026-07-27** (branch `feature/ratatui-m3`; contracts:
`docs/superpowers/specs/2026-07-27-m3-tutorial-contracts.md`, incl. the
integration addendum §9 — recorded deviations: call1 not call0, hand-rolled
top strip not tui-popup, host-side statblock-fence fill, the county content
check pinned to the honest LIVE epistemic surface (the pilot's own row is
fixture-fed — Gate 3 agenda), verb_log not reset on bind, the M2 Esc
rail-defocus arm shipped here after the port found it missing). The full
24-step WAYNE arc is green against the Rust client (13-test harness incl.
two-run byte-identical transcript determinism); **BD Gate 3 itself remains
the Director's ceremony — Task 29 stays open until it runs.**

- [x] **Task 27 — Tutorial overlay.** `tutorial_state_json()` (current step id, `overlay_text`, per-predicate completion) polled per frame; `Clear`+`tui-popup` render; `escape` dismiss. Python: adapter over `game/tutorial_runtime.py::TutorialRuntimeProgress` (predicates stay Python — Rust never evaluates them). Snapshot goldens per overlay fixture. Commit `feat(tui): tutorial overlay`.
- [x] **Task 28 — Headless BDD harness.** Port the semantics of `tests/unit/tui/test_tutorial_pilot.py` (**the real 1,172-LOC parity gate** — rev 1 miscited the dead 53-LOC `shell/bdd/harness.py`): a Python pytest harness (`tests/unit/tui/test_tutorial_pilot_rs.py`) that builds a real `GameSession` over `WayneCountyScenario`, wraps it in `RustClientHost`, feeds each `TutorialStep`'s `when` as scripted input through `babylon_tui.run(..., headless, script=...)`, and asserts each `then` predicate completes **in order**; records the transcript (step sequence + per-step `TestBackend` frame) to `tests/unit/tui/transcripts/wayne_opening_arc.json` (golden, regenerate-freely doctrine).
- [ ] **Task 29 — Gate run.** The full WAYNE arc passes against the Rust client; transcript blessed; **BD Gate 3 (#262) is run here, on the Rust client — a combined content+client session (BD-8; informal Textual playtests remain free anytime as the early-warning mitigant)**; results appended to ADR150 (status note: parity PROVEN, cutover unblocked). Commit `test(tutorial): WAYNE arc green against Rust client — parity gate`.

# M4 — Topology + the 3D lane (task-level; expand at kickoff) — **release-blocking half of BD-4**

- [ ] **Task 30 — Topology host surface.** `topology_json(kind, focus)` → `projection/topology/{paoh,levi,incidence}.py` payloads as JSON; `field_state_json()` → the T3 field-state dossier projection. Contract tests. Commit `feat(tui): topology + field host surface`.
- [ ] **Task 31 — 2D renderers (the glyph floor).** PAOH bars (`BarChart`/canvas), Levi ego-tree (canvas `Line`/`Points` layout — layout computed deterministically), incidence matrix (cell grid). Goldens per fixture (port `test_egotree_directive.py`/`test_matrix_directive.py` fixtures). Commit `feat(rust): topology 2D views`.
- [ ] **Task 32 — Raster pipeline hardening.** Promote the M0 walking skeleton to production: `scene` module (builder traits), `Camera` interaction state (rotate/zoom keys — camera is client state; `(scene, camera, cols, rows) → frame` stays a pure function), blit path sized to the view rect, glyph-floor golden harness shared by Tasks 33–34/39/42. Commit `feat(rust): raster pipeline`.
- [x] **Task 33 — Hypergraph 3D builder (release-blocking).** Generic builder: `(id, pos, radius, color)` nodes + edge/hull lists → `SceneGraph3D` (`Node3` + fan-triangulated hull `Face`s + `Strut`s) — hypergraph-rs `cylinder.rs` is the verified template, minus its DeckWorld/spectral coupling; positions come from the topology payload (layout stays Python/rustworkx-side). Read-only (Amendment D). Frame goldens at 2 camera angles. Commit `feat(rust): hypergraph 3D render`.
- [x] **Task 34 — Contradiction-field surface (release-blocking).** Generic scalar-surface builder over `(x, y, scalar)` triples → IDW/quad-grid `Face` mesh with heat coloring — `terrain.rs`'s loop is already scalar-generic (the research's "closest to a direct lift"); fed by `field_state_json`. Frame goldens. Commit `feat(rust): contradiction-field 3D surface`.
- [x] **Task 35 — Pixel tier (gated).** Resolve the two design-§11 opens at kickoff (ratatui-image probe-once constructor for ADR097 D4; kitty/TGP under tmux/SSH). If green: the D-T3 `Rgb → bytes` bridge over `render_pixels()` + `ratatui-image` `StatefulImage` behind `render_tier: pixel`, honoring the recorded probe (never re-probing at runtime). If not green: record the deferral in ADR150 — the glyph floor ships regardless and is the ADR099 insurance. Commit `feat(rust): kitty pixel tier` (or the deferral note).
- [x] **Task 36 — M4 close-out.** Gates; smoke: rotate the live hypergraph + field surface in a real campaign; `ai/state.yaml`. Commit `docs(state): raster cutover M4`.

# M5 — Maps (task-level; expand at kickoff)

- [x] **Task 37 — Choropleth host surface.** `choropleth_json(tier, lens)` over `projection/topology/choropleth_aggregation.py` (reads `v_hex_state_asof` ONLY — spec-089 discipline) + county WKT polygons from the reference read path; `MapTier = county|state`, `lens = value|tension|fog`. Contract tests with fixture cells. Commit `feat(tui): choropleth host surface`.
- [x] **Task 38 — MapView canvas.** `Canvas` + `HalfBlock` marker; custom `Shape` impl filling WKT polygons colored by lens band (port band thresholds from `tui/map_room.py::_band_color` as host-shipped data — bands are DATA, not Rust literals); centroid dots for hexes. Snapshot goldens per lens. Commit `feat(rust): map view canvas`.
- [x] **Task 39 — Extrusion mode (best-effort, BD-4).** Prereq in hypergraph-rs (its repo, consumed via rev bump): real `h3o::CellIndex::boundary()` corner extraction (today's `ground_hex` is a synthetic regular hexagon) + a fan-triangulate-any-polygon fn generalizing it. Then: extruded-choropleth builder (`sankey.rs`'s ground-polygon + riser-`Strut` + height pattern) behind a map mode key. **Sliceable: if this would delay v1.0, it slips to v1.1 with a one-line ADR150 note — before it delays anything else.** Commit `feat(rust): choropleth extrusion mode`. *(EXECUTED AS THE SLICE: slipped to v1.1 per the task's own BD-4 rule — ADR150 M5 STATUS NOTE records it; prerequisite h3o boundary work not built at the pinned rev.)*
- [x] **Task 40 — Lenses + zoom/pan + M5 close-out.** `1/2/3` lens switch; `+`/`-`/arrow pan & zoom (x/y bounds math); hover → region peek (LayoutRegistry entity = region_id). Gates; smoke on tri-county; `ai/state.yaml`. Commit `feat(rust): map lenses + viewport`.

# M6 — Stock Market (task-level; expand at kickoff)

- [x] **Task 41 — Trend host surface.** `trend_json(last_n)` reads declared view `v_national_trend` (NEVER raw `tick_summary` — II.11) → `NationalTrendView` rows; `dashboard_view_json()` → `EconomyView`. Contract tests over `tests/unit/projection/test_registry.py` fixture shapes. Commit `feat(tui): trend host surface`.
- [x] **Task 42 — Market dashboards + ridgelines (best-effort).** Stacked single-axis `Chart`s (terminal-idiomatic, design §11): imperial rent (+Δ), price⟷value scissors (`price_log` vs `fictitious_log` + deltas), cumulative corrections counter, c/v/s + exploitation/profit rates; `Sparkline` strip; `Gauge` for O=C/B overshoot — all ratatui built-ins (research: fully sufficient; no chart crate needed). Ridgeline mode (best-effort, BD-4): stacked offset trend curves as quad-strip `Face`s through the Task-32 pipeline (~100–150 LOC; no crate exists anywhere — confirmed custom). Goldens. Commit `feat(rust): stock-market dashboards + ridgelines`.
- [x] **Task 43 — M6 close-out.** Gates; smoke over a 520-tick save; `ai/state.yaml`. Commit `docs(state): raster cutover M6`.

# M7 — Cutover ceremony (task-level; expand at kickoff; requires M3 gate — inside v1.0 per BD-5)

- [x] **Task 44 — The packaging flip (BD-5; design risk R6).** `babylon-tui` moves from the `tui` group into the default dependency set; CI gains a Rust build leg (toolchain from the flake devshell; wheel caching); the T7 uv2nix player closure gains `rust/` in `projectSrc` + the maturin build (coordinate with T7-beta, which builds strictly post-cutover per the 2026-07-23 ruling — this lands the closure work exactly once); closure-size audit recorded. Commit `build(tui): rust client in the default install`.
- [x] **Task 45 — Flip the default client.** `--client` default becomes `rust`; textual remains available one release behind a deprecation warning. (Consistent now: Task 44 made the wheel a default dep — rev 1's opt-in/default contradiction is resolved.) Commit `feat(cli): rust client is the default`. *(SUPERSEDED mid-execution by Director ruling 2026-07-28 — "delete the textual code outright": landed as `feat(cli)!: rust client is the only terminal client` (8950a531), no deprecation window; contract §7 row 1.)*
- [x] **Task 46 — The ceremony commit.** Single declared commit `test(cutover)!: retire Textual Archive lane`: delete Textual widgets/tests/`__snapshots__` (221 Pilot `run_test` sites / 574 test fns / ~27 SVG goldens), remove `textual`/`textual-image`/`textual-plotext`/related deps from `pyproject.toml` + `uv.lock`, re-point the `tutorial_coverage` sentinel at Rust bindings, update `AGENTS.md`/`CLAUDE.md` (client section), `ai/architecture.yaml`, `ai/tooling.yaml`, `docs/`. Body records the parity-gate evidence (M3 transcript hash). `mise run check` + `qa:regression` MUST be green post-delete; no baselines touched.
- [x] **Task 47 — Program close.** ADR150 status → implemented; `ai/state.yaml` final; design + plan moved to done; hypergraph-rs rev pin recorded at its final value.

---

## Self-Review Notes

- **Rev 2 (2026-07-27):** spec (rev 2) coverage — §3 rulings → header/constraints/Task 5/Task 8/Task 29/M4 block/Task 39/Task 42/Task 44; §5 layout → Tasks 1–4; §6 seam + gap ledger → Tasks 6, 17 (backlinks build), 22 (salience as data), 25 (nav persistence), 30 (field_state_json); §7.1 3D lane → Tasks 5, 32–35, 39, 42; §8 testing → Tasks 3, 6, 28, 46; §9 milestones → the reordered map; §10 risks → R1 (Task 11–12), R2 (Task 14), R3 (Tasks 5/32–35 + the BD-4 split), R4 (Task 3 GIL discipline + Task 21), R5 (gates), R6 (Task 44), R7 (Task 5 + rev-bump rule). Rev-1's Task-4 CI-stub step and custom wikilink scanner are retired unbuilt (research: in-tree needs no stub; pulldown-cmark 0.13 is native).
- **Deferred granularity:** M2–M7 tasks carry interfaces/test-intent/acceptance but expand to bite-sized steps at milestone kickoff (stated in Program Map) — deliberate; the one API that rev 1 deferred as unverified (`cells3d`) is now read, verified, and templated (design §3.1/§7.1).
- **Type consistency:** `lobby_catalog_json`, `read_page`, `known_subjects_json`, `backlinks_json`, `subject_view_json`, `advance_tick`, `issue_verb`, `verb_plate_view_json`, `endgame_status_json`, `pin_watchlist`, `watchlist_json`, `save_nav_state`, `tutorial_state_json`, `choropleth_json`, `topology_json`, `field_state_json`, `trend_json`, `dashboard_view_json`, `bind_session`, `AppConfig::from_json`, `App::new`, `render_frame`, `host_calls`, `run_interactive`, `render_page`, `parse_babylon_uri`, `LayoutRegistry::{register,hit}`, `raster_bridge::blit` — used consistently across tasks.
