# Crusader Kings III — UI/UX Deep Dive for the Babylon Living Map

**Research date:** 2026-07-11
**Why CK3 matters to Babylon:** it is the Paradox title that solved
"explain a deeply nested causal model to a new player without a wall of
spreadsheets," via the **nested/recursive tooltip system** (later borrowed
wholesale by Victoria 3), and it is the clearest public case study of
**character-first information architecture layered over a map game** — the
map is the stage, but the portraits, relationships, and events are the
protagonists. It also carries the most-cited de jure/de facto **border
duality**, directly relevant to Babylon's "borders redraw as revolution
progresses" mandate, and a well-documented, metrics-driven **onboarding
rewrite** (Game Developer / Gamasutra deep dive) that is a template for how
to *measure* whether a tutorial works rather than assume it does.

---

## 1. Overall Information Architecture — what floats vs. what takes over

CK3's screen is a **fixed chrome frame around a 3D/painterly map**, with a
strong bias toward "flicking between menus, the map, and popups" rather than
full-screen takeovers. Paradox's own console-port UX lead stated the design
goal explicitly: avoid "lots of fullscreen menus that potentially break the
immersion of gameplay," and instead let players "quickly move between any
open screen elements as well as allowing easy Map Interaction."

Concretely, the desktop screen breaks into five persistent regions:

- **Top bar (two zones).** Left zone: a row of **event/process icons**
  (active wars, intrigues, ongoing schemes) that deep-link to their owning
  window on click, and are dismissible per-icon via right-click if the
  player doesn't want to deal with them yet. Right zone: the **four-resource
  strip** (Gold, Prestige, Piety, Renown) plus troop count and domain limit —
  always-visible vitals, same "ambient truth strip" pattern as Victoria 3's
  top bar. A multicolored **Issues icon** sits left of gold and doubles as
  both a warning light (succession problems, missing heir) and an
  opportunity feed (available wars, ransom offers, claims).
- **Bottom-left — the Character Panel.** Clicking the ruler's 3D portrait
  opens a panel (not a fullscreen takeover) containing: portrait + consort
  attitude indicators, personality traits, five attributes + prowess,
  religion/culture affiliations, main title + government type, resource/army
  summary, domain-limit gauge, titles-held and claims lists, and **four
  relationship tabs** (family, social relations, court, vassals). A small
  **stress meter** sits beside the portrait — acting against your character's
  traits raises stress, which risks a "mental break." This panel is the
  spine of CK3's character-first IA: almost everything else in the game is
  reachable *from* a character, not just from the map.
- **Right edge — seven icon tabs + Outliner.** Kingdom (domain/vassals/
  succession law, 3 sub-tabs), Military (armies/mercenaries/orders, 3
  sub-tabs), Council, Court, Intrigues, Factions, Decisions — each opens a
  slide-in panel, map stays visible behind it. An **Outliner icon** above
  these gives one-click access to manually-pinned armies, counties, and
  characters — CK3's answer to Victoria 3's auto+manual outliner split, but
  narrower (mostly manual pins; no auto-surfaced "situations" list the way
  Victoria 3 does).
- **Bottom-right — time/meta bar.** Current date, speed controls, and
  buttons for encyclopedia, settings, game rules, multiplayer status, and
  character-switching (for playing multiple characters across a session);
  map-mode picker also lives here.
- **Center — the 3D map**, at three zoom tiers (see §2), always live and
  always clickable; nothing in CK3 blocks map interaction except modal event
  popups.

**Sources:** [Interface — CK3 Wiki](https://ck3.paradoxwikis.com/Interface),
[Interface Guide — gamepressure.com](https://www.gamepressure.com/crusader-kings-3/interface-description/z2f0f6),
[Console Dev Diary #3: UI/UX and Controls](https://www.paradoxinteractive.com/games/crusader-kings-iii/news/ck3-console-dev-diary-3-uiux-and-controls)

**Babylon read:** CK3's structure maps almost 1:1 onto the Babylon chrome
vocabulary already in the design brief: top bar's resource strip →
`TopBar` (Imperial Rent gap, national radicalization index, active
contradictions); the character panel → `InspectionStack` anchored on a
selected class/organization/key-figure node instead of a ruler; the
right-edge icon tabs → `ActionDock` (mobilize/educate/campaign/etc. as the
seven verbs, each opening a slide-in, not a takeover); the Outliner icon →
`OutlinerOverlay` for pinned organizations/hexes/contradictions; event
popups → `EventToasts`/wire takeover for Rupture events. The one gap CK3
doesn't fill (see §6) is a *reason-for-alert* explanation on its Issues
icon — Babylon's alert surface should say **why** a contradiction fired, not
just that it did.

---

## 2. Map: zoom tiers, modes, and the de jure/de facto border duality

### Three zoom tiers, each a different visual register

CK3's map deliberately **changes rendering style, not just detail, per zoom
level** — this is the single most transferable idea for Babylon's "hexes are
just deep-zoom tiles, county borders are the starting cartography" mandate:

1. **Far zoom — the "paper map."** The world map becomes a **stylized
   medieval parchment on a table**, with painterly decoration (ships, sea
   monsters at map edges) — explicitly designed for "an easy overview and
   stylish screenshots," and it visually marks the map's edge-of-knowledge
   (where the game world ends) as an *in-fiction* frayed parchment edge
   rather than a hard UI cutoff.
2. **Mid zoom — the "political map."** 3D terrain reappears, realms are
   colored fills with clean typography for realm/title names — this is the
   "diplomatic gameplay" register, the one most players spend the most time
   in.
3. **Near zoom — the "detail map."** Realm color washes are stripped away
   entirely; county names, individual baronies, and terrain classification
   become readable — the "tactical" register for army movement and building
   placement.

**Source:** [CK3 Dev Diary #2 — The Medieval Map (search summary)](https://forum.paradoxplaza.com/forum/developer-diary/ck3-dev-diary-2-the-medieval-map.1274052/),
[Crusader Kings 3 Map — GameWatcher](https://www.gamewatcher.com/crusader-kings-3-map)

**Babylon read:** this directly validates the mandate's "county borders as
starting colonial cartography → hexes as deep-zoom tiles" structure — CK3
proves that *changing the rendering language itself* (parchment → clean
political fill → stripped-down tactical) at each zoom band reads as
intentional game-world detail rather than a LOD hack, and gives the player a
free "you have zoomed into a different kind of truth" cue. Babylon could
use: national/continental zoom = a stylized "situation map" (matches Cold
Collapse's dark/cyan palette instead of parchment); state/county zoom = the
political fill map (faction/class-control coloring); hex zoom = the
tactical/production detail register.

### Map modes: 11 total, grouped 7 primary + 4 secondary

The **lower-right corner** exposes seven primary map modes as always-visible
buttons: **Realms, Empire Titles, Kingdom Titles, Duchy Titles, Houses,
Cultures, Faiths.** A **plus-button toggle** reveals four secondary modes:
**Counties, Terrain, Development, Government.** Striped/hatched fill on a
county is the recurring visual grammar for "this county's culture or faith
differs from its liege's" — a cheap, at-a-glance way to show latent tension
without a separate mode.

**Source:** [Crusader Kings 3 Map — GameWatcher](https://www.gamewatcher.com/crusader-kings-3-map),
[Map modes — gamepressure.com](https://www.gamepressure.com/crusader-kings-3/map-modes/z3f0f7)

**Babylon read:** the **primary/secondary split with a "+" overflow** is a
clean pattern for Babylon's map-lens picker — put the lenses players check
constantly (class control, faction control, contradiction intensity) as
always-visible buttons, and bury denser/rarer ones (terrain, development,
government type equivalents) behind one overflow toggle instead of a long
dropdown. The **striped-fill "latent mismatch" convention** is directly
reusable for Babylon: a hex/county whose dominant class differs from its
controlling faction's ideology, or whose solidarity network crosses a
border, could use the same hatch treatment instead of a whole extra lens.

### De jure vs. de facto — the border duality Babylon needs most

CK3's default map mode shows **de facto** control — who actually holds each
county right now. Switching to the Duchy/Kingdom/Empire **Titles** modes
shows **de jure** land instead — "which lands you need to conquer to obtain
a given title, not which lands are currently included in it." A county can
be de jure part of a kingdom while de facto ruled by someone else entirely;
this mismatch is core to CK3's whole succession/conquest game (a "De Jure
Drift" mechanic slowly reassigns which kingdom a county is de jure part of,
based on who's actually held it for a long time).

**Source:** [De Jure Drift — Steam Community guide](https://steamcommunity.com/sharedfiles/filedetails/?id=2290291243),
[Map modes — gamepressure.com](https://www.gamepressure.com/crusader-kings-3/map-modes/z3f0f7)

**Babylon read:** this is the single most load-bearing transferable concept
for the "borders redraw as revolution progresses" mandate. Babylon's map
should support **two simultaneously-legible border layers**: (1) *de facto*
— who currently exercises control on the ground (garrisons, organization
presence, class control of a hex), the default view; and (2) *de jure* —
the "official"/colonial/claimed political map (state lines, national
borders as currently recognized) that a revolutionary movement is trying to
override. A liberated county that's still nominally part of a
collapsing/reactionary state is exactly CK3's de-jure-vs-de-facto mismatch,
and the striped-fill convention (§ above) is the right visual language for
it. CK3's "drift" mechanic (borders slowly reassign based on sustained
control) is also a good model for how Babylon might *animate* the eventual
border redraw rather than snap it instantly on a Rupture event.

---

## 3. The Nested Tooltip System — interaction design

This is CK3's most-cited UX contribution to the genre, later adopted
wholesale by Victoria 3 for its economy ("Victoria 3 will use Crusader
Kings 3's tooltip system so you don't need an econ degree" — PCGamesN).

**Mechanics, as documented:**
- **Trigger: hover**, not click. Any highlighted (usually colored/underlined)
  term in a tooltip's own body text is itself hoverable.
- **Recursion:** hovering a highlighted term *inside* an already-open
  tooltip spawns a **second tooltip flying out from that word**, which can
  itself contain further highlighted terms — "you can keep digging through
  the game's knowledgebase indefinitely this way." In practice the design
  literature notes recursion has to be depth-limited by content design
  ("an infinitely deep tooltip chain doesn't make sense") even though the
  UI technically supports arbitrary nesting.
- **Effect:** the tooltip system becomes "the game's official wiki sitting
  right alongside your mouse pointer" — definitions, formulas, and modifier
  breakdowns are all reachable without alt-tabbing to a wiki, because every
  number that comes from a calculation shows its inputs, and every unusual
  term used in that breakdown is itself a hover target.
- A UX blog analysis (Philip Ardeljan) frames the pedagogical case for the
  pattern directly: "when learning something new, there might be a
  supporting concept or idea I don't fully grasp. A nested tooltip is a
  great solution" — but the same piece flags known weaknesses: **cursor
  precision** (moving the mouse into a nested tooltip risks dismissing the
  parent one before you get there), **no accessibility fallback** (the
  interaction is fundamentally hover/mouse-based, hostile to touch and to
  keyboard-only or motor-impaired play), and the **unbounded-depth
  question** (the pattern needs an editorial limit, not just a technical
  one).

**Sources:** [Tooltips in tooltips — Philip Ardeljan](https://philip.design/blog/tooltips-in-tooltips/),
[Interface — CK3 Wiki](https://ck3.paradoxwikis.com/Interface) (`.gui`
container/child architecture — tooltips are built from the same
nestable-container primitives as every other window),
[Victoria 3 will use Crusader Kings 3's tooltip system — PCGamesN](https://www.pcgamesn.com/victoria-3/nested-tooltip-system)

**Babylon read:** this is a near-perfect fit for Babylon's "Victoria-3-style
nested recursive inspection panels where every number explains itself"
mandate — and the mandate is right to phrase it as *panels* (Victoria 3's
evolution of the pattern) rather than literal flyout tooltips, precisely
because of the two weaknesses above. Recommendation: build Babylon's
`InspectionStack` as **pinned, stacked panels** (click to push a new panel
onto the stack, not hover-to-fly-out) — this sidesteps both the cursor-
precision problem and the accessibility gap, while keeping CK3's real
insight, which is *recursive drill-down on any number*: Imperial Rent Φ
should show its Wc/Vc inputs, and Wc itself should be clickable to show its
own formula inputs, arbitrarily deep, with a **hard depth limit** (CK3's own
practice — "not infinite") set by what a coefficient tree in `GameDefines`
actually needs, likely 3-4 levels for something like the Fundamental
Theorem stack.

---

## 4. Event Popups — narrative device

Events surface as **modal-ish pop-up notifications**: either presenting a
branching choice, or simply informing the player of consequences with an
acknowledge button. Two trigger families exist — **triggered-only events**
(direct consequence of an action, fire "immediately or within a short
amount of time") and **pulse events** (recurring background events on
weighted timers, from yearly up to five-year intervals, used for ambient
world texture). Portraits in event windows are **3D and animated**, with
multiple portrait "slots" so an event can stage more than one character at
once (e.g., a plotting rival and a pleading spouse in the same event). A
quiet but important accessibility/pacing detail: **if the player leaves a
choice event untouched for four in-game months, the game auto-selects the
first option** rather than blocking forever — events don't hard-pause the
simulation indefinitely.

**Sources:** [Events — CK3 Wiki](https://ck3.paradoxwikis.com/index.php?title=Events),
[Console Dev Diary #3: UI/UX and Controls](https://www.paradoxinteractive.com/games/crusader-kings-iii/news/ck3-console-dev-diary-3-uiux-and-controls)

**Babylon read:** Babylon's "wire" news feed is closer to CK3's *pulse
events* (ambient, weighted, recurring texture) than to its *triggered
choice* events — but CK3's pattern of **staging animated portraits inside
the popup itself** is a strong argument for Babylon's Rupture-event
"wire takeover" to show the actual key-figures/organizations involved
(even as simple iconography, not full 3D) rather than pure text, since
CK3's data suggests character presence is what makes an event register as
*consequential* rather than ambient. The **4-month auto-resolve safety
valve** is a good low-cost pattern for Babylon's own choice-bearing wire
events (if any exist) — never let an unanswered popup silently stall
simulation time.

---

## 5. Character-First IA over a Map Game

CK3's structural bet — argued at length in genre criticism (e.g. Bret
Devereaux's "Teaching Paradox" series on CK3, widely discussed in strategy-
game design circles) — is that **the map is the stage but characters are the
protagonists**: nearly every panel (Council, Court, Intrigues, Factions,
even Military via commander portraits) is reached *through* a character
rather than through the map or a spreadsheet. The practical UI consequence
is that the character panel (bottom-left, §1) isn't one tab among many — it
is the hub every other system hangs off of, and the map is mostly used to
answer "where is this person/army/county," not "who controls what" in the
abstract.

**Babylon read:** Babylon's material is classes/factions/territories, not
individual nobles, so a literal character-hub port doesn't fit — but the
underlying lesson generalizes: **give every abstract number a face.**
Babylon already has `key_figure` nodes in the topology (per
`ai/architecture.yaml`); CK3's pattern argues for surfacing key figures
*prominently* in `InspectionStack` when inspecting an organization or class
node (who leads this union, who's the fascist demagogue driving this
faction) rather than treating key-figures as a buried node type — a
Φ-gap number is abstract, but "General Secretary X's organization is
losing solidarity edges" is legible the way CK3 makes politics legible.

---

## 6. Onboarding & the Suggestions Gap

### The measured tutorial rewrite

Paradox's own Game Developer deep dive is unusually candid and metrics-
driven. The **original tutorial** was "67 sequential boxes full of
explanatory information" — and telemetry showed **completing the tutorial
was not a significant predictor of whether a player kept playing**, meaning
the exhaustive version wasn't earning its length. The rewrite:
- Cut to roughly **20–22 key messages** (about a third of the original),
  applying **progressive disclosure** explicitly — e.g. gradually revealing
  fields in the character panel rather than showing the whole panel at
  full complexity on first open.
- Reframed remaining tutorial content as **narrative events** (using the
  game's own event system, not a separate overlay) — teaching mechanics
  *in-fiction* rather than via a modal instruction box.
- Used **the live game itself as the prototyping tool** to minimize the gap
  between prototype and shipped feature.
- Kept **both old and new tutorials live simultaneously** post-launch
  specifically to A/B the telemetry — "knowing what doesn't work can be
  valuable" was treated as a legitimate outcome, not a failure.

**Source:** [Deep Dive: Refreshing the Crusader Kings III tutorial mode
through optimized UX — Game Developer](https://www.gamedeveloper.com/design/deep-dive-refreshing-the-crusader-kings-iii-tutorial-mode-through-optimized-ux)

### Devs' own stated interface-accessibility philosophy

Paradox programmer Matthew Clohessy, on the broader interface-accessibility
push: *"We have put a lot of effort into making the interface more
accessible. All so that the difficulty and complexity of the game would
depend on how the game is played and the use of good strategies instead of
things like looking for a number you need, bringing up the tooltip to get
some information, or googling what a particular term means."* — i.e., the
explicit design target is to push all difficulty into **strategy**, and
zero difficulty into **information retrieval**. Concrete levers cited:
expandable tooltip hints, a rebuilt tutorial, and better highlighting of
events that need immediate attention ("issues that require immediate
player's attention are properly highlighted so that players don't miss
them by accident").

**Source:** [Crusader Kings 3 devs want interface to be more accessible —
gamepressure.com](https://www.gamepressure.com/newsroom/crusader-kings-3-devs-want-interface-to-be-more-accessible/z61a22)

### The gap: no ongoing suggestions/advisor system

Notably, and unlike some later Paradox titles, CK3 has **no built-in
ongoing "suggested action" or advisor system** that proactively recommends
what to do next after the tutorial ends — onboarding is front-loaded into
the linear tutorial/narrative-event sequence, then the player is on their
own with the Issues icon (§1) as the only standing "here's what you could
do" surface. This is a real gap relative to genre peers and worth Babylon
noting as something to *improve on*, not copy.

**Babylon read:** three actionable lessons. (1) **Measure, don't assume** —
Babylon should instrument its own onboarding (however it ends up designed)
with the same "does finishing it predict retention" telemetry lens CK3
used, rather than trusting tutorial-length as a proxy for effectiveness.
(2) **Progressive disclosure belongs in the panels, not just the tutorial**
— CK3's rewrite explicitly extended progressive disclosure into the
character panel itself (gradual field reveal), which argues Babylon's
`InspectionStack` should default new/first-time players to a reduced field
set per node type, expanding as the player demonstrates engagement, rather
than dumping every coefficient at once. (3) **Close CK3's own gap** — build
a lightweight standing "why is this contradiction escalating / what verb
would help" suggestion surface into `ActionDock` or the alert stream,
something CK3 never shipped and its community still asks for via mods
(Steam Workshop "Better AI"/advisor-style mods exist precisely because
the base game doesn't do this).

---

## 7. Community UX Criticism — top complaints

1. **Borders/fills "all look the same" across scales.** A frequently-cited
   Steam Community complaint ("HATE THE UI" / "Fairly Detailed List on Why
   I HATE The UI" threads) is that barony/county/duchy/kingdom/realm color
   fills are visually similar enough that players struggle to tell which
   political layer they're looking at at a glance, especially before
   learning the zoom-tier conventions in §2. **Babylon implication:** each
   Babylon map-lens/zoom-tier needs a genuinely distinct visual register
   (not just a palette swap) — exactly the "different rendering style per
   zoom band" lesson from §2, taken as a warning as well as a technique:
   CK3 sometimes reuses similar fills *across* those registers and players
   notice.
2. **Overwhelming information density with insufficient early guidance.**
   Multiple reviews (Checkpoint Gaming, forum "reality check" threads)
   describe CK3 pre/at-launch as "a mess of menus" where "an overwhelming
   amount of information [is] on screen between the side panels and the
   alerts tab at the top," compounded by a beginner tutorial that "only
   taught bare basics." This is the same problem CK3's own §6 tutorial
   rewrite was commissioned to fix — i.e., even Paradox agreed the
   complaint was valid. **Babylon implication:** front-load Babylon's own
   progressive-disclosure discipline rather than treating "the math is
   complex, so the UI must be complex" as inevitable — CK3's post-launch
   telemetry proved that assumption wrong for its own tutorial.
3. **No built-in accessibility options (notably colorblind support).**
   Community and Steam Workshop activity (a dedicated "Colorblind Friendly"
   mod and a "Visual Impairment Accessibility Support" mod, plus repeated
   forum requests — "Colorblind mode, again...") indicate CK3 shipped
   without native colorblind-safe palettes for distinguishing enemy/own/
   ally borders and troops, forcing reliance on third-party mods (which
   also breaks multiplayer parity, since CK3 disables mods in multiplayer
   unless all players share them). **Babylon implication:** the ratified
   "Cold Collapse" dark/cyan palette should be checked against a colorblind
   simulator *before* ship, particularly for any faction-control or
   solidarity/contradiction-intensity color coding on the map — this is a
   cheap, durable fix relative to retrofitting it post-launch the way CK3's
   community had to via mods.

**Sources:** [HATE THE UI — Steam Community](https://steamcommunity.com/app/1158310/discussions/0/2914346777826585083/),
[Fairly Detailed List on Why I HATE The UI — Steam Community](https://steamcommunity.com/app/1158310/discussions/0/2943620809065843131/),
[Crusader Kings III Review — Checkpoint Gaming](https://checkpointgaming.net/reviews/2020/09/crusader-kings-iii-review-a-worthy-heir/),
[Crusader Kings III: UI, balance, functionality, bugs, launch — reality
check (Paradox forums)](https://forum.paradoxplaza.com/forum/threads/crusader-kings-iii-ui-balance-functionality-bugs-launch-reality-check.1417859/),
[Colorblind Friendly — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=2224739277),
[Visual Impairment Accessibility Support — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=2225500765),
[Colorblind mode, again... — Paradox forums](https://forum.paradoxplaza.com/forum/threads/colorblind-mode-again.1420804/)

---

## 8. Console Port — a natural experiment in "focus switching"

CK3's console adaptation (Lab42, covered in Console Dev Diary #3) is a
useful natural experiment because it had to solve "how do you navigate a
PC-dense information game without a mouse" — directly relevant if Babylon
ever needs a controller/keyboard-only navigation mode for its own
`InspectionStack`/`OutlinerOverlay` stack:

- **Rejected** using a thumbstick as a literal virtual-mouse overlay, after
  research showed the D-Pad was the preferred navigation input and needed
  to stay free for direct game interaction rather than being consumed by
  menu-cursor duty.
- **Control hierarchy:** shoulder buttons for top-level menu switching,
  bumpers for sub-menu navigation, with **context-specific control prompts
  always shown at the bottom of the screen** rather than a fixed universal
  button legend.
- **"Focus switching"** as the core innovation: a dedicated mechanism to
  jump between any currently-open screen element and the map itself,
  without closing/reopening panels — explicitly framed as preserving the
  PC version's "frequent menu-flicking" play pattern console players
  specifically said they wanted kept, rather than "simplified away."
- **Two radial wheels** (Character Wheel, Map View Radial) for fast modal
  switching (map modes, character-context actions) "without in-depth menu
  navigation" — a console-native alternative to hovering over small
  top-bar icons.

**Source:** [Console Dev Diary #3: UI/UX and Controls — Paradox
Interactive](https://www.paradoxinteractive.com/games/crusader-kings-iii/news/ck3-console-dev-diary-3-uiux-and-controls)

**Babylon read:** lower priority (Babylon is a desktop/web cockpit, not
console-bound), but the **"focus switching" principle** — a single
consistent gesture to bounce between "whatever panel is open" and "the map
under it" without a close/reopen round-trip — is worth stealing directly
for keyboard power-users of `InspectionStack`, independent of any
controller support.

---

## Summary Table — CK3 Pattern → Babylon Surface

| CK3 pattern | Babylon surface | Verdict |
|---|---|---|
| Nested hover tooltips (recursive, depth-limited) | `InspectionStack` | Adapt to click-pinned panels, not hover-flyouts (accessibility) |
| Top bar resource strip + Issues icon | `TopBar` + alert surface | Adopt; add *why* explanations CK3 lacks |
| Character panel as universal hub | `InspectionStack` on key_figure nodes | Adapt: surface key figures inside class/org/faction inspection |
| Right-edge verb tabs (Kingdom/Military/Council/...) | `ActionDock` (9 verbs) | Direct structural match |
| Outliner icon (manual pins only) | `OutlinerOverlay` | Adopt, but add Victoria-3-style auto-pinned "situations" too |
| 3 zoom tiers, distinct rendering per tier (parchment/political/tactical) | map lens system | Adopt directly — core to the "hexes as deep-zoom tiles" mandate |
| 7 primary + 4 secondary (+button) map modes | map lens picker | Adopt the primary/overflow split |
| De jure vs. de facto border duality + striped mismatch fill | border-redraw system | Adopt directly — core to "borders redraw as revolution progresses" |
| Animated multi-portrait event popups | `EventToasts` / wire takeover | Adapt: stage key-figures/orgs in Rupture events |
| 4-month auto-resolve on unanswered events | wire choice events (if any) | Adopt as a safety valve |
| 67-box tutorial → 20-22 narrative-event tutorial, telemetry-validated | onboarding design | Adopt the *measurement methodology*, not just the shorter number |
| No standing suggestions/advisor system | `ActionDock` alert stream | Explicitly improve on — this is a known CK3 gap |
| No native colorblind support | Cold Collapse palette QA | Explicitly improve on — check before ship, not after |
| Console "focus switching" between panel and map | `InspectionStack` keyboard nav | Adopt the gesture, skip the controller-specific parts |
