# M5 Maps — seam contracts (Tasks 37–40)

**Charter:** plan `docs/superpowers/plans/2026-07-26-ratatui-client.md`
§M5. Recon: 3-scout sweep `wf_88f20c03-3eb`. Branch `feature/ratatui-m5`
off post-Wave-1 dev. Design row of record: "Live choropleth: county/state
tiers from `v_hex_state_asof` (never raw `dynamic_hex_state`); lenses
value|tension|fog; zoom/pan; county WKT polygons from reference data via
host."

## 1. `choropleth_json(args_json)` — call1 (host), Task 37

- Arg `{"tier": "county"|"state"|"ea", "lens": "value"|"tension"|"fog"}`
  (field order pinned; the M4 `topology_json` args pattern). Unknown tier
  or lens raises a loud `ValueError` through the session (the M4
  out-of-vocabulary precedent — never laundered to `null`).
- Three-layer shape, verbatim from M4: `CampaignHandle.choropleth_view(
  tier, lens) -> dict | None` (@runtime_checkable Protocol member) →
  `GameSession.choropleth_view` (compute fresh, never cache) →
  `RustClientHost.choropleth_json` thin passthrough; `"null"` string is
  the sole absence signal (unbound session, or an empty tier).
- **_FakeCampaign Protocol-widening sweep (the M4 trap, pre-scouted):**
  exactly 4 files carry `isinstance(..., CampaignHandle)` seams and need
  the no-op stub — `test_app_dashboard_live`, `test_app_hud_live`,
  `test_app_action_bar_live`, `test_app_lobby_flow`. The ~9 deliberately
  narrow doubles stay untouched.
- Envelope (hand-built dict, the M4 discipline):
  `{"tier", "lens", "verified_tick", "bands": [[threshold|null, role]...],
  "cells": [{"region_id", "value": float|null, "wkt": str|null,
  "centroid": [lon, lat]|null}...]}`.
  - `bands` ships AS DATA per lens (plan ruling: bands are data, not
    Rust literals). Lens `value` keeps the `map_room._band_color` table:
    `[[null,"panel"],[1.0,"dim"],[2.0,"gold"],[null,"crimson"]]`. Lens
    `tension` ships the DIVERGING table over `w ∈ [-1,1]` (ADR170
    ruling 3): `[[null,"panel"],[-0.15,"crimson"],[0.15,"dim"],
    [null,"gold"]]` — negative = Φ-source (bled), positive = Φ-recipient
    (bribed). Lens `fog` ships the status table
    `[["exact","gold"],["approximate","dim"],["unknown","panel"]]`
    (categorical, not thresholds). Roles are §9b tokens the client maps
    to its parity-guarded constants.
  - `cells` values — **GRAPH-FIRST at county grain (amended 2026-07-28,
    supersedes the ledger-view rows; §6 records why)**: lens `value` =
    `tick_exploitation_rate` read per county-bearing territory node
    (inf-is-present convention kept); lens `tension` =
    `projection.topology.tension.county_tension_cells(graph)` (the
    ADR170 `county_extraction` witness — `v` recovered as `s/e` from the
    co-present `tick_total_surplus`/`tick_exploitation_rate` stamps,
    ratio-of-sums θ, poisoned `0.0` fallbacks = absence); lens `fog` =
    `projection.fog.county_status.county_fog_status(graph,
    player_org_id, ledger, tick, ...)` with the three defines-fed
    epistemic_horizon bounds. State tier aggregates the SAME graph
    attrs by `county_fips[:2]` ratio-of-sums; the hex-ledger views
    (`v_county_value_aggregate` / `v_hex_state_asof`) are recorded as a
    tri-county tick-0 enrichment path, NOT wired this milestone.
  - `wkt` comes from `persistence.tiger_ingestion.
    fetch_county_geometries_wkt(pool, geoids)` — **the first production
    caller of that wired-but-unconsumed seam** (county tier only; state
    tier ships `wkt: null` + centroid-less cells this milestone —
    dissolving state boundaries from county WKT is real work with no
    consumer pull yet). EPSG:4269 ≈ WGS84 for CONUS; the client treats
    lon/lat as plain x/y (equirectangular — honest at county scale).
- **LENS RULINGS (amended 2026-07-28 — the Director's four-question
  slate, ADR170, supersedes the original absence disclosure):** all
  three lenses have producers. `tension` = `county_extraction`
  principal (candidates 2/3 shadow-chartered to the engine train);
  θ is US-INTERNAL; rendering is the diverging `w` channel (the
  `(1-w)/2` damping DROPPED as redundant); the national-oppression
  axis is RULED (ADR171, 2026-07-28, over
  `reports/national-oppression-proposal.md`): it enters as a DECLARED
  STATIC REFERENCE OVERLAY — never a fourth lens (the lens enum stays
  closed) — keyed `"national_overlay"` in the envelope once the
  Phase-0 incidence artifact exists (B+C+I named-nations partition
  with a declared overlap policy; channels: E extensive-mass PRIMARY,
  Λ per-capita secondary, Ω̂ bribe alongside; labelled a reference
  layer per Amendment V/II.8 — identical every tick until the engine
  transport lands). UNTIL the artifact lands the envelope carries
  `"overlay_absent": "national overlay ruled (ADR171); Phase-0
  incidence artifact not yet built"` — the §9.9 pin-goes-red
  mechanism announces the flip. `lens_absent_reason`
  remains the envelope's honest-absence channel for the cases that
  stay real: a graph with zero data-bearing counties (tension), and
  the fog `approximate` tier being structurally unreachable until the
  INVESTIGATE intel-stash + `action_result` read-path wiring lands on
  the modern runtime (chartered W-C motion — the modern path has ZERO
  `persist_action_results` callers; the harness pins the absence, pin
  goes red when the wiring lands).
- **EA tier**: `ea_choropleth_cells` returns None by design (no bridge)
  — the envelope for `tier: "ea"` is `"null"` absence; the client's tier
  cycle SKIPS ea until a producer exists (recorded here, not a bug).
- **TUTORIAL-CAMPAIGN DISCLOSURE:** the WAYNE scenario's territories
  carry NO `county_fips` and NO hex-state rows — the live tutorial
  campaign's map pane renders honest absence naming this cause. The
  harness pins the absence line (pin-goes-red = producer-landed
  announcement, the M4 §9.9 mechanism). Non-empty rendering is proven by
  fixture-fed tests (tri-county FIPS 26163/26125/26099 + real TIGER WKT
  fixtures).

## 2. MapView canvas (Task 38)

- `views/map.rs`, the TopologyView shape verbatim: `MapView { tier,
  lens, payload: Option<ChoroplethPayload>, payload_failed: bool,
  viewport }` chrome-owned on `PlayChrome`; `ingest_choropleth(raw)`
  with the loud parse-failure flag; `args_json()`; `mod
  view_state_tests` pure state-machine pins.
- Render: `ratatui::widgets::canvas::Canvas`, `Marker::HalfBlock`
  (2-colors-per-cell — the richest cell form), `x_bounds`/`y_bounds`
  from the cells' WKT bounding box (plus margin).
- **Polygon fill is a hand-written `Shape`** (recon: NO built-in
  polygon fill exists in ratatui 0.30) — scanline point-in-polygon over
  the painter grid within the polygon's bbox, colored by the band the
  cell's value falls in (bands from the ENVELOPE, resolved to the
  parity-guarded theme constants by role name). Exterior-ring-only for
  v1.0 (county WKT holes are negligible at braille resolution — noted).
- Absent `value` (null) fills with PANEL (the absence band row);
  absent `wkt` renders the region as a labeled centroid dot only, or —
  when every cell lacks geometry — the honest-absence line naming the
  tier's state. `lens_absent_reason` renders as a one-line CRIMSON
  banner over the canvas (declared, the pixel-degradation precedent).
- Labels: `Context::print` for region ids at centroids (labels always
  draw on top — recon).

## 3. Keys + chrome integration (Tasks 38/40)

- The Pane::Map absence fence arm is REPLACED by `chrome.map.render`;
  `refresh_map()` mirrors `refresh_topology()` (pane-entry '2' fetch +
  `refresh_after_tick` gate — without it the pane goes silently stale).
- Map-pane key block inserted BEFORE the wiki fallthrough (the
  match-arm-order trap, third time): **`l` cycles lens** value→tension→
  fog→value (RECORDED DEVIATION from the plan's "1/2/3 lens switch" —
  digits 1–4 are the GLOBAL pane-switch keys and cannot be shadowed
  pane-locally without breaking the '1'-'4' contract §1 wire; the 'g'
  kind-cycle precedent applies instead). **`t` cycles tier**
  county→state (ea skipped per §1)... `t` is the GLOBAL tick key —
  ALSO taken. Tier cycle key: **`y`** (mnemonic: tier). Zoom `+`/`-`,
  arrows pan, `0` reset viewport (topology camera-key parity), `Esc` →
  wiki. Every arm returns the `MapAction` mirror of `TopologyAction`.
- Wheel (Wave 1 §5): the `Pane::Map` no-op arm in the region:center
  wheel route becomes zoom in/out (the topology-3D precedent).
- Keybar: `KeybarSurface::Map` variant — `l lens · y tier · +/- zoom ·
  ←→↑↓ pan · 0 reset · Esc wiki` + the global tail; help section added
  (one source of truth holds).
- Viewport math (Task 40): pan shifts both bounds by 10% of span;
  zoom scales spans by ×0.8 / ×1.25 about the center, clamped to the
  data bbox ±1 span; `0` restores the fitted bbox. Pure functions,
  unit-tested.

## 4. Extrusion (Task 39) — SLICED to v1.1; Task 39-P — Patches (FOLDED IN)

Extrusion: the prereq is NOT built at the pinned hypergraph-rs rev: h3o
0.10's `CellIndex::boundary()` exists in the dep but nothing converts
spherical boundaries to scene space; `sankey.rs::ground_hex` is still a
synthetic regular hexagon; no general fan-triangulate exists. The
plan's own BD-4 rule ("if this would delay v1.0, it slips to v1.1 with
a one-line ADR150 note — before it delays anything else") FIRES:
sliced, ADR150 note lands with the M5 close-out. No sibling-repo work
this milestone.

**Task 39-P — the Patches 3D scene (Director fold, 2026-07-28;
directive of 2026-07-27).** The golden snub-nosed monkey renders as a
real scene through the Task-32 raster pipeline: a `patches` scene3d
builder (stylized low-poly monkey from triangle Faces — the
`field_surface` local-construction pattern; ksbc gold body/crimson
accents per the Director's palette), `compute_bounding_box`,
`CameraState` reuse, insta `<scene>_{front,3q}` goldens (the raw-frame
lane). Surfaces in the tutorial arc as Patches' visual presence beside
the strip (exact placement recorded as a deviation when the layout
lands). **Plain-terminal fallback is ALREADY SHIPPED**: the M3 text
strip (24 Patches lines, harness-pinned) is the glyph-floor Patches —
the 3D scene is an enrichment tier, never a dependency. **ratty/RGP
disposition (the 2026-07-27 evaluation charter): SUPERSEDED** — the
in-tree M4 pipeline (scene3d + blit-to-rect + the pixel tier) already
does inline 3D without a Bevy dependency; adopting ratty would add a
heavyweight engine against a working lane. Recorded here for Director
veto; the hard import ban on `hypergraph_rs::raster::{instruments,
deck,ingest}` holds for this builder too.

## 5. Testing + goldens

- TDD red-first per unit. Python: contract tests over choropleth_view /
  choropleth_json with tri-county fixtures (real FIPS, small synthetic
  WKT squares — deterministic, no drive reads: CI never touches the
  babylon-data drive); the spec-089 multi-tick guard pinned; the lens
  disclosure pinned; the 4 _FakeCampaign stubs.
- Rust: view_state_tests (lens/tier cycles, loud parse flag, viewport
  math); a frame-content golden per lens over a 3-cell fixture — pin
  CONTENT (band-colored braille/halfblock cells present, absence
  banner text) never just titles (the golden-certifies-what-you-render
  class, strike three's lesson).
- The M3/M4 harness: the WAYNE arc's map-pane beat flips from the
  MAP_FENCE line to the pane's own honest-absence line — transcript
  golden regen (wheel-first), the fence pin in
  `test_pane_fences_render_under_the_strip` updates for Map.
- Smoke (Task 40 close-out): fixture-fed tri-county render at county
  tier — 3 filled polygons in 3 bands; the "tri-county" ambiguity the
  recon flagged (WAYNE ≠ detroit_tri_county) is resolved AS the
  fixture-fed reading: the detroit_tri_county scenario's committed
  golden is county-aggregated, matching the county tier exactly.

## 6. Deviations discovered during implementation

- **2026-07-28 — county tier goes GRAPH-FIRST (supersedes §1's original
  `county_choropleth_cells`-over-`v_county_value_aggregate` ruling).**
  The 3-scout data-path recon (wf_e7c72977-b24) proved the hex-ledger
  view is NOT a nationwide source: `dynamic_hex_state` is written ONLY
  at tick 0, ONLY for the tri-county hydration set, ONLY when the TIGER
  drive symlink resolves at campaign creation — and
  `GameSession.advance_tick` never re-writes hex rows (no
  `hex_state_rows` in the envelope), so `v_county_value_aggregate` is
  frozen at tick 0 or empty. The live nationwide medium is the GRAPH:
  `USScenario` seeds 3,153 county territories and TickDynamics stamps
  the `tick_*` block per county-bearing territory. The ledger views
  keep their existing consumers/tests untouched.
- **2026-07-28 — the tension v-pole is RECOVERED, not stamped.** No
  nationwide per-territory `v` write exists (the one real writer,
  `Vol2CirculationStep`, is tri-county-locked; employment is never
  written — the Program-17 100k-placeholder gap). Where
  `tick_exploitation_rate` (e) and `tick_total_surplus` (s) are
  co-present and positive, `v = s/e` is exact; elsewhere the county is
  honestly absent. Extending `Vol2CirculationStep` past the tri-county
  fixture is chartered wiring-doctrine work (W-C), not M5's.
- **2026-07-28 — fog ships reach-complete, ledger-empty.** The
  `county_fog_status` producer is fully built and tested, but the
  modern runtime has no INVESTIGATE intel-stash writer and no
  `action_result` read path (`persist_action_results`: zero production
  callers) — so live campaigns pass an empty `IntelLedger` and the
  `approximate` tier is structurally dead until that chartered W-C
  wiring lands. Reach (`organizing_reach` over PRESENCE/TENANCY/
  SOLIDARITY with the epistemic_horizon defines) is fully live.
- **2026-07-28 — nationwide WKT needs a bulk ingest at first use.** The
  reference data genuinely carries 3,222 county geometries
  (`dim_county_geometry`), but the live Postgres
  `immutable_reference_tiger_county` table only ever receives the
  hydration set's rows (≤3) — Task 37's first call must bulk-ingest via
  `ingest_tiger_counties_from_sqlite` (or read the reference DB
  directly) rather than assume the table is populated.
- **2026-07-28 — tension/fog band thresholds are module presentation
  constants** (the `map_room._band_color` precedent), NOT `GameDefines`
  entries — they are projection presentation data, not simulation
  coefficients; folding the lens tables into a defines category is
  deferred to a declared sweep (avoids a defines_hash-only ceremony
  for band edges).
- **2026-07-28 (Task 38b) — labels gate on legibility, not
  unconditionally.** §2 says "labels: `Context::print` for region ids at
  centroids"; drawn literally at the nationwide county fit, 3,000+
  five-char ids shred the canvas into noise. A WKT cell's label draws
  only when its bbox is wide enough (in canvas columns) to hold the id
  and one text row tall — labels appear as the player zooms in. A
  centroid-dot cell (no WKT) stays labeled unconditionally per §2's
  "labeled centroid dot". In-window anchors that would clip at the right
  bound shift left to fit (a truncated FIPS reads as the WRONG id).
- **2026-07-28 (Task 38b) — role `panel` resolves to the module-local
  `topology::PANEL` (`#200404`)**, the Textual `map_room._band_color`
  absence fill — NOT `theme::MUTED_DARK` (a §9b token with a different
  value); the parity guard forbids adding a non-§9b constant to
  `theme.rs`, so the topology module's own precedent constant is reused.
- **2026-07-28 (Task 39-P) — the portrait camera + the strip-side width
  gate.** The mascot goldens (and the live placement) frame Patches at
  the front camera stepped fully in (5 × `DIST_STEP`, landing on the
  `DIST_MIN` clamp) — built only from the public discrete-step API, no
  bespoke camera literals. Placement: wide raster terminals (≥120 cols,
  strip ≥4 rows) carve 26 columns off the strip band's right edge;
  narrower terminals and raster-less builds keep the full-width text
  strip (the M3 Patches lines are the glyph floor). The 100×30 headless
  harness sits below the gate by design, keeping transcript goldens
  free of raster nondeterminism concerns.
- **2026-07-28 (Task 40) — solid fills blit as `█`.** The HalfBlock
  grid renders a cell as `▀` only when its two half-pixels differ;
  same-color pairs merge to `█` — content asserts over transcript text
  must accept any of `▀▄█` (the smoke's own first red run caught this).
