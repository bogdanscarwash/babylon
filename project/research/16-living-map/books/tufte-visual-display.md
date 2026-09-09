# The Visual Display of Quantitative Information — Edward R. Tufte (2nd ed., 2001)

Research pass for Program 16 ("Living Map"). Read strategically: front matter + TOC, then
Part I (Graphical Practice, ch. 1–3 partial) and the full Part II (Theory of Data Graphics,
ch. 4–9), which is where the reusable design law lives. Citations are `p.NNN` to the printed
page numbers (title page through p.191; PDF page = printed page − 5 in this scan).

Babylon surfaces referenced throughout: the **map** (base layer + lens overlays), the
**inspection stack** (Victoria-3-style nested panels, "every number explains itself"), the
**top bar** (global chrome: date, speed controls, resource summary), **event toasts** (the
transient notification queue), the **action dock** (verb bar: mobilize/educate/campaign/
attack/aid/investigate/move/negotiate/reproduce), and the **wire feed** (scrolling news
narration).

---

## 1. The data-ink ratio is the single most actionable law in the book

**What the book says.** "A large share of ink on a graphic should present data-information...
Data-ink is the non-erasable core of a graphic, the non-redundant ink arranged in response to
variation in the numbers represented." Tufte defines it precisely:

> data-ink ratio = data-ink / total ink used to print the graphic
> = 1.0 − proportion of a graphic that can be erased without loss of data-information

He then walks a real scatterplot from a ~0.1 ratio (grid overwhelms the 76 data points) up to
~0.9 by successively erasing non-data-ink: first the grid ticks, then restoring only reference
curves, then re-labeling axes to read left-to-right (ch. 4, "Data-Ink," p.93–105, esp. the five
redraws of the atomic-volume periodicity chart, p.101–105). The two governing principles: "Erase
non-data-ink, within reason" and "Erase redundant data-ink, within reason" (p.96, p.100).

**Application to Babylon.** The cockpit's stat tiles, inspection panels, and the top bar are
exactly the kind of "workaday, routine design" this chapter targets. Concretely:
- **Inspection stack panels**: every card currently drawn with a full rectangular border, a
  filled background chip, AND a redundant icon repeating what the label already says is
  over-inked by Tufte's count (three ways to say one number, cf. p.96's "35.9" bar shown six
  redundant ways). Cut to at most two encodings per datum (position + one of {color, label}).
- **Top bar resource readouts**: if a number is shown as a filled meter bar *and* a percentage
  label *and* a color-coded icon, that's triple redundancy — keep the meter (data-ink) and the
  numeral (precision), drop the icon unless it disambiguates a *different* variable.
- Ratchet this as a build gate, not just a guideline: **any new inspection-panel or HUD
  component must be able to state its data-ink ratio informally** ("what could I erase here
  without losing a number?") before merge.

## 2. Chartjunk has three named villains — the map/lens system must avoid all three

**What the book says** (ch. 5, "Chartjunk: Vibrations, Grids, and Ducks," p.107–121). Tufte
catalogs three widespread failure modes:

1. **Unintentional optical art / moiré vibration** — cross-hatching, tight parallel-line fills,
   and closely-spaced textures that "interact with the physiological tremor of the eye to
   produce the distracting appearance of vibration" (p.107). He surveyed 8 major journals and
   found moiré vibration in 2–21% of published graphics, worse in computer-generated ones
   (p.110–112). His fix: replace cross-hatch/moiré fills with **flat gray tint screens**, and
   label regions with words instead of encoding them with a hatch-pattern legend (p.111).
2. **The dreaded grid** — "the grid should usually be muted or completely suppressed so that
   its presence is only implicit — lest it compete with the data... Dark grid lines are
   chartjunk" (p.112). His redesign sequence for a train schedule (Marey chart, p.115–116) goes
   dark-grid → thinned-grid → **gray grid**, each step raising legibility. Rule of thumb: "If
   the paper is heavily gridded on both sides, throw it out" (p.116).
3. **The duck** — borrowing Venturi's architecture term: "when a graphic is taken over by
   decorative forms... when the overall design purveys Graphical Style rather than quantitative
   information" (p.116). Fake 3-D perspective on bars/pyramids is the modern epidemic case
   (p.118–120): it adds a phantom variable (depth) that encodes nothing, and it actively
   obscures data (pyramids conceal each other, back planes optically flip via the Necker
   illusion — p.109).

**Application to Babylon.**
- **Map hex/tile fills**: the "Cold Collapse" palette must use flat, muted tint values for
  density/intensity choropleth lenses (population, unrest, control) — never diagonal hatching
  or repeating micro-patterns to distinguish categories on the base map; that is the textbook
  moiré trap at map scale (dense hex grids at deep zoom are exactly the "closely spaced
  parallel lines" geometry Tufte warns about).
- **County/state border grid**: borders are structurally necessary data-ink (they *are* the
  data — territorial claims), but the underlying **graticule/hex grid should never be drawn at
  full black weight** once real cartography is visible; it should default to implicit/off, and
  only fade in as a thin gray reference at deep zoom when the player needs to count hexes.
- **No 3-D bars, no fake bevels/gradients on inspection-panel stat bars.** Flat is correct;
  drop-shadow-as-depth on a meter bar is a duck.
- **Hatch/pattern fills are banned for map lenses** — the ramp lens legend must be color (or
  gray-value) only, never texture, or it becomes what Tufte calls a "puzzle graphic," discussed
  next.

## 3. Graphical integrity: the Lie Factor and "show data variation, not design variation"

**What the book says** (ch. 2, "Graphical Integrity," p.53–70). Tufte's central metric:

> Lie Factor = (size of effect shown in graphic) / (size of effect in the data)

Factors outside 0.95–1.05 indicate "substantial distortion... Lie Factors of two to five are not
uncommon" (p.57). Two extended case studies matter for a *time-varying map*:
- **The disappearing/shifting baseline and irregular scale intervals** (p.60–61): the Nobel
  Prize chart's scale silently switches from 10-year to 4-year buckets at the right edge,
  producing an optical "decline" that is a pure artifact of *design variation* masquerading as
  *data variation*. Principle: **"Show data variation, not design variation"** (p.61).
- **The multi-scale OPEC oil-price chart** (p.61–63): five different vertical scales and two
  different horizontal scales are used across one image, so identical dollar amounts occupy
  areas differing by 15×. Lie Factors of 9.4 and 9.5 appear in contemporaneous news graphics of
  the same data (p.62). His fix in every case: **hold the coordinate system fixed and plot in
  constant (inflation-adjusted) units** so the eye's built-in expectation — "a scale moving in
  regular intervals is expected to continue... to the very end" (p.60) — is never violated.
- Two integrity principles fall out (p.56): (1) "the representation of numbers, as physically
  measured on the surface of the graphic itself, should be directly proportional to the
  numerical quantities represented," and (2) "clear, detailed, and thorough labeling should be
  used to defeat graphical distortion and ambiguity."

**Application to Babylon.**
- **Map lenses that redraw borders as the game progresses (liberation/collapse) must never
  silently change projection, scale, or color-ramp domain between ticks.** If the "unrest"
  ramp is [0,100] on tick 40, it cannot rescale to [0,60] on tick 41 just because max observed
  unrest dropped — that is the exact Nobel-Prize-chart trick (apparent change from design
  variation, not data variation). Ramp domains should be fixed per campaign/session, or any
  rescale event must be visually flagged (a one-tick "scale changed" toast), never silent.
  **Directive: color-ramp domain changes emit an explicit UI event; they are never inferred
  silently from the current frame's data range.**
  Note (2026-07-11): grep of `src/frontend` shows no ramp/rescale logic yet — this is a
  forward-looking constraint on the lens system to build, not a regression to fix.
- **Time controls (pause/speed)**: elapsed-time labeling in the top bar (e.g. "Tick 52 / Year
  3") must use one consistent unit throughout a session — don't switch between "ticks" and
  "years" as the primary axis label depending on zoom, the way the OPEC chart switched between
  "Yearly" and "Quarterly" buckets (p.61) and fooled readers into seeing acceleration that
  wasn't there.
- **Any comparison of economic quantities across ticks (imperial rent, wages, GDP-analog) that
  are subject to in-game inflation/devaluation mechanics must be displayed in constant
  (deflated) units by default**, mirroring Tufte's principle for time-series of money (p.68):
  "In time-series displays of money, deflated and standardized units of monetary measurement
  are nearly always better than nominal units." A raw nominal-currency sparkline in the
  inspection stack is a Lie-Factor risk if the sim has inflation.

## 4. Multifunctioning graphical elements — data-based grids and labels (sparkline ancestor)

**What the book says** (ch. 7, "Multifunctioning Graphical Elements," p.139–159). Core
principle: "Mobilize every graphical element, perhaps several times over, to show the data"
(p.139). Concrete devices:
- **Data-built data measures**: the stem-and-leaf plot builds the *shape* of a histogram out of
  the digits of the data themselves (p.140) — the ink that shows magnitude is literally made of
  numerals, not a separate bar.
- **Ayres' triple-functioning data measure** (p.141, p.155): a cumulative time-series of WWI
  troop divisions built entirely from typed division numbers, arranged so the same ink
  simultaneously shows (1) count per month, (2) which specific division, (3) duration in
  theatre. This is Babylon's most directly reusable precedent: **a compact, text-dense,
  self-labeling time-series glyph** — functionally a proto-sparkline crossed with a Gantt chart,
  built from the labels a wire-feed entry would need anyway.
- **Data-based grids and labels** (p.145–152): rather than a generic axis with round-number
  ticks (0, 10, 20, 30, 40), replace the ticks with the *actual* min/max/marginal values
  realized in the data (p.149, the "range-frame with range-labels"). "Why not use the ink to
  show data?" A worked example (p.152) shows a scatterplot's entire frame replaced by the exact
  numeric values of each point positioned in the margin — no ticks, no legend, no scale
  decoration, just the numbers doing double duty as both label and axis.
- **Puzzles and hierarchy** (p.153–154): the warning side of multifunctioning — a 16-color
  choropleth of "male cardiovascular mortality × household crowding" is legible only by running
  "little phrases through their minds" to decode a color legend; "a sure sign of a puzzle is
  that the graphic must be interpreted through a verbal rather than a visual process" (p.153).
  Gray-scale ramps avoid this because they have a natural, learned visual ordering that color
  usually lacks (p.154).
- **Viewing architecture** (p.154–159): design graphics to have "at least three viewing
  depths" — (1) overall structure seen from a distance, (2) fine structure seen up close, (3)
  what's implicit/behind the graphic — and multiple *stable* viewing angles (always horizontal
  or always vertical per variable), so a complex multivariate display stays legible instead of
  becoming "a puzzle" even when dense (Ayres' chart again, p.155, uses three fixed sightlines:
  horizon-profile for the time-series, vertical for composition, horizontal for duration).

**Application to Babylon.**
- **Inspection-stack sparklines / mini-timeseries**: follow the range-frame-with-range-labels
  pattern (p.149) — a tension/solidarity/wage sparkline in a nested panel should label its
  actual min and max endpoints in-line (not round numbers, not a separate legend), so "every
  number explains itself" without a hover tooltip. This is a direct, literal fulfillment of the
  Victoria-3 promise using 60-year-old graphical law.
- **Wire feed entries**: consider an Ayres-style compact multi-functioning glyph for org/faction
  activity history — e.g., a org's action log rendered as stacked tick-labels (verb abbreviation
  doubling as both the label and the vertical position) rather than a generic bar chart plus
  a separate legend, when space is at a premium in a feed card.
- **Choropleth map lenses must ship a gray-value fallback or ordering-safe ramp**, not an
  arbitrary hue wheel, specifically to avoid the "puzzle graphic" failure (p.153) — a player
  should be able to *feel* "more red = more unrest" the way they feel "darker gray = denser"
  without consulting the legend twice.
- **Directive for the map + inspection stack jointly**: define fixed viewing axes per data
  type before drawing — e.g., population density always reads left→right as time or
  west→east as geography, never remapped per-lens — so switching lenses doesn't force the
  player to re-learn which direction means "more."

## 5. Data density and small multiples — the eye tolerates far more than designers assume

**What the book says** (ch. 8, "Data Density and Small Multiples," p.161–175). Tufte's
empirical measure:

> data density of a graphic = (number of entries in data matrix) / (area of data graphic)

He surveys real published graphics and finds enormous variance: a bloated 5-color chart at
**0.15 numbers/sq-inch** (p.162) versus Bertin's map of French communes at **9,000 numbers/sq
inch** (p.166) versus his own galaxy map, the "current record" at **110,000 numbers/sq inch**
(p.166). A cross-publication survey (p.167) shows median data density ranging from 0.2
(*Pravda*) to 48 (*Nature*) — "the average published graphic is rather thin... about 50 numbers
shown at the rate of 10 per square inch" (p.168). His prescription: **"Maximize data density and
the size of the data matrix, within reason"** (p.168) — "graphics can be shrunk way down... many
data graphics can be reduced in area to half their currently published size with virtually no
loss in legibility." This directly motivates **small multiples**: "a series of graphics, showing
the same combination of variables, indexed by changes in another variable... the design remains
constant through all the frames, so that attention is devoted entirely to shifts in the data"
(p.170). His checklist for a well-designed small-multiple (p.175): inevitably comparative,
deftly multivariate, shrunken/high-density, based on a large data matrix, drawn almost entirely
with data-ink, efficient to interpret, and "often narrative in content, showing shifts in the
relationship between variables as the index variable changes." Closing aphorism: **"For
non-data-ink, less is more. For data-ink, less is a bore."** (p.175)

**Application to Babylon.**
- **The inspection stack's nested recursive panels are structurally a small-multiples
  opportunity**: when a player drills from country → region → territory → hex, each level
  should render with an *identical* mini-layout (same stat-tile grid, same sparkline position)
  so the only thing that changes is the data — exactly Tufte's "design remains constant... so
  attention is devoted entirely to shifts in the data" (p.170). Don't let deeper levels
  introduce new chart types; reuse the shell.
  Note: current React 19 + deck.gl cockpit does not yet have this nested-panel structure — this
  is a forward design constraint for Program 16, not a fix to existing code.
- **Time-scrubbing / history playback**: a filmstrip of past-N-ticks map snapshots (border
  redraws, unrest heat) is a direct small-multiples application (cf. the 23-hour LA smog
  small-multiple, p.170) and would let the player *see* the revolution/collapse trajectory
  rather than only reading a single current frame.
- **Push back on low-density defaults**: a single stat tile showing one number in a large card
  is close to the 0.15-numbers/sq-inch failure case (p.162). Where the player is data-hungry
  (a fully-open inspection panel, not a glance-tile), the panel should be willing to be dense —
  a compact numeric table or micro-grid rather than one giant number in whitespace — because
  Tufte's evidence is that **viewers tolerate far more density than designers assume**, and
  low-density design reads as evasive ("what is left out, what is hidden, why are we shown so
  little?", p.168).
- **Directive**: any new HUD/panel component's data-density should be sanity-checked against
  its purpose — glance tiles (top bar, event toast) intentionally stay low-density for legibility
  at a glance; opened inspection panels should default toward the *Nature*/JASA end of the
  density spectrum (dense tables, sparklines, small multiples), not the *Pravda* end.

## 6. Tables beat bar charts for small, precise, or heavily-labeled data — and pie charts are banned

**What the book says** (ch. 9, "Aesthetics and Technique," p.177–183, section "The Choice of
Design"). "Tables are clearly the best way to show exact numerical values... Tables are
preferable to graphics for many small data sets" — Tufte's rule of thumb elsewhere in the book
is **20 numbers or fewer favors a table over a chart** (echoed at p.56). On pie charts: "a table
is nearly always better than a dumb pie chart; the only worse design than a pie chart is several
of them... given their low data-density and failure to order numbers along a visual dimension,
pie charts should never be used" (p.178, citing Bertin calling multi-pie displays "completely
useless"). He also introduces the **supertable** — a large, topically-grouped table (his own
410-number NYT election table, p.179) that "is likely to attract and intrigue readers through
its organized, sequential detail and reference-like quality. One supertable is far better than a
hundred little bar charts." Finally, **data/text integration**: "Data graphics are paragraphs
about data and should be treated as such" (p.181) — captions, labels, and small explanatory
phrases belong ON the graphic, not segregated into a separate legend block requiring the eye to
dart back and forth (his worked contrast table of "friendly" vs. "unfriendly" graphics, p.183,
is a ready-made component checklist — see below).

**Application to Babylon.**
- **No pie charts anywhere in the cockpit** — faction/class composition breakdowns (e.g. share
  of population by class, resource allocation by sector) should be **ordered horizontal bars
  or a compact table**, never a pie or (worse) a small-multiple of pies.
- **Inspection panels with ≤20 discrete stats** (e.g. a territory's full stat block: population,
  wages, control, garrison, etc.) should default to a **dense labeled table layout**, not a grid
  of separately-chromed stat tiles — this is cheaper to build, scans faster, and matches
  Tufte's evidence directly.
- **A faction/organization "dossier" view is a supertable candidate**: one well-organized,
  topically-grouped table (grouped by economic / military / political stats, in a fixed reading
  order) beats a dozen small charts scattered across tabs.
- **Friendly-graphic checklist (p.183) — adopt verbatim as a component review checklist:**
  words spelled out (no cryptic abbreviations); labels run left-to-right, never rotated onto a
  vertical axis; inline "little messages" explain data instead of a legend; no cross-hatch/color
  code that needs decoding — label directly on the shape instead; type is upper-and-lower case
  with serifs or a humanist sans, never all-caps; ramps and lenses must remain legible to the
  color-deficient (5–10% of players) — blue-based contrasts are safest, red/green as the *only*
  distinguishing pair is explicitly flagged as unfriendly (p.183).

## 7. Graphical elegance = simplicity of design × complexity of data (not simplicity of data)

**What the book says** (ch. 9 opening, p.177). Framed against Minard's Napoleon map and the
galaxy map: "Graphical elegance is often found in simplicity of design and complexity of data...
Visually attractive graphics also gather power from content and interpretations beyond the
immediate display of some numbers. The best graphics are about the useful and important, about
life and death, about the universe. Beautiful graphics do not traffic with the trivial." His
practical checklist for attractive routine (non-Minard) design (p.177): a properly chosen format
and design; words, numbers, and drawing used together; balance/proportion/relevant scale;
accessible complexity of detail; a narrative quality — "a story to tell about the data"; drawn in
a professional manner; content-free decoration (chartjunk) avoided.

**Application to Babylon.** This is the strongest available justification, from outside game
design entirely, for the stated goal of ditching "corporate dashboard" polish for something that
reads as consequential. A revolution/collapse map with real county-level cartography carries
exactly the kind of "life and death" data weight Tufte says elegance requires — the fix is not
more chrome, it's **exposing the underlying complexity of the simulation's actual data (real
borders, real class composition, real unrest gradients) through simpler, less decorated
graphical form**, not dumbing the data down to fit decorative UI chrome. Concretely: resist any
temptation to simplify the map's real geography/complexity to make room for game-UI ornament;
instead simplify the *ornament* (chrome, panel borders, redundant icons) so the actual simulation
data can carry the visual weight.

---

## Summary of testable directives

1. **Data-ink ratio.** Every new inspection-panel or HUD component ships with at most two visual
   encodings per datum (e.g., position + one of {color, label}); a reviewer must be able to name
   one element that could be erased without losing a number, and if they can't, the panel passes.
2. **No hatch/texture fills.** Map lenses and inspection-panel stat bars use flat color or
   gray-value tint only; cross-hatching, diagonal stripe fills, and fake-3D bevels/drop-shadows-
   as-depth are rejected in review.
3. **Fixed lens domains.** A color-ramp's numeric domain (min/max) is fixed for the session/lens
   and never silently rescales between ticks; any domain change fires an explicit UI event
   (toast or legend flash), never inferred silently from the current frame's data range.
4. **Constant units for money/time.** Any in-game economic time-series subject to inflation
   displays in deflated/constant units by default; any elapsed-time axis (top bar, sparklines)
   keeps one fixed unit (ticks or years, not both) across a single view.
5. **Range-labeled sparklines.** Inspection-stack mini-timeseries label their actual realized
   min/max inline on the trace itself, not via a separate legend or round-number axis ticks.
6. **No pie charts.** Any composition/breakdown display (class share, resource allocation, vote
   share) renders as an ordered bar list or table; pie charts and multi-pie small-multiples are
   rejected in review, full stop.
7. **Consistent nested-panel shell.** Every depth level of the inspection stack (country → region
   → territory → hex) reuses one fixed layout shell; only the data changes between levels, never
   the chart type or arrangement.
8. **Friendly-graphic checklist enforced.** New components are checked against: spelled-out
   labels (no unexplained abbreviations), left-to-right text only (no rotated/vertical axis
   labels), inline explanatory micro-text instead of a separate legend where feasible, and
   color-deficiency-safe contrast (never red/green as the sole distinguishing pair).

---

*Source: Edward R. Tufte, The Visual Display of Quantitative Information, 2nd ed. (Graphics
Press, Cheshire, CT, 2001). PDF at
`/home/user/Downloads/babylon_books/ux/ux_edward-tuft_visual-display-of-quantitative-information.pdf`,
191 printed pages. Chapters read in full: 1 (partial, p.13–17, 51), 2 Graphical Integrity
(p.53–70), 4 Data-Ink and Graphical Redesign (p.91–105), 5 Chartjunk (p.107–121), 6 Data-Ink
Maximization and Graphical Design (p.123–137), 7 Multifunctioning Graphical Elements (p.139–159),
8 Data Density and Small Multiples (p.161–175), 9 Aesthetics and Technique (p.177–183, opening +
Choice of Design sections). Chapter 3 (Sources of Graphical Integrity) and the latter half of
chapter 9 (Friendly Data Graphic detail, p.183–190) and the Epilogue (p.191) were not read in
this pass; if a future task needs sophistication-vs.-cynicism editorial guidance or the
epilogue's design gallery, resume there.*
