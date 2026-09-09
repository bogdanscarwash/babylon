# Stellaris — UI/UX Deep Dive for the Babylon Living Map

**Research date:** 2026-07-11
**Why Stellaris matters to Babylon:** it is the genre's most-cited "HUD as
the real game" case study — the **outliner**, not the galaxy map, is where
most Stellaris play actually happens; the map is spectacle and a spatial
index, while empire management lives in a persistent side panel plus
full-screen drill-downs. Stellaris also has the most publicly-documented
*failure and repair* cycle for information architecture of any Paradox
title: the 2018 tile→district planet rework, the notification/situation-log
rework line running through 2020 and again in 2026 (Dev Diary #417), and a
decade of community backlash on notification fatigue and nested tooltips.
Babylon's `OutlinerOverlay` + `EventToasts` + `InspectionStack` design should
treat Stellaris as the cautionary-tale-plus-fix reference, not just a
positive model.

---

## 1. Overall Information Architecture — what floats vs. what takes over

Stellaris's screen has three persistent floating layers and one takeover
layer:

- **Top bar.** Empire resources left-to-right (energy, minerals, food,
  consumer goods, alloys, then influence/unity, research, strategic
  materials), each showing current stock **and monthly net gain** in the
  same widget. The date sits centrally with pause/speed controls. Alerts
  occupy the right side, **color-coded by urgency** (red = critical, orange
  = moderate, green = informational).
- **Outliner (right edge, persistent).** Four tabs — Government (situations,
  sectors, planets, starbases), Ships (fleets, civilian vessels, armies),
  Politics (factions, observation posts), Structures (megastructures).
  Toggleable with `O`; double-clicking an entry recenters the camera on it.
  This is the panel players interact with more than the map itself — it is
  effectively the game's true HUD, doubling as a navigation index and a
  live status board (population/happiness/output glyphs update per-planet
  in the list without opening anything).
- **Bottom bar.** Map-mode toggle (Empire/ownership, Diplomatic, Intel,
  Opinion, AI Attitude, Immediate Neighbors) plus graphic overlays (sector
  borders, trade routes, hyperlanes) that layer independently of the
  selected map mode.
- **Situation Log (left sidebar, floating panel).** A persistent tracker for
  ongoing objectives, anomalies, and victory conditions, marked with
  exclamation-icon badges when something needs attention. Opens as an
  overlay panel, not a takeover.
- **Full-screen takeovers.** Planet management, ship designer, species
  screen, tech tree, and the diplomacy screen replace the map entirely with
  a dedicated management UI reached via the outliner or top bar — the map is
  never visible behind them. This is the sharpest contrast with Victoria
  3's slide-in side panels (see the Babylon Victoria-3 note): Stellaris
  treats planet/fleet/tech management as *destinations*, not overlays.

**Sources:** [Main interface — Stellaris Wiki](https://stellaris.paradoxwikis.com/Main_interface),
[User interface — Stellaris Wiki (Fandom)](https://stellaris.fandom.com/wiki/User_interface),
[Empire interface — Stellaris Wiki](https://stellaris.paradoxwikis.com/Empire_interface)

**Babylon read:** the outliner-as-true-HUD pattern is directly applicable —
Babylon's `OutlinerOverlay` (organizations, territories, factions) should be
the thing players glance at constantly, with the map as a secondary spatial
lookup. But Babylon should NOT copy the full-screen-takeover pattern for
`InspectionStack` — the project's own mandate (Victoria-3-style nested
panels that never hide the map) argues for slide-in panels over Stellaris's
map-replacing management screens.

---

## 2. Map Modes / Lenses

Stellaris ships a small, flat set of map modes rather than Victoria 3's
larger grouped lens system: **Empire (ownership)**, **Diplomatic**,
**Intel**, **Opinion**, **AI Attitude**, and **Immediate Neighbors**,
switched via a single bottom-bar dropdown/button row. Independent toggle
overlays (sector borders, trade route lines, hyperlane visibility) stack on
top of whichever map mode is active, so "what color is the map" (mode) and
"what lines are drawn on it" (overlay) are separate, orthogonal controls
rather than one long menu.

Community modding pressure (Enhanced UI Project, Better Galaxy Map UI) is
almost entirely about **icon and banner density at the galaxy scale** — the
base game's per-system banners (resource icons, ownership flags, fleet
counts) get so large and numerous that "it's actually hard to see the stars
and planets behind the massive banners hovering over every starbase,
planet, mining or research hub, and ship." The fix mods apply is shrinking
low-priority icons and enlarging only the ones relevant to the current
context (threats, opportunities).

**Sources:** [Main interface — Stellaris Wiki](https://stellaris.paradoxwikis.com/Main_interface),
[Best UI Mods for Stellaris — FandomSpot](https://www.fandomspot.com/stellaris-ui-mods/),
[Better Galaxy map UI — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/better-galaxy-map-ui.997748/)

**Babylon read:** two lessons. (1) Keep the mode/overlay split — Babylon's
map lenses (control, solidarity, imperial-rent-gap, ecological overshoot)
should be one mutually-exclusive picker, with independent toggle overlays
(county borders, hex grid, corridor mesh) stacked on top, not a combinatorial
menu. (2) Icon/banner density is a real failure mode at galaxy-equivalent
zoom (Babylon's state/national view) — icon size and count must be
LOD-gated and context-prioritized (show contradictions and active struggles
large; show routine production small or hidden) or the full-bleed map
mandate collapses into banner soup exactly as Stellaris's base UI did.

---

## 3. Galaxy ↔ System Zoom LOD

Stellaris's camera has **14 discrete zoom levels in galaxy view** for smooth
zooming, and a separate camera mode in system view that supports rotation
(top-down to angled) plus its own zoom range. The two are distinct camera
rigs, not one continuous zoom: clicking a system from the galaxy view snaps
the camera into system view and auto-zooms in, rather than the player
continuously scrolling from one scale to the other. Within system view,
detail is LOD-gated by zoom: **orbit lines are hidden at most zoom levels
and only fade in as you zoom out** near the system-view/galaxy-view
boundary, and system-navigation arrows are deliberately sized small to
avoid becoming visual noise. Exiting a system back to the galaxy view resets
the camera to a consistent "further-out" zoom rather than preserving the
exact prior state.

**Sources:** [Galaxy Map and System View zooming — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/galaxy-map-and-system-view-zooming.962148/),
[Move camera to system without zooming in? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/1639792569852006960/)

**Babylon read:** the "snap to a dedicated view on selection, don't force a
continuous drag-zoom" pattern is worth borrowing for the national-view →
county-view → hex-view transition: clicking a state/county should snap-zoom
with a deliberate camera move, not require the player to manually scroll
through every intermediate LOD. Gating secondary detail (orbit lines →
Babylon's hex grid or corridor lines) to only the zoom band where it's
legible is a direct, provable pattern to apply to hex-tile visibility.

---

## 4. Situation Log — a documented three-generation UX repair

The Situation Log is Stellaris's most publicly iterated disclosure surface,
and it is the single best evidence trail in this research set for "how a
Paradox team actually diagnoses and fixes a progressive-disclosure failure."

**The failure mode (pre-2026):** a flat, undifferentiated feed mixing
crises, resource deficits, first contacts, anomalies, dig sites, and
precursor chains in one list with no hierarchy. Per the community and dev
diary framing, players could **lose a critical resource-deficit notice
under a pile of first-contact entries** — i.e., routine exploration noise
buried urgent economic alarms, the textbook progressive-disclosure failure
of "everything is equally loud, so nothing is."

**Dev Diary #417 (2026) rework**, per UX designer Doğa (quoted): the
redesign targeted "pain points of navigation and lack of ordering, lack of
grouping and hierarchy, desire for personalization, and also generally make
it look better." The fix:

- **Unified entries.** Events that previously fragmented into several
  separate log entries are now collapsed into one.
- **Collapsible categories ordered by urgency**, not chronology: Tutorial →
  Crises → Priority (user-pinned) → Urgent (e.g., resource deficits) →
  Empire Concerns → Precursors → Developments.
- **A dedicated "Findings" tab** pulled entirely out of the main feed for
  First Contacts, Dig Sites, Anomalies, and Astral Rifts — i.e., the
  "interesting but not urgent" exploration content got its OWN surface
  instead of competing for attention in the urgent-affairs feed.
- **User pinning.** Players can pin any entry to a top-level Priority
  category — manual override on top of the system's automatic urgency
  sort.

**Sources:** [Stellaris Dev Diary #417 — Situation Log Updated (summary via xpgained.co.uk)](https://xpgained.co.uk/patch-notes/stellaris-dev-diary-417-situation-log-updated-notes-23rd-april-2026),
[Stellaris Dev Diary #417 — Paradox Forums thread](https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-417-situation-log-updated.1918530/)

**Babylon read:** this is the single most transferable pattern in this
report for Babylon's `EventToasts`/wire feed and any situation-log
equivalent. Concretely:

1. **Separate urgency streams from discovery streams by construction**, not
   by filter toggle — Babylon's wire/news feed should split "contradiction
   escalations / rupture warnings" (urgent) from "narrative color / faction
   gossip / investigation results" (findings-equivalent) into different
   tabs from day one, rather than retrofitting a filter after players
   complain about noise.
2. **Category-collapse, urgency-first ordering** beats
   chronological-only ordering for any feed that mixes severities.
3. **Manual pin + automatic urgency sort should coexist** — don't force a
   choice between "the system decides what's important" and "the player
   decides"; Stellaris ships both.
4. Treat this as a warning: Stellaris shipped the *broken* flat-feed
   version for years before fixing it in 2026. Babylon should design the
   split from the start rather than assume a flat wire feed will scale.

---

## 5. Alerts, Notification Fatigue, and Dismissal Patterns

**Two-tier alert system.** Square alerts persist "until the circumstance
changes or the player right-clicks them to dismiss" (durable state alerts —
e.g., an unemployed-pop warning). Round alerts are ephemeral and
"disappear automatically after brief display" (transient event pings). This
square/round + color-urgency (red/orange/green) coding is the top-bar's
entire triage vocabulary.

**This is Stellaris's most consistently criticized UI surface across a
decade of community feedback**, and the complaints cluster into three
repeated failure modes:

- **Notification overload / noise-to-signal collapse.** Players report
  wanting popups about their OWN empire to be prominent while "drama
  between other empires" (AI-vs-AI diplomatic events) gets "automatically
  turned into pointless notifications" that compete for the same channel.
  ("Too many notifications I don't care about," Steam Community.)
- **Duplicate/fragmented notifications for one underlying event class** —
  e.g., three separate anomaly-available notifications firing individually
  instead of one notification listing multiple locations. Players
  explicitly proposed the fix Stellaris later shipped as the Findings tab
  (grouping by category rather than by individual event instance).
- **Auto-dismiss timing mismatch** — round/ephemeral alerts can disappear
  "before players have time to act on them," a direct tension between
  keeping the screen decluttered and giving players enough reaction window.
  A long-running Paradox Forums thread ("Don't auto-dismiss notifications")
  argues the auto-timeout should be removed or made player-configurable
  rather than fixed.

**Mitigation the game already ships:** a settings toggle to disable
full-screen popup interruption so notifications only ever appear as
top-bar indicators (never pausing the game or blocking the screen) — i.e.,
the game learned to let players trade "interruptive" for "ambient" as a
global preference.

**Sources:** [Too many notifications I don't care about — Steam Community](https://steamcommunity.com/app/281990/discussions/0/3781371514569030827/),
[Does anyone else find the incessant notifications frustrating? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/4203492762829778326/),
[Don't auto-dismiss notifications — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/dont-auto-dismiss-notifications.1010326/),
[UI Issues — Steam Community](https://steamcommunity.com/app/281990/discussions/0/357285562496225464/)

**Babylon read:** this is the strongest cautionary evidence in the whole
report for `EventToasts` design.

1. **Distinguish "your empire" events from "world" events at the channel
   level**, not just severity — Babylon should give player-actionable
   contradiction alerts a different visual/audio channel than ambient
   wire-feed narrative about factions the player doesn't control, exactly
   the split Stellaris players have been begging for since launch.
2. **Group by category before firing**, don't fire one toast per raw event
   — batch multiple simultaneous low-priority events (e.g., three counties
   crossing a solidarity threshold in the same tick) into one toast with an
   expand-affordance, the fix the community asked for years before Findings
   shipped.
3. **Auto-dismiss timing must be generous or player-configurable** for
   anything actionable — an ephemeral toast that vanishes before the
   player can click it is worse than no toast. Persistent (square-alert
   equivalent) state should be used for anything requiring a decision;
   ephemeral (round-alert equivalent) only for pure flavor/narration.
4. **Ship the "ambient-only" global toggle from day one** — a setting that
   collapses all `EventToasts` into a quiet top-bar/`OutlinerOverlay`
   indicator strip with zero interruption, for players who want the wire
   feed as texture, not as a chore.

---

## 6. Tooltip / Progressive Disclosure System

Stellaris uses **nested tooltips** ("Concepts") — hovering a term inside a
tooltip opens a second tooltip explaining that term, originally scoped to
the galaxy setup screen and gradually expanded to species traits, alternate
government unlocks via ascension paths, and tech-tree terminology (the
"More Informative Tech Trees with Nested Tooltips" mod exists specifically
because the base game's coverage was inconsistent).

**Community critique is split but converges on one rule.** A Paradox
Forums thread titled "Reminder to the devs that nested tooltips is bad UX
design" argues against overusing them (full thread text was behind a
bot-check wall and not independently retrievable in this research pass —
flagging as unverified beyond the title/topic), but the surviving community
counter-discussion converges on a specific principle: **nested tooltips are
valuable for composite/variable data (e.g., what does this Origin actually
grant right now, given my current tech and species) but actively harmful
for constant/reference data** (e.g., "what does Standard of Living mean" —
a static glossary definition should not require a hover-chain; it should be
inline or in a persistent reference panel).

**Sources:** [Concepts (nested tooltips) barely used at all? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/3828666917399193408/),
[Reminder to the devs that nested tooltips is bad UX design — Paradox Forums (title/topic only, content unretrievable)](https://forum.paradoxplaza.com/forum/threads/reminder-to-the-devs-that-nested-tooltips-is-bad-ux-design.1702017/),
[More Informative Tech Trees with Nested Tooltips — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/more-informative-tech-trees-with-nested-tooltips.1598392/)

**Babylon read:** directly actionable rule for `InspectionStack`'s
"every number explains itself" mandate — **reserve nested/recursive
tooltip drill-down for numbers that are actually computed from current
state** (imperial rent gap, a specific contradiction's current tension
value, a territory's live control ratio), and use flat inline glossary text
or a static reference panel for constant definitions (what IS "imperial
rent," what IS a "rupture event" — these don't change tick to tick and
nesting them wastes a click). Confirms the project's own instinct
(Victoria-3-style recursive inspection) is sound specifically for *dynamic*
values, with a documented community-sourced caveat about where it breaks
down.

---

## 7. Planet/Pop Management Drill-Down — the rework history

This is the deepest and most transferable case study for Babylon's
territory/county drill-down design, because Stellaris has rebuilt this
exact surface twice at the architecture level.

### Generation 1 → 2: the 2018 tile-to-district rework (2.2 "Le Guin")

The original planet UI was a **grid of 25 fixed tiles**, one pop and one
building per tile, directly clickable on a rendered planet surface. Per the
dev-diary framing, this "worked well for visualization and early-game
economic decisions, but was constrictive" — the hard 25-tile/pop/building
cap didn't scale to the mid/late game, and clicking individual tiles to
manage buildings became repetitive busywork rather than a decision.

The 2.2 rework replaced tiles with **districts** (categories of
developable land: housing, mining, generator, etc., each with a numeric
capacity rather than discrete visual slots) and **jobs** as the unit
resources flow through. Jobs split into **capped jobs** (limited by how
many districts of the relevant type exist — e.g., mining jobs capped by
mining-district count) and **uncapped jobs** (limited only by available
pop labor). Stated design goals: "simulating a wide variety of different
societies," "more interesting choices about how to develop planets," and
**reducing micromanagement** — i.e., the fix deliberately traded visual
tile-clicking for numeric/list-based allocation to cut the click count per
decision.

### Generation 2 → 3: the 4.0 "Phoenix" pop-groups rework (2026)

By the 4.0 update, individual per-pop simulation had become both a UI
burden (long, undifferentiated pop lists) and a performance bottleneck.
The fix: **Pop Groups** — pops are aggregated by species × strata × ethics
× faction, and "most things that previously affected or manipulated 1 pop
would now affect or manipulate groups of 100." Pop Groups generate
**Workforce**, which fills Jobs; planets (not individual pops) produce
resources from that workforce. This is explicitly a **scale change to the
underlying data model to serve the UI**, not just a visual tweak — the
devs "took the opportunity to streamline some aspects of planetary
management and improve the planet UI" while doing it.

Concrete UI deltas shipped alongside 4.0: clearer visual cues for when
District Specializations can be built; the Build Queue now auto-opens
inside the Surface/Management tab if it's non-empty (surfacing queued work
instead of requiring a manual tab click to discover it); and a Management
tab that shows **which faction each pop group has joined and which
ethics/factions are represented within it** — i.e., political composition
became a first-class, always-visible planet-level readout rather than
something requiring per-pop inspection.

### The performance regression as a documented cautionary tale

Community post-launch analysis of 4.0 (Paradox Forums, "Why Did Late Game
Pop Lag Exist in the First Place?" and "Some observations on late-game
lag") reports that **the rework moved lag from late-game to mid/early-game
and, by some player accounts, made overall performance worse than the
pre-rework build** despite performance being the explicit stated goal.
Root-cause discussion in the community centers on ship-count (not
pop-count) driving most of the per-tick cost via daily modifier/check
recomputation multiplied across the whole galaxy, suggesting the pop
aggregation fix targeted the wrong bottleneck.

**Sources:** [Stellaris Dev Diary #121 — Planetary Rework (part 1 of 4)](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-121-planetary-rework-part-1-of-4.1115043/),
[Stellaris Dev Diary #122 — Planetary Rework (part 2 of 4)](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-122-planetary-rework-part-2-of-4.1115992/),
[Stellaris Dev Diary #370 — 4.0 Changes Part 4](https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-370-4-0-changes-part-4.1728047/),
[Stellaris Dev Diary #371 — 4.0 Changes: Part 5](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-371-4-0-changes-part-5.1729244/),
[Why Did Late Game Pop Lag Exist in the First Place? — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/why-did-late-game-pop-lag-exist-in-the-first-place.1545278/),
[Some observations on late-game lag — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/some-observations-on-late-game-lag.1773613/)

**Babylon read:** four transferable lessons for territory/county/hex
drill-down:

1. **Discrete visual slots (tiles) don't scale; aggregate-with-drill-down
   does.** Babylon's hex→county→state aggregation is already
   architecturally aligned with the district/job model rather than the old
   tile model — validate that county-level panels show aggregate capacity
   (e.g., total organizing capacity in a county) with drill-down into
   contributing hexes, not a flat grid of every hex as a clickable tile.
2. **Political/faction composition should be an always-visible readout at
   the aggregate level**, mirroring the 4.0 Management tab's per-planet
   faction breakdown — Babylon's county/territory inspection panel should
   show class/faction composition inline, not require opening each
   social-class node individually.
3. **Surface pending/queued work automatically** — the Build Queue
   auto-open pattern (show players what's in-flight without a manual tab
   click) applies directly to any Babylon panel with a mobilize/organize
   queue.
4. **Aggregation-for-performance and aggregation-for-UI-clarity are not
   automatically the same fix** — Stellaris's 4.0 lesson is a direct
   warning: don't assume that simplifying the *visual* representation
   (fewer distinct pop rows) will fix the *simulation* cost (still O(n)
   per-tick work somewhere else, in Stellaris's case ships). Babylon's own
   26-system tick cost and Territory↔FIPS work (see
   `tick52-territory-fips-fixed.md`) should be profiled independently of
   any UI aggregation decisions, not assumed to move together.

---

## 8. Onboarding

Coverage across guide sites and reviews converges on: Stellaris pairs raw
complexity with a **built-in AI-companion tutorial** that is explicitly
player-controllable — players can dismiss individual tips permanently while
choosing to keep others recurring, rather than an all-or-nothing tutorial
toggle. Reviewer consensus (aggregated via guide/review search, not a
single citable long-form piece) frames this as one of Paradox's stronger
onboarding efforts for the genre: the tutorial "interferes" only as much as
the player allows, and the sheer system count is still acknowledged as
overwhelming for genre newcomers even with the tutorial active.

**Sources:** [Beginner's guide — Stellaris Wiki (Fandom)](https://stellaris.fandom.com/wiki/Beginner's_guide) *(aggregated search summary; individual guide-site claims not independently re-verified)*

**Babylon read:** low-confidence/secondary source for this section —
flagging that this claim set came from an aggregated search summary rather
than one fetched long-form article, unlike Sections 4–7. The actionable
takeaway that IS well-supported by the pattern (regardless of source
quality) is the **per-tip dismissal model**: let players permanently
silence a specific onboarding tip while keeping others live, rather than a
single global "skip tutorial" toggle — directly applicable to any Babylon
first-run coachmark/tooltip system.

---

## 9. Accessibility

No official colorblind mode exists in Stellaris; the game's status/alert
iconography relies heavily on green/red coding, which the community has
flagged as a problem for colorblind players since at least 2018 (Twitter
thread from the official Stellaris account referenced in search results,
undated forum requests continuing through recent years). The gap is filled
entirely by community mods (e.g., "Color coded pop status icons," tuned for
Protanopia/Deuteranopia). A Paradox developer forum response (source:
community-summarized, not independently fetched) reportedly characterized a
full colorblind-mode UI redesign — bundled with keyboard-navigation
support — as too large an undertaking, with interfacing to an external
accessibility tool floated as a more realistic long-term path instead of a
native in-game mode.

**Sources:** [Colorblind support — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/colorblind-support.1064744/),
[Color blind mode for resource display on galactic map — Steam Community](https://steamcommunity.com/app/281990/discussions/0/350532795330779034/),
[Color coded pop status icons (Color Blind) — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=784045344)

**Babylon read:** a clear anti-pattern to avoid, not a pattern to copy.
Given Babylon's "Cold Collapse" palette is dark/cyan-accented (already a
constrained hue range, which helps), status/alert states (contradiction
severity, control ratio, solidarity vs. repression) should be
differentiable by **shape/icon or luminance, not hue alone**, decided at
design time rather than retrofitted after a decade of player requests the
way Stellaris has left this unresolved.

---

## 10. Top 3 Community UX Complaints (synthesized across sections above)

1. **Notification/alert fatigue from an undifferentiated single channel** —
   player-actionable alerts compete with ambient world-flavor events for
   the same visual/audio channel, entries fragment per-instance instead of
   grouping, and auto-dismiss timing doesn't match reaction time needed.
   (Section 5.) Stellaris's own Situation Log fix (Section 4, 2026) is the
   template for how to *actually* resolve this, seven-plus years after the
   complaints started.
2. **Overloaded galaxy-scale iconography ("banner soup")** — resource,
   ownership, and fleet icons at galaxy zoom grow large and numerous enough
   to occlude the map itself, addressed only by third-party UI mods, never
   fully fixed in the base game. (Section 2.)
3. **UI/data-model rework introducing new performance and complexity costs
   even when framed as a simplification** — the 4.0 Pop Groups rework was
   sold as both a UI clarity win and a performance win, but community
   post-mortems report the performance goal partly failed (lag
   redistributed rather than reduced) because the actual bottleneck (ship
   count) was elsewhere. (Section 7.) This is a process lesson as much as
   a UI one: a UI simplification pass should not be assumed to fix an
   unrelated performance problem without profiling first.

---

## Summary Table

| Surface | Stellaris pattern | Babylon translation |
|---|---|---|
| Outliner | Persistent right-edge panel, 4 tabs, is the real HUD | `OutlinerOverlay` should be primary nav, map secondary |
| Map modes | Small flat set (6) + independent overlay toggles | Keep lens picker + overlay toggles orthogonal |
| Zoom LOD | Discrete snap-zoom levels; detail gated to zoom band | Snap-zoom state→county→hex; gate hex grid visibility to zoom |
| Situation log | 2026 rework: urgency-ordered categories + Findings tab + pinning | Split urgent contradiction feed from narrative wire feed from day one |
| Alerts | Square (persistent)/round (ephemeral) + red/orange/green | `EventToasts`: persistent for actionable, ephemeral (generous timing) for flavor |
| Tooltips | Nested "Concepts," criticized when used for constant data | Nest only for computed/dynamic values; flat glossary for constants |
| Planet drill-down | Tiles→districts→pop groups, each a full architecture rework | Aggregate-with-drill-down (hex→county→state), not flat tile grids |
| Onboarding | AI companion, per-tip dismissal, not global skip | Per-coachmark dismissal, not all-or-nothing tutorial |
| Accessibility | No native colorblind mode; hue-only status coding | Use shape/luminance, not hue alone, for status states |

---

## Source List

- [Main interface — Stellaris Wiki](https://stellaris.paradoxwikis.com/Main_interface)
- [User interface — Stellaris Wiki (Fandom)](https://stellaris.fandom.com/wiki/User_interface)
- [Empire interface — Stellaris Wiki](https://stellaris.paradoxwikis.com/Empire_interface)
- [Best UI Mods for Stellaris — FandomSpot](https://www.fandomspot.com/stellaris-ui-mods/)
- [Better Galaxy map UI — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/better-galaxy-map-ui.997748/)
- [Galaxy Map and System View zooming — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/galaxy-map-and-system-view-zooming.962148/)
- [Move camera to system without zooming in? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/1639792569852006960/)
- [Stellaris Dev Diary #417 — Situation Log Updated (Paradox Forums thread)](https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-417-situation-log-updated.1918530/)
- [Stellaris Dev Diary #417 — Situation Log Updated notes (xpgained.co.uk summary)](https://xpgained.co.uk/patch-notes/stellaris-dev-diary-417-situation-log-updated-notes-23rd-april-2026)
- [Too many notifications I don't care about — Steam Community](https://steamcommunity.com/app/281990/discussions/0/3781371514569030827/)
- [Does anyone else find the incessant notifications frustrating? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/4203492762829778326/)
- [Don't auto-dismiss notifications — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/dont-auto-dismiss-notifications.1010326/)
- [UI Issues — Steam Community](https://steamcommunity.com/app/281990/discussions/0/357285562496225464/)
- [Concepts (nested tooltips) barely used at all? — Steam Community](https://steamcommunity.com/app/281990/discussions/0/3828666917399193408/)
- [Reminder to the devs that nested tooltips is bad UX design — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/reminder-to-the-devs-that-nested-tooltips-is-bad-ux-design.1702017/) *(title/topic verified; body text unretrievable behind bot-check)*
- [More Informative Tech Trees with Nested Tooltips — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/more-informative-tech-trees-with-nested-tooltips.1598392/)
- [Stellaris Dev Diary #121 — Planetary Rework (part 1 of 4)](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-121-planetary-rework-part-1-of-4.1115043/)
- [Stellaris Dev Diary #122 — Planetary Rework (part 2 of 4)](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-122-planetary-rework-part-2-of-4.1115992/)
- [Stellaris Dev Diary #370 — 4.0 Changes Part 4](https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-370-4-0-changes-part-4.1728047/)
- [Stellaris Dev Diary #371 — 4.0 Changes: Part 5](https://forum.paradoxplaza.com/forum/threads/stellaris-dev-diary-371-4-0-changes-part-5.1729244/)
- [Stellaris Dev Diary #372 — Modding: Pop Groups and Jobs](https://admin-forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-372-modding-pop-groups-and-jobs.1729994/)
- [Why Did Late Game Pop Lag Exist in the First Place? — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/why-did-late-game-pop-lag-exist-in-the-first-place.1545278/)
- [Some observations on late-game lag — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/some-observations-on-late-game-lag.1773613/)
- [Stellaris review — PC Gamer](https://www.pcgamer.com/stellaris-review/) *(cited via search-engine summary; full article body not independently fetched — see note below)*
- [Beginner's guide — Stellaris Wiki (Fandom)](https://stellaris.fandom.com/wiki/Beginner's_guide) *(aggregated search summary)*
- [Colorblind support — Paradox Forums](https://forum.paradoxplaza.com/forum/threads/colorblind-support.1064744/)
- [Color blind mode for resource display on galactic map — Steam Community](https://steamcommunity.com/app/281990/discussions/0/350532795330779034/)
- [Color coded pop status icons (Color Blind) — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=784045344)

**Note on source access:** several Paradox Forums threads (dev diary #417
thread body, the nested-tooltips-is-bad-UX thread) and the PC Gamer review
page returned Cloudflare bot-check pages or membership-wall content when
fetched directly; those claims are sourced from search-engine result
summaries instead of primary-fetched text and are flagged inline above.
Direct dev-diary quotes (Doğa on Situation Log rationale) were recovered via
a third-party patch-notes aggregator (xpgained.co.uk) that reproduced the
forum post text.
