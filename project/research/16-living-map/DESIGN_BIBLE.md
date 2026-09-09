# The Living Map — Design Bible

Program 16 / spec-113. Synthesized 2026-07-11 by Fable from the 21-agent Phase-R corpus
(10 UX books, 5 Paradox titles, Maxis + Paradox studio dives, babylon-data survey, 2 MIM
ideology reads, cockpit audit — full reports in this directory). This document governs
every design decision in the Living Map program; `specs/113-living-map/architecture.md`
governs structure. Where the two conflict, §9 here records the amendment.

---

## 1. North star

**The map is the argument.** Babylon's UI is not a dashboard reporting on a simulation;
it is a political map of a collapsing empire that the player reads, interrogates, and
acts on. Three research findings converge into one doctrine:

- Paradox: the map is the permanent full-viewport background; every panel is an overlay,
  never a room you leave the map to visit.
- Tufte/Norman: complex systems must push knowledge into the world — every number
  explains itself in place, spatially anchored to what it describes.
- Amin/Cope (via MIM): what the map SHOWS is itself the thesis — extraction, not
  "development"; colonial claims over an immutable land substrate; wages never shown
  without the rent that pays them.

Five pillars, each traceable to the corpus:

1. **Map never occluded, never left** (Shneiderman, Rules of Play's magic circle,
   Stellaris's full-screen-takeover anti-pattern). Chrome floats INSIDE the map's frame.
2. **Every number explains itself, by click-pin not hover-maze** (Vic3's #1 launch
   complaint; CK3/EU4's one-hover-then-pin ceiling; Norman's knowledge-in-the-world).
3. **One interaction grammar, one meaning per visual channel** (Norman consistency,
   Sylvester's channel discipline, Krug's neighbor test).
4. **Borders are claims, land is substrate** (Constitution + CK3 de-jure/de-facto +
   Amin: borders are colonial artifacts of the wage hierarchy — the starting map IS the
   colonial baseline, and its dissolution must be legible as exactly that).
5. **Voice rides on real numbers** (MIM's anti-formalism: cut any copy that performs
   tone without communicating an actual state change).

## 2. The map (cartographic substrate)

### 2.1 Layers (bottom → top)

| # | Layer | Source | Notes |
|---|-------|--------|-------|
| 0 | Land/water backdrop | Natural Earth v5.1.2 sqlite (110m/50m world frame; 10m US-adjacent context: coastline, lakes, major rivers) | muted, near-monochrome; the "table" the map sits on |
| 1 | County mesh (de jure substrate) | TIGER 2024 `tl_2024_us_county` → simplified TopoJSON keyed by GEOID (5-digit FIPS) | IMMUTABLE. Hairline county borders; state dissolve renders heavier "colonial" borders |
| 2 | Political claims (de facto) | polity membership from balkanization/sovereignty data → client-side `topojson.merge` over member counties | fills + thick claim borders; REDRAWS as counties change hands |
| 3 | Contest/mismatch | counties where de facto ≠ de jure | striped/hatched fill (CK3 convention), never a separate lens |
| 4 | Lens recolor | active lens fill function | recolors layers 1–2, never replaces geometry |
| 5 | Flows & markers | LODES commuter arcs, tribute/solidarity edges, org pins, event pins, AIANNH sovereignty overlay | full-saturation budget lives HERE only |
| 6 | Hex tiles (deep zoom) | H3 res-7 via existing member_h3 | visible only at deep zoom + as frontline shading inside contested counties |

### 2.2 Cartography rules

- **Geometry never animates; claims do.** County/state/hex boundary geometry never
  morphs (Debord agent + Constitution). A border "redraw" is a re-dissolve of polity
  fills along immutable county arcs, animated ≥600ms with a wire toast naming the cause
  (Rules of Play discernibility). CK3's drift idiom is the model: gradual claim
  transitions, instant snaps reserved for Rupture events.
- **Zoom tiers are registers, not magnification** (CK3): national = state-dominant
  political register; regional = county-dominant with polity fills; deep = hex tactical
  register with strictly MORE data per screen area than the tier above (Tufte
  micro/macro: zooming in must reveal, never merely enlarge).
- **Snap-zoom between tiers** (Stellaris), inertial pan/zoom inside a tier (Rules of
  Play's mechanical "play").
- **AK/HI/PR insets** — standard cartographic repositioning; naive bbox lets Alaska
  dominate (data survey). Territories (AS/GU/MP/VI) included in data, inset or listed.
- **Basemap**: self-authored minimal MapLibre style (land/water only, Cold Collapse
  values). The Carto Dark Matter dependency is removed — the political layer IS the map.
- **One cartographic frame across lenses**: lens switches change ONLY per-feature fill
  functions; camera, projection, geometry identical pre/post switch (Tufte's constant
  frame; testable byte-compare).
- **Pipeline**: TIGER shp → mapshaper (npx, verified available; no GDAL on this box) →
  TopoJSON with shared arcs; also consider `dim_county_geometry` (reference DB, full-res
  WKT — 46KB/county, must be simplified, never served raw). Target ≤ ~2 MB national.

## 3. Lenses

### 3.1 Taxonomy (fixed now, with headroom — EU4's 40-mode lesson)

Four groups, ~6–8 primary lenses visible as a flat bar, overflow behind one labeled
toggle (HOI4 cap; CK3 primary/overflow split). "Look" lenses and "act" targeting are
distinct systems (Vic3): verb targeting recolors eligibility on TOP of the active lens,
using one 5-state eligibility grammar everywhere.

| Group | Lens | Backing | Notes |
|-------|------|---------|-------|
| **Extraction** | **Imperial Rent Φ** | economy metrics | **THE DEFAULT LENS.** Choropleth of net extraction, signed (drained ↔ enriched). A GDP-style default would reproduce the "backwardness" ideology the game refutes (Amin) |
| Extraction | Wage hierarchy W_c/V_c | metrics | class-stratification view, one toggle from Φ (Cope) |
| Extraction | Unequal exchange σ | trade-flows | edge overlay (color+thickness on tribute/wage flows), rides on Φ or wages |
| **Struggle** | Heat / tension | exists | single-hue ramp |
| Struggle | Solidarity network | communities/edges | lines-of-communication lens (Debord): full network drawn persistently while active |
| Struggle | Control ratio P(S\|R) vs P(S\|A) | metrics | rupture-proximity shading |
| **Political** | Claims (stance/faction/collapse) | balkanization | the de-facto polity view; striped mismatch built in |
| Political | National oppression | class composition + AIANNH | oppressed-nation territories vs settler-core (MIM lexicon: legend language) |
| **Reproduction** | Habitability / overshoot | exists | ecology NEVER a siloed green tab — paired with Φ (Amin's second dimension) |
| Reproduction | Dispossession | eviction/foreclosure (ref DB, future) | data-grounded expansion |

Future data-grounded lenses (survey): LODES commuter arcs, QCEW employment, coercive
infrastructure (repression icons grounding P(S|R)), Gini, broadband organizing-capacity,
FAF freight corridors. The taxonomy has slots for all of them.

### 3.2 Lens rules

- One lens = one dominant variable = one legend (HOI4/SimCity). No blended heatmaps.
- Quantitative ramps: single-hue light→dark within Cold Collapse; never >2 hues per
  variable; never rainbow (Tufte). Ramp DOMAIN fixed per session/lens — silent rescaling
  between ticks is banned; domain changes fire a legend flash (Tufte integrity).
- Full-saturation color is reserved for small discrete markers (event pins, selected
  units, active conflicts). Large polygon fills stay in Imhof's muted range (Tufte).
- Legend always visible while a non-default lens is active, adjacent to the map, swatches
  rendered against map-canvas tone (not panel white); legend shows a marker for where
  the current world state sits on the ramp (Sylvester).
- Every lens hotkey-bound; Q/E cycles (existing); number keys = speed (HOI4 gap).
- Colorblind-safe redundancy in every lens (shape/luminance/pattern, not hue alone) —
  HOI4/CK3/Stellaris all shipped without it and paid; QA against a simulator pre-ship.
- Selecting a lens syncs the relevant panel (Vic3: map and panel state never siloed).

## 4. Progressive disclosure — the InspectionStack

The recursive drill-down is the game's pedagogy: it teaches Cope/Amin through play.

- **Click-to-pin stacked cards, not hover chains.** Hover preview = one level max
  (EU4's ceiling); anything deeper is a click that pushes a breadcrumbed card (Krug:
  `>`-separated trail, current node bold). Vic3 paid a patch cycle to learn this.
- **Human-value labels first, coefficients on drill** (Sylvester): the top of a county
  card says "besieged / organized / bleeding value", not `k=0.73`.
- **Wages never naked** (Cope, binding): any wage figure renders in the same view as
  value produced and the imperial-rent transfer explaining the gap.
- **The rebuttal layer**: wherever a wage/productivity number appears, the apologist
  explanation ("skill premium") is one click from its refutation (Cope Part III) —
  this is the game's version of Tufte's full-population rule.
- **Every stat row shows its comparison baseline** (Tufte VE): distribution position,
  trend sparkline (with realized min/max labeled inline), or paired comparator.
- **Depth is content-limited, not technically infinite** (CK3); terminal frames are
  FormulaCards: expression, inputs (each recursively explainable), constants with
  provenance. Backed by the additive `/explain/` endpoint (architecture §2.4).
- **One fixed layout shell across all depth levels** (Tufte VDQI): country → region →
  county → hex reuse one card anatomy; only data changes.
- **Same-name discipline** (Krug): a drill-down card is titled with the EXACT term used
  in the parent row that opened it.
- **Read/write symmetry** (Paradox's most-cited Vic3 failure): any card stating a
  changeable fact links to the verb that changes it (ActionDock deep-link).
- Every StatChip in the TopBar is itself a Probe — disclosure starts at the top bar.

## 5. Chrome and the event system

### 5.1 Layout

- Four strata per architecture §0; every panel is a FloatingPanel INSIDE the map frame
  (magic circle). Content-to-chrome ratio ≥80% per panel (Tufte's kiosk test).
- TopBar: 4±1 clusters (Universal Principles), not a flat row: [identity/date/speed]
  [Φ chip + overshoot chip, paired, sharing color grammar] [alerts] [takeovers].
  Φ and O=C/B are the two permanent vitals (MIM theory agent).
- OutlinerOverlay: the always-on index of the player's organizations (Stellaris: the
  outliner is the real HUD); filterable, compact-density mode AT LAUNCH (Paradox lesson:
  list surfaces degrade with player success).
- ActionDock: ≤3 primary verbs at first contact + labeled "more" (Shneiderman
  progressive disclosure); verbs show live cost/eligibility on the button (EU4
  governing-capacity lesson); predicted delta arrows before commit (Sylvester); only
  currently-legal verbs rendered prominent (Universal Principles context sensitivity);
  bulk-apply escape hatch for multi-territory orders (Vic3 click-tax).
- Every overlay remembers its last-viewed subject per session (EU4 complaint).

### 5.2 Events: two streams, three channels, two lifetimes

- **Two streams** (Stellaris 2026 rework, designed-in from day one): URGENT/actionable
  vs AMBIENT/narrative. The wire feed is the ambient stream — a newspaper, not a log
  (SimCity: headlines with voice get read; toasts get tuned out).
- **Severity tiers** (Norman alarm-fatigue): ambient / notable / critical. Only critical
  interrupts. Multiple criticals in one tick merge into one queued surface, never
  stacked equal-weight alerts.
- **Critical events fire on three channels at once** (Sylvester): wire entry + toast +
  map-anchored visual on the affected geography.
- **Two toast lifetimes** (Stellaris): persistent-until-acted for decisions;
  ephemeral-with-generous-timing for flavor; simultaneous low-priority events batch into
  one expandable toast.
- **Recoverable dismissal tray** (HOI4): a missed toast is retrievable, not gone.
- **Per-category mute controls, in-context on each toast** (EU4 message settings), and
  a planned consolidation pass as event volume scales (Vic3 needed −50% post-launch;
  budget it now).
- **Positive-feedback loops get early map warnings** (Rules of Play/LeBlanc): solidarity
  snowballs and fascist consolidation show accelerating-trend glyphs before
  irreversibility.
- Autopause already exists; CriticalEventModal gives it its face (architecture §4.2),
  with a bounded default action so an ignored popup never stalls the game (CK3).

### 5.3 Interaction grammar (one, everywhere)

- Left-click = inspect (push card). Right-click = act (verb context at the clicked
  point). Drag = pan; scroll = zoom; Q/E = lens cycle; space = pause/resume; 1/2/3 =
  speed. This grammar NEVER varies by lens/zoom/panel (Norman standardization).
- Verbs with spatial targets are executable ON the map (Shneiderman direct
  manipulation); the dock form is the fallback for non-spatial verbs. Failed spatial
  actions snap back with a rejection animation, not a modal (Shneiderman).
- Legal-target highlighting: action-implying color appears ONLY on currently-legal
  targets for the selected org (Shneiderman/Rules of Play conveyance).
- Every interactive map element carries a signifier beyond its geometry (hover glow,
  cursor, halo); decorative elements never reuse that treatment (Norman/Krug).
- First paint of any lens/panel within 200ms; data may refine progressively after.

## 6. Visual language (Cold Collapse, evolved)

The audit found two visual languages coexisting: the takeovers' phosphor/scanline
diegetic voice vs the shell's neutral devtools micro-typography. **The reskin extends
the first into the second** — the game already knows how to look like itself.

- Palette: Cold Collapse ratified values untouched. Additions are ROLE tokens (chrome
  elevation, claim-border, contest-stripe, severity tiers) not new hues. Cyan
  ornamentation stays on frames/chrome, never over numbers or map fills (Vic3 trade-panel
  lesson). Red enters as the single structural urgency accent (MIM visual culture:
  red as rule/banner, not mood lighting).
- Grids/borders: hex outlines, gridlines, dividers at 20–30% the alpha of the data they
  host (Tufte); no solid-black boxed chrome outside critical modals; run the 1+1=3
  audit on every divider (Envisioning Information).
- Encodings: ≤2 visual encodings per datum; no pie charts anywhere (composition = ordered
  bars/BreakdownBar); no fake-3D, no texture fills in data surfaces (VDQI).
- Type: two registers matching the two voices — terse mono/numeric register for HUD and
  wire headlines; longer serifed/periodic register reserved for deep drill-down theory
  panes (MIM's newspaper vs theory registers). No ALL-CAPS shouting; urgency via
  weight/size/color (MIM pitfalls).
- Motion: 200ms response floor; 600ms+ for meaning-bearing transitions (border
  re-dissolves, endgame); every animated/tick-scrubbed view keeps a persistent tick/date
  readout (Tufte's dequantified-animation ban); a subtle tick pulse on the time control
  so advancement is never silent (Norman feedback).
- The un-slick principle (MIM visual culture): woodcut/newsprint sensibility over gloss.
  Icon-grid badges (one small glyph per cause/verb) over hero art. If illustrated events
  ever ship, they are agit-poster, credited, not stock.

## 7. Voice and lexicon

- **Two registers** (MIM): wire/HUD = newspaper (short declaratives, actor + action +
  tick by the second clause, scare-quoted state euphemisms); drill-down = theory
  (periodic sentences, enumerations, citations).
- **Copy rules** (binding, from mim/voice-and-lexicon.md): headlines lead with actor +
  action + date; name the driving contradiction explicitly; tone modulated to actual
  stakes; imperative second person ONLY for opted-in confirmations; endgames are never
  neutral scoreboard text; every stat is one click from its formula; one glossary fixes
  every recurring name's spelling everywhere.
- **Lexicon adopted**: labor aristocracy, superprofits/superexploitation (threshold
  badge, not slider), united front (alliance mechanic), mass line (educate/investigate
  loop), principal contradiction (HUD label), revisionism/opportunism (drift labels),
  cardinal principles (org founding), national oppression / oppressed vs settler nation
  (lens language), self-determination (late-game verb distinct from liberate), vanguard
  (top-tier org), comprador (peripheral ruling class).
- **Explicitly NOT adopted**: orthographic stunts ("Amerika", "$", "kkk") — achieve the
  same force through mechanics (territories renamed on liberation); real-world factional
  invective; the "gender aristocracy" frame (flagged weakest/essentialist); figure
  hagiography (channel into generated leaders instead).
- **Purge the admin voice**: every empty state ("No world state loaded yet.") rewritten
  in-register (the Wire already proves the codebase can do it — "neutrality is
  hegemony"). Component vocabulary de-IDE'd where player-facing.

## 8. Playability details

- Scenarios start paused (Vic3 community norm), on the Φ lens, camera framed to the
  scenario region with the world-context backdrop visible.
- Onboarding: no 67-box scripted tutorial (CK3 telemetry); per-coachmark dismissal
  (Stellaris); new mechanically-significant conventions (claim redraw, contested
  stripes, siege badges) must occur in a guaranteed low-stakes early context before
  they carry weight (Sylvester); instrument completion-vs-retention from day one.
- Under-explain verbs, let cheap legible failure teach (Wright's possibility space);
  the wire + InspectionStack are the query tool that makes failure legible.
- Trunk test (Krug) as a standing QA ritual: from any blurred mid-tick screenshot, a
  first-time viewer names faction, date, active lens, and the way back to the default
  map.
- Loud Failure (Constitution III.11) extends into the UI: stale/sentinel/fallback values
  render flagged, never as plausible color (SimCity 2013's sin, inverted).

## 9. Amendments to architecture.md (from research)

1. **Default lens is Imperial Rent Φ**, not political-topology (MIM theory agent;
   §3.2's lens roster reorders accordingly). Political claims remain the layer-2
   substrate under every lens.
2. **HexTooltip.tsx and FramingSelector.tsx ownership → Lane B** (audit found them
   unowned in §5). FramingSelector inverts per the Carto addendum: county/state are
   primary framings, hex is the deep-zoom register — default framing changes and its
   semantic weight flips.
3. **Lane E additions**: recoverable toast tray, per-category in-context mutes,
   two-lifetime toasts, urgent-vs-ambient stream split (all §5.2 here).
4. **Lane Carto additions**: AIANNH tribal-areas overlay (867 features, sovereignty
   lens material); AK/HI/PR insets; `dim_county_geometry`/`bridge_county_h3` noted as
   existing DB endpoints (simplification still required; res-8 discrepancy vs
   Constitution II.13 flagged to the Program 11 owner, not ours).
5. **Eligibility grammar**: one 5-state target-eligibility color grammar shared by all
   verbs/lenses (Vic3), owned by Lane B, consumed by Lane F's TargetPicker map mode.
6. **OrgSelect / faction `<select>` controls** (audit): restyled in-register in Phase D;
   map-native org picking queued as a follow-on, matching TargetPicker's pick-on-map.
7. **TimeseriesChart** stays hosting-only in Lane E but is flagged for a BblData-idiom
   restyle in Phase D (audit: last chart-first component).

## 9b. THE INSTALLER — owner aesthetic ruling (2026-07-11, binding; amends §6)

Percy's direction, given with three Guix 1.5.0 installer screenshots and their Kitty
theme (`~/.config/kitty/themes/ksbc-new.conf`): *"the guix GUI is how I'd like the
'aesthetic vibe aura' of Babylon to feel... the menus and dead space and what not in
the style of GUIX, but with my Kitty configuration color scheme"* — while still
rendering good maps.

### Palette (Kitty `ksbc-new`, verbatim)

| Role token | Value | Kitty source |
|---|---|---|
| field (dead space) | `#1a0000` | background — near-black, blood-red undertone |
| text | `#e8e8e8` | foreground |
| accent-crimson | `#dc143c` | cursor, active border — borders, urgency, cursor accents |
| accent-gold | `#ffd700` | selection bg, bell, url — SELECTION, action, solidarity (Article VII's GOLD) |
| selection text | `#000000` | selection_foreground — inverse video |
| muted | `#404040` / `#c0c0c0` / `#202020` | inactive greys |
| supporting | green `#228b22/#32cd32`, royal `#4169e1`, cyan `#008b8b` | ANSI ramp — data accents |

Cold Collapse canon tokens REMAIN (contract add-only; wire/dialectic/data ramps keep
their language). The CHROME skin moves to ksbc: crimson borders, gold selection/action,
near-black field. §6's "cyan on frames" clause is superseded for chrome.

### Degraded palette (256-color, ADR097 D5)

The crimson/gold truecolor canon has a DECLARED 256-color fallback (ADR097 D5): terminals without
truecolor render the same roles at their nearest xterm-256 index. This is honest degradation, never
silent — `babylon doctor` prints the palette verdict. The mapping is computed by
`nearest_xterm256` and lives in `babylon.render.tiers.DEGRADED_256_PALETTE` (the single source of
truth; `tests/unit/render/test_design_bible_parity.py` fails if this table drifts from the code).

| Role token | Truecolor | xterm-256 |
|---|---|---|
| `field` | #1a0000 | 232 |
| `text` | #e8e8e8 | 254 |
| `accent_crimson` | #dc143c | 161 |
| `accent_gold` | #ffd700 | 220 |
| `selection_text` | #000000 | 16 |
| `muted_dim` | #404040 | 238 |
| `muted_light` | #c0c0c0 | 250 |
| `muted_dark` | #202020 | 234 |
| `green_dark` | #228b22 | 28 |
| `green_bright` | #32cd32 | 77 |
| `royal` | #4169e1 | 62 |
| `cyan` | #008b8b | 30 |

### The dialog anatomy (newt idiom → FloatingPanel/modal reskin)

- **Dead space is composition**: takeovers, lobby, login, scenario select, endgame =
  a flat `#1a0000` field with ONE centered plate — the full Guix treatment (these
  surfaces are literally menus; the map is absent there anyway).
- **Plate**: flat panel tone, **hard offset shadow, zero blur** (`box-shadow: 8px 8px 0
  #000`), square corners, generous inner padding.
- **Title tab breaks the border**: panel titles render as a label interrupting the top
  border line (fieldset/legend idiom), crimson text, `┤ ├` feel.
- **Inner wells**: double-line borders (CSS `border-style: double` or layered) in
  crimson around scrollable content regions.
- **Selection = inverse video**: full-width gold bar, black text. Focus/hover the same
  grammar at lower weight.
- **Buttons are keys**: chunky, hard-shadowed; press = shadow collapse + 2px translate.
  A keyboard-hint footer row on every dialog (the installer's soul; we already have
  Q/E/space/1-2-3).
- **Scrollbars**: styled hatched/dotted tracks, square thumbs.
- **Monospace-first** for all chrome (JetBrains Mono/Departure Mono already shipped);
  proportional reserved for long-form drill-down prose.
- **In-game floating chrome** (over the live map): same anatomy at reduced opacity —
  plates go `#1a0000`-translucent so the map reads through dead space, but borders/
  title-tabs/selection grammar identical. The MAP CANVAS itself keeps §2/§6
  cartographic discipline (muted fills, Tufte alpha rules) with crimson/gold as its
  marker duo.

### Juice-pass re-aim (supersedes the gradient items in the ledger inventory)

OUT: gradient hairlines, conic corner accents, rounded anything, soft glows on chrome.
IN (all compositor-budget rules still binding): inverse-video selection sweeps; hard-
shadow button presses; title-tab blink-accents on critical dialogs (cursor-blink
rhythm, `#dc143c`); gold flash on selection commit; claim-redraw shimmer stays but in
gold; CRT/scanline effects stay confined to takeovers; phosphor bloom becomes SUBTLE
and gold-tinted on display numerals only.

## 10. Acceptance gates (Phase V additions)

- Lens-switch frame invariance (camera/geometry byte-identical pre/post).
- Ramp-domain stability across ticks (no silent rescale).
- Render-count stability for MapStage under chrome toggles (existing pattern).
- Border re-dissolve produces clean rings (vitest fixture: county set → ring count).
- Trunk test + neighbor test on the five core chrome surfaces.
- Colorblind simulator pass on every lens.
- Content-to-chrome ≥80% on TopBar/ActionDock/InspectionCard.
- Every wire headline names actor + action + tick (copy lint).

## 11. The weather grammar (Wave 3, adopted 2026-07-14 — binding)

Source: `ai/_inbox/lawvere-visualization.md` + `ai/_inbox/weather-visualization.md`, verified and
scoped in `reports/wave3-weather-implementation-map.md`. Four laws; the rendering rules ARE the
physics. They govern every new visualization from Wave 3 on and bind all existing lens work
retroactively where cheap.

1. **Extensive renders as stuff, intensive renders as color.** Labor-hours, value masses,
   population, Φ flows → particles, flux width, stacked volume. Rates (r, s/v, OCC, heat,
   contradiction intensity) → hex fill and edge hue ONLY. Payoff: zoom-merge visibly SUMS stuff
   (ink conserved) while color fields re-render — conservation becomes perceptible; you cannot
   missum an intensive because color doesn't add.
2. **Quantities flow, qualities cut.** Continuous animation is reserved for field evolution and
   value flows. Every enum flip — regime change, edge-mode transition, phase state — is a hard
   cut, one frame to the next, never tweened. Qualitative change is discontinuous; the screen
   teaches it without a word (Anti-Pattern VIII.1 at the presentation layer).
3. **One motion budget per tick.** Only the principal contradiction pulses — the engine already
   names it (`principal_field`, max |df/dt|, System 20). Everything else is static between
   ticks. Tufte-compliant; makes the principal contradiction do player-facing work.
4. **Curvature is terrain, not decoration.** Geometry deforms only when someone ACTS (topology
   change), never ambiently. HELD until κ actually exists in live runs — Ollivier-Ricci is
   currently dormant math with zero production callers; do not fake terrain from anything else.

Corollaries: storm/rupture markers appear as hard cuts (law 2) with intensity = the engine's own
Mao score; the vector lens (gradient wind) is law 1's first extensive citizen; reduced-motion
users get law 3's static frame with the principal contradiction marked, not animated.
