# Bevy visual assets — `Implementation Plan`

<!-- This executable plan needs project API, command, and asset terms that the STE word list omits. -->
<!-- vale ste.UnapprovedWords = NO -->

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans. Do this plan one task at a time. A checkbox
> (`- [ ]`) shows the status of each step.

**Goal:** Ship a licensed sixteen-image visual-development suite and a typed Bevy 0.18.1 load,
presentation, and gallery surface. Do not change simulation behavior.

**Architecture:** Opaque illustrations that AI generates ship as compact WebP files. Authors supply
transparent graphics for the interface as PNG files exported from SVG masters. `VisualAssetsPlugin` embeds every
raster and exposes named handles plus typed atlas layouts. Separate presentation and gallery plugins
keep load, production use, and review concerns independent.

**Tech Stack:** Bevy 0.18.1 (`embedded_asset!`, `ImageNode`, `TextureAtlasLayout`, WebP feature), Rust
1.91.1, and SVG. Export tools are `inkscape`, `magick`, and `cwebp`. Static contracts use Python
3.12, `tomllib`, and Pillow.

**Spec:** `docs/superpowers/specs/2026-08-22-bevy-visual-assets-design.md`

## Global Constraints

- All new visual assets are `AGPL-3.0-or-later` and have explicit provenance.
- The runtime catalog contains exactly sixteen rasters: ten illustrations and six interface files.
- Every tracked file stays below the repository's 1 MiB ordinary-blob limit.
- AI-generated images contain no baked text, named faction emblem, canonical victory, or new outcome.
- Interface SVG files use only `#1a0000`, `#e8e8e8`, `#dc143c`, `#ffd700`, `#404040`, `#202020`,
  `#4169e1`, `#228b22`, `none`, and opacity.
- Illustration and texture never cover the live county map after the player dismisses the story card.
- This work adds no runtime SVG loader or new direct crate dependency.
- The current Bevy dependency gains the `webp` feature.
- Cargo resolves `image-webp` and `quick-error` as transitive decoder packages.
- All new loops use compile-time fixed bounds. All asset-load polls stop after 64 updates.
- Engine state, rules, tick ordering, and deterministic hash bytes do not change.

---

### Task 1: Authored interface pack and static contract

**Files:**

- Create: `tests/unit/render/test_bevy_visual_assets.py`
- Create: `design/bevy-assets/sources/title-mark.svg`
- Create: `design/bevy-assets/sources/interface-atlas.svg`
- Create: `design/bevy-assets/sources/marker-atlas.svg`
- Create: `design/bevy-assets/sources/provenance-atlas.svg`
- Create: `design/bevy-assets/sources/frame-atlas.svg`
- Create: `design/bevy-assets/sources/surface-atlas.svg`
- Create: `design/bevy-assets/manifest.toml`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/title-mark.png`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/interface-atlas.png`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/marker-atlas.png`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/provenance-atlas.png`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/frame-atlas.png`
- Create: `rust/crates/babylon-client/src/visual_assets/embedded/surface-atlas.png`

**Interfaces:**

- Consumes: the eight RGB role colors in `rust/crates/babylon-client/src/palette.rs`.
- Produces: six named interface assets in `manifest.toml`. Task 2 uses the final manifest schema.

- [ ] **Step 1: Write the red static interface contract**

Create a test module with these fixed contracts:

```python
from __future__ import annotations

import hashlib
import re
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image

_ROOT = Path(__file__).resolve().parents[3]
_DESIGN = _ROOT / "design" / "bevy-assets"
_RUNTIME = _ROOT / "rust" / "crates" / "babylon-client" / "src" / "visual_assets" / "embedded"
_INTERFACE_IDS = (
    "title-mark",
    "interface-atlas",
    "marker-atlas",
    "provenance-atlas",
    "frame-atlas",
    "surface-atlas",
)
_PALETTE = {
    "#1a0000", "#e8e8e8", "#dc143c", "#ffd700",
    "#404040", "#202020", "#4169e1", "#228b22",
}
_HEX_COLOR = re.compile(r"#[0-9a-fA-F]{6}")


def _manifest_assets() -> tuple[dict[str, object], ...]:
    data = tomllib.loads((_DESIGN / "manifest.toml").read_text(encoding="utf-8"))
    return tuple(data["asset"])


def test_interface_sources_and_rasters_match_the_manifest() -> None:
    manifest = _manifest_assets()
    assert len(manifest) in (6, 16)
    assets = manifest[:6]
    assert tuple(assets[index]["id"] for index in range(6)) == _INTERFACE_IDS
    for index in range(6):
        asset = assets[index]
        source = _ROOT / str(asset["source"])
        runtime = _ROOT / str(asset["runtime"])
        ET.parse(source)
        with Image.open(runtime) as image:
            assert image.size == (asset["width"], asset["height"])
            assert image.mode == "RGBA"
        assert runtime.stat().st_size < 1_048_576
        assert hashlib.sha256(runtime.read_bytes()).hexdigest() == asset["sha256"]
        assert asset["license"] == "AGPL-3.0-or-later"


def test_interface_svg_colors_are_palette_roles() -> None:
    colors = set()
    for index in range(6):
        source = _DESIGN / "sources" / f"{_INTERFACE_IDS[index]}.svg"
        colors.update(value.lower() for value in _HEX_COLOR.findall(source.read_text(encoding="utf-8")))
    assert colors <= _PALETTE
```

- [ ] **Step 2: Run the contract and confirm red**

Run:

```bash
mise run test:q -- tests/unit/render/test_bevy_visual_assets.py
```

Expected: failure because `design/bevy-assets/manifest.toml` and the six masters do not exist.

- [ ] **Step 3: Author the six SVG masters**

Use explicit `viewBox` and output dimensions that match the spec. Give every semantic cell a `<g
id="...">` name. The atlas cell IDs must be exactly:

```text
interface: play pause step speed lens map story beat roster material topology flow pin inspect warning close
marker: hover selection pin event origin target
provenance: material absent not-computed redacted
frame: neutral selected critical absent
surface: concrete hatch paper
```

Use only the palette literals in `_PALETTE`. Use `fill="none"` for transparency. Keep symbol
silhouettes distinct at one-quarter scale and keep frame borders within an 8-pixel inset so
nine-slice corners remain intact.

- [ ] **Step 4: Export and optimize the PNG files**

Run these six bounded exports from the repository root:

```bash
inkscape design/bevy-assets/sources/title-mark.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/title-mark.png
inkscape design/bevy-assets/sources/interface-atlas.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/interface-atlas.png
inkscape design/bevy-assets/sources/marker-atlas.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/marker-atlas.png
inkscape design/bevy-assets/sources/provenance-atlas.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/provenance-atlas.png
inkscape design/bevy-assets/sources/frame-atlas.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/frame-atlas.png
inkscape design/bevy-assets/sources/surface-atlas.svg --export-type=png --export-filename=rust/crates/babylon-client/src/visual_assets/embedded/surface-atlas.png
optipng -quiet rust/crates/babylon-client/src/visual_assets/embedded/*.png
```

- [ ] **Step 5: Populate the six manifest rows and make the test green**

Each `[[asset]]` row declares `id`, `category`, `source`, `runtime`, `format`, `width`, `height`,
`columns`, `rows`, `sampler`, `sha256`, and `license`. Get the digest from the real raster:

```bash
sha256sum rust/crates/babylon-client/src/visual_assets/embedded/title-mark.png
```

Copy the printed lowercase digest into the row. Repeat for the five other files. Do not enter a
synthetic or empty digest.

Run:

```bash
mise run test:q -- tests/unit/render/test_bevy_visual_assets.py
```

Expected: all tests of the interface contract pass.

- [ ] **Step 6: Commit the interface pack**

```bash
git add tests/unit/render/test_bevy_visual_assets.py design/bevy-assets rust/crates/babylon-client/src/visual_assets/embedded
mise run commit -- $'feat(assets): add authored Bevy interface pack\n\nCo-Authored-By: Codex <noreply@openai.com>'
```

### Task 2: Hero art, banners, concept plates, and provenance

**Files:**

- Create: `design/bevy-assets/prompts/hero-red-apparatus.md`
- Create: `design/bevy-assets/prompts/hero-empire-anatomized.md`
- Create: `design/bevy-assets/prompts/concept-bunker-oracle.md`
- Create: `design/bevy-assets/prompts/concept-living-map.md`
- Create: `design/bevy-assets/prompts/concept-carceral-circuit.md`
- Create: `design/bevy-assets/prompts/concept-metabolic-rift.md`
- Create: `design/bevy-assets/prompts/banner-counties.md`
- Create: `design/bevy-assets/prompts/banner-carceral.md`
- Create: `design/bevy-assets/prompts/banner-topology.md`
- Create: `design/bevy-assets/prompts/banner-collapse.md`
- Create: ten `.webp` files that match under `rust/crates/babylon-client/src/visual_assets/embedded/`
- Create: `design/bevy-assets/PROVENANCE.md`
- Create: `design/bevy-assets/README.md`
- Change: `design/bevy-assets/manifest.toml`
- Change: `tests/unit/render/test_bevy_visual_assets.py`
- Change: `LICENSING.md`

**Interfaces:**

- Consumes: the manifest schema and runtime directory from Task 1.
- Produces: ten image-generation prompt records and ten selected WebP outputs.
- Produces: a complete sixteen-row manifest and license evidence for all later tasks.

- [ ] **Step 1: Extend the static contract and confirm red**

Add fixed category IDs and a complete-runtime assertion:

```python
_ILLUSTRATION_IDS = (
    "hero-red-apparatus",
    "hero-empire-anatomized",
    "concept-bunker-oracle",
    "concept-living-map",
    "concept-carceral-circuit",
    "concept-metabolic-rift",
    "banner-counties",
    "banner-carceral",
    "banner-topology",
    "banner-collapse",
)


def test_illustrations_decode_and_match_the_manifest() -> None:
    manifest = _manifest_assets()
    assert len(manifest) == 16
    assets = manifest[6:16]
    assert tuple(assets[index]["id"] for index in range(10)) == _ILLUSTRATION_IDS
    for index in range(10):
        asset = assets[index]
        assert (_ROOT / str(asset["source"])).is_file()
        runtime = _ROOT / str(asset["runtime"])
        with Image.open(runtime) as image:
            assert image.size == (asset["width"], asset["height"])
            assert image.mode == "RGB"
            assert image.format == "WEBP"
        assert runtime.stat().st_size < 1_048_576
        assert hashlib.sha256(runtime.read_bytes()).hexdigest() == asset["sha256"]
        assert asset["license"] == "AGPL-3.0-or-later"


def test_manifest_and_runtime_directory_have_exactly_the_same_files() -> None:
    assets = _manifest_assets()
    assert len(assets) == 16
    declared = {Path(str(assets[index]["runtime"])).name for index in range(16)}
    actual = {path.name for path in _RUNTIME.iterdir() if path.is_file()}
    assert actual == declared
```

Run the scoped test and expect failure because the ten illustration rows and files are absent.

- [ ] **Step 2: Generate and curate the ten illustration masters**

Use the image-generation tool once per prompt record. Every prompt states the exact subject from the
spec, the requested aspect ratio, “no words, no letters, no logos,” and the shared family:

```text
Severe materialist constructivist editorial art for the strategy game Babylon: near-black field,
crimson pressure, gold relation lines, bone highlights, rust and concrete neutrals, hard geometry,
engraved diagram marks, cinematic depth without neon cyberpunk, no glossy UI, no heroic victory
pose, no named organization or national flag, no words, no letters, no logo, no watermark.
```

Record the complete final prompt verbatim in its Markdown file. Apply three checks. The subject must
read at thumbnail size. The negative space must support a Bevy text overlay. The image must not
invent game-behavior claims.

- [ ] **Step 3: Normalize the selected outputs to runtime WebP files**

Use `magick` only for deterministic resize and crop operations. Use `cwebp` only to encode runtime
files. Hero
outputs become 1536 by 864, concept outputs become 1024 by 1024, and banner outputs become 1536 by
384. Use center crop unless the prompt's recorded safe-zone note requires a named gravity.

```bash
magick selected-source.png -resize '1536x864^' -gravity center -extent 1536x864 /tmp/babylon-asset.png
cwebp -quiet -q 82 -m 6 /tmp/babylon-asset.png -o rust/crates/babylon-client/src/visual_assets/embedded/hero-red-apparatus.webp
```

Repeat with the exact declared dimensions for each of the ten fixed outputs. If any file exceeds
1,048,575 bytes, lower WebP quality in increments of two until it passes. Do not change dimensions.

- [ ] **Step 4: Complete the manifest, provenance, and licensing records**

Add ten manifest rows with `format = "webp"`, `sampler = "linear"`, `columns = 1`, `rows = 1`,
their exact source prompt paths, dimensions, and SHA-256 digests. `PROVENANCE.md` records image
generation with OpenAI, the 2026-08-22 creation date, human-directed curation, deterministic resize
and encode operations, and
AGPL-3.0-or-later. `README.md` lists the asset families and exact export commands.

In `LICENSING.md`, explicitly add `design/bevy-assets/**` and
`rust/crates/babylon-client/src/visual_assets/embedded/**` to the AGPL section and state that this
new estate does not change the current CC0 audio classification.

- [ ] **Step 5: Run the complete static contract and Vale**

```bash
mise run test:q -- tests/unit/render/test_bevy_visual_assets.py
vale design/bevy-assets/README.md design/bevy-assets/PROVENANCE.md design/bevy-assets/prompts/*.md LICENSING.md
```

Expected: all static tests pass and Vale reports no errors or warnings.

- [ ] **Step 6: Commit the illustration estate**

```bash
git add design/bevy-assets rust/crates/babylon-client/src/visual_assets/embedded tests/unit/render/test_bevy_visual_assets.py LICENSING.md
mise run commit -- $'feat(assets): add Babylon hero and concept art suite\n\nCo-Authored-By: Codex <noreply@openai.com>'
```

### Task 3: Typed embedded Bevy catalog

**Files:**

- Create: `rust/crates/babylon-client/src/visual_assets/mod.rs`
- Create: `rust/crates/babylon-client/src/visual_assets/catalog.rs`
- Create: `rust/crates/babylon-client/tests/visual_assets.rs`
- Change: `rust/crates/babylon-client/src/lib.rs`
- Change: `rust/crates/babylon-client/Cargo.toml`

**Interfaces:**

- Consumes: the sixteen stable filenames and atlas layouts from Tasks 1 and 2.
- Produces: `VisualAssetsPlugin`, `VisualAssets`, `VisualAssetId`, five typed atlas-index enums, and `VISUAL_ASSET_CATALOG`.

- [ ] **Step 1: Write the red Rust catalog test**

Create `tests/visual_assets.rs` with an app that installs `MinimalPlugins`, `AssetPlugin`,
`ImagePlugin::default()`, and the not-yet-created plugin. Assert the catalog and typed atlas bounds:

```rust
#[test]
fn typed_catalog_declares_all_sixteen_images_and_bounded_atlases() {
    assert_eq!(babylon_client::visual_assets::VISUAL_ASSET_CATALOG.len(), 16);
    assert_eq!(babylon_client::visual_assets::InterfaceIcon::COUNT, 16);
    assert_eq!(babylon_client::visual_assets::MarkerIcon::COUNT, 6);
    assert_eq!(babylon_client::visual_assets::ProvenanceIcon::COUNT, 4);
    assert_eq!(babylon_client::visual_assets::FrameKind::COUNT, 4);
    assert_eq!(babylon_client::visual_assets::SurfaceKind::COUNT, 3);
}
```

Run `cargo test -p babylon-client --test visual_assets --locked` from `rust/`. Expect unresolved
`visual_assets` symbols.

- [ ] **Step 2: Declare the typed catalog**

In `catalog.rs`, define `VisualAssetId` with sixteen variants in manifest order, descriptor fields
`id`, `label`, `width`, `height`, `columns`, and `rows`, and a
`pub const VISUAL_ASSET_CATALOG: [VisualAssetDescriptor; 16]`. Define each atlas enum with
`#[repr(usize)]`, an exact `COUNT`, and:

```rust
#[must_use]
pub const fn index(self) -> usize {
    self as usize
}
```

The enum declaration itself fixes the valid domain. Do not add text-to-index parsing.

- [ ] **Step 3: Add `VisualAssetsPlugin` and the named resource**

In `mod.rs`, register every file explicitly with `embedded_asset!`, load every handle explicitly
with `load_embedded_asset!`, create atlas layouts with `TextureAtlasLayout::from_grid`, and insert:

```rust
#[derive(Resource, Clone)]
pub struct VisualAssets {
    pub hero_red_apparatus: Handle<Image>,
    pub hero_empire_anatomized: Handle<Image>,
    pub concept_bunker_oracle: Handle<Image>,
    pub concept_living_map: Handle<Image>,
    pub concept_carceral_circuit: Handle<Image>,
    pub concept_metabolic_rift: Handle<Image>,
    pub banner_counties: Handle<Image>,
    pub banner_carceral: Handle<Image>,
    pub banner_topology: Handle<Image>,
    pub banner_collapse: Handle<Image>,
    pub title_mark: Handle<Image>,
    pub interface_atlas: Handle<Image>,
    pub marker_atlas: Handle<Image>,
    pub provenance_atlas: Handle<Image>,
    pub frame_atlas: Handle<Image>,
    pub surface_atlas: Handle<Image>,
    pub interface_layout: Handle<TextureAtlasLayout>,
    pub marker_layout: Handle<TextureAtlasLayout>,
    pub provenance_layout: Handle<TextureAtlasLayout>,
    pub frame_layout: Handle<TextureAtlasLayout>,
    pub surface_layout: Handle<TextureAtlasLayout>,
}
```

Add `pub fn image(&self, id: VisualAssetId) -> Handle<Image>` as one exhaustive match over all
sixteen variants. Export the module from `lib.rs` and change the Bevy feature list to
`["pan_camera", "webp"]`.

- [ ] **Step 4: Prove all embedded images load with a fixed bound**

Add a second test that installs the plugin, reads all sixteen handles from the typed catalog, and
updates the app at most 64 times. The polling loop is exactly `for _ in 0..64`. After each update,
use `AssetServer::is_loaded_with_dependencies`. On timeout, collect the descriptor IDs whose handles
did not load and fail with those IDs.

Run:

```bash
cargo test -p babylon-client --test visual_assets --locked
cargo clippy -p babylon-client --all-targets --locked --no-deps -- -D warnings -D clippy::pedantic
```

Expected: both tests pass and clippy reports no warnings.

- [ ] **Step 5: Commit the typed loading surface**

```bash
git add rust/crates/babylon-client/Cargo.toml rust/crates/babylon-client/src/lib.rs rust/crates/babylon-client/src/visual_assets rust/crates/babylon-client/tests/visual_assets.rs rust/Cargo.lock
mise run commit -- $'feat(client): embed a typed visual asset catalog\n\nCo-Authored-By: Codex <noreply@openai.com>'
```

### Task 4: Production title and story-banner presentation

**Files:**

- Create: `rust/crates/babylon-client/src/visual_assets/presentation.rs`
- Change: `rust/crates/babylon-client/src/visual_assets/mod.rs`
- Change: `rust/crates/babylon-client/src/main.rs`
- Change: `rust/crates/babylon-client/tests/visual_assets.rs`

**Interfaces:**

- Consumes: `VisualAssets`, `SelectedStory`, and `StoryCardVisible`.
- Produces: `VisualPresentationPlugin`, `TitleMark`, `ReadableTitle`, and `StoryBanner` components.

- [ ] **Step 1: Write red presentation tests**

Build a headless app with the asset and presentation plugins plus explicit `SelectedStory` and
`StoryCardVisible` resources. Assert one title-mark `ImageNode`, one readable `Text("BABYLON")`, and
one story-banner `ImageNode`. Add a second test that starts with counties, changes
`SelectedStory` to `carceral`, updates, and compares the banner handle against the named resource
field. Set `StoryCardVisible(false)`, update, and assert `Visibility::Hidden`.

Run the integration test and expect unresolved presentation symbols.

- [ ] **Step 2: Add the title lockup**

`spawn_title_lockup` creates exactly two entities:

```rust
commands.spawn((
    ImageNode::new(assets.title_mark.clone()),
    Node {
        position_type: PositionType::Absolute,
        top: px(20),
        left: px(24),
        width: px(144),
        height: px(36),
        ..default()
    },
    TitleMark,
));
commands.spawn((
    Text::new("BABYLON"),
    TextFont { font_size: 28.0, ..default() },
    TextColor(crate::palette::GOLD),
    Node {
        position_type: PositionType::Absolute,
        top: px(58),
        left: px(24),
        ..default()
    },
    ReadableTitle,
));
```

The readable text remains independent of the image.

- [ ] **Step 3: Add bounded story-banner selection**

Spawn a 480-by-120 image at `top: px(8)` and `right: px(24)`. Use one two-arm selector:

```rust
#[must_use]
fn story_banner(assets: &VisualAssets, story: &crate::story::Story) -> Handle<Image> {
    match story.id {
        "counties" => assets.banner_counties.clone(),
        "carceral" => assets.banner_carceral.clone(),
        unknown => panic!("no visual banner declared for story {unknown:?}"),
    }
}
```

`sync_story_banner` updates the handle when `SelectedStory` changes and makes visibility exactly
match `StoryCardVisible`. Register it in `VisualPresentationPlugin`. It reads no engine resource.

- [ ] **Step 4: Wire production and prove it does not change determinism**

In `main.rs`, import `visual_assets`, add `VisualAssetsPlugin` and `VisualPresentationPlugin` after
`DefaultPlugins`, and delete the old local `spawn_title` system. Extend the test with two independent
`EngineSession`s, add visual plugins only to one app, advance both sessions once, and assert equal
`report.after` hashes. This proof uses the engine seam, not a screenshot assertion.

Run:

```bash
cargo test -p babylon-client --test visual_assets --locked
cargo test -p babylon-client --test determinism --locked
cargo clippy -p babylon-client --all-targets --locked --no-deps -- -D warnings -D clippy::pedantic
```

- [ ] **Step 5: Commit the production presentation**

```bash
git add rust/crates/babylon-client/src/main.rs rust/crates/babylon-client/src/visual_assets rust/crates/babylon-client/tests/visual_assets.rs
mise run commit -- $'feat(client): present Babylon title and story banners\n\nCo-Authored-By: Codex <noreply@openai.com>'
```

### Task 5: Review gallery and full verification

**Files:**

- Create: `rust/crates/babylon-client/examples/visual_assets.rs`
- Create: `rust/crates/babylon-client/src/visual_assets/gallery.rs`
- Change: `rust/crates/babylon-client/src/visual_assets/mod.rs`
- Change: `rust/crates/babylon-client/tests/visual_assets.rs`
- Change: `design/bevy-assets/README.md`

**Interfaces:**

- Consumes: `VISUAL_ASSET_CATALOG` and `VisualAssets::image`.
- Produces: `VisualAssetGalleryPlugin` and `GalleryAssetLabel`, with one inspectable card per asset.

- [ ] **Step 1: Write the red gallery contract**

Add a test that installs the asset and gallery plugins, updates once, queries
`With<GalleryAssetLabel>`, and asserts the exact fixed count `16`. Also collect label `Text` values
and compare them with the sixteen descriptor labels in catalog order.

- [ ] **Step 2: Add the gallery plugin**

`spawn_gallery` creates a camera, a full-screen vertical scroll root, and one card per entry in
the fixed catalog. Each card contains an `ImageNode` from `VisualAssets::image(entry.id)` and a
readable `Text` label. Hero and banner nodes preserve their aspect ratios. Square concepts and atlas
files use 256-pixel previews. The only explicit loop iterates the compile-time
`VISUAL_ASSET_CATALOG: [VisualAssetDescriptor; 16]`.

The example is exactly:

```rust
use babylon_client::visual_assets::{VisualAssetGalleryPlugin, VisualAssetsPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(babylon_client::palette::FIELD))
        .add_plugins((VisualAssetsPlugin, VisualAssetGalleryPlugin))
        .run();
}
```

- [ ] **Step 3: Run scoped verification and an eyes-on gallery pass**

```bash
cargo test -p babylon-client --test visual_assets --locked
cargo clippy -p babylon-client --all-targets --locked --no-deps -- -D warnings -D clippy::pedantic
cargo run -p babylon-client --example visual_assets --locked
```

Inspect all sixteen labeled entries for crop quality, legibility, unintended text, repeated anatomy,
and visual-family coherence. Record the command, not a GPU screenshot golden, in the README.

- [ ] **Step 4: Run the complete gate suite**

Run sequentially from the worktree root:

```bash
mise run test:q -- tests/unit/render/test_bevy_visual_assets.py tests/unit/render/test_rust_theme_parity.py
vale docs/superpowers/specs/2026-08-22-bevy-visual-assets-design.md docs/superpowers/plans/2026-08-22-bevy-visual-assets.md design/bevy-assets/*.md design/bevy-assets/prompts/*.md LICENSING.md
mise run rust:check
mise run check
```

Expected: all commands exit zero. If an unrelated pre-existing failure appears, preserve the full
output and stop. Do not relax the gate.

- [ ] **Step 5: Commit the gallery and verification record**

```bash
git add rust/crates/babylon-client/examples/visual_assets.rs rust/crates/babylon-client/src/visual_assets design/bevy-assets/README.md rust/crates/babylon-client/tests/visual_assets.rs
mise run commit -- $'feat(client): add visual asset review gallery\n\nCo-Authored-By: Codex <noreply@openai.com>'
```

## Self-review

- Spec coverage: Tasks 1–2 deliver and license all sixteen assets.
- Spec coverage: Task 3 embeds and types the assets.
- Spec coverage: Task 4 provides the two production uses and the determinism proof.
- Spec coverage: Task 5 provides complete review and verification.
- Placeholder scan: the plan names every file, command, enum, dimension, and manifest field.
- Placeholder scan: the implementation gets digests from real bytes before it commits the rows.
- Type consistency: the plan adds `VisualAssets`, `VisualAssetId`, and `VISUAL_ASSET_CATALOG` before
  their consumers.
- Type consistency: the plan adds `VisualPresentationPlugin` and `VisualAssetGalleryPlugin` before
  their consumers.
