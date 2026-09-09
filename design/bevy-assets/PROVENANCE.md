<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->

# Bevy asset provenance

This file is the authoritative provenance and license record for the sixteen assets in
`manifest.toml`. Every asset in this estate uses `AGPL-3.0-or-later`.

## Original interface work

The project created the six interface assets as original SVG work on 2026-08-22. The Director
approved the visual direction. Codex authored the geometry and exported the PNG runtime files under
that direction. The work does not copy third-party art, icons, logos, or font outlines.

| Asset | Editable source | Runtime export |
| --- | --- | --- |
| `title-mark` | `sources/title-mark.svg` | `title-mark.png` |
| `interface-atlas` | `sources/interface-atlas.svg` | `interface-atlas.png` |
| `marker-atlas` | `sources/marker-atlas.svg` | `marker-atlas.png` |
| `provenance-atlas` | `sources/provenance-atlas.svg` | `provenance-atlas.png` |
| `frame-atlas` | `sources/frame-atlas.svg` | `frame-atlas.png` |
| `surface-atlas` | `sources/surface-atlas.svg` | `surface-atlas.png` |

Inkscape produced the runtime PNG files from the SVG masters. The source geometry, runtime bytes,
and SHA-256 digests remain in the repository for review.

## OpenAI-generated illustrations

OpenAI's built-in image-generation tool created the ten illustration masters on 2026-08-22. The
process used no input image or third-party reference image. Each linked prompt record contains the
complete final prompt verbatim.

The work used human-directed curation: the Director approved the three-mode visual family and its
constraints, and Codex selected and inspected one output per prompt. Selection required a clear
thumbnail subject, usable negative space for a Bevy text overlay, and no invented gameplay claim.

| Asset | Prompt record | OpenAI generated-master identifier |
| --- | --- | --- |
| `hero-red-apparatus` | `prompts/hero-red-apparatus.md` | `exec-0905f191-1fe1-4057-8bf9-f340c018e319.png` |
| `hero-empire-anatomized` | `prompts/hero-empire-anatomized.md` | `exec-5e9bbdfc-ac62-40a9-950e-38cba7867392.png` |
| `concept-bunker-oracle` | `prompts/concept-bunker-oracle.md` | `exec-9fa2af1f-592f-4162-b907-22f7e3824ead.png` |
| `concept-living-map` | `prompts/concept-living-map.md` | `exec-bb026837-8a2c-4c68-86a5-807785dfd2cb.png` |
| `concept-carceral-circuit` | `prompts/concept-carceral-circuit.md` | `exec-10d63720-b2c6-4dec-bd90-9b2cf405d82f.png` |
| `concept-metabolic-rift` | `prompts/concept-metabolic-rift.md` | `exec-d826bc34-1842-4c00-8cb8-6185575f8a42.png` |
| `banner-counties` | `prompts/banner-counties.md` | `exec-855a37a8-7b47-4c48-8c0a-85d456277915.png` |
| `banner-carceral` | `prompts/banner-carceral.md` | `exec-567de97a-c26d-4c6a-beb8-62b7be67dc7c.png` |
| `banner-topology` | `prompts/banner-topology.md` | `exec-48bfea31-cdc9-45e4-b2fb-b9e0ace76b56.png` |
| `banner-collapse` | `prompts/banner-collapse.md` | `exec-1ad583b3-1b97-4a74-b6d5-cd86b88135df.png` |

ImageMagick applied only deterministic resize and center-crop operations. `cwebp` encoded each
selected result at quality 82 with method 6. The committed WebP files record the selected outputs.
`manifest.toml` pins their dimensions and SHA-256 digests.

## License disposition

The project distributes the SVG masters, prompt records, interface PNG files, and generated WebP
files under `AGPL-3.0-or-later`. This disposition covers all sixteen manifest rows. It does not
change the separate CC0-1.0 classification of Babylon's shipped audio estates.
