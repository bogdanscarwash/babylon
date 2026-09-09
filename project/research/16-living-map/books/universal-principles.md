# Universal Principles of Design — research notes for the Babylon "living map"

Source: *Universal Principles of Design* (William Lidwell, Kritina Holden, Jill Butler), EPUB at
`/home/user/Downloads/babylon_books/ux/ux_universal_principles_of_design.epub`. The book is a
single-volume encyclopedia of ~125 design principles, organized alphabetically, each as a
self-contained 1-2 page entry (definition → mechanism → guidelines → "see also" cross-refs →
examples). Read strategically: table of contents (`toc.ncx`) extracted first to select the ~18
entries most relevant to a map-first strategy-game UI (progressive disclosure, chunking,
hierarchy, wayfinding, aesthetic-usability, performance load, plus supporting Gestalt/memory
principles the target entries cross-reference). Page numbers are not present in this EPUB edition
(no fixed pagination); citations below use the book's own entry titles, which is how the book is
indexed and cross-referenced internally.

Babylon surfaces referenced throughout: **map lenses** (Paradox-style data overlays — solidarity,
imperial rent, control, contradiction fields, etc.), **inspection stack** (Victoria-3-style nested
recursive panels where every number explains itself), **top bar** (time controls, global stats),
**action dock** (the 9 verbs: mobilize, educate, campaign, attack, aid, investigate, move,
negotiate, reproduce), **wire feed** (news/event ticker), **event toasts/popups**, **county/state
cartography layer** (the redrawing borders), **hex deep-zoom tiles**.

---

## 1. Progressive Disclosure

**What the book says:** "A strategy for managing information complexity in which only necessary
or requested information is displayed at any given time." Separate information into layers and
show only what's relevant now. Concealing infrequently-used controls behind a "More" affordance
keeps the default view clean while keeping power available on request. Information given before a
person needs or wants it is *noise*, not signal — disclosed progressively, it is processed better
and perceived as more relevant. Errors and recovery time drop when this is used. Physical-world
analogue: theme parks progressively disclose the length of a queue so no one — in or out of line —
ever sees the whole thing at once. *See also Chunking, Errors, Layering, Performance Load.*

**Application to Babylon:** The **inspection stack** is the canonical progressive-disclosure
surface — clicking a county shows headline stats only (population, dominant class, control
ratio); clicking further reveals the Leontief/imperial-rent breakdown; clicking the rent figure
itself reveals the tensor math. Never default-expand more than one level. The **action dock**
should show only the verbs currently legal for the selected organization/context (context
sensitivity, reinforced below under Visibility) rather than all 9 grayed out. The **map lens**
picker should default to a single "headline" lens (e.g., Control) and progressively disclose the
fuller lens catalog (Contradiction Field, Solidarity, Metabolic Rift, etc.) behind a lens-drawer
toggle, mirroring the "More Choices" search-dialog example in the book.

---

## 2. Chunking

**What the book says:** Short-term memory holds roughly 4±1 chunks. Break lists/strings into
small groups (`704-555-6791` not `7045556791`) to make them retainable. Critically, this is a
**memory** principle, not a general simplicity principle — misapplying it to *reference* material
that is scanned rather than memorized (e.g. a dictionary) actually hurts performance by increasing
scan time for no benefit. Apply chunking when people must recall/retain/problem-solve with
information; do not apply it to searchable/browsable material. *See also Errors, Mnemonic Device,
Performance Load, Signal-to-Noise Ratio.*

**Application to Babylon:** Chunk the **top bar** global-stats readout into 4±1 grouped clusters
(e.g., Economy | Solidarity | Repression | Time) rather than a flat row of a dozen numbers — this
is a recall-relevant HUD players glance at repeatedly. Do NOT chunk the **county list / search
panel** in the inspection stack — that's scan/reference material, and forcing it into 4-item
groups would slow lookup for no gain. The **wire feed**, if players need to recall recent events
under time pressure (e.g., before pausing to react), should batch related events into a single
digest chunk ("3 uprisings in the Rust Belt") rather than 3 separate ticker lines.

---

## 3. Hierarchy

**What the book says:** "Hierarchical organization is the simplest structure for visualizing and
understanding complexity." Perceived hierarchy comes mainly from relative position (left-right,
top-down), reinforced by proximity, size, and connecting lines. Three representational forms:
**trees** (parent above/left of child — good for moderate-complexity overviews, gets tangled with
shared children), **nests** (Venn-style containment — good for simple groupings, breaks down when
dense), **stairs** (indented outline — good for complex/volatile hierarchies, but implies false
sequence and needs progressive reveal to stay browsable). "Explore ways to selectively reveal and
conceal the complexity of hierarchical structures to maximize their clarity and effectiveness."
*See also Advance Organizer, Alignment, Five Hat Racks, Layering, Proximity.*

**Application to Babylon:** The **inspection stack** is structurally a *stair* hierarchy
(hex→county→state→national, or class→faction→organization) — conceal child levels until the
parent row is expanded, exactly as the book prescribes for stair structures, to avoid implying a
false linear sequence between sibling entities. The **cartography layer** itself (county borders
nested inside state borders nested inside the national outline) is a *nest* structure and should
use nest-style visual containment (border weight/opacity increasing with administrative level) so
players read state boundaries as "containing" county boundaries at a glance. Reserve *tree*
diagrams for one-off explainer views (e.g., an organization's chain-of-command popup), not for
persistent chrome.

---

## 4. Layering

**What the book says:** Organize information into related groupings, present only some groupings
at a time. Two-dimensional layering shows one layer at a time (linear — beginning/middle/end, like
a story; or nonlinear — hierarchical/parallel/web, revealed by navigation). Three-dimensional
layering shows multiple layers simultaneously, stacked as **opaque** planes (elaboration without
switching context, e.g. popup windows) or **transparent** planes (overlays that combine to reveal
relationships, e.g. weather maps). *See also Chunking, Five Hat Racks, Progressive Disclosure.*

**Application to Babylon:** **Map lenses are three-dimensional transparent layering by the book's
own example ("weather maps")** — this is the strongest, most literal match in the whole survey.
A lens (Solidarity, Contradiction Field, Imperial Rent) should render as a semi-transparent
overlay on top of the base cartography, not as a full lens-swap that replaces the map, so
relationships between the base political map and the data overlay stay visible simultaneously.
The **inspection stack**, by contrast, is opaque 2D layering — each drill-down level replaces or
stacks atop the previous panel without needing to see the base map through it. Multiple
simultaneous lenses (if ever supported) should be capped hard — the book's weather-map analogy
works because weather maps rarely stack more than 2 transparent layers before becoming unreadable.

---

## 5. Wayfinding

**What the book says:** Four stages, all directly applicable to a game map: **orientation**
(divide space into distinct subspaces with landmarks + signage — "signage is one of the easiest
ways to tell people where they are and where they can go"), **route decision** (minimize
navigational choices, prefer shorter routes, use maps over narrative directions when the space is
large/complex — "especially true when the person navigating is under stress"), **route
monitoring** (paths need a clear beginning/middle/end and sight lines; "breadcrumbs" — visual cues
of the path already taken — aid recovery from a wrong turn), **destination recognition** (clear,
consistent identities; dead-ends or barriers signal arrival). *See also Errors, Mental Model,
Progressive Disclosure.*

**Application to Babylon:** This is the map's core discipline, not a side concern. **Orientation**:
every hex/county/state needs a persistent, legible label at the zoom level where it's the
addressable unit — landmarks (capitals, major industrial hexes, contested fronts) should get
distinct icon treatment so players can navigate by them, not just by coordinates. **Route
decision**: when a player is moving an organization or tracing a supply chain, minimize the
decision points shown at once (don't render every possible adjacency at every zoom level — reveal
choices progressively as the player zooms/hovers). **Route monitoring**: give the "move" verb's
in-flight path a visible breadcrumb trail so a player who alt-tabs away and returns mid-tick can
re-orient instantly. **Destination recognition**: give territory selection (click target) a
distinct highlight state that's unambiguous from hover state — under Percy's stated stress case
("the player is directing chaos"), ambiguous selection state is a wayfinding failure.

---

## 6. Performance Load

**What the book says:** "The greater the effort to accomplish a task, the less likely the task
will be accomplished successfully." Splits into **cognitive load** (mental effort — perception,
memory, problem-solving; reduced via minimizing visual noise, chunking, memory aids, automating
computation) and **kinematic load** (physical effort — number of steps/movements/force; reduced by
cutting steps, minimizing travel distance/range of motion, automating repetition). The GUI's shift
from memorized command-line syntax to browsable menus is the canonical cognitive-load win; Morse
code's shortest symbols for the most frequent letters is the canonical kinematic-load win.
*See also 80/20 Rule, Chunking, Cost-Benefit, Hick's Law, Fitts' Law, Mnemonic Device, Recognition
Over Recall.*

**Application to Babylon:** Directly indicts any UI requiring memorized hotkeys or multi-menu
digging for the **9 core verbs** — they should be one click/keypress from the selected-entity
context, not buried in a settings-style menu (kinematic load). The **top bar**'s always-visible
stats should be the "critical 20%" (see 80/20 Rule below) that players check every tick; everything
else belongs behind a drill-down (cognitive load reduction via reduced visual noise). Give
frequent, repeated actions (e.g., "mobilize this organization again next tick") a low-kinematic
shortcut, analogous to Morse code's frequency-weighted encoding — assign the shortest interaction
path to the most-repeated verb in actual playtest telemetry, not by guess.

---

## 7. Aesthetic-Usability Effect

**What the book says:** "Aesthetic designs are perceived as easier to use than less-aesthetic
designs — whether they are or not." This is not a cosmetic nicety: aesthetic designs get *higher
adoption and more tolerance for real usability flaws*; positive relationships with a design
"catalyze creative thinking and problem solving," while negative relationships "narrow thinking and
stifle creativity" — and this effect is *stronger under stress*, since stress already degrades
cognitive performance. First impressions bias all subsequent interaction and resist revision.
*See also Attractiveness Bias, Form Follows Function, Golden Ratio, Law of Prägnanz, Ockham's
Razor, Rule of Thirds.*

**Application to Babylon:** This is the direct theoretical justification for the whole "make it
feel like a game, not a corporate dashboard" mandate — it is not subjective taste, it is a
documented usability lever. The ratified "Cold Collapse" dark/cyan palette should be applied with
maximal consistency across every chrome surface (top bar, dock, inspection stack, wire feed) so
the *first* map a new player sees establishes the positive first impression the book says is
"resistant to change." Given Babylon's genuinely high information density (26 systems, dozens of
formulas), the aesthetic-usability effect is the mechanism by which players tolerate that density
without bouncing — invest disproportionately in the map's *first-five-seconds* visual polish
(smooth border rendering, legible typography, restrained color use) over feature count.

---

## 8. Signal-to-Noise Ratio

**What the book says:** Maximize signal (accurate, minimally-degraded information) and minimize
noise (extraneous elements). Signal degradation happens through unclear writing, wrong chart
types, or ambiguous icons; it's fixed with simple/concise presentation and, notably,
**highlighting or redundant coding of key elements**. Noise reduction means removing every
unnecessary data item, line, or symbol — "every element in a design should be expressed to the
extent necessary, but not beyond." Grid/table lines specifically called out: thin them, lighten
them, or remove them. *See also Alignment, Layering, Ockham's Razor, Performance Load.*

**Application to Babylon:** A direct critique target for the current "corporate dashboard" feel —
deck.gl/MapLibre chrome defaults (heavy grid lines, boxed panels, dense borders) read as noise by
this principle. Redesign the **inspection stack**'s tables to use minimal/no rule lines, relying on
whitespace and typographic weight instead. On the **map** itself, county/state borders that are
*not* part of the active lens's signal should be rendered thin/low-opacity; the active lens's
signal (e.g. contradiction-field intensity) should be the highest-contrast element on screen at
any moment — never competing decoratively with base cartography.

---

## 9. Recognition Over Recall

**What the book says:** People recognize far more easily than they recall (multiple-choice beats
fill-in-the-blank) because recognition provides memory cues that narrow the search space; the
GUI's shift from command-line to browsable menus is the canonical example. Recognition memory is
also easier to form and longer-lasting than recall memory. Crucially, this extends to
**decision-making**: "A familiar option is often selected over an unfamiliar option, even when the
unfamiliar option may be the best choice" (the peanut-butter blind-taste-test example). *See also
Exposure Effect, Serial Position Effects, Visibility.*

**Application to Babylon:** The **action dock** should always show the 9 verbs as visible,
recognizable icons/labels rather than requiring the player to recall which key does what — this is
non-negotiable for any player who hasn't memorized shortcuts yet. The **lens picker** and
**inspection-stack breadcrumb trail** should let players *browse* available drill-down paths (what
can I click next?) rather than requiring them to recall that, say, "clicking the rent figure reveals
the tensor." Be aware of the bias risk this principle also names: if the UI always surfaces the same
familiar lens/verb first, players will default to it over a better option — vary the *default*
selection cautiously, or explicitly surface "other lenses you haven't tried."

---

## 10. Visibility

**What the book says:** Systems are more usable when status, possible actions, and consequences of
actions are clearly visible — but the book explicitly warns against "kitchen-sink visibility"
(trying to show everything all the time), which paradoxically makes relevant information *harder*
to find under information overload. The fix is **hierarchical organization** (categorize and hide
behind a visible parent control) plus **context sensitivity** (show only what's relevant to the
current context; hide/minimize irrelevant controls). The Three Mile Island case study is used as a
cautionary tale: under real operational stress, blind spots and delayed feedback (a status printer
that could only print 15 lines/minute) made the crisis unsolvable. *See also Affordance, Mapping,
Mental Model, Modularity, Progressive Disclosure, Recognition Over Recall.*

**Application to Babylon:** This is the principle that most directly argues against dumping every
stat onto the **top bar** at once — Babylon's 26-system, 56-formula depth makes "kitchen-sink
visibility" a live risk, not a hypothetical. Use **context sensitivity** aggressively: the
**action dock** should show only verbs legal for the current selection/turn phase (an organization
mid-cooldown shouldn't show "attack" as a live button); irrelevant lenses for the current zoom
level (e.g., a hex-level metabolic-rift lens at national zoom) should gray out or hide rather than
render uselessly. The TMI case is a direct warning for the **wire feed** under crisis conditions
(mass uprising, collapse cascade): if events queue faster than the feed can surface them, that is
exactly the "status information more than an hour behind" failure mode — the feed needs
digest/summarization under load, not a growing unread backlog.

---

## 11. Mapping

**What the book says:** "A relationship between controls and their movements or effects. Good
mapping ... results in greater ease of use." Good mapping is a function of similarity of *layout*
(stovetop knobs arranged like the burners), *behavior* (turning a wheel left turns the vehicle
left), or *meaning* (red = stop). Avoid one control serving multiple functions; where unavoidable,
use visually distinct modes. Be cautious relying on cultural conventions, since they vary by
population (the UK light-switch example). *See also Affordance, Interference Effects, Proximity,
Visibility.*

**Application to Babylon:** Camera/pan controls and any drag-to-move interaction on the map should
obey behavioral mapping (drag direction = pan direction, no inversion) without a settings toggle
defaulting to the wrong sense. Verb-icon coloring in the **action dock** should use meaning-mapping
consistently with the established Cold Collapse palette (e.g., if red is used anywhere for
"attack," it must not also mean "danger to me" elsewhere — one meaning per color, one color per
meaning, system-wide). Avoid overloading a single map click with multiple functions (select vs.
move vs. inspect) — if click must be multi-purpose, use a visually distinct cursor/mode indicator
per the book's mode-guidance, not silent context-dependent behavior.

---

## 12. Proximity

**What the book says:** A Gestalt principle — elements placed close together are perceived as more
related than elements placed farther apart, and this cue "will generally overwhelm competing visual
cues (e.g., similarity)." Labels should be positioned directly on/near what they describe (direct
labeling beats a legend/key). The book's own trailhead-sign example shows how bad proximity
literally sends people the wrong way. *See also Chunking, Performance Load, Similarity.*

**Application to Babylon:** Favor **direct labeling on the map** (territory names/stat callouts
placed on the hex/county itself) over a detached legend panel wherever screen space allows —
legends should be the fallback, not the default, for lens value ranges. In the **inspection
stack**, keep a number's explanatory sub-breakdown (e.g., rent = wages − value) spatially adjacent
to the number it explains, not in a separate tab, so the "every number explains itself" goal reads
as proximity-driven grouping rather than requiring recall of where the explanation lives.

---

## 13. Figure-Ground Relationship

**What the book says:** Perceptual systems split any scene into a "figure" (object of focus,
clearer shape, feels closer, receives more attention/memory) and "ground" (undifferentiated
background). Visual cues that push an element toward "figure": definite shape vs. shapeless
background; appearing to sit in front of/closer than the background; and — notably — position
below rather than above a horizon line, and in *lower* regions of a composition rather than upper
regions. Ambiguous figure-ground (the Rubin vase) is perceptually unstable and should be avoided
unless intentional. *See also Gutenberg Principle, Law of Prägnanz, Top-Down Lighting Bias.*

**Application to Babylon:** The active **lens overlay** and any selected-territory highlight must
read unambiguously as figure against the base cartography ground — give it a definite, closed
shape and a "closer" treatment (drop shadow, brighter saturation, slight elevation in the
three-dimensional-layering sense) rather than blending into the base map's color range. Floating
game chrome (dock, top bar, event toasts) sitting in the lower half of the viewport gets a
figure-perception assist "for free" per this principle — reinforces putting the action dock at the
bottom of the screen rather than the top, and explains why HUD elements pinned to the top of a
screen often feel more like passive "ground" (status bar) than active "figure" (controls).

---

## 14. von Restorff Effect

**What the book says:** Noticeably different items are recalled far better than common ones,
whether the difference is contextual (the one digit in a string of letters) or experiential (an
atypical event). "If everything is highlighted, then nothing is highlighted, so apply the technique
sparingly." Useful specifically for boosting recall of *middle* list items, which otherwise suffer
under Serial Position Effects (below). *See also Highlighting, Serial Position Effects, Threat
Detection.*

**Application to Babylon:** **Event toasts** for genuinely rare/critical events (a Rupture Event,
an endgame-adjacent threshold crossing) should get a visually distinct treatment (motion, color,
size break) that is *not* reused for routine events — reserving the "unique" treatment for the
handful of moments the book says will make it memorable. If every wire-feed item gets an urgent
red flash, the flash stops working; ration it to the small number of state changes that are
narratively/mechanically pivotal (e.g., endgame trigger proximity, a faction's collapse).

---

## 15. Serial Position Effects

**What the book says:** Items at the start (primacy) and end (recency) of a list are recalled
better than items in the middle; for visual lists, primacy dominates influence on interpretation;
for auditory lists, recency dominates; and when a decision follows immediately after the last item,
recency wins even for visual material. This also produces general **order effects** in choice —
first/last options in any list are disproportionately selected regardless of merit (the book cites
ballot-position research). *See also Advance Organizer, Chunking, Classical Conditioning, Operant
Conditioning.*

**Application to Babylon:** In the **lens picker** and **verb dock**, do not let arbitrary
alphabetical or code order determine which verb/lens sits first or last — that position carries
disproportionate selection weight per this principle, so the most important/most-intended-for-new-
players options (e.g., "mobilize," "investigate") should deliberately occupy the primacy/recency
slots, and options the design does *not* want over-selected (a high-risk/irreversible verb like
"attack") should sit in the middle rather than at an edge. For the scrollable **wire feed**, since
it's read visually, the newest (primacy, if newest-first) or a pinned "most important" event should
anchor an edge position rather than getting lost mid-list.

---

## 16. Feedback Loop

**What the book says:** Systems are made of interacting feedback loops. **Positive feedback**
amplifies (growth or decline) and, left unmoderated, causes runaway/collapse — the book's football-
helmet case study shows how a well-intentioned safety redesign (positive feedback on player risk-
taking) produced *more* injuries. **Negative feedback** dampens toward an equilibrium (the Segway's
100 corrective adjustments/second) but excessive negative feedback causes stagnation. Key
takeaway: "changing one variable in a system will affect other variables in that system and other
systems" — design elements must be considered in relation to the whole. *See also Convergence,
Errors, Shaping.*

**Application to Babylon:** This is squarely about the *simulation's* dynamics (spirals toward
revolution/fascism/collapse), but it has a direct UI corollary: the map/UI must make **positive
feedback loops visible as they accelerate**, not just their end state — e.g., a rising contradiction
field or an accelerating repression spiral should get a visibly intensifying lens treatment (not a
single threshold pop) so players can read the system's *trajectory*, not just its current value, the
way the book's Segway/thermostat framing implies feedback is continuous, not a single reading.

---

## 17. Control

**What the book says:** "The level of control provided by a system should be related to the
proficiency and experience levels of the people using the system" — beginners do best with reduced,
structured control (training wheels); experts do best with more direct, less-structured control.
Systems should generally offer at most **two** interaction methods for a given task — one for
beginners, one for experts (e.g., File→Save menu vs. keyboard shortcut) — because supporting more
than two multiplies complexity. Systems used only rarely/by first-timers (museum kiosks, ATMs)
should *not* try to accommodate expertise tiers at all. *See also Constraint,
Flexibility-Usability Tradeoff, Hierarchy of Needs.*

**Application to Babylon:** Give every core verb a menu/click path (beginner method) *and* a
keyboard shortcut (expert method) — but resist adding a third or fourth way to invoke the same
verb, per the book's complexity-cost warning. Since Babylon is a deep sim meant to be played
repeatedly (not a kiosk/ATM one-off), it is exactly the kind of system the book says benefits from
expert shortcuts and eventually from **customization** (the "highest level of control") — e.g.,
letting experienced players customize which lenses/stats populate the top bar's chunked clusters.

---

## 18. Five Hat Racks

**What the book says:** There are exactly five ways to organize any body of information:
**alphabet** (referential, nonlinear lookup — dictionaries), **time** (chronological sequence —
timelines, schedules), **location** (geographical/spatial — maps, exit plans; use "when orientation
and wayfinding are important"), **continuum** (magnitude ordering — leaderboards, search-result
ranking), and **category** (similarity/relatedness clusters — a store's department layout). *See
also Advance Organizer, Consistency, Framing.*

**Application to Babylon:** Use this as a checklist when designing any list surface in the game:
the **wire feed** should default to *time* organization (chronological) but offer a *category*
filter (economic events / military events / political events); a "search for a territory" panel
should default to *alphabet* for direct lookup but offer *location* (click-to-select on the map
itself) as the primary path, since the book explicitly recommends location-organization "when
orientation and wayfinding are important" — which is Babylon's whole map-first premise. Any
leaderboard-style ranking (e.g., factions by solidarity) is a natural fit for *continuum*
organization.

---

## Summary table

| Principle | Primary Babylon surface | One-line directive |
|---|---|---|
| Progressive Disclosure | Inspection stack, action dock, lens drawer | Show headline only; drill down on request |
| Chunking | Top bar, wire feed digest | Group HUD stats into 4±1 clusters; never chunk searchable lists |
| Hierarchy | Inspection stack, cartography nesting | Stair for drill-down, nest for border containment |
| Layering | Map lenses (3D transparent), inspection stack (2D opaque) | Lenses overlay, never replace, the base map |
| Wayfinding | Cartography, move-verb paths | Persistent labels, breadcrumbed movement, unambiguous selection |
| Performance Load | Action dock, top bar | Shortest path for most-repeated verb; critical stats always visible |
| Aesthetic-Usability Effect | All chrome | Invest in first-five-seconds polish; apply palette with total consistency |
| Signal-to-Noise Ratio | Inspection tables, map borders | Thin/remove non-signal lines; max contrast on the active lens |
| Recognition Over Recall | Action dock, lens picker | Icons/labels always visible; never require memorized shortcuts alone |
| Visibility | Top bar, action dock, wire feed | Context-sensitive, not kitchen-sink; digest the feed under crisis load |
| Mapping | Camera controls, verb color coding | One meaning per color; behavior matches expectation, no silent modes |
| Proximity | Map labels, inspection stack | Direct-label on the map; keep explanations adjacent to the number |
| Figure-Ground | Lens overlay, selection highlight, dock placement | Selected/active elements read as closer/figure; dock chrome low on screen |
| von Restorff Effect | Event toasts | Reserve the "unique" visual treatment for genuinely rare/critical events |
| Serial Position Effects | Lens picker, verb dock, wire feed | Put priority options at list edges, not the middle |
| Feedback Loop | Contradiction/repression lenses | Show trajectory/acceleration, not just current-value snapshots |
| Control | Action dock | Exactly two invocation methods per verb: menu + shortcut, no more |
| Five Hat Racks | Wire feed, search panel, leaderboards | Match organization scheme to task: time for feed, location for search, continuum for rankings |
