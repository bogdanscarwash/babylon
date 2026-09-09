# M6 Stock Market — seam contracts (Tasks 41–43)

**Charter:** plan `docs/superpowers/plans/2026-07-26-ratatui-client.md`
§M6. Recon: 2-scout sweep `wf_8a749695-cd2`. Branch `feature/ratatui-m6`
off post-M5 dev (M5 implementation lands first; this contract only pins
the seams). Plan rows of record: Task 41 "trend host surface:
`trend_json(last_n)` over `v_national_trend` (never raw `tick_summary`,
II.11) + `dashboard_view_json`"; Task 42 "market dashboards: ratatui
`Chart`/`Sparkline`/`Gauge` built-ins, ridgeline best-effort"; Task 43
close-out.

## 1. Host surface (Task 41)

- **`dashboard_view_json()` — nearly free.** `CampaignHandle.
  dashboard_view` already exists (`src/babylon/tui/app.py:368`), already
  implemented (`GameSession.dashboard_view`, `session.py:764`), already
  consumed by Textual. Task 41 adds ONLY the `RustClientHost` thin
  passthrough: `None` session → `"null"`, else
  `self.session.dashboard_view().model_dump_json()`. No Protocol change
  → no _FakeCampaign sweep for this method.
- **`trend_json(args_json)` — the real new work.** NOTHING in
  production reads `v_national_trend` today (only tests). Arg
  `{"last_n": int}` (pinned field order; loud `ValueError` on
  non-positive, the M4 out-of-vocabulary precedent). Four new layers,
  bottom-up:
  1. `fetch_national_trend(runtime, session_id, last_n)` in
     `persistence/postgres_aggregation.py` — the exact
     `fetch_national_aggregate` shape (lines 180–203); column order from
     `declared_view("v_national_trend")`, never hand-duplicated
     (II.11: the declared view IS the interface; raw `tick_summary` is
     prohibited).
  2. `GameSession.trend_view(last_n) -> tuple[NationalTrendView, ...]`
     — compute fresh, never cache; rows oldest→newest.
  3. New `CampaignHandle.trend_view` Protocol member (docstring carries
     the "computed HOST-SIDE by the composition root" projection-purity
     litany verbatim, per every existing member).
  4. `RustClientHost.trend_json` passthrough — envelope
     `{"verified_tick", "rows": [...], "national_value": {...}|null}`
     (hand-built dict + `json.dumps`, the `watchlist_json` convention;
     rows via `model_dump(mode="json")` — `session_id` is a UUID).
- **_FakeCampaign Protocol-widening sweep (third strike, pre-scouted):**
  the SAME 4 files as M5 need the `trend_view` no-op stub —
  `test_app_hud_live.py:196`, `test_app_lobby_flow.py:130`,
  `test_app_action_bar_live.py:302`, `test_app_dashboard_live.py:175`.
- **`national_value` snapshot (resolves the plan's "c/v/s +
  exploitation/profit rates" bullet):** no TIME-SERIES producer exists —
  `tick_summary.total_c/v/s/exploitation_rate/profit_rate` are written
  as permanent `None` (`projection/tick_summary.py:291`), and
  `v_national_trend`'s own docstring excludes them ("a trend of a
  permanently NULL column is not a signal"). The LIVE producer is the
  declared view `v_national_value_aggregate` (`NationalView.c_sum/
  v_sum/s_sum/k_sum`) — a single-tick snapshot. Ruling: the envelope's
  `national_value` object carries `{c_sum, v_sum, s_sum, k_sum,
  exploitation_rate, profit_rate}` with rates derived RATIO-OF-SUMS in
  the projection producer (s/v and s/(c+v); the intensive-aggregation
  law), `null` when the view has no row. The c/v/s TIME-SERIES stays a
  declared absence — the client renders the absence line naming the
  missing producer; the harness pins it (pin-goes-red = the §9.9
  producer-landed announcement). EconomyView is NOT widened.

## 1b. Migration 0039 — window the five LIVE playability columns

**Addition beyond the plan bullet (flagged for Director veto in the PR).**
Migration `0035_playability_series.sql` added five genuinely-computed
`tick_summary` columns (`crisis_pop_share`, `bifurcation_score_mean`,
`wage_compression_mean`, `capital_stock_total`,
`unemployment_rate_mean` — written live by `build_tick_summary_kwargs`,
`projection/tick_summary.py:188-210`) that `v_national_trend` does not
window: live signal invisible to ANY trend read. The view's stated
exclusion rationale covers only permanently-NULL columns, so windowing
these is consistent by its own logic, and they ARE the playability
series a market dashboard exists to show. Contract: migration
`0039_trend_playability.sql` re-declares `v_national_trend`
(`DROP VIEW` + `CREATE VIEW`, the 0038 idiom — never
`CREATE OR REPLACE`) adding the five columns + five LAG deltas;
`NationalTrendView` grows 10 fields (same `float|None` honest-absence
discipline); registry entry + the pinned 10-column tuple in
`tests/unit/projection/test_registry.py:66-84` become 20 columns
(declared contract change, TDD'd); the integration suite
`test_tick_summary_trend_view.py` gains the new-column legs.

## 2. DashboardView (Task 42)

- `views/dashboard.rs`, the TopologyView shape: chrome-owned on
  `PlayChrome` (`dashboard` field + `PlayChrome::new` init), `ingest_*`
  with loud `payload_failed` flags, serde structs mirror the Pydantic
  optionality EXACTLY (`Option<f64>`/`Option<i64>`) — charts GAP-SKIP
  `None` points, never fabricate 0 (the `EndgameSlot::{Absent,
  Unreadable,Bound}` precedent, `hud.rs:169-193`).
- **Chart pages** (focus-cycled, one rendered large + keybar shows
  position): imperial rent (`GraphType::Line` level + `Bar` delta);
  price⟷value scissors (`price_log` vs `fictitious_log`, two named
  `Dataset`s, `Marker::Braille`, legend distinguishes); corrections
  (`Sparkline` — data is **u64-only**, no float/negative:
  `market_corrections` cumulative count fits; delta as second strip);
  the five 0039 playability series (Line charts); the `national_value` +
  `EconomyView` snapshot panel (FT verdict, Φ tri-decomposition —
  currently all-None, honest absence — Vol-III p/i/r/t split + shares,
  matter book).
- **Gauge veto (the panic trap, pre-scouted):** `Gauge::ratio()`
  `assert!`s `[0.0, 1.0]` — `overshoot_ratio` O=C/B is UNBOUNDED above
  1.0 and O>1 IS the signal. Do not use `Gauge`; use the in-crate
  hand-drawn bar idiom (`hud.rs::bar_glyphs`, clamp + label carries the
  true value) for O and any share that can exceed 1.
- **Ratatui gotchas:** `Dataset.data` borrows `&[(f64,f64)]` — own the
  `Vec` across the `frame.render_widget` call; keep axis labels ≤3
  (>3 mispositions the middle ones, upstream issue 334).
- **Chrome integration** (the `Pane::Topology` wiring as live
  precedent, all sites pre-scouted): `DASHBOARD_FENCE` arm
  (`app.rs:698`) → `chrome.dashboard.render`; `refresh_dashboard()`
  (pulls `trend_json` + `dashboard_view_json` via `self.recording()`)
  gated at BOTH sites — pane-entry `'1'` (`app.rs:926` region) and
  `refresh_after_tick` (`app.rs:2039` region); key block inserted
  BEFORE the wiki fallthrough (match-arm-order trap, fourth time) with
  a `DashboardAction` enum; the `Pane::Dashboard` wheel no-op arm
  (`app.rs:1268`) becomes chart-focus cycling.
- **Keys:** `c` cycles chart focus (wheel mirrors); `m` toggles the
  ridgeline 3D mode (§3); `Esc` → wiki. Digits/t/r/a/l/y stay
  untouched (global + M5-Map keys).
- **Keybar/help:** new `KeybarSurface::Dashboard` variant with its own
  `hints()` arm + help row; the shared `AbsencePane` "MAP / DASHBOARD"
  row splits (Map has its own variant by then — M5 lands first; record
  the actual split shape in §5 deviations if sequencing differs).

## 3. Ridgeline 3D (Task 42, best-effort)

Stacked offset trend curves through the Task-32 raster pipeline. The
`scene3d.rs::field_surface` pattern verbatim: per-segment quad → TWO
triangle `Face`s (`{verts:[Vertex3;3], fill, opacity}` — no new Face
shape), vertex closure walks consecutive `(x_i, scalar_i)` pairs per
series with a per-ridge depth offset (1D curves, not the 2D IDW grid),
`heat_ramp` color, `compute_bounding_box`, `CameraState` reused
unchanged. HARD import ban holds: never reach into
`hypergraph_rs::raster::{instruments,deck,ingest}` — follow the
pattern, reimplement locally (the `idw_height` precedent,
`scene3d.rs:220`). ~100–150 LOC per the plan's own estimate.
Best-effort: if it threatens the milestone, it slips to v1.1 with a
one-line ADR150 note (BD-4) — before it delays anything else.

## 4. Testing + goldens (Task 43 close-out)

- TDD red-first per unit. Python: `trend_view` contract tests
  (unbound → `"null"`; `last_n` windowing oldest→newest; first-tick
  deltas `None`; ratio-of-sums derivation; loud `ValueError` on bad
  `last_n`; the 4 stubs; the 0039 registry re-pin). Postgres-touching
  legs live in the integration tier (`test_tick_summary_trend_view.py`
  extension), unit legs stay fixture-fed — CI's unit shard has no DB.
- Rust, the RULED golden split (m4-contracts §"Golden strategy"):
  2D charts → **explicit substring/field asserts over `buffer_text`**
  (+ `style_at` for series colors — insta cannot assert color) exactly
  like `hud_view.rs`/`topology_2d.rs`; ridgeline raster frames →
  `insta::assert_snapshot!` fixture-named `<scene>_{front,3q}` like
  `scene3d_goldens.rs`.
- Harness: the WAYNE arc's dashboard beat flips from `DASHBOARD_FENCE`
  to the pane's own rendered content; the fence pin in
  `test_pane_fences_render_under_the_strip` updates; transcript golden
  regen (wheel-first).

## 5. Deviations discovered during implementation

- **2026-07-28 — the migration number is 0041, not 0039.** This contract
  was pinned pre-P26-merge; P26 landed `0039_domain_contracts.sql` +
  `0040_receipts_vocabulary.sql` first, and the runner globs `00*.sql`
  sorted — the next free slot was 0041 (`0041_trend_playability.sql`).
- **2026-07-28 — `national_value` carries its own `tick`.** The hex
  ledger is written at hydration and never re-written by
  `advance_tick` (the M5 recon's finding of record), so the §1 object
  gains a `tick` field as the staleness disclosure the client renders
  ("as of tick N") — honest staleness beats a laundered snapshot
  (III.11). Rates stay ratio-of-sums with `None` on zero denominators.
- **2026-07-28 — the snapshot panel is TWO columns.** A single column
  ran ~34 rows and clipped its own tail inside the play chrome's
  ~15-row center at the 100×24 floor geometry (caught by the first
  frame-content test): economy dossier left (wage_balance riding the
  FT-verdict line — every row is contended), national_value + matter
  book right, and the harness-pinned `no c/v/s time-series` declared
  absence placed HIGH in the right column so the floor never clips it
  (wording shortened to fit: "— no producer yet").
- **2026-07-28 — the wheel's chart cycle is direction-agnostic.** The
  `Pane::Dashboard` wheel arm fires one forward `'c'` cycle either
  direction — a 5-page cycle has no meaningful reverse, and a reverse
  map would be a second dispatch table for no player value.
- **2026-07-28 — `KeybarSurface::AbsencePane` DELETED.** The dashboard
  was the last fence pane; with it real, no pane maps to the absence
  surface and `render_pane_absence` + the M3 fence consts are gone.
  The parametrized `test_pane_fences_render_under_the_strip` retired
  with them — each pane's case had already moved to its own
  `test_<pane>_renders_real_content_under_the_strip`.
- **2026-07-28 — ridgeline LANDED (best-effort delivered, no BD-4
  slip).** Per-ridge normalization to each series' own `[min, max]` is
  deliberate and documented (a ridgeline shows SHAPE; magnitude
  comparison is the 2D charts' job); flat series ridge at half height;
  the dashboard's `m` mode renders eight ridges (headline trio + five
  playability) through the raster pipeline with its own absence ladder.
- **2026-07-28 — the client trend window is pinned `last_n: 120`**
  (~2.3 simulated years — enough for every chart at braille
  resolution); the host does not cap, the client chooses its window.
