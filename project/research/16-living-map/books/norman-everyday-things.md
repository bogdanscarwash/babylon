# The Design of Everyday Things — Don Norman

**Source:** `ux_don-norman_design-of-everyday-things.pdf` (Revised & Expanded Edition, Basic Books,
2013), 369 pp. Read strategically: full Chapter 1 ("The Psychopathology of Everyday Things," pp.
1-36), full Chapter 2 ("The Psychology of Everyday Actions," pp. 37-54 core), Chapter 3 selections
("Knowledge in the Head and in the World," pp. 104-122 — knowledge tradeoffs, natural mapping,
culture), and Chapter 4 selections ("Knowing What to Do: Constraints, Discoverability, and
Feedback," pp. 123-160 — four constraint types, forcing functions, the faucet case study, sound as
signifier). Citations below are `(Norman, p.N)`.

This book is the single best theoretical grounding for Program 16 (the Living Map / game-chrome
rework). Its entire argument — that good design is a *communication problem* between a system and
a person, not a decoration problem — maps almost one-to-one onto what "feels like a corporate
dashboard" is diagnosing in the current Babylon cockpit: signifiers are missing, mappings are
arbitrary, feedback is inconsistent, and there's no single legible conceptual model of what the map
*is*.

---

## Core Lessons

### 1. Affordances are not properties — they are relationships, and only *perceived* affordances matter to design

**What the book says:** An affordance is "a relationship between the properties of an object and
the capabilities of the agent that determine just how the object could possibly be used" (p.11). It
is not an attribute of the object alone. Critically, "affordances exist even if they are not
visible. For designers, their visibility is critical" (p.13) — the moment an affordance isn't
perceivable, it needs a **signifier**, a separate, deliberate signal, to communicate it (p.14-15).
Norman spends the 2013 revision's biggest addition distinguishing the two precisely because
designers had been misusing "affordance" to mean "the circle I drew to say tap here" — that's a
signifier, not an affordance (p.13-14, the "Affordances and Signifiers: A Conversation" dialogue,
p.19-20).

**Application to Babylon:** The map itself has real affordances (hexes are clickable, territories
are selectable, org pins are draggable) but almost none of them are currently *signified*. A county
that can be clicked to open the inspection stack looks identical to a county that can't. Every
interactive layer in the cockpit needs an explicit signifier vocabulary: a **hover glow + cursor
change** for clickable territories, a **halo ring** for actor-selectable org pins, a **dashed vs.
solid border** distinguishing contested (interactable) vs. settled (inert) hex boundaries. Do not
rely on the county-border geometry itself to imply interactivity — geometry is the affordance;
color/glow/cursor is the signifier, and only the signifier is what a first-time player perceives.

### 2. Discoverability and understanding are the two pillars of good design — and doors that need "PUSH" signs have already failed

**What the book says:** "Two of the most important characteristics of good design are
*discoverability* and *understanding*. Discoverability: Is it possible to even figure out what
actions are possible and where and how to perform them? Understanding: What does it all mean? How
is the product supposed to be used?" (p.3). The book's recurring "Norman door" thesis: "Whenever you
see hand-lettered signs pasted on doors, switches, or products, trying to explain how to work them,
... you are also looking at poor design" (p.19).

**Application to Babylon:** Any tutorial-toast, tooltip-only, or wiki-page explanation of "how the
map lenses work" is a hand-lettered PUSH sign — evidence the underlying signifiers failed. The test
for every new surface (lens switcher, verb dock, inspection panel) should be: *can a first-time
player discover this control's existence and meaning purely from what's rendered on screen, with
zero onboarding text?* If not, redesign the control before writing help copy.

### 3. The Seven Stages of Action and the Gulfs of Execution/Evaluation — every player action is two bridges, and the map must build both

**What the book says:** People face two gulfs when using something: the **Gulf of Execution**
("how do I work this? what can I do?") and the **Gulf of Evaluation** ("what happened? is this what
I wanted?") (p.38-39, Fig. 2.1). The full loop is seven stages: Goal → Plan → Specify → Perform
(execution, bridged by signifiers/constraints/mappings/conceptual model) → Perceive → Interpret →
Compare (evaluation, bridged by feedback and a conceptual model) (p.41, Fig. 2.2). "The gulf is
small when the device provides information about its state in a form that is easy to get, is easy
to interpret, and matches the way the person thinks about the system" (p.39).

**Application to Babylon:** Every player-facing verb (mobilize, educate, campaign, attack, aid,
investigate, move, negotiate, reproduce) has to close both gulfs independently. Execution-gulf work:
the action dock icons must visibly signify *what org, what target, what verb* is currently
composable before the player commits (this is what the "how do I work this" question demands).
Evaluation-gulf work: after a verb resolves, the wire feed / event toast / map delta must answer
"what happened, and did it match what I wanted" within the same visual frame the player was looking
at — not buried three panels away. A verb whose only feedback is a wire-feed line two scrolls down
is an evaluation gulf failure exactly analogous to Norman's elevator button with no light (p.23).

### 4. Feedback must be immediate, informative, prioritized, and not excessive — the beep/burp anti-pattern

**What the book says:** "Feedback must be immediate: even a delay of a tenth of a second can be
disconcerting" (p.23). But "too much feedback can be even more annoying than too little" — the
"backseat driver" problem: continuous alerts become "an irritating distraction" and get disabled
wholesale, "which means that critical and important ones are apt to be missed" (p.24). "Feedback
has to be planned... prioritized, so that unimportant information is presented in an unobtrusive
fashion, but important signals are presented in a way that does capture attention" (p.25).

**Application to Babylon:** The tick engine runs 26 systems per turn; if every system emits a wire
event, the feed becomes cacophony and the player tunes it all out — exactly the "hospital operating
room… nuclear power control plants… airplane cockpits" alarm-fatigue case Norman cites (p.25). The
wire feed needs an explicit **severity tier** (ambient flavor / notable / critical-to-player-goals)
with distinct visual weight, and critical events (an org destroyed, a rupture threshold crossed)
should get a modal or map-anchored toast, not just another feed line. Conversely, routine tick
completion needs *some* feedback (a subtle pulse on the time control) — a silent tick-advance is the
elevator-button problem: the player doesn't know if their input registered at all.

### 5. Conceptual models: the single most important principle, and the designer's only lever is the "system image"

**What the book says:** "A conceptual model is an explanation, usually highly simplified, of how
something works. It doesn't have to be complete or even accurate as long as it is useful" (p.25). The
designer's model, the artifact itself, and the user's model form a triangle; because "designers
cannot communicate directly with users, the entire burden of communication is on the **system
image**" — everything perceivable about the device: its structure, documentation, signifiers (p.31,
Fig. 1.11). "Good conceptual models are the key to understandable, enjoyable products: good
communication is the key to good conceptual models" (p.32).

**Application to Babylon:** The map needs ONE legible conceptual model of what a "hex" and a
"territory" and a "state border" *are* to the player — not the engine's actual data model (H3 res-7
cells aggregated to counties/FIPS via multiple joins), but a simplified, sufficient fiction: e.g.
"you are looking at political control, borders redraw as control shifts, zoom reveals the terrain
underneath." Every visual choice (lens colors, border weight, zoom-triggered relabeling) is system
image and must reinforce that one model consistently. The refrigerator anti-pattern (p.28-30, two
knobs implying two independent thermostats when there's actually one shared sensor) is the exact
failure mode to avoid: **do not let the UI structure imply a data model that isn't true** — e.g. if
county borders visually imply county-level sovereignty but the actual unit of contest is the hex,
players will form the wrong model and every subsequent surprise reads as a bug.

### 6. Natural mapping: spatial correspondence between controls and effects, ranked by strength

**What the book says:** "Natural mapping, by which I mean taking advantage of spatial analogies,
leads to immediate understanding" (p.22). Three ranked levels: **best** — controls mounted directly
on the item controlled; **second-best** — controls as close as possible to the object; **third-best**
— controls arranged in the same spatial configuration as the objects (p.115). The stove-burner
example (p.113-117) is the canonical failure: four burners in a 2D rectangle, four controls in a 1D
row — "there are four possible mappings... The only way to know which control works which burner is
to read the labels." Norman's own fix: mount switches on a tilted floor-plan replica of the room
(p.137-138, Fig. 4.5) — literally turning the control panel into a tiny natural-mapped map.

**Application to Babylon:** This is a direct validation of "map lenses" and "inspection panels
anchored to the map" over a sidebar-driven dashboard. Any control that manipulates a *place*
(territory, hex, org stationed somewhere) should live spatially at or adjacent to that place on the
map — not in a docked panel elsewhere that requires the player to re-map "panel row 3" to "that hex
over there" the way stove controls require re-mapping "front-left knob" to "back-right burner."
Concretely: verb targeting should click-anchor context menus at the clicked hex, not open a
detached modal; the nested inspection stack ("every number explains itself") should expand *outward
from the map point clicked*, not reset to a fixed side panel — that is Norman's best/second-best
mapping applied to a strategy-game UI.

### 7. Four kinds of constraints (physical, cultural, semantic, logical) let people assemble correct understanding even with zero instructions

**What the book says:** Demonstrated with a 15-piece Lego motorcycle kit that untrained subjects
assembled correctly with no instructions, because "physical constraints limit alternative
placements. Cultural and semantic constraints provide the necessary clues... and logical constraints
[handle] the one piece left [with] only one place it could possibly go" (p.123-124, Fig. 4.1).
Cultural constraints: things "everyone from that culture knows" (red = brake light, p.129) but which
**can and do vary by culture and change over time** (yellow headlights used to be European standard,
p.129). Semantic constraints rely on situational meaning (the rider faces forward because that's
what "rider" means). Logical constraints are last-piece-only-fits-one-place elimination.

**Application to Babylon:** Design every new icon/color/glyph in the cockpit to stack multiple
constraint types rather than relying on one. E.g., a "fascist consolidation risk" hex overlay should
combine a **physical constraint** (can't be clicked while an animation is resolving — genuinely
disabled, not just styled gray), a **cultural constraint** (red = danger, a near-universal convention
worth exploiting rather than fighting), and a **semantic constraint** (the icon should visually
resemble what it represents — a fist, a flame — not an abstract glyph requiring a legend lookup).
Where Babylon invents its OWN political-map conventions (border redraw semantics, contested-hex
dashing) that have no real-world cultural constraint to borrow, that is exactly where a **persistent,
always-visible legend is mandatory** — Norman's point that cultural constraints "are likely to change
with time" and must be actively taught when novel (p.129).

### 8. The legacy problem and the standardization principle of desperation

**What the book says:** Inelegant designs persist because "too many devices use the existing
standard—that is the legacy" (p.127), and switching costs relearning for everyone. When a genuinely
better design can't win because of legacy lock-in, Norman's fallback: "If all else fails,
standardize... simply design everything the same way, so people only have to learn once ... If you
can't put the knowledge on the device (knowledge in the world), then develop a cultural constraint:
standardize what has to be kept in the head" (p.155). Crucially: "the standards should reflect the
psychological conceptual models, not the physical mechanics" (p.155) — his faucet-blade case shows
consistency should track how people *conceptualize* the action (push/pull), not the literal physical
rotation, even when that means the two blades rotate in mechanically opposite directions (p.152).

**Application to Babylon: adopt ONE interaction grammar and hold it everywhere.** If left-click
selects and right-click opens a context/verb menu on the map, that must be true on every map layer,
every zoom level, every lens — never silently swap to right-click-drag-pans on one lens and
right-click-menus on another. If color intensity means "solidarity strength" on the political lens,
it cannot mean "population density" on the demographic lens without an explicit, signified lens
switch. Internal consistency, once chosen, is more valuable than any single "more logical" choice
(p.149: "Consistency in design is virtuous... it is better to be consistent" than marginally
"more correct").

### 9. Knowledge in the world vs. knowledge in the head — the tradeoff table, and why the map should carry the burden

**What the book says:** Table 3.1 (p.110) lays out the tradeoff precisely: knowledge in the world
is "readily and easily available whenever perceivable," has "ease of use at first encounter" that
is high, but "can be ugly and inelegant... if there is a need to maintain a lot of knowledge." Knowledge
in the head is efficient once learned but "ease of use at first encounter is low" and requires
"considerable" learning. "The unaided mind is surprisingly limited. It is things that make us smart.
Take advantage of them" (p.104).

**Application to Babylon:** A 4X/grand-strategy audience will tolerate *some* knowledge-in-the-head
burden (they expect to learn systems), but the game should still put as much of the "why is this
happening" burden into the world as possible — this is the direct justification for Victoria-3-style
recursive inspection panels ("every number explains itself"): each stat should be a signifier that,
on click, reveals the world-knowledge (the formula, the contributing factors) rather than requiring
the player to have memorized the mechanic. This is Norman's tradeoff resolved in the world's favor
specifically because Babylon's math (imperial rent, survival calculus, contradiction fields) is
genuinely complex enough that memorization is an unreasonable ask.

### 10. Activity-centered vs. device-centered controls — group by what the player is doing, not by what subsystem owns the control

**What the book says:** "Activity-centered controls" group everything a task needs in one place
(lecture mode = light + sound + slide controls together) versus "device-centered" controls that
scatter related functions across separate screens by subsystem ownership, forcing "a horrible
cognitive interruption to the flow" as the user hunts across panels (p.140). But Norman also warns
activity-centered controls fail on exceptions: his own story of a "give a talk" mode that locked the
projector screen down when he tried to raise the room lights mid-Q&A — "invoking the manual settings
should not cause the current activity to be canceled" (p.141).

**Application to Babylon:** The action dock and inspection stack should be organized around player
*intents* (assess this territory / direct this org / respond to this event), not around engine
subsystem ownership (economy panel / topology panel / OODA panel). But build the escape hatch Norman
missed: a player mid-verb-composition who clicks elsewhere on the map to check something should not
lose their in-progress verb — exceptions must not cancel the activity, only pause it.

### 11. Sound and non-visual signifiers — silence itself can be a design failure

**What the book says:** In the electric-car case study, Norman shows that removing a signifier
(engine noise) that people had unconsciously relied on for decades created a real safety hazard for
pedestrians and the blind — "the absence of sound can lead to the same kinds of difficulties we
have already encountered from a lack of feedback" (p.157). Designed sound has to be **alerting**
(indicates presence), give **orientation** (location/speed/direction), and avoid **annoyance**
(infrequent/tolerable) (p.160).

**Application to Babylon:** This licenses sparing, purposeful audio signifiers for the wire feed and
critical events (a distinct low tone for endgame-adjacent thresholds, a soft ambient shift as
contradiction intensity rises) — but only if held to the same three-part discipline: an audio cue
must alert, orient (which region/actor), and never become a background nuisance the player mutes
wholesale (Norman's alarm-fatigue warning, lesson 4, applies identically to sound).

### 12. The paradox of technology — more features, more legibility debt

**What the book says:** "The same technology that simplifies life by providing more functions in
each device also complicates life by making the device harder to learn, harder to use. This is the
paradox of technology and the challenge for the designer" (p.34). Illustrated with the modern
digital watch that does everything and remembers nothing about how (Fig. 1.8, p.27) versus a pair of
scissors whose "conceptual model is obvious" from two holes and two blades (p.27).

**Application to Babylon:** Every new mechanic (26 systems, 5 endgames, 9 verbs, multiple lenses)
adds to this debt. Program 16 should treat "does this feature increase perceived complexity faster
than it increases perceived capability" as an explicit design gate — Babylon's fundamental theorem
and survival calculus are genuinely complex (that complexity is the point, per project memory's
"full vision, no MVP" stance), so the burden falls entirely on interface legibility carrying that
complexity, not on hiding it. This is the strongest argument for the nested/progressive-disclosure
inspection stack over a flat dashboard: complexity should be *available on demand*, never dumped on
the surface (his scissors-vs-watch contrast, applied at the system level).

---

## Directives (imperative, specific, testable)

1. **Every clickable map element must carry a distinct, perceivable signifier (hover glow, cursor
   change, or halo) separate from its underlying geometry** — never rely on hex/county shape alone
   to imply interactivity. Test: screenshot the map with no tooltip open; a first-time viewer must be
   able to point to every clickable element without hovering.

2. **No control's meaning may be explained only by a tooltip, help-page, or onboarding toast on first
   use.** If a control requires that explanation, the control's signifier is broken and must be
   redesigned before shipping. (Norman door test.)

3. **Every player verb must produce visible feedback anchored at or adjacent to its target within
   one animation frame of resolution** (map-pinned toast, hex pulse, or org icon state change) —
   not solely a wire-feed line requiring the player to scroll to find it.

4. **Wire-feed events must carry an explicit severity tier with distinct visual treatment** (ambient
   flavor text vs. notable vs. critical-to-player-goals), and only critical-tier events may
   interrupt with a modal or forced-attention toast. No tick may emit more than one critical-tier
   event without visually merging or queuing them — never a stacked wall of equal-weight alerts.

5. **Any control that manipulates a specific map location (territory, hex, stationed org) must be
   spatially anchored at or immediately adjacent to that location on screen** — verb menus,
   targeting UI, and the inspection stack must open from the clicked point, never from a
   fixed-position side panel disconnected from where the player clicked.

6. **Establish one interaction grammar (what each mouse button/gesture does) and enforce it
   identically across every map lens, every zoom level, and every panel** — a QA pass must confirm
   left-click/right-click/drag semantics never silently change between lenses.

7. **Any color, icon, or overlay convention Babylon invents that has no real-world cultural
   precedent (e.g., border-redraw states, contested-hex dashing, solidarity-strength ramps) must ship
   with an always-visible, non-modal legend whenever that lens or overlay is active** — never gated
   behind a menu the player must remember to open.

8. **Every numeric stat surfaced in the inspection stack must be clickable/expandable to reveal its
   contributing formula and inputs (progressive disclosure)** — no stat may be a dead-end number the
   player must memorize the meaning of from documentation.
