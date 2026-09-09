# Graph-render lane ruling — outsource complex visuals to the graph substrate (BD, 2026-07-21)

> BD (verbatim intent): "the TUI can show svg and png images within the tui itself
> through kitty or something so that way more complex stuff like the map can be
> outsourced to rustworkx and xgi and our rust package which can use that."

Binding on T4-integration / T8 client work. Complements (does not weaken) the T6
text-assertion ruling: assertions stay text; this rules where complex RENDERING comes from.

## The ruling

Complex structural visuals are NOT hand-built as bespoke Textual widgets. They are
rendered **graph-natively** and displayed in-terminal through the ADR099 raster lane
(kitty graphics via textual-image, detect-before-boot, glyph floor beneath):

- **rustworkx.visualization** (`graphviz_draw` / `mpl_draw`) for topology views straight
  off the live BabylonGraph: solidarity networks, the coupling graph, class-fraction
  structure, supply/tribute flows. The graph substrate draws itself — zero widget build.
- **XGI** for hypergraph views (candidate dependency — add to pyproject only when the
  unit lands; matplotlib-based, seed/style pinned).
- **hypergraph-rs** (our Rust package) as the eventual high-performance renderer —
  PAUSED by owner ruling; resume via the subrepo's plans/EXECUTION-STATE.md; nothing
  here un-pauses it. XGI is the Python-side interim.
- Map-room complexity (choropleths, hex overlays at nationwide scale) may likewise be
  emitted as SVG/PNG artifacts rather than cell-painted, where richer than the glyph
  floor can carry.

## Contract discipline (keeps the T6 ruling intact)

- **Assert the SOURCE, display the RENDER.** The behavioral/golden artifact for an
  outsourced render is its text-form input — DOT source, layout JSON, the projection
  data handed to the renderer — never the rasterized bytes (graphviz/mpl bytes are not
  cross-version stable; the DOT text is). SVG/PNG bytes are presentation-tier,
  regenerate freely, aesthetic review only (T8 KSBC pass styles them: crimson/gold on
  near-black, DESIGN_BIBLE §9b).
- **Glyph floor stays mandatory** (ADR099): every raster view has a text-mode floor the
  BDD suite can assert; kitty raster is progressive enhancement, detect-before-boot,
  never load-bearing for playability (the game must be fully playable glyph-only, e.g.
  over plain ssh).
- **Determinism**: renders are P-tier projections — never in the tick hash; generated
  from committed projection data so a given tick's DOT source is reproducible even when
  the raster bytes are not.
- **Layering**: render-source generation lives in projection (reads graph, emits DOT/
  JSON); the TUI displays files/bytes it is handed. No new tui→topology import; the
  import-linter contracts are untouched.

## Placement

- T4-integration gains a unit: graph-render lane plumbing (projection emits render
  sources + rasterizer step + raster/glyph display seam in the relevant screens).
- T8 aesthetic pass styles the outsourced renders (graphviz/mpl theme = KSBC palette).
- Tutorial/BDD coverage: scenarios assert the DOT/layout source text + the glyph floor;
  raster presence is environment-conditional, never asserted as behavior.
