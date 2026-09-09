# Designing Games (Tynan Sylvester) — Applied to Babylon's Living Map

Source: *Designing Games: A Guide to Engineering Experiences*, Tynan Sylvester, O'Reilly 2013,
415 pp. Read strategically: Ch.1 "Engines of Experience" (pp.7–44), Ch.2 "Elegance" (pp.49–54),
Ch.4 "Narrative" case study (pp.111–117), Ch.5 "Decisions" (pp.119–146), Ch.8 "Motivation and
Fulfillment" (pp.211–218), Ch.9 "Interface" (pp.219–238). Page numbers below cite the book's own
printed page numbers (footer), not PDF page indices.

Context this report targets: Babylon's map-first UI — the political map with lenses (imperial
rent, solidarity, contradiction, etc.), the floating inspection stack (Victoria-3-style nested
panels), the top bar / time controls, the action dock (9 verbs), the wire feed (event ticker), and
event toast popups.

---

## 1. Mechanics generate events, events provoke emotion — design the generator, not the moment

**What the book says (Ch.1, pp.7–8):** Screenwriters and novelists author events directly. Game
designers can't — they author *mechanics*, and mechanics interact with play to *generate* events
that never repeat exactly. "Game designers don't design events. We design systems of mechanics
that generate events. This layer of indirection is the fundamental difference between games and
most other media." The corollary (p.15): a *reveal of information* is emotionally equivalent to a
*change* — learning a fact and a fact becoming true produce the same human-value shift.

**Application to Babylon:** Every map surface that currently *reports* state (a number changing in
a panel) should be evaluated for whether it can instead be the *site of the reveal* — i.e., the
map itself un-fogs, a border visibly redraws, a hex's fill color visibly shifts — rather than a
sidebar simply updating a digit. The wire feed should not just narrate what already happened in
the data; wire items that *reveal* previously-hidden state (a faction's true allegiance, a hidden
solidarity edge) carry as much emotional weight as items that report a change, and should be
visually distinguished as "reveals" (an unveiling animation/icon) versus "events" (a delta icon).

## 2. The primacy of emotion is subtle, plural, and not "fun" — track human-value shifts, not deltas

**What the book says (Ch.1, pp.8–15):** "To be meaningful, an event must provoke emotion," but
most valuable emotions are subtle pulses below conscious awareness, not spikes. "The emotions of
play are not limited to fun." An event is emotionally relevant not because of what it *is* but
because of the shift in a **human value** it implies — `[life/death]`, `[victory/defeat]`,
`[friend/stranger/enemy]`, `[wealth/poverty]`, `[freedom/slavery]`, `[knowledge/ignorance]`,
`[skilled/unskilled]`, etc. "The more important the human value and the more it changes, the
greater the emotion" (p.13) — losing a pawn is trivial in the opening, gutting when it's the pawn
guarding the king. Emotions also fire on *anticipated* change, not just realized change (p.14).

**Application to Babylon:** Babylon's simulation already tracks the exact vocabulary of human
values this book describes — `[wealth/poverty]` (Imperial Rent, wages vs. subsistence),
`[freedom/slavery]` (dispossession, reserve army), `[together/alone]` (solidarity edges,
atomization), `[knowledge/ignorance]` (consciousness), `[life/death]` (survival calculus,
endgames). The map/UI should not surface raw formula outputs (`Φ = 0.34`) as the primary signal;
it should surface **which human value just crossed a threshold and in which direction** — a hex
that was `[safe]` becoming `[besieged]`, a class that was `[atomized]` becoming `[organized]`. Name
UI elements by the value they track, not the formula. The inspection stack's outermost layer
(what the player sees before drilling in) should always be a human-value label, never a
coefficient; the coefficient lives one layer deeper, per the "every number explains itself"
brief.

## 3. Immersion = flow (erase the world) + arousal (mechanics) + fiction (label the arousal)

**What the book says (Ch.1, pp.41–44):** Via the two-factor theory of emotion (Schachter-Singer,
the Capilano Canyon bridge study, p.16), all intense emotions are physiologically similar arousal
states that differ only in their *cognitive label*. "To create an experience that mirrors that of
a character, we construct it out of three parts. First, we create flow to strip the real world out
of the player's mind. Second, we create an arousal state using threats and challenges in the game
mechanics. Finally, we use the fiction layer to label the player's arousal to match the character's
feelings" (p.42). Doom's marine is scared *because Doom's fiction says so*; the underlying arousal
is identical to Geometry Wars.

**Application to Babylon:** The Cold Collapse palette and the wire feed's narration are doing the
"fiction labels arousal" job — but only if the *mechanics* are actually producing arousal (time
pressure, stakes visibly rising, a countdown to a rupture event). A calm, static map with a news
ticker cannot be immersive no matter how well-written the wire copy is, because there is no
arousal state to label. Concretely: when a Contradiction is approaching its rupture threshold, the
map itself (not just a panel) should visibly tense — pulsing border, rising ambient tension color
on affected hexes — so the fiction ("wire: unrest spreading in Detroit") has real arousal to
attach to. Conversely, avoid decorative "tension" effects (screen shake, red vignettes) on hexes
where nothing is actually at stake — per §2, that's noise, not signal, and (per Ch.1 spectacle
discussion) cheap spectacle without mechanical backing "leaves players numb" (p.25).

## 4. Elegance: mechanics that interact with many others, and that ration information cheaply, beat isolated ones

**What the book says (Ch.2, pp.49–54):** "Good design means maximizing the emotional power and
variety of play experiences while minimizing players' comprehension burden and developer effort.
This form of efficiency is called elegance." Elegance comes from **emergence** — mechanics that
"don't just add together, but multiply into a rich universe of possibility" (shoot + look + move
example, p.50–51) — and is *recognizable* by four smell tests (pp.52–54):
1. "Mechanics that interact with many other mechanics smell like elegance."
2. "Simple mechanics smell like elegance" — reducing complexity is as valuable as adding benefit.
3. "Mechanics that can be used in multiple ways smell like elegance" (multirole trade-offs, e.g.
   *Resident Evil*'s guns are both offense and defense — creates a genuine dilemma).
4. "Mechanics that reuse established conventions and interfaces smell like elegance" because they
   leverage knowledge players already have.

**Application to Babylon:** Audit the map lens system against smell test #1: a lens (e.g. the
imperial-rent ramp lens) is elegant only if its color-coding *also* informs decisions the player
makes with other systems (does a high-rent hex change where you'd deploy the Aid verb? does it
change negotiate targets?) — if a lens is purely decorative and touches nothing else, it's
inelegant weight. Smell test #3 argues for making map interactions **multirole**: clicking a hex
should never be single-purpose (just "select") when it could simultaneously scope the inspection
stack, filter the wire feed to that hex, and preview verb-target eligibility — one click, three
payoffs, per the "trade-off, not tool proliferation" principle. Smell test #4 says: don't invent a
bespoke Babylon iconography for map lenses from scratch — reuse the choropleth/heatmap
conventions (light=low, dark=high, diverging red-blue for contested) that political-map games and
real cartography already teach players, so the legend does less work.

## 5. Decisions are the only emotional trigger unique to games — engineer the future, not just the present

**What the book says (Ch.5, pp.119–124):** "Understanding decisions is critical in game design
because decisions are the only emotional trigger that is unique to games." Crucially, decisions
generate emotion about **possible futures**, not present events — "something doesn't have to
happen to generate emotions. The player need only sense the possibility of it happening" (p.121,
the skyscraper-ledge vs. porch-ledge thought experiment). For that sensing to work, the game's
systems must be **consistent** (same rules apply everywhere) and **comprehensible** (simple enough
to model in the player's head) — "prediction of a possible future depends on it being driven by a
consistent, comprehensible system" (p.124). An AI or system that is *too* smart or *too*
unpredictable is bad design here: "the more the AI thinks, the less the player is able to think"
(p.126).

**Application to Babylon:** The 26-system engine is deterministic and hash-verified — good, that
satisfies "consistent." The open question is **comprehensible**: does the player's map UI let them
build an accurate mental model of what *will* happen if they mobilize here, negotiate there? The
inspection stack's job, per this chapter, is not just to explain the current number but to let the
player *feel the future* — e.g. hovering a hex before committing a verb should preview the
predicted directional shift ("Solidarity: likely ↑"), not just show current state. Where the
engine's real logic is too deep to be predictable at a glance (23 hot-swappable formulas,
compounding systems), the UI must supply a simplified, honest predictive model at the top of the
inspection stack (directional arrows, confidence bands) even if the full math is one drill-down
away — this is "comprehensible" without "dumbing down the sim."

## 6. Information balance: starvation and glut are both design failures, and they're invisible to the designer

**What the book says (Ch.5, pp.127–139):** "The same decision can be made incomprehensible with
too little information, fascinating with the right amount of information, and trivial with too
much information." **Information starvation** (p.127–130) collapses play into "reactive
thrashing" — the *authored challenge preparation problem* is named explicitly: games that ask
players to prepare for a future challenge they have no way to predict (e.g. RPG character creation
before the player knows anything — *Mass Effect 2*'s classes example, 80% picked the most
familiar option out of confusion, not preference, p.129). Critically: "Information starvation is
an insidious problem because designers can't see it due to their unique knowledge of the game" —
a useful community FAQ is *itself* a warning sign that the game under-informs. **Information glut**
(p.130–134) is the opposite failure — when the answer is already visible, "the thought process
vanishes; the decision is no longer a decision" — illustrated by the *Modern Warfare 2* heartbeat
sensor, fixed by two deliberate information *cuts* (Ninja perk invisibility; discrete pulses
instead of continuous tracking) that reintroduced decisions "without changing any mechanics
interactions at all" (p.131). The poker case study (pp.133–135) shows the entire multi-century
evolution of the game as a search for the exact point of information balance via community
cards. Also: **fictional ambiguity** (p.135–137) — "the player has no way of knowing which of
these possibilities is real in a given game just by looking" (the roast-turkey example) — the fix
is to build puzzles/decisions out of *mechanics the player already knows work*, not out of
fictional plausibility. And **metagame information** (p.137–139) — knowledge from outside the
game (genre convention, "the designer wouldn't do that to me") silently defangs threats the
fiction is trying to sell.

**Application to Babylon:** This is the single most load-bearing chapter for the map UI.
- **Starvation risk**: a first-time player opening the political map has zero calibration for
  what "high imperial rent" or "solidarity 0.7" *means* relative to the game's own scale — exactly
  the authored-challenge-preparation problem. Every lens legend must show not just a color ramp
  but *where the current world sits on it* (a marker on the gradient, not just min/max labels),
  and ideally a relative/percentile framing ("more contradiction than 80% of counties") rather
  than a bare absolute number, because absolute numbers are information-starved without a learned
  baseline.
- **Glut risk**: showing every one of the ~56 formula outputs simultaneously on hover is the
  *Mass Effect 2* mistake in reverse — glut, not starvation — it kills the decision by pre-solving
  it or by drowning the one number that mattered. The inspection stack should default to 2–3
  headline numbers per layer (mirroring the heartbeat-sensor fix: dial down to a "slow drip"),
  with drill-down for the rest.
- **Fictional ambiguity**: the map must never let iconography imply an action is possible that the
  9-verb system doesn't actually support (a "roast turkey" problem — e.g. a hex that looks
  clickable/attackable via visual convention but isn't a legal Attack target). Every visual
  affordance on the map (a highlighted border, a pulsing icon) must correspond to an actual verb
  legality, or be explicitly styled as "informational only, not actionable."
- **Metagame information**: players who've absorbed enough Paradox-genre convention will assume
  "the game won't let the revolution just lose to bad luck." If Babylon's endgames are meant to
  feel earned and real (including FRAGMENTED_COLLAPSE), the UI should occasionally signal genuine
  stakes are live (a visible, mechanically-grounded "this is not scripted" cue — e.g., showing the
  actual deterministic hash/tick-seed) to counteract the reflexive genre-savvy discounting the book
  describes.

## 7. Decisions have scope (nondecision → twitch → tactical → profound → impossible) — the map must offer a mix, and flow requires a constant drip

**What the book says (Ch.5, pp.140–146):** Decision scope is "the amount of thought a decision
takes to make," from *nondecisions* (habit, no real choice) through *twitch* (<1s), *tactical*
(1–5s), *profound* (10s+, drawing on the player's whole knowledge — the Kasparov example), to
*impossible* (beyond the player's capacity to model, degenerating to noise). Flow is a "cup with a
hole in the bottom" that must be fed a constant stream of appropriately-scoped decisions —
too-large gaps ("flow gaps," p.142) or too much simultaneous load ("overflow," p.144) both break
it. "The only hard-and-fast rule of flow pacing is that it should vary" (p.146) — mixing scopes,
not holding one scope for long stretches, sustains engagement.

**Application to Babylon:** A grand-strategy map inherently trends toward *profound* decisions
(should I commit to revolution in this state) with long gaps between them — a known genre risk of
flow gaps during multi-tick simulation waits. The action dock and event toasts should supply a
steady stream of **tactical**-scope decisions (a wire event with a 2–3 option quick-response) to
fill the gap between profound turns, exactly as the book recommends varying scope rather than
letting the player sit idle waiting for the next big call. Conversely, resist adding *twitch*
decisions with no real stakes (busywork clicks) purely to keep hands moving — the book explicitly
calls these "nondecisions" that "don't contribute to flow since they don't engage the mind"
(p.140).

## 8. Predictable systems, not smart AI — organizations should be legible automata, not black boxes

**What the book says (Ch.5, pp.125–127):** "A character who follows straightforward, predictable,
consistent rules often contributes more to a play experience than a realistically chaotic mind
simulation." Real-time-strategy soldiers work as automata precisely *because* their predictability
lets players plan and strategize with confidence — an unpredictable "smarter" AI would rob the
player of the ability to think ahead at all.

**Application to Babylon:** OODA-driven organizations (the Action phase system) should expose
enough of their decision logic in the inspection stack that a skilled player can *predict* what a
rival faction will do next turn, the same way a chess player predicts an opponent. If organization
behavior is a total black box, players cannot form profound decisions around it (per §5) — they
can only react. Consider a "likely next action" hint surfaced in an organization's inspection
panel once the player has enough intel/visibility on that faction (gated by fog-of-war / the
"Investigate" verb), turning enemy AI legibility itself into a rewarded, verb-gated resource
rather than either omniscience or total opacity.

## 9. Rewards alignment: don't bolt score/XP onto the map — reward what players already want to do

**What the book says (Ch.8, pp.211–218):** Extrinsic rewards can "displace and even destroy the
intrinsic fulfillment of play" (Deci's classic study, p.213) — this damage is worst on
*exploratory or creative* tasks, exactly Babylon's genre. "Rewards alignment is how closely the
activities encouraged by a reward system resemble those the player would have engaged in without
it" (p.214) — the goal is "to construct a system that can detect and appropriately reward
everything the player already wants to do," illustrated by *Skate 3*'s finely-tuned trick-scoring
system versus a crude "score points for jumping through hoops" alternative that would "antagonize
the creative play." Games that maximize motivation without regard to fulfillment produce **player's
remorse** (p.216) — the book explicitly flags this as an ethical hazard of Skinner-box design
(*Cow Clicker*, p.217).

**Application to Babylon:** Babylon has no explicit "score" — its reward structure *is* the
narrative and systemic outcome (endgames, faction fortunes, the wire feed's story). This is
already well-aligned by construction: the closest thing to a "reward" is watching a contradiction
you cultivated actually rupture, or a solidarity network you built actually hold. The design risk
is adding gamified UI candy (achievement toasts, streak counters, XP-style level-ups for
organizations) that isn't grounded in what the simulation actually detects as meaningful — per
this chapter, that would be pure motivation-without-fulfillment and should be avoided. Any toast/
notification system should reward *detectable, player-intended* outcomes (a rupture you set up
several ticks ago finally landing) rather than arbitrary milestone counts.

## 10. Metaphor and vocabulary: the map's fiction layer must be a taught, consistent vocabulary, not ambient decoration

**What the book says (Ch.9, pp.220–224):** "The entire fiction layer of a game is a giant
metaphor" that lets players reason about a system via a familiar shape instead of learning it from
scratch — the chess-on-a-2D-plane example (p.223) is called out as "nearly universal" for spatial
metaphor specifically, directly analogous to a political map. But metaphor is dangerous precisely
because "only a small subset of the functionality of the real object is actually implemented in
game mechanics" (p.223) — players cannot tell by looking which parts of the fiction are real
mechanics. The fix: "A game must establish a metaphor vocabulary that indicates which elements are
simulated mechanically. It must then remain consistent with this vocabulary" (p.224), introduced
early, in a context where the player is guaranteed to test it (the *Prince of Persia* climbable-
brick example).

**Application to Babylon:** County borders, hex fills, faction colors, and iconography on the map
each need to be established, early and low-stakes, as *load-bearing signals* the first time they
appear — e.g. the first time a border visibly redraws should happen in a clearly-telegraphed,
low-consequence early-game moment so the player learns "redrawing borders = mechanically real,
not decorative" before it matters strategically. Never let two different map affordances share
the same visual vocabulary (e.g. don't use the same red glow for "under military attack" and for
"high imperial-rent extraction" — per Zelda's crumbling-wall lesson, an ambiguous visual costs the
player a genuine, unresolvable guess). Maintain one canonical meaning per visual channel: color =
one specific lens value, glow/pulse = imminent threshold, border thickness = political claim
strength, icon = verb availability. Document this vocabulary once (a legend/glossary artifact) and
never silently overload a channel later.

## 11. Signal, noise, and visual hierarchy: complex map art actively degrades comprehension unless deliberately tuned

**What the book says (Ch.9, pp.225–230):** "Noise is signal that fails to transmit meaningful
information" — and critically, *realistic/detailed art is a primary source of noise*, not
polish: "Puzzles that were intuitive suddenly become impenetrable... just replacing gray surfaces
with art has made the game become unplayable, even though the mechanics design has not changed at
all" (p.226, the graybox-to-art anecdote). The fix used by *Team Fortress 2*, *Portal*, and
*Mirror's Edge* is a deliberately simplified, mechanically-legible art style, not more detail.
**Visual hierarchy** (p.227–230) is the tool for balancing signal density across skill levels:
"everything is displayed at once, but more important pieces of information are made more visible
so that people notice them first" — bigger/closer/brighter/faster draws the eye first, and this
lets novices and experts each unconsciously perceive only what they can currently use, without a
difficulty-tier toggle. The ammo-counter example (p.229) walks through the full spectrum from
"whisper" (small print, ignorable) to "shout" (fills half the screen), arguing designers should
tune deliberately along that spectrum per element.

**Application to Babylon:** The "real US county borders aggregated into states" cartography goal
is exactly the graybox-to-art risk described here: photorealistic or overly-detailed basemap
rendering will bury the political signal (which counties are contested, which classes dominate)
under geographic noise. Babylon's map rendering should treat county/state geometry the way *Team
Fortress 2* treats its characters — simplified, high-contrast, mechanically-legible shapes first,
beauty second — with real terrain/imagery detail reserved for deep-zoom hex tiles where it doesn't
compete with the strategic read. Apply visual hierarchy explicitly to the HUD: the wire feed,
faction color, and the currently-active lens value should be "shout"-tier (large, high-contrast);
secondary stats (population, raw formula coefficients) should be "whisper"-tier (small, only
noticed on deliberate inspection) — and this tuning should be revisited per element the way the
book revisits the ammo counter, not left to default component styling.

## 12. Redundancy and indirect control: never gate critical information behind a single, missable channel

**What the book says (Ch.9, pp.230–235):** Players can be looking anywhere, distracted, or
mid-conversation when something critical happens on screen — unlike film or prose, "games can
[cause players to] miss it" and the designer bears the blame anyway (p.230). The fix is
**redundancy**: repeat critical information through multiple channels (homogenous: same message
twice; diverse: dialogue + animation + level geometry + HUD marker, the quadruple-redundancy
window-jump example, p.231) rather than forcing observation (interrupting dialog boxes), which
"shatter[s] flow and immersion." **Indirect control** — nudging (default options, lit doorways),
priming (activating concepts before a decision), and social imitation (NPCs modeling the intended
behavior) — lets designers guide behavior "without the player realizing that they're being guided"
(p.232), cheaper and less immersion-breaking than explicit instruction.

**Application to Babylon:** A critical rupture event or endgame-adjacent threshold crossing must
never live in only one place (only the wire feed, say) — it should hit the wire feed (diverse:
text), a toast popup (diverse: interrupt-lite), and a map-level visual pulse on the affected hex
(diverse: spatial), so a player who's mid-inspection-stack-drilldown elsewhere still catches it —
this is literally the quadruple-redundancy pattern from the book, applied to a strategy-game
"something important just happened" moment instead of a shooter's "jump out the window" moment.
For guiding new players toward legal/sensible verb usage without a tutorial-mode interrupt, use
nudging (make the legal targets for a selected verb visually "lit," per the lit-doorway example)
and social imitation (if AI-controlled factions visibly use a verb effectively, players will copy
the pattern) rather than modal how-to popups.

---

## Summary Table: Book Concept → Babylon Surface

| Concept | Babylon surface | Directive |
|---|---|---|
| Emotion = human-value shift | Lens legends, inspection stack headline | Label by value crossed, not raw coefficient |
| Immersion = flow+arousal+fiction | Map tension states, wire narration | Only narrate arousal the mechanics actually produce |
| Elegance smell tests | Map lens system, click interactions | Every lens must feed ≥2 other systems; every click must have ≥2 payoffs |
| Decisions feel futures | Inspection stack, verb hover preview | Show predicted directional shift before commit, not just current state |
| Information starvation/glut | Lens legends, hover tooltips | Legend must show current-value marker on the ramp; hover shows 2–3 headline numbers max |
| Decision scope variety | Action dock, event toasts | Feed tactical-scope decisions between profound strategic turns |
| Predictable AI | Organization inspection panel | Surface a legible "likely next move" once player has Investigate-gated visibility |
| Rewards alignment | Toast/notification system | Only fire toasts for outcomes the simulation actually detects as player-caused |
| Metaphor vocabulary | Map visual channels | One canonical meaning per color/glow/icon channel; teach it in a low-stakes early moment |
| Visual hierarchy vs. noise | Basemap rendering | Simplify county/state geometry for legibility; save detail for deep-zoom hex tiles |
| Redundancy | Rupture/endgame threshold events | Fire on wire feed + toast + map pulse simultaneously (diverse redundancy) |
| Indirect control | Verb targeting UI | Nudge via "lit" legal targets, not modal tutorials |
