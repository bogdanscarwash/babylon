# Victoria 3 — UI/UX Deep Dive for the Babylon Living Map

**Research date:** 2026-07-11
**Why Victoria 3 matters most to Babylon:** it is the closest genre sibling — a
society/economy simulation (pops, classes, production chains, standard of
living, radicals vs. loyalists, revolutions) that also tries to be a *map
game* rather than a pure spreadsheet, and it has a public, well-documented
five-year iteration story (dev diaries #30 → #74+) on exactly the tension
Babylon faces: how much economic/political truth to expose, and how to keep
a deeply nested data model from turning the map into wallpaper behind a wall
of panels.

---

## 1. Overall Information Architecture

Victoria 3's screen is a **fixed chrome frame around a 3D globe**, not a
series of full-screen takeovers:

- **Top bar (two rows).** Row 1: country flag/rank (click → country tab) +
  four capacity meters (Bureaucracy, Authority, Influence, Convoys) each
  shown as a fill gauge with surplus/deficit scaling, + treasury balance.
  Row 2: GDP, Literacy %, Standard of Living, total Population,
  Radicals/Loyalists counts, Infamy. This is the "vitals strip" — the whole
  nation's health in one glance, always visible, never a click away.
- **Left bar.** Twelve icon tabs (Politics, Budget, Buildings, Market,
  Military, Power Blocs, Diplomacy, Technology, Society, Population,
  Journal, Companies, Ledger) — each opens a **full detail panel that slides
  in from the left**, not a new screen; the map stays visible and playable
  behind/beside it.
- **Bottom bar.** Six lens/action buttons (location search, Production,
  Political, Diplomatic, Military, Trade) plus a **map-mode picker** —
  the primary way the player *acts on the map* rather than through menus.
- **Right side.** An **outliner** — auto-pinned "situations" (active laws,
  diplomatic plays, revolutions, ongoing events) at the top, manually-pinned
  items (armies, tracked goods, tracked characters) below that, and a
  **scrolling event/news feed** in the bottom-right corner.
- **Center.** A real 3D globe that, per Rock Paper Shotgun's review, "sits in
  the middle of the screen for almost the entire time you play, but you
  almost never have to click on it" — the map is often ambient/spectacle
  rather than the primary interaction surface, because most of the *verbs*
  live in the side panels and lenses.

**Source:** [User interface — Victoria 3 Wiki](https://vic3.paradoxwikis.com/User_interface),
[Dev Diary #30 — User Interface Overview](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-30-user-interface-overview.1507166/),
[Victoria 3 review — Rock Paper Shotgun via search summary](https://www.rockpapershotgun.com/victoria-3-review)

**Babylon read:** this is close to the mandate — "floating game chrome" over
a full-bleed map. The key structural lesson is the **two-tier split between
ambient vitals (top bar) and drill-down panels (left bar)**: nothing
important is *only* in a panel — the top bar always shows the headline
numbers, and panels are where you go to find out *why* a headline number is
what it is. Babylon's `TopBar` should carry the same "vitals strip" role
(imperial rent gap, national radicalization index, active contradictions
count) with every number tap-through into `InspectionStack`.

---

## 2. Map Modes / Lenses — structure and switching

Victoria 3 separates **static map modes** (informational, no side effects)
from **action lenses** (tool modes that change what building/army/decree
buttons do). This is a meaningful distinction Babylon should keep explicit.

### The five action lenses (bottom bar, always visible)
1. **Production Lens** — construct production buildings; shows GDP and state
   traits.
2. **Political Lens** — construct government buildings, issue decrees, take
   state actions (incorporate states, move capital); shows political
   strength/active decrees.
3. **Diplomatic Lens** — declare interests, establish colonies, conduct
   diplomacy; shows attitudes, wars, diplomatic plays.
4. **Military Lens** — create armies/fleets, set strategic objectives, plan
   naval invasions; shows army/fleet positions.
5. **Trade Lens** — build infrastructure (construction sectors, ports,
   railways), establish trade routes; shows local goods prices.

Each lens both **changes the cursor/tool behavior** (what clicking a
territory does) and **recolors the map** to show only the data relevant to
that tool (a Paradox-standard "mode = filter + tool" fusion). Ineligible
targets are greyed/orange, eligible ones green, in-progress blue — a
consistent five-color eligibility grammar (orange = ineligible, green =
available, blue = in progress, red = currently ineligible, white = cannot
easily meet conditions) reused across every lens rather than invented per
lens.

### The ~18 pure map modes (opened via the small world-map button next to
the trade lens; picking one also opens the matching ledger page)
Countries at Peace (default), Country Attitude, Country Relations, Cultures
Overview, Population, Literacy, Global GDP, National GDP, Markets,
Migration, Standard of Living, Loyalists, Radicals, Pollution Impact,
Infamy, Military, States, Strategic Regions — plus context-only overlays
(Budget, Local Goods Prices, National Cultures, Interest Groups) that open
automatically when you open a related panel rather than living in the main
picker.

**Source:** [Map modes — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Map_modes),
[User interface — Victoria 3 Wiki](https://vic3.paradoxwikis.com/User_interface)

**Babylon read:** Babylon's map-lens toolbar should mirror the **lens =
tool+filter fusion**, not a bare palette dropdown. Concretely: an
"Organize" lens (mobilize/educate/campaign) recolors hexes by solidarity
density and constrains click-targets to valid organize actions; an "Attack"
lens recolors by control-ratio/repression and constrains targets to
contested hexes; a pure "Inspect" mode (imperial rent gradient, contradiction
intensity, class composition) needs no tool attached at all. Keep the
eligibility color grammar (available/ineligible/in-progress) identical
across all lenses so players learn it once.

---

## 3. The Nested Tooltip System (the single most citable Vic3 pattern)

Victoria 3 (following Crusader Kings III) treats **every highlighted value
in the UI as a hyperlink into its own explanation**. Hovering a value opens
a flyout tooltip; that tooltip itself contains further highlighted terms;
hovering those opens a further nested flyout, recursively, "so you don't
need an econ degree" to understand what "market access" or "dividend taxes"
or "pop qualifications" mean while playing.

> "If you hover over any highlighted text in the game, a tooltip will fly
> out from your mouse... Within that text, you'll see more highlighted
> terms, which will in turn lead to more tooltip flyouts... You can keep
> digging through the game's knowledgebase indefinitely this way — it's
> basically like having the game's official wiki sitting right alongside
> your mouse pointer."

Paradox's stated intent (per their own messaging): *"Our intention is still
to allow you to learn even the most advanced concepts the game is based on
as you play,"* pairing the tooltip web with a stronger tutorial rather than
treating either as sufficient alone.

**Dev Diary #74 (UX Improvements, post-launch)** documents concrete
iteration on this system once it shipped:
- **Positioning fix.** The Map Interaction panel was "restructured to make
  it possible to navigate directly to another panel," specifically to stop
  players **accidentally triggering a different tooltip while moving the
  mouse toward the nested tooltip they actually wanted** — a real,
  named failure mode of infinite nested flyouts.
- **Notification-count reduction (~50%)** by consolidating low-value
  messages (e.g. merging "Interest Activated"/"Interest Deactivated" spam
  into one actionable "You can now conduct Diplomacy" alert).
- **Information reorganization**: surfaced high-value fields higher in the
  visual hierarchy of a panel and collapsed/hid low-value fields by default,
  rather than showing everything flat.
- **Explicit causal tooltips** replacing vague ones — e.g. market-access
  tooltips now *explain why* a state lacks full connectivity instead of
  just stating the number; battle tooltips now show troop-calculation
  math and which commander/HQ is responsible for what.
- **Pop-panel overhaul**: pop counts, earnings, spending, and
  standard-of-living impact all visible without scrolling, using
  collapsible sections instead of a wall of rows.
- **Outliner got richer inline data** (states show available labor +
  qualifications; interest groups show approval rating inline) so fewer
  clicks were needed to get the number you actually wanted.
- **Pinning** was added for goods and characters so recurring lookups don't
  require re-navigating the nested chain every time.

**Source:** [PCGamesN — nested tooltip system](https://www.pcgamesn.com/victoria-3/nested-tooltip-system),
[Dev Diary #74 — UX Improvements](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements)

**Community reception is genuinely split**, which matters for Babylon:
- Praise: "a great innovation and step forward for Paradox games... once
  players examine nested tooltips a few times and gain knowledge, they
  become completely unobtrusive."
- Complaint (forum thread "Reminder to the devs that nested tooltips is bad
  UX design"): the recurring criticism is that **nested tooltips force
  players to re-derive information every time** rather than surfacing it
  persistently — you have to hover-chase the same three-deep chain
  repeatedly instead of the game remembering you already looked once.
  Multiple post-launch reviews (Kotaku-style aggregation, Destructoid)
  echoed this as: "the information needed is fragmented, spread across many
  screens and tabs, sometimes hidden deep in nested tooltips, and at no
  point does the player ever have all the information they need readily at
  hand."

**Babylon read (high-value pattern, with the failure mode named):**
`InspectionStack` should implement the **recursive drill-down** ("every
number explains itself") but must avoid Vic3's launch mistake of making
*every* lookup transient-hover-only. Two concrete mitigations, both of which
Vic3 itself converged on post-launch:
1. **Pin-to-panel**, not just hover-flyout — let the player promote any
   nested tooltip node into a persistent stacked panel (this is exactly
   what `InspectionStack`'s "nested recursive inspection panels" framing
   already implies; make pinning a first-class, discoverable gesture, not
   a secondary feature bolted on in patch 74).
2. **Cursor-path-aware flyout placement** — Vic3's #74 fix for "opening the
   wrong tooltip while reaching for the nested one" is a concrete
   implementation detail worth carrying over literally: flyouts should open
   in the direction that doesn't force the mouse to cross other hoverable
   elements.

---

## 4. "Panels-over-map" IA and the spreadsheet tension

Victoria 3's own 2D Art Lead (Kenneth Lim, Dev Diary #30) frames the UI as
three primitives — **Panels, Buttons, Icons** — with a stated design rule:
*"a high level of detail with intricate elements, but used sparingly so it
does not become cluttered and overwhelming,"* colour-coded (green = positive
condition, red = negative) for at-a-glance scanning, Art Nouveau/Victorian
ornamentation reserved for chrome (not data), and infographics used
specifically where "menus are very text heavy" to replace prose with a
visual.

Despite that stated intent, **the recurring critical consensus at launch and
after** was that Victoria 3 did not fully escape the "spreadsheet with a
skin" problem endemic to the genre:

- Destructoid: *"Victoria 3 wants to be a social simulation, but it plays
  like accounting software... poring over numbers, adjusting those numbers,
  and seeing if you get new (and better) numbers in return."*
- Multiple reviews independently converged on the same complaint pattern:
  *"very good and at times very convoluted, with many things involving
  multiple menus that can leave players lost."*
- The **technology tree** was singled out repeatedly as the worst offender
  — described as "octopian," "so bad" one reviewer resorted to picking
  techs at random, "a cramped mess" on smaller displays — a cautionary
  example of a *dense relational structure rendered as a literal node graph
  with no progressive disclosure*.
- The **trade panel** was criticized for prioritizing charm over legibility:
  "pretty pictures of all the items flowing in and out of your nation, which
  is charming until you realize the icons are in the way" — decoration that
  actively degrades the number you came to read.
- Late-game **event/pop-up volume**: "an endless deluge of pop-ups for so
  many countries and events took up a lot of screen space" — the
  Journal-entry system (Vic3's replacement for classic Paradox event
  pop-ups, designed to avoid arbitrary historical scripted events firing at
  a fixed date regardless of state) still produced volume problems once the
  world got large and eventful.

**Source:** [Dev Diary #30](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-30-user-interface-overview.1507166/),
[Destructoid review](https://www.destructoid.com/reviews/review-victoria-3-pc-paradox-interactive-strategy/),
[Inverse review](https://www.inverse.com/gaming/victoria-3-review-pc),
search aggregation of Kotaku/Metacritic/community threads.

**Babylon read:** the stated Art Nouveau design rule ("ornament the chrome,
never the data") is exactly right and worth adopting verbatim for Cold
Collapse — cyan accents and chrome ornamentation belong on panel *frames*,
never layered on top of numbers or icons a player needs to read quickly.
The tech-tree failure is a direct warning for anything in Babylon that
threatens to become a literal graph-of-everything screen (e.g. a
contradiction-web view, or an organization-relationship map) — those need
progressive disclosure (collapse by category, expand on demand) rather than
one giant node-and-edge diagram dumped on screen at once.

---

## 5. Pops/Buildings drill-down chain (closest analog to Babylon's class/hex model)

Victoria 3's pop→building→market chain is architecturally the nearest
sibling to Babylon's social-class/territory/production graph:

- **Buildings panel**: grouped by type per state, expandable to individual
  buildings; **each building opens an in-depth balance-sheet + workforce
  view**; production methods can be set per-building or bulk-applied to all
  buildings of that type nationwide (a "set all" shortcut against
  drill-down fatigue).
- **Predictive tooltips** on production-method changes show the *anticipated*
  balance-sheet and employment delta *before* the player commits — i.e. the
  UI simulates the consequence and previews it inline rather than making the
  player guess and check next tick.
- **Market system**: goods never move producer→consumer directly; every
  transaction is an abstracted buy/sell order against a market-wide price.
  Every building and every pop independently generates buy/sell orders,
  and the **Market panel** (plus its own dedicated map mode, "Local Goods
  Prices") is where the player inspects price formation.
- **Pop Needs** panel (post-#74 rework) surfaces standard-of-living impact
  and need-satisfaction inline in the pop list itself, rather than requiring
  a separate click per pop to see whether needs are met — the fix was
  specifically about "surfacing relevant details higher in the visual
  hierarchy" instead of nesting them one click deeper.

**Source:** [Building — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Building),
[Market — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Market),
[Needs — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Needs),
Dev Diary #74.

**Babylon read:** two directly portable patterns —
1. **Predictive/preview tooltips on action commit** — before a player
   confirms "mobilize" or "attack," `InspectionStack` should preview the
   projected delta to solidarity/control-ratio/repression, the same way
   Vic3 previews a production-method swap's balance-sheet impact.
2. **Bulk-apply escape hatch** — Vic3's "set all buildings of type X"
   exists precisely because per-instance drill-down doesn't scale once a
   player owns 200 buildings; Babylon's `ActionDock` should offer an
   equivalent for organizations managing many hexes/classes at once
   (e.g. "apply this campaign to all contested hexes in this state")
   so the recursive inspection model doesn't become a click-tax at scale.

---

## 6. Alerts / Outliner / Time controls (chrome mechanics)

- **Outliner** (right side): auto-pinned "situations" (active laws,
  diplomatic plays, revolutions, ongoing events) always at top; below that,
  manually pinned items (armies, tracked goods/characters); a "current
  situations" button shows a badge count and **glows gold when new
  information is available** — a single ambient attention cue rather than a
  modal interrupt.
- **Alerts are color-coded by priority** (red = high priority), grouped by
  category, individually dismissible, with a **refresh/undo** to bring back
  a dismissed alert — i.e. dismissal is reversible, reducing the fear of
  losing information by clearing clutter.
- **Event/news feed**: a small scrolling ticker bottom-right for
  foreign/domestic events, separate from the modal Journal-entry popups —
  ambient narrative flow vs. blocking decision points are two different UI
  surfaces, not one conflated feed.
- **Time controls**: top-right date bar with week/day-phase granularity
  (morning/afternoon/evening/night, sun/moon icon), Roman-numeral speed
  selector I–V, Space to pause, numpad 1–5 for speed — game starts **paused
  by default** to let the player get oriented before the clock runs.

**Source:** [User interface — Victoria 3 Wiki](https://vic3.paradoxwikis.com/User_interface)

**Babylon read:** the "glow badge, don't interrupt" pattern for the
outliner and the "reversible dismiss" pattern for alerts are both directly
applicable to `OutlinerOverlay`/`EventToasts` — avoid modal event popups for
anything below a defined severity threshold, and make toast dismissal
undoable via a small "recent" tray rather than a one-way clear. Starting
paused-by-default is a cheap, high-value onboarding win Babylon should
adopt for first tick after any scenario load.

---

## 7. Onboarding / tutorial

The "Learn the Game" tutorial track exists but is widely described by the
community as insufficient on its own — new players are routinely advised to
replay the tutorial multiple times, watch third-party YouTube guides, and
start with a small/simple country (Cape Colony, Hudson Bay Company, Haiti,
Brunei) specifically to reduce the number of simultaneous systems in play
while learning. Inverse's review calls out **poor task sequencing** —
tutorial steps can block on a prerequisite (e.g. an ungated technology) the
tutorial itself never told the player to unlock.

**Source:** [Beginner's guide — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Beginner's_guide),
[Inverse review](https://www.inverse.com/gaming/victoria-3-review-pc),
Steam community threads on onboarding.

**Babylon read:** if Babylon ships a tutorial, favor a **reduced-scope
starting scenario** (fewer classes/organizations in play) over a scripted
walkthrough of the full system — Vic3's community-invented workaround
(start small, learn incrementally) is itself evidence that a designed
"easy nation" onboarding scenario would have been the right fix.

---

## 8. Top 3 community UX complaints (synthesized across sources)

1. **Nested tooltips fragment information instead of consolidating it.**
   Getting one answer often requires hover-chasing a multi-level chain
   every single time, with no persistent memory of "I already looked this
   up" — named explicitly in the Paradox forum thread "Reminder to the devs
   that nested tooltips is bad UX design," and echoed by review language
   like "at no point does the player ever have all the information they
   need readily at hand."
2. **Dense relational data rendered as a literal graph, not progressively
   disclosed** — the technology tree is the canonical bad example: an
   "octopian," "cramped" node web that multiple reviewers explicitly gave up
   trying to parse and started clicking through randomly instead.
3. **Notification/pop-up volume scales badly with game size** — by the
   late game, "an endless deluge of pop-ups for so many countries and
   events took up a lot of screen space," a problem serious enough that
   Paradox's own post-launch patch (#74) targeted a **~50% notification
   reduction** as a headline fix.

---

## 9. Direct recommendations for Babylon

1. **Keep a permanent "vitals strip" (TopBar)** carrying the handful of
   numbers that matter every tick (imperial rent gap, radicalization,
   active contradictions), with every value tap-through into
   `InspectionStack` — mirrors Vic3's top-bar design and avoids burying
   headline numbers inside panels.
2. **Fuse map lenses with tool-mode**, not a bare palette switch: each lens
   both recolors the map for its domain and constrains what clicking a hex
   does, using one consistent five-state eligibility color grammar across
   every lens.
3. **Build `InspectionStack`'s nested drill-down with pin-to-panel from day
   one**, not hover-only flyouts — this is the single most-repeated
   complaint about Vic3's launch tooltip system, and Paradox's own #74 fix
   converged on exactly this (pinning goods/characters, promoting hover
   data into always-visible panel fields).
4. **Never render a dense relational structure (contradiction web,
   organization network) as one big node graph.** Use collapsible
   categories and progressive disclosure — this is the direct lesson of
   Vic3's most-hated tech tree.
5. **Preview action consequences inline before commit** (solidarity/
   control-ratio delta on hover of a verb button), following Vic3's
   predictive production-method tooltips.
6. **Provide a bulk-apply escape hatch** in `ActionDock` for
   multi-hex/multi-organization actions, mirroring Vic3's "set all
   buildings of this type" — recursive per-instance inspection does not
   scale to a large empire/movement without one.
7. **Separate ambient narrative (wire ticker) from blocking decision
   popups.** Only true decisions should take over the screen; news should
   flow in a dismissible, reversible toast/ticker (`EventToasts`), following
   Vic3's outliner-glow + undo-dismiss pattern rather than modal event
   chains.
8. **Cap notification volume deliberately and revisit it as the world
   scales** — Vic3 needed an explicit ~50% cut post-launch; budget for the
   same kind of alert-consolidation pass once Babylon's late-game
   organization/contradiction count grows.
9. **Ornament the chrome, never the data** — apply Cold Collapse's cyan/dark
   Art Nouveau-equivalent treatment to panel frames and borders only; keep
   numbers, icons, and map fills clean and legible, per Vic3's own stated
   (if imperfectly executed) design rule and the trade-panel "icons in the
   way" complaint as the cautionary counter-example.
10. **Start paused, and favor a reduced-scope onboarding scenario** over a
    full-system scripted tutorial — Vic3's community had to invent this
    workaround themselves ("start as a small country"); design it in.

---

## Sources

- [Dev Diary #30 — User Interface Overview](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-30-user-interface-overview.1507166/) (Paradox Forums)
- [Dev Diary #74 — UX Improvements](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements) (Paradox Interactive)
- [User interface — Victoria 3 Wiki](https://vic3.paradoxwikis.com/User_interface)
- [Map modes — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Map_modes)
- [Building — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Building)
- [Market — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Market)
- [Needs — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Needs)
- [Beginner's guide — Victoria 3 Wiki](https://vic3.paradoxwikis.com/Beginner's_guide)
- [PCGamesN — "Victoria 3 will use Crusader Kings 3's tooltip system so you don't need an econ degree"](https://www.pcgamesn.com/victoria-3/nested-tooltip-system)
- [Paradox Forums — "Reminder to the devs that nested tooltips is bad UX design"](https://forum.paradoxplaza.com/forum/threads/reminder-to-the-devs-that-nested-tooltips-is-bad-ux-design.1702017/)
- [Paradox Forums — "User Inter-fiasco: Victoria III's Abysmal UI"](https://forum.paradoxplaza.com/forum/threads/user-inter-fiasco-victoria-iiis-abysmal-ui.1561896/)
- [Destructoid — Victoria 3 review](https://www.destructoid.com/reviews/review-victoria-3-pc-paradox-interactive-strategy/)
- [Inverse — "Victoria 3 is 2022's most promising world-building sim — with a few caveats"](https://www.inverse.com/gaming/victoria-3-review-pc)
- [Rock Paper Shotgun — Victoria 3 review](https://www.rockpapershotgun.com/victoria-3-review)
- [PC Gamer — Victoria 3 review](https://www.pcgamer.com/victoria-3-review/)
- [Kotaku — Victoria 3 review](https://kotaku.com/victoria-3-review-paradox-pc-kotaku-impressions-verdict-1849669988)
- [Gamecritics.com — Victoria 3 review](https://gamecritics.com/mitch-zehe/victoria-3-review/)
- [Gamepressure — Victoria 3 Accessibility](https://www.gamepressure.com/victoria-3/accessibility-features/z11055f)
