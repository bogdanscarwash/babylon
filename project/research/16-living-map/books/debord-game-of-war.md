# A Game of War (Alice Becker-Ho & Guy Debord) — UX research for Babylon's living map

**Source:** *A Game of War*, Atlas Press, 2011 (trans. Donald Nicholson-Smith), original *Le Jeu de la
Guerre*, Gérard Lebovici, 1987. PDF at
`/home/user/Downloads/babylon_books/ux/A Game of War -- ALICE BECKER-HO; GUY DEBORD -- 2011 -- Atlas
Press -- 9781900565387 -- 8bb9bbe479112382b89e37a317f9ecfd -- Anna's Archive.pdf`.

**Coverage note:** this edition is short — 27 pages total, fully read (pp. 11–35 of the printed book:
title/copyright, then "The Rules of the Game of War" §§1–7, then "Explanatory Diagrams" with a
symbol key and worked Figures 1–6). It is the rules-and-diagrams booklet only; it does **not** contain
the longer Situationist essays on strategy, biography, or historical commentary that accompany some
other editions/printings of this title. Every lesson below is cited to this text as read; nothing is
inferred from outside knowledge of Debord's other writings.

Despite the brevity, the rules booklet is dense with transferable UI lessons because Debord designed
it explicitly as **a legible, arithmetic, deterministic model of strategy** — which is exactly
Babylon's own claim ("Graph + Math = History"). The kinship is structural, not just tonal: both are
kriegspiel-descended systems where a player must be able to *see the state that produces the outcome*,
not just the outcome.

---

## Core lessons

### 1. Terrain is an absolute, not a suggestion — and the UI must show which squares are absolute

> "Mountains constitute an absolute barrier to troop movement and completely obstruct fire.
> Similarly, they block all lines of communication between the armies and their arsenals and
> communications units." (§1, p. 13)

Debord's board has exactly four terrain types (plain, mountain, mountain pass, fort) and each one
changes what is *computable* at that square — mountains hard-block movement, fire, and communication;
passes and forts change a unit's defensive factor (6→8→10 for infantry, 8→10→12 for artillery, per the
Table of Forces, p. 12). There is no ambiguous terrain — a square is either open, absolutely closed, or
a chokepoint with a stated numeric bonus.

**Application to Babylon:** hexes/territory tiles that gate imperial-rent flow, supply-chain edges, or
OODA movement (mountains, contested/blockaded corridors from the transport substrate) must be visually
distinguishable at the *base* map lens, not just discoverable by clicking. A player should be able to
tell, from color/pattern alone at any zoom, which hexes are (a) freely traversable, (b) chokepoints
that grant a stated bonus/penalty, (c) absolutely impassable. Any hex that structurally alters a
formula (rent multiplier, transport conductivity, control-ratio threshold) earns a distinct terrain
glyph in the base cartography layer — not a lens-only overlay a player might not have toggled on.

### 2. Lines of communication are the actual object of strategic attention — make the supply network a first-class, always-inspectable overlay

> "All a fighting unit's offensive and defensive value, and all its mobility, are entirely dependent on
> the necessity for that unit to remain in communication with one or another of its army's arsenals...
> A unit can neither move nor engage in combat unless it remains on a square which is in communication,
> either direct or indirect, with one of its own arsenals." (§4, p. 17)

> "In view of the vital importance of communication, strategy in this game is more often concerned with
> movement against the adversary's communications than with either offensive action directed against
> [...] enemy arsenals, or with the wearing down of enemy strength by means of enduring superiority on
> the battlefront." (§4, p. 19)

The rules give this its own worked diagrams (Figures 1–3, pp. 29–30) showing every permanent line of
communication for each side as a literal drawn network of rays from each arsenal — before a single unit
moves. This is treated as more fundamental to reading the board than piece count.

**Application to Babylon:** the supply-chain / TRIBUTE / imperial-rent edge network and the transport
substrate's corridor mesh deserve a dedicated map lens (a "lines of communication" lens) that draws the
full active network as persistent lines from source (arsenal-equivalent: capital/production hub) to
every connected node — not just on hover, not just in the inspection panel. A node cut off from its
network must be visibly distinct on the base map (not merely a stat in a side panel) — parallel to
Debord's "isolated unit... defenceless against attack" (§4, p. 18): in Babylon this is the
`ImperialRent`/`Dispossession`/`ReserveArmy` systems' inputs going to zero, and it should read on the
map as starkly as a besieged fort reads on Debord's board.

### 3. A unit cut off doesn't just lose stats — it becomes paralyzed and undefendable, and that state must be legible before the kill

> "A fighting unit may move into a square where it is no longer in communication [...] the isolated
> unit is condemned to immobility and stripped of all offensive and defensive capacity. It is
> defenceless against attack and may be destroyed at will by any enemy unit stationed [...] within
> firing range." (§4, p. 18)

This is a specific, named failure state (isolation) with its own rules for how it is entered and exited
(direct/indirect communication, relief by a friendly force "penetrating the enemy front"). It is not
merely "low HP."

**Application to Babylon:** endgame-adjacent conditions (organizations losing solidarity connectivity,
territories entering `Dispossession`/atomization) should get a named, iconographically distinct map
state — e.g. a "cut off" badge/hatch pattern on the node — that appears *before* the terminal collapse
tick, giving the player the same read Debord gives a wargamer: "this piece is already lost unless
relieved." This is a direct instruction for the inspection stack's node header (a status chip: nominal /
isolated / under siege) and for the wire feed (an isolation event should fire distinctly from a combat
event).

### 4. The stat table is compact, columnar, and comparative — not prose

The "Table of Forces in Play, with their Characteristics" (p. 12) puts every unit type in one table:
Type / No. / Offensive factor / Defensive factor (broken out by terrain: ordinary/pass/fort square) /
Range / Mobility. A player can compare any two unit types at a glance without reading a paragraph.

**Application to Babylon:** the Victoria-3-style recursive inspection panel's *first* screen for any
entity (organization, territory, class) should be exactly this shape — a compact table of the handful
of numbers that feed the formulas that act on it (e.g. for a territory: control ratio, defensive
modifiers by terrain/fortification state, current rent extraction, solidarity edge count), with
"explain this number" drill-down available per cell, but the top-level view is table, not narrative
paragraph.

### 5. Combat resolution shows its arithmetic in the open — the popup should show the sum, not just the verdict

Figures 4 and 5 (pp. 31–33) walk the exact same attack (South attacks square H8) twice with different
force compositions, and for each, the text gives the literal per-unit contribution and the running
total: *"North's total defensive factor with regard to the square under attack is thus: 6 + 6 + 8 + 8 =
28... South's total offensive factor... 34... Since South's total offensive factor (34) surpasses
North's total defensive factor (28), the attack is successful."* The second case (Figure 5) is the
mirror result (41 vs 40, attack fails) shown with the same transparency, and the text goes further to
note the exact margin that would have changed the outcome ("Had the attackers mustered two more
points... this would have obliged the infantry unit... to retreat", p. 33).

**Application to Babylon:** any event/attack toast or wire story generated from a system resolution
(e.g. a StruggleSystem confrontation, a FascistFaction contest, a Sovereignty contest) should offer a
one-click "show the math" expansion listing every contributing edge/node and its numeric contribution
to the winning/losing side, plus — where meaningful — the margin, mirroring "had the attackers mustered
two more points." This is the map-adjacent event-toast equivalent of Debord's worked figures, and it
directly serves the game's own "Graph + Math = History" mantra: the player should be able to verify the
history, not just be told it.

### 6. A minimal, fixed, unambiguous symbol legend covers the entire game

"Key to Symbols" (p. 28, "Explanatory Diagrams") lists exactly nine glyphs — arsenal, fort, mountains,
mountain pass, foot artillery, mounted artillery, cavalry, infantry, foot/mounted communications units,
and "square under attack" — each a distinct pictogram, and that is the *entire* iconographic vocabulary
of the game. Every diagram in the book (Figures 1–6) uses only these nine marks, drawn consistently.

**Application to Babylon:** each map lens (imperial rent, solidarity, transport, contradiction field,
etc.) should ship with its own small, fixed legend (analogous scope: under a dozen glyphs) that is
*always visible whenever that lens is active* — not hidden in a settings drawer — and every symbol used
anywhere in that lens's rendering must appear in the legend. "Square under attack" is a useful
precedent specifically: Babylon's event/combat highlight state (a contested hex, a live Struggle/
Sovereignty contest) should get one single, consistent glyph reused across every lens, exactly as
Debord reuses one mark for "square under attack" regardless of which figure it appears in.

### 7. Strategic layer and tactical layer are explicitly different views of the same board — support both, don't collapse them

> "This emphasis also affects tactics, for the order of battle adopted at each moment must take into
> account not only the best positioning for the purposes of defence and counter-attack but also the
> best means of covering one's lines of communication." (§4, p. 19)

> Figure 6 (pp. 34–35) is captioned entirely in strategic vocabulary — "pivot of manœuvre," "flanking,"
> "axes of attack open to South's cavalry" — drawn as arrows over the whole board, distinct from
> Figures 4–5's square-level arithmetic.

The rules never blend these: Figures 1–3 are pure communication-network views, Figures 4–5 are
square-level combat arithmetic, Figure 6 is whole-board maneuver arrows. Each figure commits to one
level of description.

**Application to Babylon:** the map-lens system (per the ratified Paradox-style-lens direction) should
keep strategic-scale lenses (imperial rent flow, contradiction field, solidarity network) and
tactical/local lenses (a single county's control ratio breakdown, a single contest's combat math)
visually and modally separate — a lens change, not a lens overload. Resist stacking maneuver-arrow
overlays and per-node arithmetic tooltips in the same view at the same time; that is precisely the
distinction Debord's own diagram set enforces figure-by-figure.

### 8. The board is asymmetric in shape, symmetric in resources — separate "immutable substrate" from "who currently holds it"

> "The two regions are asymmetrically disposed, but each contains the following: two arsenal squares,
> three fort squares, one mountain-pass square, and nine mountain squares." (§1, p. 13)

> "A fort, regardless of ownership at the start of play, serves whichever side is in possession of it:
> as soon as an enemy unit seizes a fort, the advantage it affords in tactical defence passes entirely
> into the hands of the new occupant. Unlike arsenals, forts are never destroyed, and they may change
> hands several times in the course of hostilities." (§5, p. 20)

Geography (mountain/pass/fort positions) is fixed at setup and never changes; what changes over the
game is *who controls* each fort. Arsenals cannot even be captured, only neutralized by occupation
while held.

**Application to Babylon:** this is close to a direct restatement of the Constitution's own rule ("the
spatial substrate is immutable; political claims are overlays") — cite it as external corroboration.
Concretely for the borders-redraw feature: county/state boundary geometry (the base cartography) should
never itself move; only the *political-claim color fill* over that geometry changes as revolution/
liberation/collapse progresses, exactly as Debord's forts stay put while only their occupying color
changes. Do not animate hex/county boundary geometry to reflect political change — animate the fill and
the control-badge instead.

### 9. Logistics/observer units are visually and functionally distinct from combat units, and their fragility is a designed tension

> "These non-fighting units have no offensive factor; their defensive factor is 1 and their range is
> two squares. They constitute a rich target in themselves for the adversary, so they need, if
> isolated, to be kept out of enemy range, and otherwise to be protected by an adequate number of
> fighting units." (§4, p. 17)

Communications units get their own two glyphs (foot/mounted) distinct from combat units, and the rules
explicitly flag them as high-value/low-defense — a designed asymmetric-risk piece.

**Application to Babylon:** organizations engaged in intelligence/observe-only actions (the `investigate`
verb, key-figure/observer nodes, wire-feed sourcing nodes) should render with a distinct, consistently
recognizable non-combat glyph on the map — not shape-shared with mobilize/attack-capable organizations —
and their exposed/unprotected state (analogous to "isolated communications unit") should be surfaced the
same way as any other vulnerability badge (see lesson 3), since losing one has an outsized informational
cost even though it has zero offensive footprint.

### 10. The designers publish their own list of what the model leaves out — and that list is itself a UI lesson

> "Three basic and universal aspects of real warfare are absent or under-represented in the game...
> first, weather conditions and the alternation of night and day; secondly, the morale and degree of
> fatigue of troops; and, thirdly, uncertainty with regard to enemy positions and movements... From the
> moment hostilities open, exact and certain knowledge of all enemy movements is vouchsafed instantly.
> To borrow the words of an old French proverb, *L'ost sait ce que fait l'ost*: each army knows what
> the other army is doing." (§7, pp. 24–26)

Debord doesn't hide this as a flaw — he states it as a designed, load-bearing simplification, so a
player never mistakes the model's legibility for a claim about real fog-of-war.

**Application to Babylon:** wherever Babylon *does* model something Debord's game admits to omitting
(fog of war via OODA's observe phase, day/night or seasonal cycles if any exist in defines, morale via
Consciousness/Survival systems), the UI should make that a **visible, named departure** from perfect
information — e.g., a wire-feed story sourced from imperfect organizational observation should be
tagged distinctly from a ground-truth engine event, so the player can tell "this is what an org
believes" from "this is what the graph/math actually computed." Precedent: Debord's own rules text
treats "the model is transparent about what it doesn't model" as part of the design, not an
apology — Babylon's UI should carry the same honesty into its narrative layer (wire feed vs. engine
truth).

---

## Directives (imperative, testable)

1. **Every hex/territory tile that structurally alters a formula (rent multiplier, transport
   conductivity, control-ratio threshold, blockade) must render a distinct terrain glyph on the base
   map lens by default — never gated behind a lens toggle the player must discover.**
2. **A dedicated "lines of communication / supply" map lens must draw the full active
   solidarity/tribute/transport-corridor network as persistent lines from each source node to every
   connected node, on demand, at all times the lens is active — not only on hover.**
3. **Any node whose upstream connectivity is severed (isolation, blockade, atomization) must display a
   distinct, named status badge on the map itself before its terminal-collapse tick — not only inside
   the inspection panel.**
4. **The first screen of the inspection panel for any entity (organization/territory/class) must be a
   compact comparative stat table (the direct inputs to the formulas acting on it), with narrative
   explanation available only on drill-down, never as the top-level view.**
5. **Every combat/contest event toast or wire story generated by a system resolution must offer a
   one-click "show the math" expansion listing each contributing edge/node's numeric contribution and
   the winning margin.**
6. **Each map lens must ship a fixed, always-visible legend (under a dozen glyphs) covering every
   symbol that lens renders; the "under contest/attack" glyph must be the same single mark reused
   across every lens, never lens-specific.**
7. **Strategic-scale lenses (rent flow, contradiction field, solidarity network) and tactical/local
   views (single-node combat arithmetic, single-county control breakdown) must never be layered
   simultaneously in one view — switching between them is a lens change, not an overlay stack.**
8. **County/state/hex boundary geometry must never animate or redraw to reflect a change in political
   control; only the political-claim fill color and control badge change — boundary geometry changes
   only when the underlying spatial substrate itself is edited (a constitutional-level event).**
