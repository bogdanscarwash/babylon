# Typography & iconography (ruled 2026-07-21)

## The stack

| role | face | notes |
| --- | --- | --- |
| text | **Iosevka** (Term variant) | sharp, narrow, futuristic; no ligatures; THE face of the game |
| icons | **Symbols Nerd Font Mono** | icon layer via fallback — width-1 glyph cells |
| emoji | **Noto Color Emoji** | narrative/chronicle prose ONLY |

This library already ships an Iosevka NF subset (`fonts` + `nerd-fonts.css` aliases) —
use monospace stacks that resolve to it. All faces are OFL/MIT: free, commercial-legal.

## The tier ladder (design for FULL, degrade honestly)

- **FULL**: NF icons + emoji + truecolor ramp gradients + raster panels (kitty lane).
- **STANDARD**: unicode floor — box drawing, sextants, geometric shapes; no icon font.
- **ASCII**: `[+] [!] [*]` markers. Everything readable, nothing decorative.

Every icon in a mock must have an obvious STANDARD/ASCII stand-in — if a glyph carries
meaning nothing else carries, the design is wrong.

## Icon vocabulary (semantic — one glyph each, consistent across screens)

- **View tabs**: Dashboard (gauge/graph glyph), Map (hex/globe), Wiki (book/archive),
  Topology (graph-nodes).
- **The 9 player verbs**: educate, reproduce, attack, mobilize, campaign, aid,
  investigate, move, negotiate — each gets one NF glyph on its verb key.
- **The 6 state/CPU verbs**: invest, police, adjudicate, subsidize, repress, trade —
  shown when the state acts in the Chronicle.
- **Severity**: informational (dim grey dot) → warning (gold triangle) → CRITICAL
  (crimson rupture mark, autopause). Severity is derived, never decorative.
- **Status**: autosave (disk), narrator (quill/dot), fog/veil (eye-slash), endgame
  axes (five mini-gauges), determinism seed (anchor).

## Composition rules

- Icons are width-1 cells inline with text — never image assets, never scaled.
- Gradients: crimson `#dc143c` → gold `#ffd700` ramps across a run of characters
  (header rules, gauge fills, active plates). Hard steps per cell are fine — this is
  a terminal, celebrate the quantization.
- Data tables: pure monospace, no emoji, no width-2 characters — alignment is sacred.
