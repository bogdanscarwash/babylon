# Typography & Iconography — the shipped face of the Archive

> STATUS: BD-directed 2026-07-21 ("native nerd-fonts that we host... Iosevka... bleeding
> edge of what's possible in a terminal... nothing that'll create lag... just make it look
> sharp"). Design spec; implementation rides T6 (doctor probe), T7 (payload), T8 (aesthetic
> pass + icon registry). The implementing units write the ADR (trains-carry-ADRs pattern).

## 0. The honest mechanism (read this first)

A terminal app does not choose its font — the emulator renders whatever the player
configured. "Native fonts we host" therefore means four concrete things, all shippable:

1. **We ship the fonts** in the installer payload (through the signed nix channel — the
   fonts arrive from OUR R2 cache, i.e. we literally host them).
2. **We offer to install them** — consent-gated, user-level only (`~/.local/share/fonts`
   + fontconfig), never sudo, removed by uninstall.
3. **We offer terminal config** — a kitty drop-in (`font_family`, `symbol_map` for the
   Nerd Font ranges, emoji fallback); equivalent snippets documented for alacritty/wezterm/
   foot. `babylon doctor --font-setup` does it interactively.
4. **We probe and degrade** — the TUI detects glyph coverage at boot (ADR097 probe-once
   discipline) and renders through a tier ladder, so a stock Ubuntu terminal still gets a
   correct, handsome game and a configured kitty gets the full bleeding edge.

## 1. The font stack (all $0, all legal for private commercial use)

| role | font | license | why |
|---|---|---|---|
| text face | **Iosevka** (Term variant — no ligatures) | OFL 1.1 | BD-named; sharp/narrow/futuristic by design; the Term variant avoids ligature shaping cost and emulator quirks |
| icons | **Symbols Nerd Font Mono** (symbols-only) | patcher MIT; icon sets OFL/Apache-2.0/MIT/CC-BY-4.0 | icons WITHOUT patching the text face — fontconfig/`symbol_map` fallback; smaller payload; Iosevka upgrades stay independent |
| emoji | **Noto Color Emoji** | OFL 1.1 | kitty renders color emoji natively; used in narrative/chronicle prose only (§4) |

- **License duty** (rides the ship-everything PROVENANCE/LICENSES file): copy each
  upstream LICENSE text verbatim into the payload at build time — the build step reads
  the license from the artifact it ships, so every row is **[V-at-build]**, no claim
  travels unverified. OFL terms: redistribution + commercial use fine; never sell the
  fonts alone; keep notices. No RFN conflict for unmodified Iosevka.
- **Custom "Babylon" Iosevka build plan** (Iosevka is designed for this; nixpkgs
  `iosevka.override { privateBuildPlan = ...; }`) = post-1.0 polish, consistent with the
  standing custom-rasterization deferral. v1.0 ships stock Iosevka Term.

## 2. Delivery (T7 units)

- Fonts enter the **nix closure** (nixpkgs: `iosevka`, `nerd-fonts.symbols-only`,
  `noto-fonts-color-emoji` — exact attr names verified at build) → served from the signed
  R2 cache with everything else. Zero new hosting machinery.
- Installer consent prompt (default yes): user-level font install + kitty drop-in under
  `~/.config/kitty/kitty.d/` (or documented include). Uninstall removes both.
- `babylon doctor --font-setup`: detects the emulator, writes the right snippet, verifies
  with the probe, prints before/after glyph samples.

## 3. Detection & the tier ladder (T6 unit; extends ADR097 probe-once + the sextant probe)

Boot probe (once, cached per terminal fingerprint): truecolor? kitty graphics? Nerd Font
PUA glyphs render at width 1? emoji at width 2? Result pins one of three tiers:

- **FULL** — NF icons + emoji + truecolor gradients + kitty raster lane. The bleeding edge.
- **STANDARD** — unicode floor: box drawing, sextants, geometric shapes; no PUA, no emoji.
  Still sharp; nothing missing semantically.
- **ASCII** — degraded honest ([+] [!] [*]); everything readable, nothing pretty.

**The icon registry is the single source** (T8): `tui/theme/icons.py`, one frozen row per
semantic icon — `(name, full_glyph, standard_glyph, ascii_glyph)` — covering: the four
view tabs, the 9 player verbs + 6 state verbs, chronicle severity (derived-CRITICAL gets
the crimson rupture mark), watchlist states, endgame-axis gauges, save/autosave, narrator
presence, fog/veil markers. NO scattered glyph literals in widgets — the registry is a
closed hand-curated registry (vocabulary-sentinel shape; a wiring-doctrine-style check
reds an unregistered glyph literal in `tui/`). Registry rows name icons semantically
(nf-md-* class names resolved at build); no codepoint is hardcoded unverified.

## 4. The look (T8 aesthetic pass) — KSBC, evolved

Palette stays the ruled KSBC signature: field `#1a0000`, text `#e8e8e8`, crimson
`#dc143c`, gold `#ffd700`, greys `#404040/#c0c0c0/#202020`. Square corners, hard
zero-blur shadows, title tabs breaking border lines, inverse-video selection — the
Guix-newt skeleton stands.

**Recorded aesthetic delta (BD 2026-07-21, supersedes-in-part the 2026-07-11 "no
gradients on chrome" clause):** truecolor **ramp gradients are now sanctioned on chrome
accents** — crimson→gold ramps across header rules, rail borders, the verb bar's active
plate, progress/tension gauges (the neovim-statusline look). The 2026-07-11 ban's intent
— no soft blurs, no rounded corners, no web-css mush — REMAINS binding. Gradients here
are per-cell color ramps on text/border glyphs: hard-edged, futuristic, zero blur.

## 5. Performance guardrails (BD constraint: no lag, no memory, no high-end kit)

- Glyphs are codepoints — **zero** runtime cost over ASCII. Fonts live on disk
  (~10–20 MB total), in the emulator's memory, never in the game process.
- No ligatures required anywhere (Term variant) — no shaping cost.
- Gradients render on **chrome only**, computed once per resize/theme-change and cached
  as styled segments — never per tick, never over data tables or the map canvas.
- Emoji confined to narrative/chronicle prose; **never in data tables or hot-path
  widgets** (width-2 cells complicate reflow on some emulators).
- The existing paint budget stands (200×50 full repaint ≈ 58 ms headless, ADR-recorded);
  the tier ladder means a weak machine on STANDARD/ASCII pays even less.
- Fallback is total: no tier requires a GPU, a specific emulator, or any font install.

## 6. Unit placement

| train | unit |
|---|---|
| T6 | boot glyph/emoji probe + tier pin; `doctor --font-setup`; doctor report shows tier |
| T7 | font packages into the closure; consent-gated install + kitty drop-in; LICENSES rows [V-at-build]; uninstall cleanup |
| T8 | icon registry + unregistered-glyph check; KSBC gradient chrome; tier-ladder verification (golden per tier for one representative screen); DESIGN_BIBLE §9b amendment recording the gradient delta |
