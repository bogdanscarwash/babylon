# Interface Wave 1 — seam contracts (discoverability + input honesty)

**Charter:** `reports/interface-master-plan.md` §3 Wave 1 + §5 (all seven
Director rulings RULED 2026-07-27/28). Recon: 3-scout sweep
`wf_b573ccdb-039` over the as-built `rust/crates/babylon-tui`. This
train pre-empts M5 (ruling 7) and designs to the **100×30 declared
floor** (ruling 1).

Defects closed: D1 (no keybar), D2 (no help overlay), D3 (mouse covers
2/~9 regions, wheel dropped), D4 (invisible focus, Tab-only), D5 (verb
plate clips at 80×24 with the tutorial strip up).

## 1. The 100×30 floor guard (ruling 1 — closes D5 structurally)

- Recon arithmetic of record: the verb plate builds EXACTLY 11 display
  lines and needs inner height 6 (= `Length(8)` with borders). At
  100×30, worst case (tutorial strip clamped at its 40% ceiling = 12
  rows) leaves 18 rows against a 17-row demand — **no deficit, ever**.
  At 80×24 the plate absorbs a 2-row deficit (largest `Length` loses
  under cassowary priorities) and F5/F6/F9 clip. The structural fix is
  the ruled floor, not plate pagination.
- `App::render_frame`: when `area.width < 100 || area.height < 30`,
  render ONLY the too-small notice (ksbc: bone on the near-black field,
  crimson accent): terminal size, the floor, and "resize to continue —
  q quits". Every key except `q`/`Esc`/`Ctrl-C` is swallowed while the
  guard is up (no hidden state mutation against an invisible UI); the
  guard lifts on the next render at ≥100×30 with all state intact.
- Headless: the guard applies identically (a `headless_size` below the
  floor renders the notice into the transcript — tests can pin it).
  The existing harness passes 120×50 and is untouched.
- Verb-plate belt-and-braces: the plate constraint stays `Length(8)`;
  a `debug_assert`-tier comment records the 11-line/inner-6 invariant
  next to `build_lines` (the floor guard is the load-bearing fix).

## 2. The persistent keybar (D1)

- Layout: a 5th `Constraint::Length(1)` row appended to the chrome
  vertical split (`[hud 3, mid Min(5), plate 8, status 1, keybar 1]`),
  to the lobby split, and to the previously-unsplit failure-page wiki
  arm — every screen has the keybar; the mid region pays the row.
- Content is CONTEXT-AWARE, a pure function of
  `(views.last, chrome{focus, pane, topology.mode()}, palette/help
  overlays open)` — the recon confirms all inputs are plain fields
  already available at render time. Per-surface hint sets (leading
  cluster = surface-specific, trailing cluster = the global trio
  `? help · / palette · q back`):
  - lobby: `↑↓ select · Enter load · n new`
  - wiki pane: `[ ] jumps · n/p links · Enter open · K peek · 1-4 panes
    · Tab focus`
  - topology pane Glyph2d: `g kind · s 3D · ↑↓ scroll · Esc wiki`
  - topology pane 3D: `←→↑↓ rotate · +/- zoom · 0 reset · s/f mode ·
    g glyph · Esc wiki`
  - rail focused: `↑↓ rows · Enter open · Esc center` (+ `p pin` on
    watchlist)
  - palette/help open: `↑↓ · Enter · Esc close`
  - map/dashboard (absence fences): `1-4 panes · Tab focus`
- Style: ksbc — DIM separators, BONE keys, GOLD key glyphs on the
  near-black field; never a second status line (the status row keeps
  its own band above it).
- CLICKABLE: each hint cell registers into the per-frame
  `LayoutRegistry` as entity `key:{name}` (the `verb:{slot}`
  precedent); `handle_mouse` maps `key:{name}` through the SAME
  dispatch the keyboard uses (synthesizes the bound `KeyCode` through
  `handle_key` — one routing authority, no second dispatch table).

## 3. The `?` help overlay (D2)

- **RECORDED DEVIATION from the plan's wording** ("a view-stack
  entry"): the as-built overlay pattern is the PALETTE FIELD
  (`Option<PaletteView>` on `App`, intercepts keys first, renders last
  over the base layout) — the help overlay follows that precedent
  exactly (`Option<HelpView>`), NOT a `View::` variant; the view stack
  only ever holds Lobby/Wiki today and the M2 port deliberately
  dissolved stacked chrome.
- Opened by `?` from ANY surface (global arm; also from the lobby).
  Esc/`?`/`q` closes. `↑↓`/`PageUp/Down` scroll.
- Content: the full binding table for the ACTIVE surface first
  (mode-scoped — the same context function the keybar uses picks the
  section), then every other section, each titled. The table is a
  static Rust const mirroring the recon's binding inventory — one
  source of truth shared by keybar hints and help sections so they
  cannot drift apart.
- Anatomy: newt-idiom centered plate (the peek overlay's
  `border_style(CRIMSON)` precedent + title-tab), max ~70×24, DIM
  scrim NOT required (ratatui overlays don't dim; the plate's own
  field color carries it).
- While open it intercepts ALL keys (palette precedent) except the
  close set.

## 4. Focus indicators + Shift-Tab (D4)

- Focused region gets `border_style(Style::new().fg(CRIMSON))` (the
  peek overlay precedent) on its Block: watchlist rail, chronicle
  rail, and the CENTER pane's own Block (wiki/topology/absence fences
  all render Blocks). Unfocused = default border. The `" ●"` title
  suffix stays (redundant channel, cheap).
- `KeyCode::BackTab` arm symmetric to Tab (recon: BackTab already
  reaches `handle_key` in crossterm 0.29 legacy mode and is swallowed
  by catch-alls today): Center → Watchlist → Chronicle → Center.
- Headless: `key_event_from_name` gains `"backtab"` (recon: unknown
  names are SILENT no-ops in transcripts — without this a BackTab test
  would vacuously pass).

## 5. Mouse parity (D3)

- `handle_mouse` gains arms:
  - `ScrollUp`/`ScrollDown` → route by POSITION (hit-test which
    region contains the cursor): center pane → wiki `scroll` ∓3 /
    topology Glyph2d `scroll` ∓3 (3D modes: wheel = camera dist ∓1
    step, the zoom affordance); watchlist/chronicle rail → that rail's
    cursor ∓1. Regions are known because each region's render
    registers a region-level rect (below).
  - `Down(Left)` on a rail row → focus that rail AND move its cursor
    to the clicked row (click-to-focus + click-to-select in one);
    double-action open is NOT required — Enter/second-click opens
    (row entities registered per-row make the second click an open).
  - `Down(Left)` on a pane title / keybar cell → dispatch per §2.
- Registry threading: `WatchlistView::render`, `ChronicleRail::render`
  gain `registry: &mut LayoutRegistry` and register (a) their region
  rect as `region:{watchlist|chronicle}` and (b) each visible row as
  `{rail}:{index}` entities; the topology + wiki + absence-fence
  center renders register `region:center`. The registry itself needs
  ZERO changes (innermost-wins hit already does the right thing:
  row hits shadow region hits).
- `handle_mouse`'s `Moved` arm keeps its current peek behavior;
  region/row/`key:` entities are EXCLUDED from peek targets (the
  `verb:` precedent).
- Headless: `ScriptStep` gains a scroll step
  (`{"scroll": [col, row, "up"|"down"]}`) so wheel behavior is
  transcript-testable.

## 6. Definition of done

- TDD: every item red-first where a Rust unit/integration test can
  express it (layout math, keybar content function, help table
  coverage, BackTab cycle, pixel_decision-style pure functions);
  transcript-tier for the rest.
- The M3/M4 harness stays green EXCEPT declared drift: the keybar row
  + focus borders change EVERY frame → `wayne_opening_arc.json`
  regenerates (wheel-first, twice-bitten rule) with two-run
  determinism re-proven; content pins (substring asserts) survive.
- A help-coverage test pins that every key handled in `handle_key`'s
  global/rail/topology arms appears in the binding const (the
  drift-guard between code and help).
- `mise run rust:check` green; `mise run check` green (modulo the
  cross-lane catalog red of record); PR with auto-merge on green CI.

## 7. Deviations discovered during implementation

1. **`headless_size`'s default moved 80×24 → 100×30** — the old M0
   default sits BELOW the ruled floor and would have rendered every
   defaulted fixture the too-small notice; `app_shell`/`hello_frame`
   backends moved to the floor and the hello-frame snapshot re-blessed.
2. **Pane-title clicks are SUBSUMED by region click-to-focus** (§5
   planned them separately): clicking anywhere in a rail — title
   included — focuses it, and the center's title row focuses Center;
   a separate title-cell dispatch would add a second affordance with
   no distinct meaning. The 1–4 pane switch stays keyboard + keybar.
3. **While the help overlay is open the keybar shows the Overlay hint
   set** (`↑↓ · Enter · Esc close`) — Enter is a no-op inside help;
   a dedicated Help surface variant was not worth a tenth enum arm for
   one mislabeled cell. Revisit if playtests trip on it.
4. **The help-coverage drift-guard is a curated mirror, not source
   parsing**: `dispatchable_hints_use_known_key_names` proves every
   keybar dispatch name resolves, and the EVERYWHERE + per-surface
   sections carry the full recon table; a handle_key-source-parsing
   sentinel was judged over-engineering for one file.
5. **Zero transcript-golden drift from U6** (§6 predicted drift only
   for U4/U5's visual rows): mouse parity adds no pixels — verified
   against the regenerated golden, not assumed.
6. **`ChronicleRow` requires `tick`/`actor` on the wire** — a fixture
   omitting them parses to the LOUD `parse_failed` state, which is the
   III.11 design working (caught writing the U6 fixture; recorded so
   the next fixture author doesn't re-learn it).
7. **The floor's height axis dropped 30 → 24 (Director field report,
   2026-07-28)**: a fullscreen 1366×768 laptop terminal at 151×27 was
   locked out of the game entirely — on that hardware no resize can
   reach 30 rows without shrinking the font, which inverts ruling 1's
   intent (refuse cramped mush, not refuse real machines). Density
   stays DESIGNED to 100×30 (`headless_size` default unchanged, no
   golden drift); the §1 verb-plate invariant is re-established at
   every admitted height by clamping the tutorial strip band against
   the play chrome's fixed rows (`PLAY_CHROME_MIN_ROWS` = 18: HUD 3 +
   mid-min 5 + plate 8 + status 1 + keybar 1) — the strip yields, the
   plate never clips. Width stays 100 (the 24-column rails need it).
   The Director's exact geometry is pinned forever:
   `floor_guard.rs::the_directors_fullscreen_laptop_geometry_renders`.
