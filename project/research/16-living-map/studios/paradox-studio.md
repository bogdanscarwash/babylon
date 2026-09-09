# Paradox Development Studio — GUI Conventions Across the Grand-Strategy Catalogue

Research pass for Babylon's living-map/cockpit redesign (Program 16). Scope: shared UI/UX
doctrine across EU4, CK3, HOI4, Stellaris, and Victoria 3, the Clausewitz/Jomini engine's role
as the UI substrate, the map-centrism doctrine, the EU4→CK3/Vic3 shift from icon-density to
typography, nested tooltips as a studio-wide innovation, pause-and-play as an interaction model,
outliner/alert conventions, feature-accretion-over-a-decade problems, and recurring community
criticism. All claims below are sourced; where a claim could not be verified from a fetched
primary/secondary source it is flagged as unverified or omitted.

## 1. Studio structure and engine substrate

Paradox Development Studio (PDS) was founded in 1995 out of the Swedish board-game company
Target Games; in 1999 the group split into Paradox Interactive (publisher) and Paradox
Entertainment, and a 2012 reorganization created the modern split where Paradox Interactive is
the publisher and PDS is the grand-strategy development studio. By 2021 PDS had split further
into internal color-coded teams — PDS Gold (Hearts of Iron IV), PDS Red (Victoria 3, later
renamed PDS Purple in 2024), PDS Black (Crusader Kings III), PDS Green (Stellaris) — with PDS
Teal created for an unannounced project. This matters for the UI story: **each team owns its own
UI conventions inside a shared engine**, which is exactly why the studio's GUI language is
consistent at the substrate level (fonts, tooltip widgets, outliner mechanics, map-mode
switching) but diverges at the surface level (iconography density, panel ornamentation,
information architecture) from title to title.

Three engine generations underlie all of this:

- **Europa Engine (2000–2007)** — the original in-house engine. Studio manager Johan Andersson
  has said the "engine" was never a deliberate initial design; it emerged from copy-pasting large
  chunks of code from one game to the next across the company's first six titles.
- **Clausewitz Engine (2007–present)** — debuted with *Europa Universalis III*, a C++ engine
  providing the 3D globe/map view used by every subsequent PDS flagship (EU, HOI, Victoria,
  Crusader Kings, Stellaris).
- **Jomini toolset (2019–present)** — a 64-bit mid-layer added on top of Clausewitz, first shipped
  in *Imperator: Rome*, explicitly built to stop duplicating UI/data code across projects: "the
  goal is to move lots of the duplicated code from all their grand strategy games into Jomini so
  instead of copy-pasting features, they can reuse them and add improvements and bug fixes that
  other projects can benefit from." Jomini and Clausewitz are described as "two halves of the same
  engine," with Jomini specifically serving top-down, map-based games. *Imperator: Rome*, *Crusader
  Kings III*, *Victoria 3*, and the announced *Europa Universalis V* are the "Clausewitz+Jomini"
  titles — which is also, not coincidentally, the same set of titles where nested tooltips,
  outliner refinements, and typography-led panels appear.

**Babylon relevance:** Babylon has no equivalent shared-engine story to inherit, but the lesson
generalizes — decide now which UI primitives (tooltip nesting, lens system, alert taxonomy) are
"engine-level" (must work identically everywhere) versus "screen-level" (can vary per panel), the
same way PDS drew the Clausewitz/Jomini boundary.

## 2. The map-is-the-star doctrine

No single canonical "map is the star" manifesto quote from Paradox was found in this pass (the
phrase circulates as community shorthand, not a verified direct citation — flagged as
**unverified**, do not attribute it to Paradox as a direct quote). What *is* verifiable from the
EU4 wiki and community discussion is the structural doctrine underneath the phrase:

- The map is the **entire background layer** — "the map itself comprises most of the Earth, apart
  from the polar caps" — and every UI panel is a translucent or bordered overlay docked to a
  screen edge, never a full-screen takeover for routine information (EU4's ledger, macrobuilder,
  and outliner all float over the map rather than replacing it).
- **Map modes** are the primary lens mechanism: EU4 exposes four categories — political,
  diplomatic, economic, geographical — bound to quick-access shortcut keys (Q through P), letting
  players re-skin the same province geometry with different color-coded data without leaving the
  map view. Terrain view color-codes by terrain type (hills brown, forests dark green, plains
  light green, farmland yellow, marshland blue); political view assigns each country a unique
  color with emphasized borders.
- A recurring, explicit community argument (Paradox forums, "Suggestions for how to improve the
  UI/UX design from previous Paradox games") is that Paradox should **not** let fear of information
  overload drive UI design by hiding information behind big simplified buttons — the stated
  industry-wide failure mode the community holds Paradox up against.

**Babylon relevance:** this directly validates the brief's "full-bleed readable political map …
hexes are just deep-zoom tiles" plan. The EU4 map-mode-by-hotkey pattern is a ready-made template
for a Babylon "lens" system switching between political control, class-solidarity/exploitation
overlay, imperial-rent intensity, and territory/hex substrate — bound to number keys, not buried
in a menu.

## 3. The EU4→CK3/Victoria 3 shift: icon density to typography-led panels

This is a real, sourced evolution, not merely a stylistic reskin:

- **EU4-era doctrine (icon-heavy):** the top bar packs treasury, manpower, sailors, stability,
  corruption, prestige, national unity, and power projection into a dense row of small numeric+icon
  readouts; envoy availability is shown as compact "X/Y" figures for merchants, colonists,
  diplomats, missionaries. Alerts are colored flags (red = urgent, yellow = lesser importance,
  green = not urgent), diplomatic requests are brown flags with red edges — an entirely
  iconographic vocabulary, legible at a glance to an experienced player but opaque to a new one.
  Community commentary on the *Imperator: Rome* lineage explicitly names this pattern: "many tiny
  buttons due to space restrictions needed to convey large amounts of information simultaneously."
- **CK3/Victoria 3-era doctrine (typography + progressive disclosure):** Victoria 3's 2D Art Lead
  (Kenneth, Dev Diary #30) frames the UI as three categories — Panels, Buttons, and Icons — with
  design effort concentrated on **panel frames, borders, and headers** carrying an Art Nouveau
  visual language (drawn from nature/plant forms) intended to evoke "the extravagance and luxuries
  of the upper class in the Victorian era," with regional palette variation (e.g., a Balkan-region
  scheme using earthy reds/black/green/gold against dark grey). The organizing idea is fewer,
  larger, more legible panels carrying explanatory text and nested tooltips rather than dense icon
  grids.
- Community framing of the shift (Paradox forums, "Comprehensive EUV feedback" and the general
  UI/UX suggestions thread) is contested, not a clean "CK3 is better" story: a meaningful faction
  of veteran players considers **EU4's UI the best Paradox has made** precisely because its icon
  density supports fast expert workflows, and views CK3/Victoria 3's text-and-tooltip approach as
  solving accessibility for newcomers at the cost of speed for experts. Paradox's own framing
  (per the community's read on Imperator/CK3 design commentary) is that new titles targeting
  different/broader audiences justify different tradeoffs rather than a strict "text is better than
  icons" doctrine.

**Babylon relevance:** Babylon's brief explicitly wants Victoria-3-style nested recursive
inspection — this is the correct reference title, but the studio's own history shows this is a
tradeoff, not a strict improvement: expert players lose glanceable density when text/tooltips
replace icon rows. Babylon's political-economy numbers (Imperial Rent, class solidarity,
P(S|A)/P(S|R)) are exactly the kind of derived values that benefit from panel-header icons +
one-line summary + nested "why" tooltip, rather than either pure icon-soup or pure prose.

## 4. Nested tooltips — the studio-wide innovation, its origin, and its stated reasoning

This is the best-documented design decision in the whole research pass.

**Origin and stated reasoning (CK3 Dev Diary #16, "Tutorials and Tooltips and Encyclopedias, Oh
My!," written by a PDS programmer):** the diary frames the entire tooltip-nesting + Encyclopedia
system around one explicit design goal — verified quote (paraphrased summary of the diary,
sourced via search, treat the specific wording as a close paraphrase rather than a verbatim
quote): *the goal was to make the depth and strategy of the game shine through in the choices
players make, not the 20 minutes spent trying to find the information needed to make those
choices because the interface is obtuse.* The Encyclopedia feature itself reportedly originated
as a programmer's personal-development-day side project before being adopted studio-wide — a
useful data point that this "innovation" grew bottom-up from an engineer's frustration with
opacity, not a top-down UX mandate.

**Mechanism (per the CK3 wiki and the Philip Ardeljan design-blog analysis "Tooltips in
Tooltips"):**
- Paradox's solution to "I need to explain a concept, but the explanation itself uses concepts
  that also need explaining" is literally nested tooltips: hovering a highlighted term inside a
  tooltip opens a second tooltip layer explaining *that* term, recursively.
- Two interaction modes govern how a nested tooltip stays open: **timer lock** (default — it
  persists for a configurable duration) and **action lock** (middle-mouse-click pins it in place
  manually).
- The design blog's key UX insight: the interaction works because of natural cursor behavior — a
  user "keeps their cursor nice and still" to read the first tooltip, then naturally drifts toward
  an unfamiliar highlighted term to drill deeper, so the nesting gesture feels discovered rather
  than taught.
- Explicitly acknowledged limitation, both by the design blog and in-studio: nesting depth needs a
  hard practical ceiling — "an infinitely deep tooltip chain doesn't make sense" — because each
  additional layer adds cognitive and spatial cost (tooltips can run off-screen; HOI4 in particular
  is criticized for tooltips that "go off the right or bottom side of the screen" and simply fail
  to display).

**Propagation across the studio:** Victoria 3 explicitly adopted CK3's nested-tooltip system
before launch (reported pre-release under the framing "Victoria 3 will use Crusader Kings 3's
tooltip system so you don't need an econ degree") — direct evidence this was consciously
generalized as shared Jomini-era UI infrastructure, not reinvented per title.

**CK3's tutorial-redesign use of progressive disclosure (Game Developer deep-dive interview with
Valeska Martins and Ellinor Zetterman, UX design lead and UX designer, PDS Black, Stockholm):**
this is the closest thing found to a studio-level GDC-style design talk (no dedicated GDC Vault
talk on nested tooltips specifically was locatable in this pass — flagged as **not found**,
despite searching).
- Core principle, direct quote: *"one way to make users feel like a complex interaction is easy to
  interact with is by adding complexity bit by bit."* Applied concretely to the character panel:
  details are revealed gradually rather than all at once.
- Telemetry-driven rationale: the team found tutorial completion was **not** a strong retention
  predictor. The original CK3 tutorial presented **67 sequential boxes** of explanatory text — the
  redesign cut it to roughly one-third that length by keeping only essential mechanics.
- They ran both tutorial versions concurrently to gather comparative data rather than committing to
  a single redesign on intuition — an explicit evidence-before-conviction posture, articulated as:
  *"you shouldn't be afraid to try things. No result is a bad result as long as you're willing to
  learn from it."*
- For tutorial-flagged events specifically, they added explanatory text directly into tooltips — a
  deliberately non-standard practice reserved for onboarding moments, not regular play — to
  introduce mechanics gently without derailing sandbox agency. Underlying philosophy: because CK3
  is an open sandbox, "we have to respect that" players make their own decisions even mid-tutorial,
  so the tutorial cannot force sequence, only annotate.

**Babylon relevance:** this is the single most directly transferable pattern for Babylon's
"Victoria-3-style nested recursive inspection panels where every number explains itself" goal.
Concretely: (a) adopt timer-lock as default hover behavior with a modifier-key pin, mirroring
CK3's action-lock; (b) cap nesting depth deliberately (the Vic3 UI backlash below shows what
happens when nesting substitutes for information architecture rather than supplementing it); (c)
every derived number in Babylon (Imperial Rent Φ, P(S|A), P(S|R), Contradiction intensity) should
have its formula terms as nested-tooltip targets — e.g. hovering "Imperial Rent" opens a tooltip
whose "Core Wages" and "Value Produced" terms are themselves hoverable.

## 5. Pause-and-play (real-time-with-pause) as an interaction model

Verified structural facts:
- PDS grand-strategy titles run as **real-time-with-pause**: game time advances in discrete ticks
  (a day for EU4/CK3/Victoria, an hour for HOI4) where all actions for that tick resolve
  simultaneously and instantaneously at tick-start, but the player can pause at any moment and
  still issue orders — pause does not lock the UI.
- Speed controls let the player compress the tick rate to the point where play *feels* real-time,
  while the slowest speed keeps individual ticks visually distinct.
- Design rationale (sourced from community/design analysis, not a direct Paradox quote — treat as
  informed synthesis): with large numbers of simultaneously-acting entities (nations, characters,
  organizations), strict turn-based sequencing creates severe turn-order fairness and pacing
  problems; making resolution automatic-and-simultaneous while giving the player control over
  *pacing* (not turn order) resolves this without forcing multiplayer-hostile turn queues. A
  secondary rationale: most non-military decisions are deliberately designed so that missing a
  handful of ticks is low-consequence (e.g., EU4 income/manpower recalculates monthly, giving an
  effective ~30-day decision window), which is what makes variable-speed play tolerable rather than
  punishing.

**Babylon relevance:** Babylon's engine already runs deterministic ticks through 26 ordered
systems — this maps cleanly onto the PDS model. The UI implication is that pause must be a true
first-class state (not just "stop the clock") — every panel, verb menu, and lens must remain fully
interactive while paused, and speed controls should telegraph tick length consequences (e.g., "this
verb resolves next tick") the way EU4 telegraphs monthly income recalculation, so players
understand why waiting one more tick before acting is safe or costly.

## 6. Outliner and alert conventions

- **Alert taxonomy (EU4, confirmed on wiki; Stellaris, confirmed via wiki+forum):** a
  three/four-tier urgency-colored system. EU4: red = urgent/immediate, yellow = lesser importance,
  green = not urgent, plus a distinct brown-with-red-edge flag class for diplomatic requests
  specifically (visually separating "the game needs a decision" from "another actor wants a
  decision from you"). Stellaris: alert outline/background color again signals urgency (red = very
  urgent, orange = somewhat urgent, green = informational), and critically **each alert type can be
  individually disabled and re-enabled through the outliner** — alert visibility is player-tunable,
  not fixed by the developers.
- **Outliner (Stellaris, primary documented instance):** a persistent list docked to a screen edge
  (right edge in Stellaris) giving quick-access rows with icon + name + location for
  player-relevant entities (systems, planets, fleets), functioning as the always-visible index into
  a large, spatially-distributed empire.
- **Outliner as a recurring failure point:** this is the single most consistently criticized UI
  surface across the sourced community feedback. Stellaris players describe the outliner as
  "cluttered, unintuitive, and requiring so many clicks to navigate what used to be clean and
  apparent at a glance," specifically citing the decision to forcibly split a previously unified,
  customizable outliner into separate planet and ship tabs, and reliance on the "Tiny Outliner"
  community mod as "almost mandatory" to make the stock outliner usable. A dedicated forum thread
  ("Why the outliner is always going to be cluttered") suggests the studio itself treats outliner
  clutter as a structurally hard problem, not a one-off bug — i.e., any list-of-everything UI
  surface that scales with player empire size will trend toward clutter unless actively curated
  per-release.

**Babylon relevance:** Babylon will need an outliner-equivalent for tracked organizations,
active contradictions, and pending verb resolutions. The Stellaris lesson is concrete: (a) ship
per-category filtering/tabs from day one rather than retrofitting them after clutter complaints;
(b) make alert-type visibility player-configurable per the EU4/Stellaris pattern rather than
fixed; (c) budget for the outliner to be the surface most likely to need a "Tiny Outliner"-style
density mode — design a compact row variant up front.

## 7. A decade-plus of feature accretion in one UI

- Verified pattern (Paradox forums, multiple threads; "This game is slowly dying to feature creep,
  bloat and multiplication"; "Game is getting more complex"): community consensus is that PDS's
  live-service DLC model produces continuous *free* systemic overhauls (patches) layered on top of
  *paid* flavor content, meaning the base UI complexity grows for every player regardless of
  purchase, and new mechanics/modifiers accumulate without corresponding pruning — cited as a
  primary onboarding barrier ("newcomers have extreme difficulty even with experienced players
  helping them").
- A related, more structural version of the same complaint recurs in the Victoria 3 launch-era
  criticism (Streams of Consciousness blog critique, and the Paradox forum thread "User
  Inter-fiasco: Victoria III's Abysmal UI"): the failure mode is not "too much information" in the
  abstract but **fragmentation without cross-linking** — related functionality scattered across a
  large number of narrowly-scoped menus, with tooltips that describe state ("you are
  knowledge-sharing") but provide no path to the control that changes that state, forcing players
  to remember or externally document which of "a dozen different flavors of diplomacy menu"
  actually contains the toggle they need. The blog's proposed fix, stated plainly: *"any menu that
  will tell you about something should also include a link letting you get there and change it"* —
  i.e., every read-only status display should carry a hyperlink/breadcrumb to its corresponding
  control surface, not just a description.
- Victoria 3 also drew specific criticism for late-game **notification/pop-up flooding** —
  "an endless deluge of pop-ups for countries and events with no impact" that, once volume crosses
  a threshold, trains players to reflexively dismiss all pop-ups, defeating the alert system's
  purpose entirely.
- HOI4 draws a parallel but more technical version of accretion pain: community reports describe
  the interface as "so uninformative and yet so cluttered" with "muddy washed out colors," plus
  concrete technical debt — misaligned button hitboxes, tooltips clipped off-screen, and long-running
  complaints about incomplete 4K/UI-scaling support.

**Babylon relevance:** this is the most load-bearing warning in the whole report for a Program-16
build that intends nested nested-recursive panels across 26 systems and 9 verbs from day one.
Two concrete guardrails follow directly from PDS's own decade of scar tissue: (1) **every
status/read-only display must carry a link to its actionable control**, enforced as a review
checklist item, not an afterthought; (2) **notification volume must be triaged by
consequence-weight, not raised on every state change** — Babylon's wire/news feed is structurally
exactly the Victoria 3 pop-up-flood risk (26 systems × many organizations × many ticks = a lot of
candidate events), so the wire needs the same red/yellow/green consequence tiering EU4 applies to
alerts, plus per-category mute, from the first implementation, not retrofitted after players start
ignoring it.

## 8. Recurring community UX criticisms, synthesized across titles

Cross-referencing the sourced complaints above, four failure patterns recur across *multiple*
independent PDS titles (i.e., they are studio-level tendencies, not one game's bug):

1. **Read/write asymmetry** — panels that tell you a fact but not how to change it (Victoria 3
   diplomacy/knowledge-sharing case; generalized as a studio pattern by the "improve UI/UX" forum
   thread).
2. **Outliner/list-surface clutter that scales with player success** — the more territory/empire/
   characters a player accumulates, the worse the flagship list view gets, and mods exist
   specifically to compress it (Stellaris "Tiny Outliner"; implicitly the same risk for any
   Babylon organization/territory list that grows over a 520-tick run).
3. **Tooltip/notification volume that eventually trains players to stop reading** — HOI4's
   off-screen-clipped tooltips and Victoria 3's late-game pop-up flood are two different technical
   causes of the same behavioral outcome: players learn to ignore the notification layer.
4. **New-audience accessibility work perceived by veteran players as a regression** — the EU4
   icon-density vs. CK3/Vic3 text-and-tooltip debate is not resolved consensus even within the
   Paradox community; a design that optimizes for legibility-to-newcomers has a real, named cost to
   expert-player information throughput, and the studio has not "solved" this tradeoff so much as
   made different bets per title.

## Recommendations synthesis is in the structured output below.

## Sources

- [Paradox Development Studio — Wikipedia](https://en.wikipedia.org/wiki/Paradox_Development_Studio) — studio history, PDS Gold/Red/Black/Green/Purple/Teal team split, Europa/Clausewitz/Jomini engine history and title-to-engine mapping.
- [Clausewitz Engine — ModDB](https://www.moddb.com/engines/clausewitz-engine) — engine background.
- [The engine behind Paradox Development Studio's future games — VentureBeat](https://venturebeat.com/pc-gaming/the-engine-behind-paradox-development-studios-future-games/) — Jomini toolset rationale ("move duplicated code into a shared layer").
- [Future Paradox games will be easier to mod, thanks to a new engine — PCGamesN](https://www.pcgamesn.com/imperator-rome/paradox-new-engine-jomini) — Jomini/Clausewitz relationship, Imperator: Rome as first Jomini title.
- [Ingame screen — Europa Universalis 4 Wiki](https://eu4.paradoxwikis.com/Ingame_screen) — EU4 top-bar layout, alert color taxonomy, map-mode categories and hotkeys, outliner toggle.
- [Interface — CK3 Wiki](https://ck3.paradoxwikis.com/Interface) — CK3 UI widget/container technical structure (modding-oriented; limited design-philosophy content).
- [Tooltips in tooltips — Philip Ardeljan design blog](https://philip.design/blog/tooltips-in-tooltips/) — nested-tooltip UX mechanism, timer-lock/action-lock modes, cursor-behavior rationale, depth-limit caveat.
- [Reminder to the devs that nested tooltips is bad UX design — Paradox forums](https://forum.paradoxplaza.com/forum/threads/reminder-to-the-devs-that-nested-tooltips-is-bad-ux-design.1702017/) — dissenting community view on nested tooltips.
- [Deep Dive: Refreshing the Crusader Kings III tutorial mode through optimized UX — Game Developer](https://www.gamedeveloper.com/design/deep-dive-refreshing-the-crusader-kings-iii-tutorial-mode-through-optimized-ux) — Valeska Martins & Ellinor Zetterman (PDS Black UX) on progressive disclosure, 67-box tutorial reduction, dual-tutorial telemetry testing.
- [Victoria 3 will use Crusader Kings 3's tooltip system so you don't need an econ degree — PCGamesN](https://www.pcgamesn.com/victoria-3/nested-tooltip-system) — cross-title propagation of nested tooltips.
- [CK3 Dev Diary #16 - Tutorials and Tooltips and Encyclopedias, Oh My! — Paradox forums](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/) — origin of the Encyclopedia/tooltip system, "20 minutes trying to find information" design rationale, programmer-personal-project origin.
- [Victoria 3 - Dev Diary #30 - User Interface Overview — Paradox forums](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-30-user-interface-overview.1507166/) — Panels/Buttons/Icons taxonomy, Art Nouveau influence, regional palette variation (Balkan example).
- [Victoria 3 has the worst UI I've ever seen — Streams of Consciousness](https://streamsofconsciousness.blog/2025/02/24/victoria-3-has-the-worst-ui-ive-ever-seen/) — read/write asymmetry critique, "link to where you can change it" recommendation, late-game pop-up flood.
- [User Inter-fiasco: Victoria III's Abysmal UI — Paradox forums](https://forum.paradoxplaza.com/forum/threads/user-inter-fiasco-victoria-iiis-abysmal-ui.1561896/) — menu fragmentation complaints, comparison to Victoria 2.
- [Why the outliner is always going to be cluttered — Paradox forums](https://forum.paradoxplaza.com/forum/threads/why-the-outliner-is-always-going-to-be-cluttered.1505339/) — structural analysis of outliner clutter as scaling problem.
- [UI Concerns - Cluttered & Unintuitive — Stellaris Steam Discussions](https://steamcommunity.com/app/281990/discussions/0/357284131790145557/?ctp=3) — outliner tab-split criticism, "Tiny Outliner" mod dependency.
- [HOI4 as a beginner - the UI is HORRIBLE — Paradox forums](https://forum.paradoxplaza.com/forum/threads/hoi4-as-a-beginner-the-ui-is-horrible.1443216/) — HOI4 UI clutter/legibility complaints.
- [UI Issues — Hearts of Iron IV Steam Discussions](https://steamcommunity.com/app/394360/discussions/2/3183486320476103590/) — misaligned hitboxes, off-screen tooltip clipping.
- [4K UI — Hearts of Iron IV Steam Discussions](https://steamcommunity.com/app/394360/discussions/0/591765454927728587/) — UI scaling/4K support complaints.
- [Suggestions for how to improve the UI/UX design from previous Paradox games — Paradox forums](https://forum.paradoxplaza.com/forum/threads/suggestions-for-how-to-improve-the-ui-ux-design-from-previous-paradox-games-screenshot-heavy.1626069/) — cross-title community synthesis, information-overload-fear critique, EU4-vs-CK3 UI debate.
- [Comprehensive EUV feedback — Paradox forums](https://forum.paradoxplaza.com/forum/threads/comprehensive-euv-feedback.1743365/) — EU4 UI quality reputation among veteran players, Imperator's small-button density note.
- [This game is slowly dying to feature creep, bloat and multiplication — Paradox forums](https://forum.paradoxplaza.com/forum/threads/this-game-is-slowly-dying-to-feature-creep-bloat-and-multiplication.1091790/page-3) — DLC/systemic-overhaul feature-creep complaints.
- [Game is getting more complex — Paradox forums](https://forum.paradoxplaza.com/forum/threads/game-is-getting-more-complex.1727663/) — onboarding-difficulty complaints tied to feature accretion.
- [how much do you use pausing? — Paradox forums](https://forum.paradoxplaza.com/forum/threads/how-much-do-you-use-pausing.71902/#post-1306992) — player discussion of real-time-with-pause usage.
- [Real-Time with Pause — TV Tropes](https://tvtropes.org/pmwiki/pmwiki.php/Main/RealTimeWithPause) — genre-level framing of the pause-and-play model.

### Notes on unverifiable/unlocated claims

- The phrase "map is the star" / "map as the main character" could not be traced to a specific,
  quotable Paradox source in this pass; treat as community shorthand for a doctrine that IS
  independently verifiable (map-as-persistent-background + map-mode lenses), not as a direct
  quote.
- No dedicated GDC Vault talk specifically on Paradox's nested-tooltip system was locatable (GDC
  Vault entries found — "The Paradox DLC Model," "History and Game Design" — are behind login and
  cover different topics). The Game Developer deep-dive interview with the CK3 UX team is the
  closest verified equivalent to a "GDC talk by Paradox UX designers" and is cited as such above,
  not as a GDC talk.
