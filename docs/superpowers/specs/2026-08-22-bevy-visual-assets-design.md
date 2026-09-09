# Bevy visual assets — `Design`

<!-- This design record needs project art and API terms that the STE word list omits. -->
<!-- vale ste.UnapprovedWords = NO -->

**Status:** Approved in chat on 2026-08-22
**License:** AGPL-3.0-or-later
**Scope:** A broad, reusable art direction and runtime asset foundation for `babylon-client`

## Purpose

Babylon's Bevy client currently has one compiled data asset, the county atlas, and no image-loading
surface. This change gives the game a recognizable face: hero art, banners, concept art, a title
mark, and reusable interface graphics. It changes presentation only. Engine state, rules, tick
ordering, and deterministic hashes remain untouched.

The direction deliberately combines three compatible modes:

<!-- These three art-direction inventories are descriptive lists, not procedures. -->
<!-- vale ste.ProcedureLength = NO -->
- **The Red Apparatus** supplies monumental constructivist key art: hard diagonals, massed civic
  architecture, broken infrastructure, and crimson pressure that moves through gold relation lines.
- **Empire Anatomized** supplies diagrammatic concept art: logistics, finance, policing, extraction,
  labor, and land shown as one material system rather than isolated spectacle.
- **The Bunker Oracle** supplies in-game atmosphere. It uses damp concrete, repaired instruments,
  engraved plates, map tables, and a private intelligence room that decodes an order in collapse.
<!-- vale ste.ProcedureLength = YES -->

These modes are a family, not three incompatible themes. Illustrations can use material neutrals,
smoke, rust, paper, and concrete beyond the strict UI palette. Data-bearing surfaces continue to
use the current Bevy role colors and must remain legible.

## Deliverable

The first visual-development suite contains sixteen Bevy-ready raster assets.

### Illustration set

- `hero-red-apparatus.webp` — 1536 by 864, the primary landscape key art.
- `hero-empire-anatomized.webp` — 1536 by 864, a second landscape key art with a systemic cutaway.
- `concept-bunker-oracle.webp` — 1024 by 1024, the intelligence-room environment plate.
- `concept-living-map.webp` — 1024 by 1024, the county map as a tactile relation instrument.
- `concept-carceral-circuit.webp` — 1024 by 1024, a `carceral` logistics and extraction plate.
- `concept-metabolic-rift.webp` — 1024 by 1024, extraction, transport, consumption, and ecological
  damage shown as one circuit.
- `banner-counties.webp` — 1536 by 384, a wide story banner for the county atlas.
- `banner-carceral.webp` — 1536 by 384, a wide story banner for the `carceral` story.
- `banner-topology.webp` — 1536 by 384, a wide banner for graph and flow surfaces.
- `banner-collapse.webp` — 1536 by 384, a wide banner for collapse and end-state presentation.

The illustrations do not depict a canonical victory, named faction, doctrine emblem, or new outcome.
They reveal material relations and atmosphere. They do not decide game rules or the ideological line.

### Interface set

<!-- This list pins the literal atlas inventory, so the item lengths follow the catalog. -->
<!-- vale ste.ProcedureLength = NO -->
- `title-mark.png` — 768 by 192, a Babylon tower-and-dialectical-split mark with transparent
  background.
- `interface-atlas.png` — 512 by 512, a 4-by-4 atlas of play, pause, step, speed, lens, map, story,
  beat, roster, material, topology, flow, pin, inspect, warning, and close symbols.
- `marker-atlas.png` — 384 by 256, a 3-by-2 atlas of hover, selection, pin, event, origin, and target
  map markers.
- `provenance-atlas.png` — 256 by 256, a 2-by-2 atlas for material, absent, not-computed, and
  redacted states.
- `frame-atlas.png` — 256 by 64, four 64-by-64 nine-slice cells for neutral, selected, critical,
  and absent frames.
- `surface-atlas.png` — 384 by 128, three 128-by-128 tiles for concrete, hatch, and paper surfaces.
<!-- vale ste.ProcedureLength = YES -->

Every interface raster has an editable SVG master. Exact prompt records and committed raster output
preserve each AI-generated illustration source. We do not trace these sources into fake vector
masters.

## Visual rules

- Keep the county map and all live charts as clean data surfaces.
- Place illustration and texture in story cards, menus, load surfaces, gallery views, and non-data
  negative space.
- Use the current Bevy palette from `palette.rs` for interface graphics.
- Lead illustration colors with near-black, crimson, gold, bone, rust, smoke, and concrete.
- Include muted earth tones, and do not restrict illustrations to the interface palette.
- Use sharp, structural, and asymmetrical geometry.
- Prefer hard diagonals, crop marks, registration marks, engraved grids, and purposeful negative
  space.
- Avoid generic neon cyberpunk, glossy sci-fi dashboards, rounded cards, empty telemetry, and
  triumphalist propaganda poses.
- Do not bake text into AI-generated art. Bevy supplies readable labels so localization and
  accessibility remain possible.
- Color is never the only meaning channel for controls, map markers, or provenance states.
- Add motion later only when a real tick, event, selection, or transition drives it.

## Source and runtime layout

Editable and provenance material lives under `design/bevy-assets/`:

- `sources/` contains SVG interface masters.
- `prompts/` contains one Markdown prompt record for each AI-generated illustration.
- `manifest.toml` records every runtime asset ID, source record, runtime path, category,
  dimensions, sampler, atlas layout when present, SHA-256 digest, and license.
- `PROVENANCE.md` records tool, date, authorship, curation, and the absence of copied third-party
  artwork.
- `README.md` is the small review index and records deterministic raster export and optimization
  commands.

Runtime assets live under `rust/crates/babylon-client/src/visual_assets/embedded/`. Bevy 0.18.1's
`embedded_asset!` macro compiles them into the executable. This choice avoids a new install-time
asset-root contract. The implementation uses PNG for transparent interface art. The current Bevy
dependency enables WebP for compact opaque illustrations. This change introduces no SVG loader or
new direct crate dependency.

Cargo resolves `image-webp` and `quick-error` as transitive decoder packages.

Every file remains below the repository's 1 MiB blob limit. The manifest pins the selected outputs
byte-for-byte. To regenerate or replace an image, update its digest and provenance.

## Bevy loading contract

`VisualAssetsPlugin` registers all sixteen raster files. A typed `VisualAssets` resource owns named
image handles and texture-atlas layouts. Typed enums expose atlas indices, so game code does not
use magic numbers or text paths.

The plugin also owns two conservative production uses:

- It renders the title mark beside the current readable `BABYLON` text.
- It renders the appropriate counties or `carceral` banner behind the tick-0 story card.
- The banner disappears when the player dismisses the card.

The other hero art, concept plates, and banners are immediately available through the typed
resource and appear in a Bevy `visual_assets` gallery example. They do not cover the live county map
or change any engine-facing module.

An asset-load failure is loud. Tests wait for Bevy's loaded state with a fixed polling bound and
report the failing typed asset ID. The implementation uses no unrelated fallback art or transparent
placeholder.

## Behavioral contracts

Implementation follows red, green, refactor.

The static asset contract verifies:

- The manifest and runtime directory contain exactly the same sixteen raster assets.
- Every SVG master is well-formed and every prompt record is present.
- Every PNG or WebP decodes with the declared dimensions and color mode.
- Every SHA-256 digest and category count matches.
- Every file remains below 1 MiB.
- Interface SVG colors belong to the current Bevy role palette.
- Every asset declares `AGPL-3.0-or-later` provenance.

The Rust contract verifies:

- All embedded images reach Bevy's loaded state within a fixed polling bound.
- The typed catalog contains sixteen images and the declared atlas cell counts.
- Every typed atlas index stays within its layout.
- The title mark retains a readable `BABYLON` text sibling.
- Story selection chooses the counties or `carceral` banner. Story-card dismissal hides it.
- The gallery creates one labeled display entry per asset.
- `VisualAssetsPlugin` leaves the engine tick and deterministic hash bytes the same.

GPU screenshots are review evidence, not CI goldens, because backend and font output vary by
machine. The source files, manifest dimensions, and selected raster bytes are the durable contract.

## Success criteria

The work is complete when all sixteen assets exist with recorded provenance. The two hero pieces,
four concept plates, and four banners must form one recognizable visual family. All files must load
through the typed Bevy resource. The production title and story card must use the appropriate art
and must not obscure live data after dismissal. The gallery must expose the complete suite. Scoped
and repository-wide verification must remain green.
