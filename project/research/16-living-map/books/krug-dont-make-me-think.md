# Don't Make Me Think, Revisited — Steve Krug

UX research pass for the Babylon "living map" HUD redesign. Source: *Don't Make Me
Think, Revisited* (New Riders, 2014, 3rd ed.), Steve Krug. 214 pages; read strategically
(TOC + Chapters 1–4, 6, opening of 5 and 7 — the guiding-principles + navigation
material most relevant to scannability, self-evident UI, and mindless choices). Page
citations below refer to the book's own printed page numbers (footer), not PDF page
index.

Krug's stated method (p.9): usability means "a person of average (or even below
average) ability and experience can figure out how to use the thing to accomplish
something without it being more trouble than it's worth." Everything below is in
service of that bar, applied to a game HUD rather than a marketing site — the same
principles, harder mode, because a game additionally *wants* the player to feel
capable and in control, not just "get the job done."

---

## Core lessons

### 1. "Don't make me think" — self-evidence is the tie-breaker (Ch.1, pp.11–19)

**What the book says:** Krug's First Law of Usability: every page should be
self-evident, or failing that, self-explanatory (p.18: "If you can't make something
self-evident, you at least need to make it self-explanatory"). The test is that a
disinterested neighbor glancing at your Home page could say "Oh, it's a ___." Every
element that forces a moment's hesitation — a cute name, an ambiguous clickable, an
inconsistent label — produces a "question mark" over the user's head (pp.12–13), and
those question marks are individually tiny but cumulative: they erode confidence in
the whole product (p.15: "every question mark adds to our cognitive workload,
distracting our attention from the task at hand"). Two categories of question mark
Krug names explicitly: (a) unclear/cute/jargon names (p.14 — "Jobs" vs. "Employment
Opportunities" vs. "Job-o-Rama"), and (b) ambiguous clickability (p.15).

**Application to Babylon:**
- Every lens name, verb icon, and panel header in the cockpit must pass the
  "neighbor test" — a first-time player glancing at the map lens selector should be
  able to say "oh, that's the ethnic-solidarity view" without reading a tooltip. Kill
  MLM-TW jargon in primary labels (`Φ`, "Imperial Rent tensor," "OODA loop") from
  anything the player clicks before they've earned the vocabulary — push it one layer
  deeper, into the inspection-stack explainer, where a curious player can drill for it.
- Every clickable element in the HUD (hex tiles, wire feed items, org icons, verb
  dock buttons) needs an obvious, consistent "this is clickable" visual language
  (shape/cursor/hover state) — see Lesson 3 below. A player should never wonder
  "is that a decoration or a button?"

### 2. We don't read, we scan; we don't optimize, we satisfice; we don't learn, we muddle through (Ch.2, pp.21–27)

**What the book says:** Three "facts of life" about real usage, backed by Klein's
naturalistic-decision-making research on fire commanders (p.24): (1) users scan for
matching words/phrases rather than reading top-to-bottom (p.22); (2) users take the
*first reasonable option*, not the best one — "satisficing" (p.24), because optimizing
is slow and the penalty for guessing wrong is usually just a Back click (p.25); (3)
users build "vaguely plausible stories" about how things work and never revisit that
understanding once it lets them get things done (p.25–26, the Prince-and-the-Pauper
seal-as-nutcracker anecdote) — muddling-through is inefficient and error-prone but
sufficient, so people do it forever unless forced to reconsider. Krug's conclusion
(p.27): "If your audience is going to act like you're designing billboards, then
design great billboards."

**Application to Babylon:**
- Design every HUD surface (top bar, lens selector, action dock, wire feed) for a
  "billboard going by at 60 miles an hour," not a document to be read. That means:
  icon + 1–3 word label over paragraphs, no "helpful" onboarding walls of text that
  assume the player will read them before playing.
- Satisficing means the FIRST plausible verb/target the player sees for a given
  intent will be clicked — so the *most common* player intents (check my faction's
  status, see what changed this tick, mobilize the nearest org) must have the most
  obvious, prominent, first-reasonable-match affordance, not be buried as the third
  option in a menu that's technically "more correct."
- Because players muddle through and never revisit a working mental model, the FIRST
  few ticks of play (the tutorial-less onboarding) permanently shape how a player
  interprets every HUD element afterward. If the first lens they open is confusing,
  they will build a wrong mental model of what lenses *are* and carry it for the rest
  of the game (this connects directly to Lesson 6, Big Bang Theory, below).

### 3. Billboard Design 101 — conventions, hierarchy, clickability, noise (Ch.3, pp.29–41)

**What the book says:** Six concrete techniques:
- **Conventions are your friends** (pp.29–32): don't reinvent widgets users already
  know how to read (scrollbars, shopping-cart icon, logo-top-left). "Innovate when you
  *know* you have a better idea, but take advantage of conventions when you don't"
  (p.32). Rule: **clarity trumps consistency** (p.33) — it's fine to be *slightly*
  inconsistent if it makes something *significantly* clearer.
- **Effective visual hierarchies** (pp.33–36): three traits — more important = more
  prominent (size/boldness/color/whitespace/position); logically related = visually
  related (grouping, shared style); "part of" relationships are shown by visual
  nesting (a section header visually spans everything that belongs to it — p.34's
  "Computer Books" example, and p.36's flawed-nesting "Bill put the cat on the table"
  analogy: a visual hierarchy that's even slightly wrong forces a moment of confused
  re-parsing, exactly like a badly punctuated sentence).
- **Break pages into clearly defined areas** (p.36): users should be able to point at
  regions and say "links to today's top stories," "things I can do here" — the
  $25,000 Pyramid test. Eye-tracking shows users decide in the first glance which
  regions matter and then *permanently* ignore the rest ("banner blindness," p.36).
- **Make it obvious what's clickable** (pp.37–38): consistent visual cues (shape,
  color, one-color-no-underline links) so users never spend a millisecond wondering
  "does that do anything?"
- **Keep the noise down** (pp.38–39): three kinds of noise — *shouting* (everything
  competing for attention because no one made hard prioritization calls), *disorganization*
  (no grid), *clutter* (low signal-to-noise ratio). Krug's rule of thumb: "start with
  the assumption that everything is visual noise... and get rid of anything that's
  not making a real contribution" (p.39, "presumed guilty until proven innocent").
- **Format text to support scanning** (pp.39–41): plentiful headings close to the
  content they introduce (not floating, p.40), short paragraphs (single-sentence
  paragraphs are fine online, p.40), bulleted lists for anything list-shaped, bold key
  terms sparingly.

**Application to Babylon:**
- **Map lenses ARE the game's convention system to build**, since no prior game has
  this exact lens set — but Paradox-style lens toggles, legends, and tooltips already
  have strong genre conventions (EU4/Vic3/HOI4 map-mode buttons: icon row, active
  state highlighted, legend bottom-left or top-right). Reuse those conventions rather
  than inventing new lens-switching UI from scratch — the "clarity trumps consistency"
  rule licenses breaking from *strict Cold-Collapse-palette* consistency only if a
  lens legend needs an off-palette color to stay legible (e.g., a diverging red/blue
  solidarity ramp against the cyan chrome).
- **Visual hierarchy directive:** the currently active lens's legend must visually
  "span" or sit adjacent to the map it explains — never floating disconnected in a
  corner unrelated by proximity (this is literally p.34's nested-heading rule applied
  to a legend).
- **Defined areas directive:** the game chrome must have a $25,000-Pyramid-testable
  layout: "the map," "time controls," "my faction's status," "the wire feed," "the
  action dock," "the inspection stack" should each be a distinct, bounded region a
  new player can name after one glance — not a single undifferentiated HUD soup.
- **Clickability directive:** every interactive hex, org icon, wire-feed card, and
  inspection-panel row must share one consistent "this is clickable" treatment
  (hover glow + cursor + consistent iconography for "drill deeper" vs. "take action").
  Do not let decorative map elements (background terrain texture, non-interactive
  county fill) share that treatment — that would create false affordances, worse than
  no affordance, per Krug's clickability logic.
- **Noise directive:** apply "presumed guilty until proven innocent" to every event
  toast, badge, and ambient effect. If a HUD element doesn't help the player either
  decide something or understand something, cut it — especially default-visible
  chrome that competes with the map itself for attention on first load.
- **Scannability directive:** in the inspection stack (Victoria-3-style nested
  numbers), every explainable number needs a short bold label before its breakdown,
  not a wall of prose; keep each breakdown row to one short line, consistent with
  Krug's "single-sentence paragraphs are fine" rule for the web, adapted to
  data-explainer rows.

### 4. Mindless choices — Krug's Second Law (Ch.4, pp.43–47)

**What the book says:** "It doesn't matter how many times I have to click, as long as
each click is a mindless, unambiguous choice" (p.43, Krug's Second Law). What matters
isn't click *count* but click *difficulty* — the amount of thought and uncertainty
each click requires. Krug invokes "scent of information" (Pirolli & Card, p.43,
footnote): links that clearly and unambiguously identify their target give off a
strong scent that the click is heading the right way; ambiguous links do not, and
users lose confidence with every ambiguous click. Rule of thumb: "three mindless,
unambiguous clicks equal one click that requires thought" (p.43). Bad, hard-to-answer
top-level choices (his "Home vs. Office" printer-site example, p.44, and the
subscriber/member/neither login maze, p.45) are worse than a longer path of easy
choices. When a hard choice is genuinely unavoidable, Krug's fix (p.47) is
"brief, timely, unavoidable" contextual guidance — his example is London's
"LOOK RIGHT" street-corner paint: minimal text, delivered exactly at the decision
point, formatted so it can't be missed.

**Application to Babylon:**
- This is the single most load-bearing lesson for the 9-verb action dock and the
  nested inspection stack: a player should be able to drill 4–5 levels deep into
  "why is this county's solidarity 0.3" without ever facing an ambiguous fork — every
  drill-down link/number should visibly "smell like" what it leads to (a solidarity
  number that expands into its formula terms, each of which is itself clickable and
  named exactly what it says on the parent row — Krug's p.76 "the name needs to match
  what I clicked" rule, generalized to numeric breakdowns).
  - Practically: name inspection-stack children EXACTLY the term used in the parent
    row's formula (if the summary row says "Solidarity Edge Weight," the drill-down
    node must be titled "Solidarity Edge Weight," not "Class Bond Strength" or some
    internal engine name).
- Never present the player a "Home vs. Office"-style false top-level fork. E.g., a
  verb-target picker that forces the player to first classify their intent into a
  category (mobilize vs. organize vs. agitate) before showing them who's available is
  exactly the p.44 anti-pattern — show the map, let the click *on a target* imply the
  intent, or let one obviously-scented click reveal the short verb list for that
  target.
- Where a genuinely hard choice can't be avoided (e.g., picking which of two
  contradictory endgame paths to commit resources toward), the guidance must be
  brief + timely + unavoidable per p.47 — a short inline label at the fork itself
  ("Investing here weakens the Sovereignty track"), not a wall of tooltip text or an
  off-screen wiki page.

### 5. Omit needless words — happy talk and instructions must die (Ch.5, pp.49–52)

**What the book says:** Krug's Third Law: "Get rid of half the words on each page,
then get rid of half of what's left" (p.49) — deliberately extreme to counteract the
instinct to over-explain. "Happy talk" — self-congratulatory welcome-blurb filler
("Welcome to our exciting new section on...") — must be cut outright; it's detectable
by ear as "blah blah blah blah" (p.50). Instructions are the second big offender
(p.51): "no one is going to read them... your objective should always be to eliminate
instructions entirely by making everything self-explanatory, or as close to it as
possible. When instructions are absolutely necessary, cut them back to the bare
minimum." His worked example cuts a 103-word form intro to 34 words (p.52) by
deleting anything the reader already knows, moving info to where it's actionable, and
keeping only what changes behavior.

**Application to Babylon:**
- Kill happy talk from every first-run panel, tooltip, and lens-intro card in the
  cockpit. No "Welcome to the Solidarity Lens! Here you can explore how..." — just
  show the lens with a self-explanatory legend.
- Tutorializing text (if any survives) should follow the 103→34-word discipline:
  cut anything the player will infer from the UI itself, keep only what actually
  changes what they do next, and place it exactly at the decision point rather than
  in an upfront wall.
- This directly supports the "full-bleed map, floating chrome" goal in the brief:
  every word of onboarding copy is pixels stolen from the map. Prefer showing (a
  glowing pulse on the first clickable hex) over telling ("Click a hex to begin").

### 6. Web (map) navigation conventions — Site ID, Sections, "You are here," breadcrumbs, tabs (Ch.6, pp.55–83)

**What the book says:** Navigation isn't a feature bolted onto content — "it is the
Web site" (p.62): it answers "where am I," "what's here," "how do I use this,"
and "who do I trust" (p.63). Persistent navigation should always expose four elements
(p.66–70): **Site ID** (top-left-ish, frames everything — p.68), **Sections**
(primary top-level choices), **Utilities** (account/help/search — capped at 4–5
visible or they get lost in the crowd, p.70), and a **search box** that follows the
"box + button + word 'Search'" formula exactly, no cute relabeling (p.71: "Fancy
wording... they'll be looking for the word 'Search'"). "You are here" indicators must
be loud, not subtle — Krug: "if you're a designer and you think a visual cue is
sticking out like a sore thumb, it probably means you need to make it twice as
prominent" (p.78). **Breadcrumbs** (p.79–80) use `>` separators, sit at the top,
bold the current (non-clickable) last item. **Page names** must exist on every page,
sit in the right visual-hierarchy position (framing the unique content, not the
nav/ads), be prominent, and — critically — **must match what the user clicked to get
there** (p.76: "the name of the page will match the words I clicked to get here" is
an implicit social contract; breaking it, even with near-misses, erodes trust every
time). **Tabs** get their own three-paragraph endorsement (p.80–81) for being
self-evident, hard to miss, and slick — but only if the active tab visually "pops"
forward with contrasting color plus a physical connection to the content below it
(no floating "connected but no contrast" half-measures, p.81). The chapter closes with
the **trunk test** (p.82–83): drop a blindfolded user on a random deep page; can they
instantly answer Site ID / page name / sections / local nav / "you are here" /
search, at a glance, slightly blurry vision, no close scrutiny required?

**Application to Babylon:**
- Treat the top bar as the Site ID + Sections + Utilities row, held to the same
  discipline: faction/game identity always visible top-left, primary game-mode
  Sections (Map, Wire, Roster, whatever the top-level game areas are) capped and
  obvious, Utilities (settings, pause, save) capped at 4–5 icons max before they
  need to fold into an overflow.
- **Map lenses are this game's "You are here" problem, doubled:** the active lens
  must be unmistakably marked on the lens selector itself (Krug's loud-not-subtle
  rule, p.78) AND the legend must make it obvious what mode the map is currently
  rendering — a player glancing away and back should re-orient in under a second.
  Apply "make it twice as prominent as feels necessary" literally when testing lens
  states in this palette (Cold Collapse's dark cyan can make subtle state changes
  invisible — verify contrast on the active-lens indicator specifically).
- **Inspection stack directive (direct breadcrumb application):** every nested
  drill-down panel needs a `>`-separated breadcrumb trail at its top (Faction >
  County > Social Class > Wages) with the current node bold and non-clickable, and a
  one-click "back to top / Home" affordance — this is the literal breadcrumb pattern,
  repurposed for Victoria-3-style recursive number inspection instead of page
  hierarchy.
- **Name-matching directive:** whatever label a player clicks (a verb button, a wire
  headline, a legend entry) must be the exact title of the panel/page it opens. If
  space forces a shortened label, the shortened and full forms must be so obviously
  equivalent that the mismatch requires zero thought (Krug's "Gifts for Him" → "Gifts
  for Men" example, p.76) — never route "Mobilize" to a panel titled "Organizational
  Activation."
- **Search-box directive:** if the cockpit ever gets a search/filter box (e.g.,
  "find a faction/organization/county"), use the box+button+word-"Search" formula
  exactly — no "Discover," no bare magnifying glass with no label on first use.
- **Tab-pattern directive:** if lens categories or inspection-panel sub-tabs use a
  tab metaphor, the active tab must visually connect to (share a border/fill with)
  the panel below it and contrast hard against inactive tabs — no "floating,
  disconnected, low-contrast" tab treatments.
- **Adopt the trunk test as a literal QA gate** for the shipped HUD: take a
  screenshot of a random deep game state (mid-tick, inspection stack three levels
  deep, non-active lens), squint at it, and verify a first-time player could still
  name what faction/tick/location they're looking at and how to get back to the
  map's default view — this is a cheap, repeatable check to run every time the
  chrome layout changes.

### 7. The Big Bang Theory of Web design — the first few seconds decide everything (Ch.7, pp.85–94, partial)

**What the book says:** In the first few seconds on any new page, a user forms
answers (correctly or not) to four questions (p.89): **What is this? What can I do
here? What do they have here? Why should I be here and not somewhere else?** Getting
this "off on the right foot" matters disproportionately because first impressions are
sticky and self-reinforcing (p.90): "if their first assumptions are wrong... they
begin to try to force-fit that explanation onto everything they encounter... if
people are lost when they start out, they usually just keep getting...loster."
Krug's four "plausible excuses" for skipping the big-picture explanation (p.91) are
all rejected: "it's obvious to us," "people will find repetition annoying,"
"our real audience already knows," "our ads already explained it" — none of these
survive contact with actual users, who frequently say in testing "Oh, is that what it
is? I'd use that all the time, but it wasn't clear what it was" (p.91). The chapter
also stresses that most users don't enter through the "front door" anymore (p.92) —
they teleport in via a deep link — so *every* page, not just the Home page, needs to
carry enough orientation to answer "what is this" on its own.

**Application to Babylon (partial chapter, but directly load-bearing):**
- The very first frame the player sees when the game boots (or when they load a
  save) must, within seconds, implicitly answer: What is this (a living political
  map of a collapsing America)? What can I do here (the verb dock + lens selector
  must be visible, not hidden behind a menu)? What's here (some indication of scale —
  states/counties, current tick/date)? Why here-not-elsewhere (what makes this
  moment/location interesting right now — a wire-feed teaser or hotspot highlight
  serves this role, like Krug's "teases," p.86).
- Because players increasingly resume mid-game (load a save, or return after a
  break) rather than always starting fresh, every screen — not just a literal
  "Home" — needs enough self-contained orientation (tick number, faction name, map
  scale, active lens) to answer the four Big Bang questions without requiring the
  player to have just come from the main menu. This generalizes the chapter's
  "people teleport into the depths of a site" warning (p.92) to loading straight into
  a saved mid-collapse America.

---

## Anti-patterns to avoid (Krug, verbatim spirit)

- Cute/internal jargon in primary labels ("Job-o-Rama" energy) — p.14.
- Ambiguous clickability: anything that looks like it might be a button/link but
  requires a guess — p.15.
- Forcing a hard top-level classification choice before showing options
  ("Home vs. Office") — p.44.
- Multiple near-identical login/access forks that require the user to self-diagnose
  which one applies to them — p.45.
- Floating headings/legends not visually connected to what they label — p.34, p.40.
- Subtle "you are here" / active-state indicators — p.78.
- Tabs that don't visually "pop" forward — p.81.
- Happy talk and unread instructions bloating first-run UI — p.50–52.
- A page/panel title that doesn't match the label the user clicked to get there — p.76.
- Treating the Home/default screen as the only place orientation matters, when most
  entries are now "deep links" (a resumed save, a clicked wire item) — p.92.

## Directives (imperative, testable — for the design-sync / implementation phases)

1. Every lens-selector control, verb-dock button, and inspection-panel header must
   read correctly to a first-time player with zero tooltip hover — verify by running
   Krug's "neighbor test" (show a static screenshot to someone unfamiliar with the
   project; can they say what it is in one sentence).
2. Legend must be visible and directly adjacent (not floating disconnected) whenever
   any non-default map lens is active.
3. The active lens, active tab, and current inspection-stack node must each use at
   least two stacked visual distinctions (not one) to mark "current state" — e.g.,
   color change AND bold, never color alone — verified with a contrast check against
   the Cold Collapse palette specifically.
4. Every clickable HUD element (hex, org icon, wire card, inspection row) must share
   one consistent hover/cursor/afforded-clickability treatment, and no non-interactive
   decorative element may use that same treatment.
5. No verb-target flow may force the player through an ambiguous top-level
   classification choice before showing selectable targets; every click must "smell"
   unambiguously of its destination.
6. Every drill-down node in the inspection stack must be titled with the exact term
   used in its parent's formula/summary row — never a renamed or paraphrased label.
7. Every inspection-stack panel must show a `>`-separated breadcrumb trail with the
   current node bold and non-clickable, plus a one-click return to the top level.
8. Any first-run or contextual instructional text must be cut to the Krug-103→34
   standard: delete anything inferable from the UI, keep only what changes the
   player's next action, and place it at the decision point, not in an upfront wall.
9. Run the trunk test on the shipped HUD after every layout change: from a random,
   deep, mid-tick screenshot, a squinting first-time viewer must be able to name the
   faction, the tick/date, the active lens, and how to return to the default map view.
