# Visual Explanations — Edward Tufte (Graphics Press, 1997)

Research pass for Babylon's "living map" game-chrome redesign. Read strategically: full
front matter + TOC, then Ch.1 Images and Quantities (pp.13–25), Ch.2 Visual and
Statistical Thinking (pp.27–51, cholera + Challenger in depth), Ch.4 The Smallest
Effective Difference (pp.73–77), Ch.5 Parallelism (pp.79–103), Ch.6 Multiples in Space
and Time (pp.105–121), Ch.7 Visual Confections (pp.121–151). Chapter 3 (Explaining
Magic — disinformation design, pp.55–72) skimmed for its closing checklist only. Total
pages actually read: ~148 of 168.

Book's own three-part identity, stated in the intro (p.10): *The Visual Display of
Quantitative Information* = pictures of numbers; *Envisioning Information* = pictures of
nouns; *Visual Explanations* = **pictures of verbs** — mechanism, motion, cause, effect,
narrative. That is exactly Babylon's problem: the cockpit currently shows nouns (a map,
some stat tiles) when the game is fundamentally about verbs (mobilize, exploit, collapse,
liberate) unfolding causally over time.

---

## Core Lessons

### 1. The central question is always "compared with what?"

**What the book says.** John Snow's cholera investigation (ch.2, pp.27–37) succeeds
because at every step he asks *compared with what*: cholera deaths compared with the
brewery workers who drank no water at all (p.30, "saved by the beer!"), compared with the
workhouse's own well (p.30), compared with population density, not just raw death counts
(p.35, "an area of the map may be free of cases merely because it is not populated").
Tufte's verdict on statistical reasoning generally (p.28, citing Dahl): "Policy-thinking is
and must be causality-thinking." A display that shows only the affected group, without the
unaffected group as a baseline, cannot support a causal claim.

**Application to Babylon.** Every stat surfaced by the inspection stack (a class's Anger,
a territory's Control Ratio, a faction's wealth) must default to showing its **reference
population or baseline**, not the raw number in isolation. When the player opens the
`SocialClassInspector` for a hex, don't just show "Anger: 0.62" — show it against the
distribution of Anger in that class across the map, or against the historical trend, or
against the class that shares the hex. This is the single most testable, highest-leverage
directive from the whole book: **no number-in-a-box without its comparison class shown in
the same view.**

### 2. Cause-and-effect displays require the full data matrix, not the convenient subset

**What the book says.** The Challenger chapter (pp.39–51) is the book's core case study.
Thiokol's 13 pre-launch charts showed only 7 of the 24 previous launches — the ones with
visible O-ring damage — and never plotted temperature against damage for the full flight
history (p.43: "Limited measure of effect, wrong number of cases... Missing are 92% of the
temperature data"). Tufte's redrawn scatterplot of **all 24 launches** (p.45) makes the
cold-weather risk immediately, starkly visible: "every launch below 66° resulted in
damaged O-rings." His diagnosis (p.44): "Numbers become evidence by being in relation
to" — i.e., a metric is meaningless without the full population it's drawn from,
including the *undamaged*, *boring*, *nothing-happened* cases. He states the general
principle explicitly (p.49): **"the little rockets must be placed in order by temperature,
the possible cause"** — ordered by the causal variable under investigation, not by
chronology (the sequence NASA's own charts used, which buried the signal).

**Application to Babylon.** This is the theoretical justification for the "every number
explains itself" provenance-chain requirement. When a player drills into why a
contradiction fired or why a rupture event triggered, the explanation panel must show the
**full comparison set the engine actually used** — e.g., if Survival Calculus fired because
`P(S|R) > P(S|A)`, show both terms, their defines-file coefficients, and where this
territory/class sits relative to the full population of territories/classes that *didn't*
rupture this tick, ordered by the causal variable (Organization/Repression ratio), not by
tick order or alphabetically. A "why did this happen" popup that shows only the winning
formula's final number, with no comparison population, replicates the 13-chart Challenger
failure. Contradiction-field and OODA decision explanations are the highest-stakes
candidates for this treatment — they are Babylon's Challenger-launch decisions.

### 3. Order data by the causal variable, not by an incidental axis (chronology, alphabet, id)

**What the book says.** Directly downstream of #2: the "wrong order" failure (p.48) — NASA's
48-rocket chart sequenced by launch date, which hid the temperature-damage link entirely.
"Sequential order... throw[s] statistical thinking into disarray." Reordering by
temperature (p.49) makes the pattern self-evident with no further annotation needed.

**Application to Babylon.** Any Babylon UI that lists entities in a fixed order (a faction
roster, a territory list, a wire feed) should offer — and default to, when the player is
investigating a specific mechanic — sorting by the variable currently under suspicion.
E.g., a "why is imperial rent spiking" investigation should let the player re-sort
territories by Φ (imperial rent) or by Wc−Vc gap, not force them to scan a fixed
alphabetical/geographic list. The wire feed itself, when filtered to a topic, should offer
"order by magnitude of change" as an alternative to strict reverse-chronological.

### 4. Chartjunk and disproportionate decoration signal — and cause — sloppy thinking

**What the book says.** Ch.2's dissection of NASA's "cute little rockets" chart (pp.46–49):
"Chartjunk indicates statistical stupidity, just as weak writing often reflects weak
thought" (quoting Ben Jonson, p.48: "Neither can his mind be thought to be in tune, whose
words do jarre... nor his reason in frame, whose sentence is preposterous.") Ch.4 opens with
the "Wound Man" ear diagram (p.73) where pointer-lines are "heavier than the linework for
the ear itself" — the strategy is stated as a formal rule: **"Make all visual distinctions
as subtle as possible, but still clear and effective"** — *just noticeable differences*, not
maximal ones. The bathymetric ocean chart (pp.76–77) is the positive exemplar: 21 subtle
blue color gradations leave enough visual "room" for a second, independent data layer
(gray survey-ship tracks) to sit on top without clutter — "minimal distinctions... increase
the number of distinctions that can be made within a single image."

**Application to Babylon.** The map's ramp/heatmap lenses (control, solidarity, rent) should
follow the bathymetric-chart model: use the *minimum* color contrast that remains legible,
specifically so state-border overlays, hex grid lines, unit icons, and event-pin badges can
coexist on top of the ramp without visual competition. Any secondary chrome element
(tooltips' pointer-lines, inspector-panel dividers, legend tick marks) should be rendered at
lower visual weight than the primary data it annotates — concretely: hairline (~0.5–1px,
low-alpha) dividers and connector lines in the inspection stack, never heavier strokes than
the data glyphs they're pointing at.

### 5. The smallest effective difference is an algorithm for layered legibility

**What the book says.** Direct continuation of #4 as a named, general design strategy
(ch.4, p.73–77): distinctions should be "definite, effective, and minimal" — strong enough
to read at a glance, no stronger. Its payoff is explicitly **more layers of information in
the same space**: the ocean chart's muted bathymetric tints "leave sufficient visual space"
for the ship-track layer; by contrast the rainbow-coded version of the same data (p.77) is
"laughed right out of the field... incoherent, with some of the original data now lost in
the soup." Tufte calls this "perhaps even an algorithm for automated design" (p.77).

**Application to Babylon.** This is the design law for the map-lens system directly: each
active lens (control-ratio ramp, solidarity heat, rent gradient) must use the *least*
saturated palette that stays legible at a glance, specifically *because* Babylon plans to
stack county borders, hex grid, unit tokens, and event pins on top simultaneously. A rainbow
or high-saturation lens (Tufte's explicit anti-example, p.77) will make every other UI layer
fight for attention and should be treated as a defect, not a stylistic choice — this gives
teeth to the existing "Cold Collapse" muted-palette ratification.

### 6. Parallelism: put comparable things in position-adjacent, structurally identical frames

**What the book says.** Ch.5 (pp.79–103): "Parallelism connects visual elements. Connections
are built among images by position, orientation, overlap, synchronization, and similarities
in content... Congruity of structure across multiple images gives the eye a context for
assessing data variation" (p.82). Two forms: parallel-in-space (Degas horse photo + x-ray,
side by side, p.79) and parallel-in-time (Repton's before/after flaps, pp.80–81) — "such
flips avoid the disorienting back-and-forth movements of the eye needed to compare adjacent
but separate images... comparisons are usually more effective when the information is
adjacent in space rather than stacked in time" (p.81). The chapter's cautionary case
(pp.102–103, HIV/AIDS mortality charts) shows **faulty parallelism**: two graphs meant to be
compared side by side use *different vertical scales* for men vs. women, so "equal vertical
distances represent different quantities" — the comparison the display exists to enable is
actively sabotaged by inconsistent axes.

**Application to Babylon.** (a) Before/after: when a territory flips from colonial to
liberated cartography, or borders redraw after a revolution succeeds, show a brief
spatially-adjacent (not merely sequential) parallel — the map lens's "history scrub" should
support a split or ghost-overlay view of before/after border states, not just a scrubber
that replaces one with the other. (b) Any place Babylon shows two comparable time-series
side by side (e.g., two territories' Anger over the last 20 ticks in a comparison panel)
must use **identical y-axis scales and identical color encodings** — this is a direct,
testable rule violated by the HIV/AIDS example and must not be violated by, say, a
faction-vs-faction wealth comparison. (c) Nested inspection panels for structurally similar
entities (two social classes, two organizations) should render in visually identical
templates — same field order, same units, same chart type — so the *position* of a number
carries meaning ("wages are always top-left") the way Gibbon's "by whom / to whom" parallel
clauses carry meaning through grammatical position alone (p.79).

### 7. Multiples: many small, structurally identical images beat one big ambiguous one

**What the book says.** Ch.6 (pp.105–121): "Multiples directly depict comparisons, the
essence of statistical thinking... Multiples enhance the dimensionality of the flatlands of
paper and computer screen" (p.105). Huygens's 32-Saturn diagram (pp.106–108) and the
letterform "A" grid (pp.112–113) show the pattern: hold everything constant except the one
variable under study, then array many small instances so the eye does the comparison
automatically. Explicit warning on **false clustering** (p.112): "Accidental communalities in
design can easily induce false groupings in the eyes of viewers... false clusterings can
result from inexpert use of color... viewers mistake the decorative tints for real
information." A second warning: **multiples depicting motion must carry an explicit
time-scale** (p.109) — Muybridge's leapfrog sequence and the continental-drift globes both
lack one, which is "dequantification all over again," the same sin as the ungridded
supercomputer storm animation from ch.1.

**Application to Babylon.** (a) Small-multiples are the right pattern for a "compare all 50
states/territories at once" overview mode — one tiny sparkline or micro-map per territory,
constant scale, arrayed in a grid, rather than forcing the player to click through 50
separate inspector panels. (b) Any icon-color coding in the cockpit (faction badges, unit
type glyphs) must not accidentally create false visual clusters unrelated to the data — e.g.
if two unrelated organization types happen to share a badge hue, group them apart or recolor;
don't let palette collisions imply kinship the sim doesn't model. (c) Any animated/scrubbing
sequence in the map (tick-by-tick playback, border-redraw animation) must carry a visible,
persistent time-scale/tick-counter on screen throughout — never let a smooth animation run
without a readable "tick N of M" or date readout, exactly the gap Tufte flags in Muybridge
and the continental-drift globes.

### 8. Confections: compartments and imagined scenes as narrative-explanatory grammar — and their two chronic failure modes

**What the book says.** Ch.7 (pp.121–151) formalizes two ways of assembling many small
image-events into a single readable "confection": **compartments** (call-out circles,
grids, boxed panels — e.g. the Ultimate Weed diagram, p.126, or Hobbes's *Leviathan*
title page, pp.136–138) and **imagined scenes** (all elements composited into one coherent
pictorial space — Pugin's skyline of 25 churches, p.124; Babar's dream battle of virtues
and vices, p.127). Two explicit failure modes: (i) **one-time, ungeneralizable navigation
schemes** — the 17th-century legal-mnemonic engraving (p.125) requires "a complex
alphanumeric code (you don't want to know)" just to find your way around; Tufte's verdict:
"Too often in such guides, keys, and codes, the apparatus itself becomes an impediment to
understanding... Ideally, structures that organize information should be transparent,
straightforward, obvious, natural, ordinary, conventional — with no need for hesitation or
questioning on the part of the reader." (ii) **content-to-chrome ratio collapse** — the
museum-kiosk case study (pp.146–150) is Tufte's most directly UI-relevant material in the
book: his own well-designed kiosk devotes "about 90% of the image" to substantive content
("the *information becomes the interface*"), while a comparison "yearbook" interface wastes
82% of the screen on ornamental icons/frames and only 18% on content — Tufte proposes this
as a **literal, measurable design metric**: "the proportion of space on the screen devoted
to content, to computer administration, and to nothing at all" (p.150), and explicitly
names the failure pattern **"television-disease: thin substance, contempt for the audience
and the content, short attention span, and over-produced styling"** (p.148), plus the
"choose one from the list below" binary-decision-tree anti-pattern for burying an overview
inside sequential menu clicks (p.148: "context and overview are lost").

**Application to Babylon.** (a) The top bar, action dock, and any modal/kiosk-style overlay
(faction briefing, tutorial popover, endgame report) should be audited with Tufte's literal
metric: **measure the percentage of pixels devoted to actual game data vs. chrome
(borders, icon frames, drop-shadows, decorative bevels) and target ≥80% content**, matching
his own kiosk rather than the 18%-content "yearbook" anti-example. (b) Reject any
navigation scheme (verb wheel, inspection-stack breadcrumbs, lens picker) that requires the
player to memorize an arbitrary non-obvious code to operate — prefer direct labels over
color/icon codes that need a legend to decode, especially for first-time players. (c) Avoid
sequential "wizard" flows (Tufte's "choose one from the list below" example) for anything
that is really a menu of parallel choices — the verb dock (mobilize/educate/campaign/...)
should present all 9 verbs flat and simultaneously (as his flat kiosk interface does with 45
options at once, p.146) rather than nesting them behind category submenus that hide the
overview. (d) Event popups/wire-feed entries that narrate a causal chain (why a faction
attacked, why solidarity spiked) are legitimate confections — use the compartment pattern
(a small annotated diagram with 2–4 labeled call-outs pointing at the actual map/graph
region involved) rather than a wall of narrative prose alone; ch.7's Potomac River
"why is this dangerous" news graphic (pp.144–145) is the closest direct analog to a wire
narrative-explainer popup and shows the target density: ~9 small diagrams + captions convey
more, faster, than continuous prose.

### 9. Every claimed number needs its provenance and its author

**What the book says.** Direct textual basis for "every number explains itself." p.40: "Public,
named authorship indicates responsibility for both the immediate audience and for the
long-term record. Readers can follow up and communicate with a named source." The
CYA-disclaimer chart (pp.46–47) is flagged as a red flag in itself: "Such defensive
formalisms should provoke rambunctious skepticism... this in this case is documented in
reports, hearing transcripts, and archives" — i.e. Tufte's own trust in the evidence comes
from being able to trace it to primary sources. Ch.1's closing "eight ethical questions" for
image integrity (implicit throughout, made most explicit at ch.3's conclusion, p.70): *Is
the display revealing the truth? Is the representation accurate? Are the data carefully
documented? Do the methods of display avoid spurious readings of the data? Are appropriate
comparisons and contexts shown?*

**Application to Babylon.** Every number surfaced in the nested inspection stack should be
traceable, in-panel, to (a) which of the 26 systems/which formula in `formula_registry.py`
computed it, (b) which `GameDefines` coefficient(s) fed it, and (c) which tick it was last
updated. This is a literal, checkable spec for "every number explains itself" — a tooltip or
expand-affordance on any stat should answer Tufte's four questions inline rather than
requiring the player to trust an opaque HUD value.

---

## Anti-patterns (explicitly named failures to avoid)

- **The convenient-subset chart** — showing only cases favorable to the point already being
  argued (Thiokol's 2-of-24 launches, p.43). Applies to: any Babylon dashboard/report
  screen that cherry-picks "notable" territories/events instead of showing the full
  population with the notable ones highlighted.
- **Chronological-order-as-default** when a causal variable exists to sort by instead
  (p.48–49).
- **Dequantified animation** — motion/change shown with no visible scale, axis, or
  time-stamp (pp.20–24 storm animation; p.109 Muybridge/continental-drift).
- **Rainbow/high-saturation encoding** for continuous quantitative data (p.77) — "laughed
  right out of the field."
- **Faulty parallelism** — visually-comparable panels that secretly use different scales,
  making the comparison the display was built for actively misleading (pp.102–103).
- **False clustering from decorative color** — palette choices that accidentally group
  unrelated items (p.112).
- **The CYA disclaimer / unnamed-author chart** — evidence with no traceable source invites,
  and deserves, skepticism (pp.40, 46–47).
- **Television-disease interfaces** — low content-to-chrome ratio, over-produced styling,
  sequential wizard flows that hide overview, hierarchy-mimicking-bureaucracy navigation
  (pp.148–150).
- **One-time, non-obvious navigation codes** that require memorizing an arbitrary legend
  instead of direct labels (p.125).
- **Disproportionate secondary elements** — pointer-lines/borders/dividers heavier than the
  primary data they annotate (p.73, the Wound Man ear).

---

## Sources (page-cited within the report above; representative subset)

- Snow, cholera map and pump-handle narrative: Tufte pp.27–37, citing John Snow, *On the
  Mode of Communication of Cholera* (London, 1855).
- Challenger: Tufte pp.38–51, citing *Report of the Presidential Commission on the Space
  Shuttle Challenger Accident* (1986) and Diane Vaughan, *The Challenger Launch Decision*
  (Chicago, 1996).
- Smallest effective difference: Tufte pp.73–77, citing T.E. Cohn & D.J. Lasley on
  just-noticeable differences, and the *General Bathymetric Chart of the Oceans* (5th ed.,
  1984).
- Parallelism: Tufte pp.79–103, incl. HIV/AIDS mortality graphs from *MMWR* 45 (1996).
- Multiples: Tufte pp.105–121, citing Huygens *Systema Saturnium* (1659), Wegener *Die
  Entstehung der Kontinente und Ozeane* (1929).
- Confections: Tufte pp.121–151, incl. Powsner & Tufte, "Graphical Summary of Patient
  Status," *The Lancet* 344 (1994) for the multiples-medical-record case; Tufte's own
  National Gallery museum-kiosk design, pp.146–150.
