# Hearts of Iron IV — UI/UX Deep Dive

Research for Babylon's "living map" program (`project/research/16-living-map/`). Focus: theater/army
outliner, alert bar, map modes, division-designer complexity management, time controls under
pressure, fog-of-war readability, and the top community UX complaints. All claims below are
attributed to a source; where I could not verify a specific mechanic (e.g. exact fog-of-war
rules) I say so rather than guessing.

## 1. Overall information architecture

HOI4's screen is organized as **persistent chrome around a full-bleed map**, not floating widgets
scattered arbitrarily:

- **Top bar** (left → center): a single horizontal row of national resource stats — Political
  Power, Stability, War Support, Manpower, Factories, Fuel, Logistics Fulfillment, Convoys,
  Command Power, Army/Navy/Air Experience, Nuclear Weapons — plus a **World Tension globe** at the
  far upper-right. This is the "vitals strip": always visible, never occluding the map, one glance
  tells you the health of the nation. (Source: [User interface —
  wiki](https://hoi4.paradoxwikis.com/User_interface))
- **Main menu bar**: the national flag + up to 10 icon buttons (Government, Decisions, Intelligence
  Agency, Research, International Market, Trade, Construction, Production, Recruit & Deploy,
  Logistics/Officer Corps), each bound to a single hotkey (Q–I). Clicking one **takes over the
  screen** with a modal-style full panel — these are the "go deep" screens, distinct from the
  map-overlay HUD. (Source: [User interface —
  wiki](https://hoi4.paradoxwikis.com/User_interface))
- **Alert bar**: alert tabs appear at the **top of the screen** as warnings that need attention;
  a right-hand cluster on the same bar carries the clock, timescale, world tension readout, main
  menu button, wiki shortcut, and a tray of dismissed alerts. Alerts are transient/floating —
  they don't force a modal, they just sit until acknowledged or resolved by state change. (Source:
  [Beginner's guide — wiki](https://hoi4.paradoxwikis.com/Beginner's_guide), corroborated by
  [User interface — wiki](https://hoi4.paradoxwikis.com/User_interface))
- **Bottom bar**: army/navy/air group portraits and the frontline/battle-plan toolbar live here,
  contextual to what's selected — this is the outliner-equivalent, docked opposite the top bar so
  it never competes with the resource strip.
- **Map modes selector**: bottom-right corner, icon buttons + hotkeys + an overflow arrow for
  extra modes (see §2).

The load-bearing pattern: **the map is always the base layer; every other panel is chrome around
its edges (top/bottom/corners) or a full-screen takeover on demand.** Nothing floats mid-map
except tooltips, unit icons, and battle-plan arrows the player is actively drawing. This is the
single most portable lesson for Babylon: keep the map full-bleed, keep persistent readouts pinned
to bars at the edges, and reserve full takeovers for genuinely deep screens (production,
diplomacy) rather than for routine glancing.

## 2. Map modes ("lenses")

HOI4 ships **15 distinct map modes**, switched via icon buttons in the lower-right corner (with an
overflow arrow for the modes that don't fit the visible row) or via hotkeys. (Source: [Map —
wiki](https://hoi4.paradoxwikis.com/Map))

The 15, as documented:

1. **Default** — a fused political + terrain view; critically, **detail is zoom-dependent**: "when
   zooming in, the terrain becomes more and more visible, integrating both map modes into one."
   This is not a toggle between political-only and terrain-only — it's one continuous mode whose
   information density scales with camera altitude.
2. **Strategic Navy** — naval regions, convoy routes, supremacy data.
3. **Strategic Air** — air superiority per region, red/yellow/green indicators.
4. **Operatives** (La Résistance DLC) — intelligence ops and spy networks.
5. **Supply** — supply network connectivity.
6. **Terrain** — colored terrain types affecting movement.
7. **Resistance** — occupied-state unrest, 0–100% with tier indicators.
8. **Compliance** — occupier control level, five progression tiers.
9. **Resource** — resource locations + trade routes.
10. **Diplomacy** — international relations, color-coded.
11. **Factions** — alliance membership by faction-leader color.
12. **States** — political boundaries + building-slot categories.
13. **Infrastructure** — infrastructure development per state.
14. **Population** — population distribution.
15. **Ideology** — communist red / democratic blue / fascist brown / non-aligned gray.

Design lessons for Babylon's lens system:

- **Zoom-coupled disclosure beats a flat toggle.** The default mode's terrain-fades-in-on-zoom
  behavior means the player never has to explicitly ask for "more detail" — it arrives as they
  get closer to the area they care about. Babylon's county→state aggregation redrawing as the
  camera zooms is the same pattern already implied by the brief; HOI4 validates it as the
  strongest single IA idea in the game.
- **One mode = one legible palette, not a stack of layers.** Every HOI4 mode recolors the whole map
  around a single variable (resistance %, compliance tier, ideology) rather than overlaying many
  variables at once. This keeps each mode readable at a glance, at the cost of needing 15 modes to
  cover the state space. Babylon's map lenses (supply/solidarity/repression/etc.) should follow
  this discipline — one dominant channel per lens, not a kitchen-sink heatmap.
- **Grouping by button row + overflow is a scaling failure mode worth avoiding.** 15 modes in one
  icon row plus an overflow arrow is already at the edge of usable — community guides exist
  specifically to help players find/remember modes (see the "Map Modes+" Steam Workshop mod,
  which exists because base-game mode-switching UX wasn't judged sufficient). Babylon should group
  lenses into a smaller number of *categories* (Political / Economic / Military / Social) with a
  radial or dropdown selector rather than a flat strip, if the lens count grows past ~6–8.

## 3. Outliner — theater / command-group / army hierarchy

HOI4's land-forces "outliner" is a three-level hierarchy, not a flat unit list:

- **Theater** — "a player-defined high-level group of command groups." Players create theaters to
  split fronts (e.g. Eastern vs Western). Theaters are named via a gear icon; theater performance
  is inspectable via a **Combat Log** (document icon next to the theater name), filterable by time
  and command group. (Source: [Land warfare —
  wiki](https://hoi4.paradoxwikis.com/Land_warfare), [User interface —
  wiki](https://hoi4.paradoxwikis.com/User_interface))
- **Command group** — a collection of divisions led by one commander; the unit of battle-plan
  assignment (front, garrison area, fallback line). A command group belongs to exactly one theater
  but can be moved between theaters at will.
- **Divisions** — the leaf unit. Critically: **"The theater interface shows only divisions that are
  assigned to a command group. Unassigned divisions are shown in the armies list."** This is a
  documented gotcha — new/reinforcement divisions can silently sit outside the theater view the
  player is looking at, and the wiki explicitly warns players to check both places.
- Commander **portraits** for each command group in the active theater sit at the bottom of the
  screen, each showing relative strength assessment and an activate-plan button — the compact
  "at-a-glance roster" the player scans before drilling in.

Lessons for Babylon's `OutlinerOverlay`:

- **A three-tier grouping (Theater → Command Group → Division) is the right depth** for an
  organization-heavy strategy game — deep enough to let the player delegate ("give this whole
  front a fallback line") without forcing them to touch every leaf unit, shallow enough that it
  doesn't need its own navigation stack. Babylon's organizations/factions could adopt an analogous
  Region-Campaign → Cell/Branch → Member-or-Asset structure if the org roster ever gets large
  enough to need it — but note this is heavier than Babylon currently needs; don't build it
  speculatively.
- **The "unassigned units disappear from theater view" gotcha is an anti-pattern to explicitly
  avoid.** If Babylon ever groups organizations/assets into player-defined bins (fronts,
  campaigns, whatever), newly created or newly available assets must default into a *visible*
  bucket (an "Unassigned" tray that's part of the main outliner, not a separate screen the player
  has to remember exists), or surface a persistent count badge ("3 unassigned") so nothing goes
  invisible by default.
- **Combat Log as a per-group drill-down** is a good precedent for Babylon's `InspectionStack`:
  every grouping level should have its own inspectable history/log, not just current-state
  numbers.

## 4. Alerts / notifications

Documented behavior: alert tabs appear at the **top of the screen** as warnings requiring
attention; a right-hand cluster on the same bar shows the clock, timescale, world tension, menu
button, wiki link, and a **tray of dismissed alerts** (so dismissing isn't destructive — the
player can retrieve what they cleared). (Source: [Beginner's guide —
wiki](https://hoi4.paradoxwikis.com/Beginner's_guide))

This is a comparatively thin documented surface — HOI4's wiki does not give exhaustive detail on
alert *categories*, priority ordering, or whether alerts can be permanently muted per-type. What
is clear from the design: alerts are **persistent tabs, not transient toasts** — they stay
attached to the bar until the underlying condition resolves or the player dismisses them, and a
dismissed-alerts tray prevents the "I lost that notification forever" failure mode that pure toast
systems have.

Lesson for Babylon's `EventToasts` / `TopBar`: consider a **dismissed-but-recoverable** affordance
— a small history tray for cleared alerts/toasts — rather than one-shot toasts that vanish
permanently on timeout or dismiss. This directly answers a known UX failure mode (player alt-tabs
or looks away, misses a toast, has no way to retrieve it) that HOI4's tray design solves cheaply.

## 5. Tooltip / disclosure system

Direct primary-source detail on HOI4's tooltip layering was not recoverable from the pages I could
fetch (the wiki's UI page is menu-functionality-focused, not a tooltip spec) — **flagging this as
unverified rather than inventing a mechanic.** What is corroborated by both the wiki and community
threads is the general disclosure *philosophy*, not implementation: base numbers on the map/HUD
are deliberately terse (a percentage, a bar), and the underlying breakdown (planning bonus math,
resistance-tier composition, division combat-width contributions) is reachable by hovering or
clicking through — e.g. the offensive-line planning bonus ("+1% attack and breakthrough per 1%
planning, up to 30%") is stated as a rule the wiki documents separately from what's shown in the
HUD number. ([Battle plan — wiki](https://hoi4.paradoxwikis.com/Battle_plan))

This matches Babylon's own "Victoria-3-style nested recursive inspection panels where every number
explains itself" goal directionally, but HOI4 should not be over-cited as a model for *deep*
disclosure — its reputation (see §7) is that too much of its math is *not* surfaced in tooltips,
pushing players to community spreadsheets and wikis instead. That is a documented failure mode
to avoid, not a pattern to imitate.

## 6. Division designer — complexity management

The division designer is HOI4's most complexity-dense screen and its most-cited example of
"spreadsheet simulator" criticism:

- It is widely described as powerful but overwhelming: "With limitless possibilities to designing
  divisions, it is easy to be overwhelmed when you open the division designer," and the game as a
  whole is characterized as being "often described as a 'spreadsheet simulator'... overly
  complicated with its numbers." (Source: [EIP Gaming — Division Designer
  Guide](https://eip.gg/hoi4/guides/division-designer/))
- The designers' own stated intent (per the original Dev Diary 6 announcement and corroborating
  community summary) was to keep customization but make it **harder to find the single "best"
  template and easier to iterate on one** compared to HOI3 — i.e. they deliberately traded away a
  single dominant optimum in favor of terrain/context-dependent tradeoffs. This mostly succeeded
  mechanically (see the No Step Back combat-width rework below) but did not reduce the *cognitive*
  load of the screen itself.
- The **No Step Back** expansion reworked combat width specifically to break the "40-width
  meta" — a single build that dominated all others regardless of context — by varying effective
  width by terrain without a clean multiplier to exploit, and by softening the penalty for
  exceeding a province's width. Community reaction settled on no single obviously-correct number
  (candidates from 12 to 36 depending on terrain/doctrine), which the devs treated as a *success
  criterion*, not a bug. (Source: [PCGamesN — Hearts of Iron 4
  meta](https://www.pcgamesn.com/hearts-of-iron-iv/meta-division-templates), [devtrackers.gg dev
  diary summary](https://devtrackers.gg/heartsofiron/p/647d25f1-dev-diary-combat-width-soviet-feedback))

Lesson for Babylon: complexity-dense customization screens (if Babylon ever needs one — e.g.
organization structure/strategy builders) benefit from **removing a single dominant optimum** more
than from simplifying the UI chrome. HOI4's screen complexity criticism persisted *even after* the
math was rebalanced — meaning the UI itself, not just the underlying system, is a real cost. If
Babylon builds anything division-designer-shaped, budget separately for (a) balancing away a
single "correct" build and (b) a genuinely disclosure-friendly UI — doing only (a) will not fix
the "overwhelming spreadsheet" perception.

## 7. Time controls under pressure

Documented mechanics:

- Pause is bound to the **space bar** in singleplayer; in multiplayer, players instead click the
  date display (since a hard pause would stop the game for everyone). (Source: [Steam Community —
  Pause button
  thread](https://steamcommunity.com/app/394360/discussions/0/1681441347882385848/), corroborated
  by hotkey guides)
- Speed control is mouse-driven by default (clicking speed icons); a persistent community
  complaint is that there is **no default keybind for jumping directly to a specific speed
  (1–5)** — players have had to hand-edit keybind config files or install workshop fixes to add
  numeric speed hotkeys. (Source: [Steam Community guide — speed
  shortcuts](https://steamcommunity.com/sharedfiles/filedetails/?id=2823045696))
- Beyond pause/speed, heavy players rely on action hotkeys to compress micromanagement under time
  pressure — e.g. **S** to split selected divisions (batch-assigning orders to a group at once)
  and battle-plan hotkeys (**Z** front line, **X** offensive line, **Shift+X** spearhead) that let
  a whole army group receive orders in one drag-gesture rather than one click per unit. (Source:
  [Battle plan — wiki](https://hoi4.paradoxwikis.com/Battle_plan), [DefKey — HOI4
  shortcuts](https://defkey.com/hearts-of-iron-4-shortcuts))

Lesson for Babylon: the gap in HOI4 (no default direct-speed hotkeys) is a concrete anti-pattern —
Babylon's `TopBar` time controls should bind speed tiers directly to number keys out of the box,
not require players to discover/mod their way there. The compensating strength — **batch order
gestures** (draw one line, whole army group obeys) — is exactly the leverage Babylon's
`ActionDock`/verb system should aim for when a player wants to direct many organizations/assets at
once (e.g. "mobilize all cells in this state" as one gesture rather than N individual verb
invocations).

## 8. Fog of war / readability

I was not able to verify HOI4's fog-of-war mechanics from the sources fetched (the Map wiki page
content retrieved did not include a fog-of-war section, and I am flagging this explicitly rather
than asserting an unverified mechanic). What is verifiable and relevant to *readability* instead:

- The **default map mode's zoom-coupled terrain fade-in** (§2) doubles as a readability mechanism
  independent of fog of war — it keeps the map legible at strategic zoom (flat political color)
  while adding tactical detail only once the camera commits to an area.
- Map-mode—specific palettes (resistance 0–100% tiers, compliance 5-tier scale, ideology 4-color)
  are each a single-variable choropleth, which is a deliberate readability choice: comparative
  literature on grand-strategy UI generally agrees single-variable choropleths are far more
  legible than multi-variable ones, and HOI4's mode design bears this out structurally even though
  I don't have a primary source explicitly stating the design rationale.

Recommendation: if a source needs to be cited for fog-of-war specifically, this should be
re-researched from primary Paradox dev-diary text rather than inferred — do not treat anything in
this section as confirmed fog-of-war doctrine.

## 9. Top 3 community UX complaints (verified)

1. **"Uninformative yet cluttered," washed-out/ambiguous color coding.** Forum discussion
   (Paradox forums, `hoi4-as-a-beginner-the-ui-is-horrible` thread, corroborated via search
   snippet) centers on muddy, low-saturation colors that make it hard to tell at a glance whether
   something is already researched or already taken — i.e. the palette fails the "read state from
   color alone" test the UI itself relies on. Defenders counter that for the game's information
   density, the interface does "an extremely good job... without overwhelming the player," so this
   is a genuinely contested point, not a unanimous verdict. (Source: [Paradox forums thread — found
   via search](https://forum.paradoxplaza.com/forum/threads/hoi4-as-a-beginner-the-ui-is-horrible.1443216/);
   direct fetch was blocked by Cloudflare bot-check, so this is sourced from the search snippet,
   not the full thread text.)
2. **Division designer / overall system as "spreadsheet simulator."** Widely repeated across
   guides and reviews (see §6) — the complaint is specifically that raw numeric complexity is not
   adequately translated into legible in-UI guidance, pushing players to external wikis/calculators
   (e.g. the fan-built [online division designer](https://taw.github.io/hoi4/)) to make decisions
   the base UI doesn't support well.
3. **UI hitbox/scaling bugs compounding perceived clutter.** A concrete, verifiable technical
   complaint: clickable areas not lining up with rendered button positions (traced by the
   community to a DirectX 9→11 rendering change), forcing players to scale the UI up, which then
   introduced blur. (Source: [Steam Community — UI Issues
   thread](https://steamcommunity.com/app/394360/discussions/2/3183486320476103590/)) This is a
   narrower, more technical complaint than the other two, but it is well-documented and speaks to
   a real risk for Babylon: **any UI-scale/DPI setting must be tested against actual click
   hitboxes, not just visual rendering**, or "clutter" complaints will conflate genuine IA problems
   with simple hit-testing bugs.

A fourth informally-verified pattern worth naming even though it didn't rise to a clean top-3
citation: **no built-in colorblind mode.** The game has no native colorblind accessibility
setting; the community has produced multiple workshop mods (e.g. a deutan-targeted "Colourblindness
Mod") to compensate for "small and unsaturated red/green bars," and forum requests for
built-in support (including black border-lines to disambiguate colored regions) have gone
unaddressed in the base game. (Source: [Steam Workshop — Colourblindness
Mod](https://steamcommunity.com/sharedfiles/filedetails/?id=699262458), [Paradox forums — Color
blind support?](https://forum.paradoxplaza.com/forum/threads/color-blind-support.911384/)) Given
Babylon's "Cold Collapse" cyan-accented palette is already ratified, this is a concrete
accessibility gap worth budgeting for deliberately (border/pattern redundancy on any
color-coded map mode, not color alone) rather than discovering it post-launch the way HOI4's
community did.

## Sources

- [User interface — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/User_interface)
- [Map — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/Map)
- [Land warfare — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/Land_warfare)
- [Battle plan — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/Battle_plan)
- [Beginner's guide — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/Beginner's_guide)
- [Hotkeys — Hearts of Iron 4 Wiki](https://hoi4.paradoxwikis.com/Hotkeys) (referenced via search;
  not directly fetched)
- [EIP Gaming — Division Designer Guide](https://eip.gg/hoi4/guides/division-designer/)
- [PCGamesN — Hearts of Iron 4 meta: combat width, division templates, and
  more](https://www.pcgamesn.com/hearts-of-iron-iv/meta-division-templates)
- [devtrackers.gg — Dev Diary: Combat Width & Soviet
  Feedback](https://devtrackers.gg/heartsofiron/p/647d25f1-dev-diary-combat-width-soviet-feedback)
- [Steam Community guide — How to add 1-5 speed
  shortcuts](https://steamcommunity.com/sharedfiles/filedetails/?id=2823045696)
- [Steam Community — Pause button
  discussion](https://steamcommunity.com/app/394360/discussions/0/1681441347882385848/)
- [DefKey — Hearts of Iron 4 keyboard shortcuts](https://defkey.com/hearts-of-iron-4-shortcuts)
- [Paradox forums — "HOI4 as a beginner - the UI is
  horrible"](https://forum.paradoxplaza.com/forum/threads/hoi4-as-a-beginner-the-ui-is-horrible.1443216/)
  (search-snippet sourced; direct fetch blocked by Cloudflare)
- [Steam Community — UI Issues technical
  thread](https://steamcommunity.com/app/394360/discussions/2/3183486320476103590/)
- [Steam Workshop — Colourblindness Mod](https://steamcommunity.com/sharedfiles/filedetails/?id=699262458)
- [Paradox forums — "Color blind
  support?"](https://forum.paradoxplaza.com/forum/threads/color-blind-support.911384/)
- [taw's online Division Designer](https://taw.github.io/hoi4/) (community tool cited as evidence
  that base-game disclosure pushes players off-platform)
- [PC Gamer — Hearts of Iron 4 review](https://www.pcgamer.com/hearts-of-iron-4-review/) (title/URL
  only; full review body was not retrievable via fetch — membership wall — so not quoted directly)

### Notes on source limitations

- The Paradox forums (`forum.paradoxplaza.com`) sit behind a Cloudflare bot-check that blocked
  direct `WebFetch` access; where forum content is cited, it is sourced from search-engine
  snippets, not the full thread, and is flagged as such inline.
- PC Gamer's review page served only its membership/nav chrome to the fetcher, not the article
  body; it is listed as a source for the review's existence and headline claims found in search
  snippets, not for direct quotes.
- No GDC talk specifically on Paradox grand-strategy UI/UX was found; this line of inquiry came up
  empty and is reported as such rather than filled with a plausible-sounding invented citation.
- Fog-of-war mechanics could not be verified from the sources fetched; §8 is written to make that
  gap explicit rather than assert unverified rules.
