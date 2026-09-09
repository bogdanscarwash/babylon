# Babylon Design System

**Product:** Babylon — The Fall of America
**Tagline:** *Graph + Math = History*
**Constitution:** VIII — Cold Collapse
**Author:** Persephone Raskova (@percy-raskova)
**Repo:** https://github.com/percy-raskova/babylon (branch: `dev`)

---

## What Is Babylon?

Babylon is a geopolitical simulation engine that models the collapse of American hegemony through Marxist-Leninist-Maoist Third Worldist (MLM-TW) theory. It is not a casual game — it is a computational political-economy simulator grounded in Marxian mathematics: imperial rent extraction, solidarity transmission, consciousness bifurcation, survival calculus, and revolutionary rupture.

**The Embedded Trinity architecture:**
- **The Ledger** (DuckDB/SQLite/Pydantic) — rigid material state, 17 JSON entity collections
- **The Topology** (NetworkX/GraphProtocol) — fluid relational state, class solidarity/exploitation edges
- **The Archive** (ChromaDB) — semantic history for AI narrative (observer only, never controls)

**Products / Surfaces:**
1. **Web App (Django + React/TypeScript)** — The primary player-facing interface ("The Cockpit"). Full viewport dashboard with hex map, topology graph panel, action composer, resource bar, event log, time series. Located at `web/frontend/`.
2. **PyQt6 Desktop GUI** — "The Synopticon" — a local dashboard for observing simulation state in real-time with ECharts and DearPyGui components. Located at `src/babylon/ui/`.
3. **Sphinx Docs Site** — Technical documentation at `docs/`. Themed in "Luxe Gothic" (deep burgundy + bronze rupture, walled off from the web UI).
4. **Rust CLI Simulation** — Run via `mise run sim:e2e-michigan`, no UI.

---

## CONTENT FUNDAMENTALS

### Voice & Tone
- **Direct, technical, materialist.** No hedging, no idealism. Babylon speaks in the language of physics and political economy.
- **Third-person institutional** for system labels. First-person plural ("we") in docs.
- **No emoji anywhere** — the terminal aesthetic forbids it. UI copy is spare and uppercase for labels.
- **Casing:** All UI labels are `UPPERCASE TRACKED` (e.g., `TICK`, `RESOLVE TICK`, `CL`, `HEAT`). Headers use `Title Case`. Body prose uses standard sentence case.
- **Precision over warmth.** Values are shown to 2–3 decimal places. Uncertainty is not softened.
- **Theoretical vocabulary used directly:** "imperial rent," "consciousness bifurcation," "solidarity transmission," "survival calculus," "rupture," "labor aristocracy." These are not explained inline — they are assumed known.
- **Epigraphs and quotes** from Marx, Engels, George Jackson, Mao — used in docs only, never in UI.
- **Urgency without drama.** "Resolve Tick" is a button, not a call to arms. The stakes are implied by the numbers.

### Example copy patterns:
- Button: `RESOLVE TICK` / `+ New Game` / `Submit Action`
- Status: `ACTIVE` / `CRITICAL` / `WARNING` / `Tick 0042`
- Error: `Login failed` / `Failed to load games`
- Label: `CL` (Cadre Labor) / `SL` (Sympathizer Labor) / `REP` / `HEAT`
- Section: `Your Games` / `Action: educate` / `TOPOLOGY` / `Time Series`

---

## VISUAL FOUNDATIONS

### Aesthetic Direction: Cold Collapse — "Concrete bunker, cyan emissions, red lasers"
The metaphor: *a scavenged terminal in a cold concrete bunker. The spire of empire still glows cyan through the smog. The threats are red lasers from outside.*

The UI is not a webpage. It is a **CRT cockpit in a damp stairwell**. Colors are not paint — they are **light emissions** in a cold, blue-grey space. Each accent encodes a verb, not a vibe.

### Color System — Constitution VIII

**Substrate** (concrete, smoke, steel — cooled toward blue-grey)

| Token | Hex | Role |
|---|---|---|
| `--babylon-void` | `#06070b` | Page background — deepest black, slightly bluer than pure |
| `--babylon-tar` | `#0d1016` | Login gradient end, deep panels |
| `--babylon-concrete` | `#11141c` | Card / panel fills |
| `--babylon-rebar` | `#1a1f2a` | Structural dividers, subtle borders |
| `--babylon-wet-steel` | `#28303d` | Default border, input outline |
| `--babylon-rust` | `#3a3530` | Decay accent, warm-grey for disabled states |

**Emissions** (text, from bone to shroud)

| Token | Hex | Role |
|---|---|---|
| `--babylon-bone` | `#d8dce0` | Primary text — never pure white |
| `--babylon-fog` | `#8a93a0` | Secondary text, gauge labels |
| `--babylon-ash` | `#5e6470` | Placeholders, inactive labels |
| `--babylon-shroud` | `#3d4250` | Lowest readable, disabled |

**Semantic accents — each one a verb**

| Token | Hex | Verb |
|---|---|---|
| `--babylon-spire` | `#4dd9e6` | **PRIMARY** — your agency, system online, active state |
| `--babylon-spire-dim` | `#2a8a93` | Hover fills, recessed active |
| `--babylon-laser` | `#ff3344` | **THREAT** — empire's violence, hostile action |
| `--babylon-thermal` | `#b8321f` | **CRITICAL** — system stress, deep red |
| `--babylon-rupture` | `#d4a02c` | **REVOLUTION** — bronze-gold, used rarely (earned) |
| `--babylon-cadre` | `#6b8fb5` | Labor aristocracy, info text |
| `--babylon-solidarity` | `#5fbf7a` | Sympathizer growth, mass-line success |
| `--babylon-rent` | `#8b4d9e` | Imperial rent extraction |
| `--babylon-heat` | `#d97a2c` | Surveillance pressure |
| `--babylon-population` | `#7a6db8` | Demographic scale |

**Luxe Gothic — print/cover ONLY** (walled off; bridges to web only via `--babylon-rupture`):
`--luxe-pitch #120004` · `--luxe-arterial #8b0a1a` · `--luxe-vellum #f4ece0` · `--luxe-buried-hope #1a3a1a` · `--luxe-forest-dim #2a6b2a`

### Typography
- **Sans (UI chrome):** `Space Grotesk` — humanist geometric with subtle character; replaces Inter
- **Mono (data, labels, IDs, terminal):** `JetBrains Mono` — generous tabular figures; replaces Roboto Mono
- **Display (titles, scarce):** `Redaction 35` — soft slabs, used sparingly for narrative moments
- **Pixel (accent, scarce):** `Departure Mono` / `VT323` — for system alerts and "old terminal" callouts only
- **No serif** in the web UI. Docs use Sphinx serif for body prose.
- Text never pure white — body uses `--babylon-bone` (`#d8dce0`)
- Labels: `UPPERCASE`, `tracking-widest` (0.4em), 10–11px, `--babylon-fog`
- Data values: `font-mono`, `font-semibold`, `--babylon-spire` (no longer gold)
- Tick counter: `font-mono text-2xl font-bold text-spire` with cyan glow

### Spacing & Layout
- Panel layout: **Left** (Topology Graph) | **Center** (Hex Map) | **Right Sidebar** (Action Composer / Results) | **Bottom** (Time Series / Events / Notifications)
- Border radius: `--radius-md` (4–6px) for buttons/inputs (sharper than V1); `--radius-lg` (8px) for cards/panels; `--radius-full` for badges/pills
- Padding: `var(--space-3)` (12px) for panels, `var(--space-2) var(--space-4)` for top bars
- Gap: `var(--space-3)` between cards, `var(--space-1)`–`var(--space-2)` for inline elements
- All panels: `border var(--border-default) bg var(--bg-surface)`

### Cards & Panels
- Background: `var(--bg-surface)` (`--babylon-concrete`)
- Border: `1px solid var(--border-subtle)` (`--babylon-rebar`)
- On hover: `border-color: var(--babylon-spire)`, subtle cyan tint background
- Shadow: `var(--shadow-card)` on rest, `var(--shadow-hover)` on hover
- No left-border accent stripe — full borders, all sides

### Backgrounds
- Never pure black — always `--babylon-void` (`#06070b`) or `--babylon-concrete` (`#11141c`)
- Login: `radial-gradient(ellipse at center, var(--babylon-tar), var(--babylon-void))`
- No other gradients in the UI chrome
- Texture layers (CRT effect): scanlines (1px / 4px / 5–18% opacity) + film grain (2–3% noise) + vignette (radial darkening)

### Borders & Dividers
- Structural dividers: `var(--border-subtle)` (`--babylon-rebar`)
- Cards / inputs: `var(--border-default)` (`--babylon-wet-steel`)
- Active / focus: `var(--border-active)` (`--babylon-spire`) + `box-shadow: 0 0 0 3px rgba(77,217,230,.1)`
- Error: `var(--border-error)` (`--babylon-laser`)

### Hover / Interactive States
- Hover: border → `--babylon-spire`, brightness `1.1`, subtle cyan tint background `rgba(77,217,230,.04)`
- Active/pressed: brief flicker + intensity spike (CSS animation)
- Focus: spire border + 3px cyan ring
- Glow on hover: cyan phosphor brightening (`var(--glow-spire)`)

### Animations
- **Quick, purposeful** — electrons don't meander
- Minimal easing: hardware switches don't smooth — `transition-duration: 150–200ms` mostly
- Flicker: opacity oscillates ±5% at 0.5–2Hz on active panels (`var(--flicker-speed)`)
- Film grain: constant subtle animation via CSS canvas layer
- Transitions: `transition-colors 500ms` for urgency tints; `transition-[width] 200ms` for panel resize

### Iconography
- Library: **Lucide React** (`lucide-react`) — stroke-based, minimal
- Size: `13` for inline, `14` for buttons
- Color: `--babylon-ash` (inactive) → `--babylon-fog` (hover) → `--babylon-spire` (active)
- Specific icons: `Settings`, `ChevronRight/Left/Down/Up`, `BarChart3`, `Vote`, `Users`, `Target`
- No emoji, no PNG icons, no custom SVG illustrations in the UI

### Data Visualization (Map Layers)
**Cold Collapse uses luminance-monotonic ramps** — one hue family per layer, lightness alone encodes magnitude (Tufte-correct, no rainbow soup):

| Layer | Ramp |
|---|---|
| **Heat** | `tar → rust → heat → laser` (single thermal axis) |
| **Consciousness** | `tar → cadre → spire` (cool axis, awakening reads as glow) |
| **Wealth** | `tar → rust → rupture` (warm axis, ends at scarcity-gold) |
| **Rent** | `tar → rent → thermal` (extraction axis, ends in violence) |
| **Biocapacity** | `thermal → shroud → solidarity` (diverging — depleted vs healthy) |
| **Population** | `tar → population` (single hue, lightness only) |

### Scrollbars
Custom dark scrollbar: `--babylon-void` track, `--babylon-wet-steel` thumb, `--babylon-fog` on hover, 8px width.

### Urgency Tints
Top-bar background shifts subtly when a metric crosses threshold:
- Normal: transparent
- Warning: `rgba(217,122,44,.06)` (heat tint)
- Critical: `rgba(255,51,68,.08)` (laser tint)

---

## ICONOGRAPHY

**Library:** Lucide React (CDN: `https://unpkg.com/lucide-react`)
**Style:** Stroke-based, single-weight, no fill. Clean geometric traces — consistent with "vector traces on a CRT screen."
**Usage:** Navigation chevrons, top-bar Settings, lens-bar BarChart3/Vote/Users/Target, sized 13–14px, colored ash/fog/spire per state.

**Brand imagery:**
- `assets/cover-art.jpg` — Dystopian cover art (armed troops in ruined city, red laser sights, cold cyan-blue tone). This is the chromatic anchor for Cold Collapse.
- `assets/babylon-falling.png` — Falling Tower of Babel (aged daguerreotype, monochrome). Used in docs / cover only.

---

## File Index

```
README.md                          ← This file
SKILL.md                           ← Agent skill descriptor
colors_and_type.css                ← All CSS custom property tokens (Cold Collapse v8)
assets/
  cover-art.jpg                    ← Chromatic anchor — armed troops, red lasers, cyan spire
  babylon-falling.png              ← Falling Tower of Babel (print only)
preview/                           ← Design system preview cards (registered in asset panel)
  colors-cold-collapse.html        ← ★ CANONICAL — Constitution VIII palette spec
  colors-primary.html              ← Substrate + emissions specimen
  colors-secondary.html            ← Semantic accents in context
  colors-data.html                 ← Map layer ramps (luminance-monotonic)
  type-scale.html                  ← Full type scale specimen
  type-specimens.html              ← Space Grotesk + JetBrains Mono + Redaction
  spacing-tokens.html              ← Space scale, radii, border width
  effects-crt.html                 ← Scanlines, vignette, spire/laser/rupture/solidarity bloom
  components-buttons.html          ← Primary / secondary / ghost / danger / resolve
  components-inputs.html           ← Text inputs, selects, checkboxes
  components-cards.html            ← Game card, metric card, Dialectic card
  components-badges.html           ← Status badges, lens tabs, notification dots
  components-resource-panel.html   ← Vanguard economy bar (CL/SL gauges, REP/$/HEAT)
  components-topbar.html           ← Tick counter, metric chips, resolve button
  components-bottom-panel.html     ← Time Series / Events / Notifications tabs
  brand-cover-art.html             ← Wordmark, cover art, falling-Babel imagery
ui_kits/
  webapp/                          ← ★ CANONICAL — Cold Collapse v8
    index.html                     ← Full interactive click-through prototype (4 screens)
    Login.jsx                      ← Login screen
    GameList.jsx                   ← Game list + nav
    GameShell.jsx                  ← Full game dashboard shell (cockpit)
    ActionPage.jsx                 ← Verb action form (educate / mobilize / attack / aid)
```

### Quick start — UI Kit

Open `ui_kits/webapp/index.html` in any browser. Navigate:
1. **Login** → enter any username + any password (except "wrong") → **Game List**
2. Click any game card → **Game Shell** (full dashboard with map, topology, resource bar, action composer, bottom panel)
3. Click any action (Educate / Mobilize / Attack / Aid) → **Action Page**
4. Submit → returns to Game Shell. Use "← Games" to go back.

### Quick start — CSS Tokens

```html
<link rel="stylesheet" href="colors_and_type.css">
<div class="babylon-root">
  <!-- All --babylon-* substrate/emission/accent tokens -->
  <!-- + semantic --bg-*, --text-*, --border-* aliases -->
  <!-- + --space-*, --radius-*, --shadow-*, --glow-* utilities -->
</div>
```

### Migration notes — V1 (Bunker Constructivism) → V8 (Cold Collapse)

If you're updating code that referenced V1 tokens, here's the mapping:

| V1 token | V8 replacement |
|---|---|
| `--babylon-gold` (`#c8a860`) | `--babylon-spire` (`#4dd9e6`) for primary; `--babylon-rupture` (`#d4a02c`) for breakthrough only |
| `--babylon-crimson` (`#e04040`) | `--babylon-laser` (`#ff3344`) |
| `--babylon-deep-crimson` | `--babylon-thermal` |
| `--babylon-data-green` (`#40c040`) | `--babylon-solidarity` (`#5fbf7a`) |
| `--babylon-royal-blue` (`#80b0e0`) | `--babylon-cadre` (`#6b8fb5`) |
| `--babylon-grow-purple` (`#a070d0`) | `--babylon-population` (`#7a6db8`) |
| `--babylon-warning-amber` (`#e0a030`) | `--babylon-heat` (`#d97a2c`) |
| `--babylon-dark-metal` | `--babylon-concrete` |
| `--babylon-soot` | `--babylon-rebar` |
| `--babylon-wet-concrete` | `--babylon-wet-steel` |
| `--babylon-silver` | `--babylon-fog` |
| `--babylon-dim-gray` | `--babylon-shroud` |
| `--babylon-blood-void` | `--babylon-tar` |
| `Inter` | `Space Grotesk` |
| `Roboto Mono` | `JetBrains Mono` |

**Discontinued tokens** (no V8 equivalent — they encoded vibe, not a verb):
`--babylon-gold-bright`, `--babylon-blood-crimson`, `--babylon-phosphor-green`, `--babylon-phosphor-red`, `--babylon-chassis`, `--babylon-luxe-bg-dark`, `--babylon-luxe-body`, `--babylon-luxe-warm-gray`. The Luxe Gothic palette has been narrowed to 5 print-only tokens (`--luxe-*`) and walled off from the web UI.
