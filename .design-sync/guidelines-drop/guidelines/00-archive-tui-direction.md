# THE ARCHIVE DIRECTION — read this before designing anything (2026-07-21)

Babylon's shipping client is now a **terminal application** (the Archive, built in
Textual). The React components in this library are the **legacy cockpit era** — use them
as vocabulary and brand reference, but NEW designs should mock the **TUI idiom**: what a
bleeding-edge terminal game looks like. You are designing screens that will be rebuilt
in a character grid, so design like the screen IS a character grid.

## The idiom (Guix-newt skeleton, KSBC blood)

- **Field**: near-black with a blood-red undertone `#1a0000`. Text `#e8e8e8`.
- **Crimson `#dc143c`** = borders, urgency, rupture. **Gold `#ffd700`** = selection,
  action, solidarity. Greys `#404040` / `#c0c0c0` / `#202020`.
- **Square corners. Hard zero-blur offset shadows. No rounded corners, no soft glows,
  no blur — ever.**
- Centered plates on a flat dead-space field; **title tabs that break the top border
  line**; double-line inner wells; **inverse-video selection bars**; chunky
  hard-shadowed button "keys"; keyboard-hint footers; hatched scrollbars.
- **Monospace-first everywhere** — Iosevka. Box-drawing characters (─│┌┐└┘├┤┬┴┼ ═║╔╗╚╝)
  are first-class layout material.
- **NEW (2026-07-21): crimson→gold RAMP GRADIENTS on chrome accents are sanctioned** —
  header rules, rail borders, the active verb plate, tension gauges. The neovim-
  statusline look: per-character color ramps, hard-edged. Never on data tables, never
  on the map canvas, never blurred.

## The shell (hybrid: tabbed main + persistent rails)

- **Main area**: tabbed between the four views — Dashboard (economy dossier), Map
  (choropleth + lens bar), Wiki (the Archive vault), Topology (orgs/institutions).
- **Left rail (persistent)**: the Chronicle — a severity-tiered event feed; derived-
  CRITICAL events get the crimson rupture treatment and autopause the sim.
- **Right rail (persistent)**: the Watchlist — pinned entities with tick-fresh readouts.
- **Bottom bar (persistent)**: the Verb Bar — the player's 9 verbs as hard-shadowed
  keys with Nerd Font icons + keyboard hints; active verb plate carries the ramp
  gradient.
- Status line: tick counter (week N of 5200), determinism seed, autosave marker,
  narrator presence dot, endgame-axis mini-gauges.

## Mocking rules

- Compose in a monospace grid; pixel-align to character cells (integer ch/lh units).
- Icons come from Nerd Font glyphs (rendered via the shipped Iosevka NF face) — see
  01-typography-iconography.md. Emoji ONLY inside narrative prose, never in tables.
- Density is a feature: this is an instrument for reading a world, not a marketing
  page. Prefer full plates of live data over whitespace.
- Every screen should feel like a phosphor terminal takeover: dark field, crimson
  frames, gold highlights, text everywhere, zero chrome fat.
