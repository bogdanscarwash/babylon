# UX Deep-Dive: Europa Universalis IV

**Genre:** Grand strategy, 1444–1821 map painter. **Developer:** Paradox Development Studio. **Released:** 2013, still receiving DLC/patches as of 2026 (12+ years of continuous UI accretion — the longest-lived UI of any Paradox title, and now a direct point of comparison against the newly-announced Europa Universalis V). Chosen for this research pass because of the **map-mode row**, the **macrobuilder as a progressive-disclosure production hub**, the **ledger**, the **peace-deal negotiation screen**, and — uniquely useful for Babylon — over a decade of hard evidence about **how DLC-driven feature growth strains a UI**, including the community's own post-mortem now that EU5 exists to compare against.

---

## 1. Overall Information Architecture

EU4's screen divides into four persistent zones, all **floating over a full-bleed map** — there is no "screen takeover" for anything except modal windows (province view, macrobuilder, ledger, peace deal), which open as large panels that still leave the map visible behind/around them.

- **Upper-left:** country shield (portal into the country interface), the macrobuilder button beneath it, a gilded resource bar (treasury, manpower, sailors, stability, corruption, prestige, national-unity stat, power projection), envoy-portrait counts (merchants/colonists/diplomats/missionaries as X/Y), a banner showing monarch points (admin/diplomatic/military), the current Age panel, and color-coded alert flags (red = urgent, yellow = lesser, green = non-urgent).
- **Upper-right:** pause/play, a speed dial (1–5, "5 = as fast as the computer can manage"), date display (hover reveals currently-playing music), a timeline/eye icon for historical playback, skip-track, score/achievements.
- **Right side:** the **outliner** — a live, user-configurable data table of the player's own nation (expand via a "+" to add more tracked fields beyond the defaults).
- **Lower-right:** the **minimap** (click-to-jump navigation, colored dots for allied/enemy unit positions, a viewport box) with the **map-mode panel** stacked above it.

Specialized government-type panels (HRE, Chinese Emperor, Papal Curia, Factions, native councils, Parliament, Mesoamerican mechanics) mount additional floating widgets **conditionally**, based on the played nation/government — an early, crude form of contextual chrome that only shows what's relevant to *this* playthrough.

**Babylon takeaway:** the "everything floats, nothing steals the whole screen except a deliberate modal" pattern is exactly the ActionDock/TopBar/OutlinerOverlay split Babylon is aiming for. The conditional-panel-by-government-type idea maps directly onto showing different ActionDock verb sets by faction type (state vs. insurgent org vs. cartel).

*Source: [Ingame screen — EU4 wiki](https://eu4.paradoxwikis.com/Ingame_screen)*

---

## 2. Map Modes: the Row Babylon Was Asked to Study

EU4's map modes are **not** a flat row of 5 icons — that's Victoria 3's approach (see sibling report). EU4 groups a much larger set into **four color-coded tabs on the minimap**, each opening a sub-panel of buttons:

| Tab | Included modes |
|---|---|
| **Diplomatic** | Coalition, Diplomatic, Federations, Opinion, Players, Rivals, Victory cards, Military Access, Truce, Trade Leagues, Great Powers |
| **Economic** | Buildings, Colonial, Development, Local Autonomy, Loot, Manpower, Technology, Institutions, Trade, Trade Goods, Trade Value, Devastation, Piracy |
| **Geographical** | Areas, Climate, Fort Level, Overseas provinces, Regions, Simple Terrain, Territories and states, Supply Limit, Terrain, Colonial and Trade Regions, Weather, Subcontinents, Great Projects |
| **Political** | Accepted Cultures, Culture, Dynastic, Estates, Government types, Imperial, Political, Rebel types, Religious, Trade league, Unrest, Religious Leagues, Metropolitan, State Edicts, Revolution |

That's **40+ distinct map modes** as of the current version — a direct, measurable artifact of 12 years of DLC each adding "just one more lens." Switching is via **hotkeys Q–P** (customizable — right-click a minimap button slot, left-click the desired mode to bind it; multiple modes can share one hotkey and cycle on repeated presses) or by clicking the minimap buttons directly.

Individual mode behaviors worth noting as design patterns:
- **Political** — flat nation-color fill; **diagonal stripes** flag anomalies (non-owner-controlled provinces) without a separate mode.
- **Religious** — color by religion; stripes flag province/owner religion mismatch; **progress bars render directly on the map** for provinces mid-conversion (no separate window needed).
- **Diplomatic** — color by relation to selected/played nation; green stripes = non-cored, green stripes outside borders = non-owned cores, yellow = non-owned claims — a single mode encoding four distinct legal-status facts through stripe color+position.
- **Coalition** — colors scale by an opinion modifier's *magnitude* relative to a selected province's owner (a genuine "how mad is everyone at me" heatmap).
- **Trade league** — while a league leader, red/green stripes preview whether an uncommitted nation *will* join if invited, before the player spends the action.

**Babylon takeaway (direct, this is the assignment's core ask):**
- The 4-tab grouping (Diplomatic/Economic/Geographical/Political) is a proven taxonomy for organizing lenses once the count exceeds ~8; Babylon's map lenses (control/political, solidarity, exploitation/rent, contradiction-tension, transport corridors, sovereignty/collapse) could pre-adopt a 3–4 group taxonomy now (Material/Relational/Political/Terrain) so growth doesn't force a painful regroup later — the EU4 forum threads below show what happens when regrouping is deferred too long.
- **Encode multiple facts in one mode via stripe/border treatment** rather than proliferating near-duplicate modes — e.g. Babylon's political lens could stripe contested/atomized hexes instead of needing a separate "atomization" mode.
- **In-map progress bars** (religious conversion) beat a modal for simple scalar progress — Babylon could render tick-countdown or consciousness-threshold progress directly on hex fill rather than requiring an InspectionStack open.
- **Preview-before-commit striping** (trade league join-preview) is a strong pattern for Babylon's OODA/verb system: striping a hex to preview "will this org's SOLIDARITY edge form if I mobilize here" before the player spends the action point.

*Sources: [Map modding — EU4 wiki](https://eu4.paradoxwikis.com/Map_modding), [Maps — EU4 wiki](https://eu4.paradoxwikis.com/Maps)*

---

## 3. The Macrobuilder: Progressive Disclosure as a Verb Hub

The macrobuilder (hotkey **B**, button beneath the country shield) is the single most relevant EU4 pattern for Babylon's ActionDock. It is **one modal window with 12 tabs** (numeric keys 1–0 jump to the first ten), each a filtered, sortable, color-coded list-view over the player's entire empire for one category of repeatable action:

1. Land units (recruit — color-codes provinces green/red for recruitment viability)
2. Naval units (recruit)
3. Province coring
4. Religious conversion (shows missionary strength + conversion progress)
5. Local autonomy adjustment (with unrest indicators)
6. Culture conversion
7. Buildings (construction with upgrade paths)
8. Province development (cost + resulting autonomy shown inline)
9. States (state-level management)
10. Diplomacy (8 further sub-tabs)
11. Exploit development (resource extraction, gated behind a DLC — sortable columns)
12. Army/Navy planner (reusable unit templates)

Every row exposes **hover tooltips with expanded detail**, **in-row progress bars** for anything already in motion, and **checkboxes to include/exclude subject or overseas provinces** from bulk actions — i.e., the macrobuilder is simultaneously a *dashboard* (what's happening across my whole empire right now) and an *action queue* (what do I want to happen next), merged into one screen instead of split across separate "status" and "orders" UIs.

**Known failure mode, confirmed by the community:** the macrobuilder's building/investment tabs **visibly lag once a nation exceeds ~50 provinces**, because — per player and modder reports — the list appears to **fully re-render from scratch on every single queue/dequeue action** rather than patching the changed row. This is a naive-invalidation bug, not a design flaw, but it's exactly the kind of thing that only surfaces at scale, and Babylon's late-game empire size (hundreds of hexes/territories under one faction) is the same shape of risk for any list-style overlay (OutlinerOverlay, a hypothetical macro-action panel) that re-renders the whole list per state change.

**Babylon takeaway:**
- A single tabbed modal, keyed 1–0/B, that merges "what's in progress" + "what can I queue" per category is a strong model for a Babylon **ActionDock expanded view** — e.g. tabs for Mobilize/Educate/Campaign/Aid/Investigate keyed to the 9 verbs, each showing eligible hexes/orgs color-coded by feasibility plus in-flight action progress bars, rather than one verb = one disconconnected modal.
- The lag bug is a concrete argument for **virtualized/windowed rendering** (only mount visible rows) in any Babylon overlay that lists per-hex or per-org state at national scale, and for diffing list updates instead of full remounts.

*Sources: [Macrobuilder — EU4 wiki](https://eu4.paradoxwikis.com/Macrobuilder), [Macro builder and province UI — Paradox forum](https://forum.paradoxplaza.com/forum/threads/macro-builder-and-province-ui.1171793/), [Literally cannot use the macrobuilder... due to ridiculous lag — Paradox forum](https://forum.paradoxplaza.com/forum/threads/literally-cannot-use-the-macrobuilder-for-buildings-or-trade-investments-due-to-ridiculous-lag.1502256/)*

---

## 4. The Ledger: Comparative Data, Not Just Self-Report

The ledger (hotkey **L**, bottom-right corner button) is a **multi-page statistical browser across every nation in the game**, not just the player's own — this is its defining, distinguishing feature versus the outliner (which is self-only). Pages are grouped thematically (Country/Military/Economy/Buildings/etc.) with a **table-of-contents jump list** at the top and a **row of quick-nav links at the bottom of every page** so the player never has to scroll back to the top to change section. Data renders as sortable graphs/charts comparing the player against all AI nations simultaneously (tech levels, manpower, income, army/navy size, force limits, inflation, gross income). Useful toggles let the player **filter the comparison set down to just Rivals or just War Enemies** — i.e., the ledger supports both "how do I compare to the whole world" and "how do I compare to the 3 nations I actually care about" without separate screens.

**Confirmed limitation** (Steam community thread, unanswered as of capture): players have asked for the ability to **customize which fields the ledger displays** or export/filter data further, and as of that thread no such customization exists — the ledger's page set is fixed by the developers, not user-configurable beyond the show/hide toggles it already has.

**Babylon takeaway:** the "compare all vs. compare only Rivals/Enemies" toggle is directly applicable to a Babylon global-stats overlay — a filter that narrows national/faction comparison tables to "factions I have SOLIDARITY/EXPLOITATION edges with" rather than the full roster. The bottom-of-page quick-nav-links pattern is a good answer for Babylon's InspectionStack when a nested panel gets deep: persistent jump anchors beat relying on scroll position.

*Sources: [Ledger — EU4 wiki](https://eu4.paradoxwikis.com/Ledger), [Modding and Ledger pages — Steam Community](https://steamcommunity.com/app/236850/discussions/0/365172547941290293/)*

---

## 5. Peace-Deal UI: A Constrained Budget as the Whole Interaction

Peace negotiation is opened by right-clicking the war emblem (bottom-right shield during an active war) and drops the player into a dedicated screen built around one core mechanic: **war score is a spendable budget**, and every demand the player adds (province cession, vassalization, tribute, religious conversion, war reparations, etc.) is priced against that budget in real time, with the UI showing the running total warscore-cost of the whole package as it's assembled. Demands *inside* the justified war goal are "free" (paid from war score only); demands *outside* it ("unjustified") additionally cost diplomatic power (2 per point of development affected, halved by a tech/idea investment) — a **second currency gate layered on top of the first**, both visible simultaneously so the player can see exactly why an ambitious peace is expensive along two independent axes.

An icon in the corner of the screen lets the player **preview AI acceptance odds** before submitting — the AI's accept/reject math (positive reasons from warscore surplus vs. negative reasons from demand cost, capped at −1000 for wildly unreasonable asks) is exposed as a legible signal rather than a black box, so the player isn't guessing whether a demand package will be rejected.

**Babylon takeaway:** the "one budget bar that fills as you stack demands, with a live accept-probability preview" pattern is directly reusable for Babylon negotiation-style verbs (negotiate, and any faction-to-faction bargaining) — a running-cost meter plus an exposed acceptance-likelihood readout turns an opaque AI decision into a legible one, which is core to the "every number explains itself" Victoria-3-style goal already stated for Babylon.

*Source: [Peace deal — EU4 wiki](https://eu4.paradoxwikis.com/index.php?title=Peace_deal&redirect=no)*

---

## 6. Tooltips, Message Settings, and Event Popups

- **Tooltips do not nest arbitrarily.** The community-documented reason: a tooltip auto-dismisses when the cursor leaves its bounding box, so a "tooltip inside a tooltip" that spawned far from the cursor would have no reliable dismiss trigger — this is a *deliberate interaction constraint*, not an oversight, and it caps how deep EU4 lets progressive disclosure go via hover alone (deeper detail requires a click into a real panel, not more hovering).
- **Message Settings** is a dedicated, granular menu (reachable from the game menu, and from a small icon in the corner of any popup) that exposes **per-message-type toggles across a large taxonomy of event/notification categories** — "a huge number of message types." Critically, the corner icon on any given popup jumps straight into the settings for *that message's type*, so muting a recurring notification is a two-click in-context action, not a trip through a global settings tree.
- **Events** render as modal popups that either present a choice or simply require acknowledgment; separatist-sentiment and similar recurring popups are explicitly requested-for and supported as individually-mutable in Message Settings (per Steam community threads).

**Babylon takeaway:** the in-context "mute this notification type from right here" affordance is a strong, low-cost pattern for Babylon's EventToasts — a small control on each toast that jumps to (or directly toggles) that event category's visibility, rather than forcing players into a separate global notification-settings screen to quiet down a specific recurring wire story. The hard tooltip-nesting limit is also a useful constraint to adopt deliberately for Babylon's InspectionStack: hover can go one level deep safely; anything deeper should require a click to pin the panel open.

*Sources: [Events — EU4 wiki](https://eu4.paradoxwikis.com/Events), [Pop up info — Steam Community](https://steamcommunity.com/app/236850/discussions/0/1457328392109896899/), [Separatist Sentiment - disable pop-up — Steam Community](https://steamcommunity.com/app/236850/discussions/0/3163209341697702736/)*

---

## 7. Onboarding

EU4's onboarding is thin by modern standards — it leans on the **in-fiction bookmark/scenario-select screen** doing double duty as a soft tutorial: choosing a start year and nation surfaces that nation's shield, description, and starting stats before the player even loads in, plus a lightbulb-icon national-ideas preview, letting players self-select an appropriately-scoped starting position (small/simple nation vs. sprawling empire) rather than the game gating complexity behind a formal tutorial track. There is a discrete in-game tutorial, but community and press commentary (see §8) consistently treats the interface itself, not a tutorial mode, as the primary teaching surface — tooltips-on-everything substituting for guided instruction. PC Gamer's review captured this tradeoff directly: *"At any given time, you can get as much or as little help as you need from the interface regarding what you should be focusing on," while conceding "convoluted underlying systems that will only be comprehensible to the most in-depth and experienced players."*

**Babylon takeaway:** self-selecting starting scope via a rich scenario-picker (rather than a mandatory tutorial) is worth weighing for Babylon's own start-screen — letting a first-time player choose a smaller/simpler starting faction naturally scopes their first exposure to the ActionDock's 9 verbs without gating content behind a forced walkthrough.

*Source: [Europa Universalis IV review — PC Gamer](https://www.pcgamer.com/europa-universalis-iv-review/)*

---

## 8. Top 3 Community UX Complaints

Synthesized from Paradox forum threads ("EU4 has an UI problem," "Opinion: The biggest issue with EU4 is the UI and not the game mechanics") and Steam community discussions:

1. **Conceptually-related features are scattered across unrelated UI locations.** The forum's own framing: *"Policies, trade policies, native policies, and naval doctrines are things that belong sort of together, but the latter three are all hidden away in awkward spots in the UI."* This is the single most-repeated complaint — grouping in the data model (all are "policy-like modifiers") does not match grouping in the UI (each type lives in a different menu, opened a different way).
2. **State/context loss between screens.** Reopening the Diplomacy screen doesn't reliably return to the country you were just looking at — *"sometimes it doesn't show the country you expect, especially when you close the interface and later open it up again, expecting your country but getting the last country viewed instead."* This is a persistence/state-management bug that reads as a design failure because it breaks the player's mental model of "the UI remembers what I was doing."
3. **Information density without hierarchy, worsened by cumulative DLC.** Forum consensus: *"The UI displays too much information with too many interactions with very little organization... an UI cleanup is needed, preferably before any new mechanics are added."* Players explicitly connect this to the DLC model — *"[the interface] will become increasingly dense as DLC is added over time"* — each expansion adding its own panel/tab/icon without a periodic pass to re-integrate them into the existing hierarchy. Notably, in the EU5-vs-EU4 comparison thread (2025-26), longtime EU4 players **retroactively praised EU4's density as "ornate and historical looking"** compared to EU5's flatter early UI, calling EU5's version bland and "lacking flavour, shape, or colour" — suggesting the community's complaint was never really about density itself but about density *without organization*; visually rich clutter that is still navigable reads as "textured," while sparse clutter reads as "broken."

**Babylon takeaway:** all three complaints are organizational, not aesthetic, and all three are preventable at design time rather than fixable after the fact: (1) group Babylon's verbs/lenses by the conceptual relationships already encoded in the graph (SOLIDARITY-adjacent actions together, EXPLOITATION-adjacent together) rather than by implementation order; (2) make every Babylon overlay/modal remember its last-viewed subject per session (InspectionStack should restore state, not reset to a default); (3) budget for a periodic "re-integration pass" once Babylon's own map-lens/verb count grows past what fits in one row — treat that as a scheduled task, not a someday-cleanup.

*Sources: [EU4 has an UI problem — Paradox forum](https://forum.paradoxplaza.com/forum/threads/eu4-has-an-ui-problem.1121519/), [Opinion: The biggest issue with EU4 is the UI and not the game mechanics — Paradox forum](https://forum.paradoxplaza.com/forum/threads/opinion-the-biggest-issue-with-eu4-is-the-ui-and-not-the-game-mechanics.1485322/), [Why YES, the UI is really an issue — Steam Community (EU5 forum, EU4 comparison)](https://steamcommunity.com/app/3450310/discussions/0/667222425710107354/)*

---

## 9. What 12+ Years of DLC Taught (Extensibility Lessons)

- **Governing Capacity dev diary (Oct 2020)** is the clearest first-party evidence of Paradox *reacting* to UI strain from feature accretion: as empires grew (a downstream effect of years of expansion content), the team added governing-capacity cost previews **directly on the building-construction row** ("buildings that affect governing capacity now show how much capacity they will remove if built in a specific province") and exposed a previously-invisible stat ("there was no previous way to see how much governing capacity a vassal had, but this information is now viewable under the subject interface") — i.e., the fix for a numbers-are-hidden complaint was to surface the number at the point of decision, not to build a new separate screen for it. This is a repeatable pattern: **when a new mechanic's cost isn't visible where the player commits to it, players experience it as the UI "hiding" information, even if a ledger page technically has it somewhere.**
- **Map modes grew from Paradox's own baseline (~10 core modes) to 40+ modes across 4 tabs** as each expansion added its own lens (trade leagues, federations, great projects, subcontinents) — never consolidated back down. The 4-tab taxonomy absorbed the growth reasonably well *because it existed from early on*; retrofitting a taxonomy after the fact is harder than establishing one before the count gets unwieldy.
- **The macrobuilder's 12-tab structure similarly absorbed new verb categories (Exploit Development, arriving via a later DLC) into an existing tab slot** rather than requiring a new top-level UI surface — evidence that a well-designed tabbed-modal container can host mechanics that didn't exist at its original design time, *if* the tab-add cost is kept low.
- **Where extensibility failed:** related-but-scattered small features (policies/trade policies/native policies/naval doctrines, §8) were each added at the time their mechanic shipped, to whatever menu was closest at hand, with no retroactive audit pass to consolidate them once the pattern ("these are all policy-like") became obvious in hindsight. The lesson is asymmetric: **container-level extensibility (tabs, mode-groups) scaled fine; ad hoc single-feature UI additions did not**, because nothing forced a "does this belong with something that already exists" check at ship time.

**Babylon takeaway:** build Babylon's lens-groups and ActionDock verb-tabs as containers designed to absorb new entries from day one (as EU4's did), and — critically — treat "which existing group does this new mechanic's UI belong to" as a mandatory design-review question for every new feature, not an optional cleanup deferred to later. The governing-capacity lesson (surface the cost where the player commits, don't bury it in a separate ledger page) should directly inform how Babylon shows OODA action costs on the ActionDock itself rather than requiring a trip to a stats overlay.

*Sources: [Dev Diary: Governing Capacity & UI — Steam News](https://store.steampowered.com/news/app/236850/view/2951506549417716894), [Developer diaries archive — EU4 wiki](https://eu4.paradoxwikis.com/Developer_diaries)*

---

## 10. Accessibility

EU4 has **no first-party accessibility settings menu** for vision/legibility beyond a manually-edited `gui_scale` value in a settings file (Documents > Paradox Interactive > Europa Universalis IV) — there is no in-game slider. Legibility and colorblind-friendliness are addressed almost entirely by the **community**: the Hyper Legible Font Mod (built on the Atkinson Hyperlegible typeface, designed for low-vision readability), the Royal Eagle UI mod (higher-contrast outlines, more vibrant color coding), and assorted UI-scaling mods. Repeated Steam-forum threads ("Text too small cant read," "Font Size," "Can you change the font size?") over multiple years indicate this is a **standing, unaddressed complaint** rather than a one-off.

**Babylon takeaway:** given Babylon's dark, cyan-accented "Cold Collapse" palette and dense data surfaces (InspectionStack numbers, wire text), a first-party text-scale and high-contrast toggle should be treated as a base requirement, not a stretch goal — EU4's 12-year gap here is a clear cautionary data point, not a pattern to emulate.

*Sources: [Hyper Legible Font Mod — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=2985308403), [Royal Eagle UI — GitHub](https://github.com/Coyote-31/eu4_mod_Royal_Eagle_UI), [Interface / Text Size — Steam Community](https://steamcommunity.com/app/236850/discussions/0/864974880507428588/)*

---

## Sources

- [User interface — EU4 wiki](https://eu4.paradoxwikis.com/User_interface)
- [Ingame screen — EU4 wiki](https://eu4.paradoxwikis.com/Ingame_screen)
- [Maps — EU4 wiki](https://eu4.paradoxwikis.com/Maps)
- [Map modding — EU4 wiki](https://eu4.paradoxwikis.com/Map_modding)
- [Macrobuilder — EU4 wiki](https://eu4.paradoxwikis.com/Macrobuilder)
- [Ledger — EU4 wiki](https://eu4.paradoxwikis.com/Ledger)
- [Peace deal — EU4 wiki](https://eu4.paradoxwikis.com/index.php?title=Peace_deal&redirect=no)
- [Events — EU4 wiki](https://eu4.paradoxwikis.com/Events)
- [Developer diaries archive — EU4 wiki](https://eu4.paradoxwikis.com/Developer_diaries)
- [Dev Diary: Governing Capacity & UI — Steam News](https://store.steampowered.com/news/app/236850/view/2951506549417716894)
- [EU4 Development Diary Shows Off UI Improvements — GameWatcher](https://www.gamewatcher.com/news/2016-08-11-eu4-development-diary-shows-off-ui-improvements)
- [Europa Universalis IV review — PC Gamer](https://www.pcgamer.com/europa-universalis-iv-review/)
- [EU4 has an UI problem — Paradox Interactive Forums](https://forum.paradoxplaza.com/forum/threads/eu4-has-an-ui-problem.1121519/)
- [Opinion: The biggest issue with EU4 is the UI and not the game mechanics — Paradox Interactive Forums](https://forum.paradoxplaza.com/forum/threads/opinion-the-biggest-issue-with-eu4-is-the-ui-and-not-the-game-mechanics.1485322/)
- [Macro builder and province UI — Paradox Interactive Forums](https://forum.paradoxplaza.com/forum/threads/macro-builder-and-province-ui.1171793/)
- [Literally cannot use the macrobuilder... due to ridiculous lag — Paradox Interactive Forums](https://forum.paradoxplaza.com/forum/threads/literally-cannot-use-the-macrobuilder-for-buildings-or-trade-investments-due-to-ridiculous-lag.1502256/)
- [Modding and Ledger pages — Steam Community](https://steamcommunity.com/app/236850/discussions/0/365172547941290293/)
- [Pop up info — Steam Community](https://steamcommunity.com/app/236850/discussions/0/1457328392109896899/)
- [Separatist Sentiment - disable pop-up — Steam Community](https://steamcommunity.com/app/236850/discussions/0/3163209341697702736/)
- [Why YES, the UI is really an issue (it's both a UI and UX one) — Steam Community, EU5 forum](https://steamcommunity.com/app/3450310/discussions/0/667222425710107354/)
- [Hyper Legible Font Mod — Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=2985308403)
- [Royal Eagle UI mod — GitHub](https://github.com/Coyote-31/eu4_mod_Royal_Eagle_UI)
- [Interface / Text Size — Steam Community](https://steamcommunity.com/app/236850/discussions/0/864974880507428588/)
