# Theory Priorities for the Living Map UI

**Question asked:** not "what does the theory say" (the engine already implements it) but **which
quantities and relations are ideologically load-bearing enough to deserve UI prominence** in a game
about the fall of American hegemony — and specifically *where* in the interface they belong.

**Sources read** (strategic/TOC-first, ~180pp total):
- Zak Cope, *Divided World Divided Class: Global Political Economy and the Stratification of Labour
  Under Capitalism* (Kersplebedeb, 2012) — cited as **Cope**, by Part/Chapter.
- Samir Amin, *The Law of Worldwide Value* (Monthly Review Press, 2010 English ed.) — cited as
  **Amin**, by Chapter.
- *MIM Theory* No. 10, "Coming to Grips with the Labor Aristocracy" (Maoist Internationalist
  Movement, 1996) — cited as **mt10**, by article.

Location: `/media/user/data/mim/DividedWorldDividedClass_ZakCope.pdf`,
`/media/user/data/mim/TheLawofWorldwideValue_SamirAmin.pdf`, `/media/user/data/mim/mt10.pdf`.

Where useful I cross-reference the concept to the concrete engine implementation found in this
repo (`src/babylon/`) so the UI recommendations are anchored to real fields, not aspirational ones.

---

## 1. Priority quantities, ranked

### 1.1 Imperial Rent Φ (net value extraction per area) — **highest priority, permanent chip**

**Theory.** Amin's central late-career thesis (Amin, Ch. 4, "Accumulation on a Global Scale and
Imperialist Rent") is that the transformation of *value* into *globalized value* is "by far the most
consequential" metamorphosis in Marxist theory, operating "in decisive fashion in all the fields of
social struggle and in international and national political conflicts of the modern world" (Amin,
Ch.4 §intro). Labor-power has a *single* global value (set by the General Intellect of the world
productive forces) but is recompensed at wildly different rates — "variations in the price of
labor-power... within the central capitalist countries... are multiplied tenfold on the global scale"
(Amin, Ch.4 §1). Imperial rent has **two dimensions**, both modeled: (1) the globalized hierarchy of
labor-power prices (§1–3), and (2) unequal access to natural resources / extractive rent (§4–6,
"Ground Rent" ch.). Amin is explicit that this is *not* incidental to capitalism but its "very precise"
imperialist essence: capitalism's worldwide spread "far from progressively 'homogenizing' economic
conditions... reproduced and deepened the contrast" between center and periphery (Amin, Ch.4 §intro).

**Engine anchor.** `ImperialRentField` (`src/babylon/domain/economics/tensor_hierarchy/types.py:463`)
already computes exactly this: `phi[a] = inflows(a) - outflows(a)` per area, signed, closed-system
conserved (`sum(phi) ≈ 0`). Positive Φ = extraction/accumulation zone (core); negative Φ = drain zone
(periphery). `ImperialRentSystem` runs a 5-phase circuit (Extraction → Tribute → Wages → Subsidy →
Decision) every tick.

**UI placement:**
- **Permanent top-bar chip**, always visible, always signed (+/−), color-coded (accumulation vs
  drain), because this is *the* number the whole simulation exists to compute — this is Amin's most
  important contribution to Marxism in his own estimation, and every other quantity below is a
  component or consequence of it.
- **Default map lens** ("Extraction" lens): choropleth of Φ per county/state, so the map's base
  read is always "who is being drained, who is accumulating" before any other layer is toggled on.
- **First row of every territory/county inspection panel** — Φ for that node, decomposed into its
  inflow/outflow components (tribute in, wages out, etc.) so the number is never presented as a bare
  fact but as a traceable sum (Victoria-3-style "every number explains itself").
- **Endgame scoring**: aggregate Φ trend (is imperial rent collapsing toward zero as the empire
  falls, or is it being defended by fascist consolidation?) should be a named axis on the endgame
  summary screen, not buried in a table.

### 1.2 Core wage vs. value produced (W_c vs V_c) — **second priority, always paired with its source**

**Theory.** This is Babylon's own "Fundamental Theorem" (`W_c > V_c` ⇒ revolution in the core is
impossible), and both books independently derive the same conclusion empirically. Cope's central
finding (Cope, Pt.II §11.3 "Conclusion"): "the profits of the capitalist class in the OECD... are
entirely derived from the superexploitation of the non-OECD productive workforce," and "the OECD
working class *tout court* receives the full value of its labour and is, to that extent, a **bourgeois
working class**" (Cope, 11.3). This is not a fringe claim requiring hedging in the UI — it is the
book's load-bearing conclusion, reached via conservative methodology (Cope explicitly notes his
assumptions are "almost certainly overly generous to the First Worldist position," 11.3).

**Quantitative grounding** (Cope, Pt.II §11.2, "Estimates of Superprofits and Super-Wages"):
- mean factoral wage ratio, OECD : non-OECD male workers = **11:1** (Cope App. I; recomputed as
  average wage factor 6.5 when population-weighted, 11.2 "Unequal Exchange through International
  Wage Differentials").
- net value transfer via unequal exchange in traded goods alone ≈ **US$2.8 trillion/year** (2010);
  combined with wage-differential transfer ≈ **US$6.5 trillion/year** — for every 1 OECD worker,
  "1.5 non-OECD producers are working unseen and for free alongside her" (Cope, 11.3).
- if the entirety of OECD profits (~US$7tn in 2009) were redistributed only to the non-OECD workers
  who produced them, their wage would still be only 70% of the average OECD wage (Cope 11.3) — i.e.
  even a *maximally generous* redistribution inside the current price structure cannot equalize
  wages; only ending the imperial rent relation can.
- mt10's domestic microcosm (`mt10`, "The White Working Class: Grass Parasitism," MC12): the same
  mechanism operates *within* U.S. borders — Black/white full-time median-wage gap in 1992
  ($6,928/worker) accounted for **21.2% of total corporate profits that year** ($52.8bn of $249.1bn),
  and U.S.-based vs. foreign-affiliate compensation for the same multinational employers differed by
  $14,285/worker, i.e. **$136.25 billion "by virtue of being in the U.S."** — proof the wage
  differential tracks national/racial position in the imperial hierarchy, not "productivity."

**UI placement:**
- **Never show a core wage number without its imperial-rent source inline** (see Framing Rules §3
  below) — this is the single most important UI *rule*, not just a quantity placement.
- **First row of every social-class node inspection** where the class is "labor aristocracy" or
  "core proletariat": show wage, value produced, and the gap, with the gap's provenance (which
  peripheral territory/tribute edge it traces to, if traceable in-graph).
- **Not a permanent top-bar chip** by itself (too granular per-class) — but a **map lens**
  ("Labor Aristocracy" lens) shading classes/territories by `W_c / V_c` ratio is warranted, since it
  visualizes the bifurcation the whole Part IV of Cope's book (III/IV, "Marxism or Euro-Marxism?")
  is built on: which national working classes have a material stake in imperialism and which don't.

### 1.3 Unequal-exchange gradient (price-value distortion, σ) — **map lens, not a chip**

**Theory.** Amin's formal mechanism for how Φ becomes concrete: "unequal exploitation is manifested
in unequal exchange... it distorts the structure of demand, accelerating self-centered accumulation
at the center while hindering dependent, extroverted accumulation in the periphery" (Amin, Ch.4 §1).
Cope operationalizes this as the price-value distortion parameter `d` in his transfer formula
`t = -vp + vdp + p + evd/p + evd` (Cope, 11.2) — i.e., value transfer is literally a function of how
far traded prices diverge from labor-time values across the core/periphery boundary. This is exactly
the σ-gradient already registered in this repo's roadmap (Constitution Amendment N, "Spectrum of
Unequal Exchange," program-10).

**UI placement:** this is inherently a *relational/edge* quantity (it lives on TRIBUTE/WAGES edges
between territories, not on a single node), so it belongs as a **map lens that renders trade/tribute
edges with a color-and-thickness gradient** (thin/pale = near parity, thick/saturated = high
distortion) rather than a chip. It is the natural "second lens" a player toggles after the Φ
choropleth, to ask "where specifically is this extraction flowing."

### 1.4 Superexploitation / superprofits — **county inspection, second row**

**Theory.** Cope distinguishes ordinary exploitation (wage < value produced) from
**superexploitation**: wages driven below the cost of reproducing labor-power itself, "often to the
point where their wages are set at levels insufficient for their households to reproduce their
labour-power" (Cope, Pt.II "Exploitation and Superexploitation"). This generates **superprofits**,
which Cope traces to three mechanisms: technological advantage, monopoly rent, and unequal exchange
(same section). mt10 grounds this in Lenin's *Imperialism* and in concrete non-wage subsidy
mechanisms: dispossessed peasantry subsidizing urban wages, unpaid domestic/reproductive labor,
below-value sales by petty producers (Cope, Pt.II, "five conditions... of superexploitation").

**UI placement:** a **binary/tiered badge** on territory and social-class nodes ("exploited" vs.
"superexploited," with the reproduction-cost floor shown as a threshold line on the wealth/wage
sparkline) — this maps directly to Babylon's existing `Vitality`/`Subsistence` mechanics and should
sit in the **second row of the county inspection panel**, directly under Φ, since superexploitation
is Φ's *proximate cause at the point of production* (as opposed to the aggregate flow itself).

### 1.5 Organization / Repression ratio — **survival-calculus overlay, event-triggered prominence**

**Theory.** This is Babylon's own Survival Calculus construct (`P(S|R) = Organization / Repression`
vs. `P(S|A) = Sigmoid(Wealth − Subsistence)`), but it is directly grounded in the texts: mt10's
Comintern review ("Lessons from the Comintern") and MIM's own 1995 Resolution treat the
**recognition of superprofits + organizational independence from the labor aristocracy** as "an
international line of demarcation" (mt10, Resolution, p.4) — i.e. organization-building against
repression, decoupled from reliance on the bribed core working class, *is* the strategic axis MIM
theory cares about. Amin frames the same tension as class struggle determining "the amount of
surplus labor appropriated," "the amount of rent drawn," and ultimately whether a rupture occurs
(Amin, Ch.4 §3).

**UI placement:** not a permanent chip (it's a *ratio of potentials*, not an observed flow like Φ),
but should surface prominently **the moment `P(S|R)` crosses `P(S|A)`** — i.e. an event-triggered
popup/banner marking a Rupture Event threshold crossing, consistent with the "event popups" pattern
already planned. This is the correct place for drama, not ambient chrome.

### 1.6 Metabolic Rift / Ecological Overshoot (O = C/B) — **secondary permanent chip, paired with Φ**

**Theory.** Amin's second dimension of imperial rent (Ch.4 §4–6): "unequal access to the utilization
of planetary resources constitutes... the second dimension, no less important than that following
from the globalized hierarchization of labor-power prices" (Amin, Ch.4 §intro). He grounds this in
Wackernagel & Rees's ecological-footprint metric: global bio-capacity 2.1 gha/capita vs. consumption
2.7 gha/capita, with the triad (Europe/N.America/Japan) at ~4x the global average — i.e. the same
extractive relation, denominated in use-value/hectares rather than price (Amin, Ch.4 §6.1). Amin is
emphatic that this is not a separate "green" concern bolted onto class analysis but the same
imperial-rent relation on its natural-resource axis, and that the discourse's capture by "weak
sustainability" (letting markets price the commons) is itself an ideological maneuver to protect
oligopoly access (Amin, Ch.4 §6.5).

**UI placement:** a **secondary permanent top-bar chip next to Φ** — not because it's less
important theoretically (Amin says explicitly it is "no less important") but because it's the
*other half* of the same imperial-rent picture, and pairing the two chips visually teaches the
player that imperial rent has both a labor axis and an ecological axis. Overshoot O > 1 should use
the same color grammar as Φ's "drain" state.

### 1.7 The "myths" — productivity, skill, militancy explanations for wage differentials — **tooltip/debunk layer, not a top-level quantity**

**Theory.** Cope's Part III, "The Ideology of Global Wage Scaling," is a sustained refutation of the
three conventional (including "Euro-Marxist left") explanations for the North-South wage gap:
productivity differences, skill differences, and workers'-militancy differences (Cope, Pt.III,
chapters III.2–III.4). He shows these are "both theoretically and empirically flawed" and function
ideologically to legitimate the world wage hierarchy "favouring the world's upper quintile" (Cope,
Pt.III intro). This matters for UI **framing**, not for a standalone quantity: whenever the game
surfaces a wage or productivity number, a contrasting "what apologists claim" vs. "what the data
shows" framing (see Framing Rule 2 below) teaches the theory by refuting the naturalized cover story
in the same breath.

---

## 2. Summary priority table

| Quantity | Theoretical centrality | UI slot |
|---|---|---|
| Imperial Rent Φ (net value extraction/area) | Amin's central late-career thesis; "by far the most consequential" metamorphosis of value | Permanent top-bar chip + default map lens + row 1 of every territory panel + endgame axis |
| Core wage vs value produced (W_c/V_c) | Cope's book-length empirical proof that the OECD working class is "to that extent, a bourgeois working class" | Row 1 of class-node panel, always paired with its rent source; secondary map lens |
| Unequal-exchange gradient (price-value distortion σ) | Amin's formal mechanism connecting exploitation to exchange; Cope's transfer formula's `d` parameter | Trade/tribute edge-rendering map lens |
| Superexploitation / superprofits | Cope's mechanism-level breakdown of Φ's proximate cause | Row 2 of county/class panel, tiered badge against reproduction-cost floor |
| Organization/Repression ratio | MIM's Comintern-derived "line of demarcation"; Babylon's Survival Calculus | Event-triggered rupture banner, not ambient chrome |
| Ecological overshoot O = C/B | Amin's explicitly co-equal "second dimension" of imperial rent | Secondary permanent chip, paired visually with Φ |
| Productivity/skill/militancy "explanations" | Cope's ideology-critique (what to *debunk*, not display) | Tooltip/contrast framing wherever a wage number appears |

---

## 3. Framing rules — how the UI should teach the theory through play

1. **Wages never appear naked.** Any UI surface showing a core-country wage number (chip, tooltip,
   inspection panel) must show, in the same visual unit, the value that class/territory actually
   produced and the imperial-rent transfer that accounts for the gap. This operationalizes Cope's
   central finding that core wages are "entirely derived from the superexploitation" of the periphery
   (Cope, 11.3) — if the UI ever lets a wage read as self-generated, it has taught the *opposite* of
   the theory.

2. **Every "productivity"/"skill" framing gets its rebuttal alongside it, not hidden in a footnote.**
   When the game (via AI narration or a tooltip) offers a productivity- or skill-based explanation for
   a wage gap — which it should, because that is the cover story real actors give — the UI should let
   the player drill into the actual mechanism (rent transfer, monopoly, militarized border) in one
   click, mirroring Cope's method of quoting the apologist claim and then walking through the data
   that refutes it (Cope, Pt.III).

3. **Borders are colonial artifacts, and the map must say so before the player asks.** Amin: the
   world system is "made up of segments that appear heterogeneous... groups of capitalist firms...
   zones that seem to be precapitalist... groups of natural resources, access to which is more or
   less obstructed, depending on the laws of the states concerned" (Amin, Ch.4 §1) — i.e. the
   boundary itself is a political-economic artifact of imperial rent extraction, not a neutral
   administrative fact. Cope is blunter: militarized borders are one of the four causal mechanisms of
   low peripheral wages, existing to "prevent the equalisation of returns to labour interzonally" and
   "ensure the perpetuation of a global wage hierarchy" (Cope, Pt.II "Understanding Capitalism and
   Imperialism," "Militarised borders"). Concretely: the starting county/state cartography should be
   labeled/framed as the *colonial* baseline (a claim-structure, not terrain), and as it redraws with
   revolution/collapse the UI's border-redraw animation should be legible as "a militarized wage-
   hierarchy boundary dissolving," not merely "territory changed hands."

4. **Superexploitation is a floor-crossing, not a slider.** Because Cope defines superexploitation as
   wages driven *below the reproduction cost of labor-power itself* (not merely "low wages"), the UI
   should render it as a threshold state (a line on the wealth sparkline, a badge) rather than a
   continuous gradient alone — the qualitative jump from "exploited" to "superexploited" is the point,
   echoing Marx's own subsistence-floor language that mt10 spends several pages defending against
   dilution into "any ridiculous luxury" (mt10, "Grass Parasitism," MC12).

5. **National/cross-class "solidarity" in the core must be shown as a material alliance, not an
   identity.** Cope: "in the core countries of the global economy a profound basis for *national*
   solidarity is created between workers and their employers" via superprofits (Cope, 11.3, emphasis
   added) — this is the mechanism behind Babylon's own bifurcation rule (agitation routes to fascism
   absent a SOLIDARITY edge). Whenever the UI shows a core social class aligning with capital/fascism
   rather than with the periphery, a one-line causal tag ("stake in imperial rent: [Φ value]") should
   accompany it so the alignment reads as materially motivated, not as an unexplained ideological
   choice.

6. **The ecological axis is not a separate "green" panel — it rides with Φ.** Per Amin's insistence
   that resource access is imperial rent's "second dimension... no less important" (Amin, Ch.4
   §intro), overshoot/metabolic-rift indicators should be visually paired with the Φ chip (same
   panel, adjacent position, shared color grammar) rather than siloed into a separate environmental
   tab — siloing it would itself be the ideological move Amin warns against (ecology "captured" by
   vulgar economics as an externality rather than integrated into the value analysis, Amin Ch.4 §6.4).

7. **Endgame scoring should foreground the two axes the books actually argue matter: rent trend and
   organizational rupture**, not a generic "victory points" tally. REVOLUTIONARY_VICTORY should read,
   mechanically, as Φ trending toward zero (imperial rent extraction ending) *combined with*
   `P(S|R) > P(S|A)` having resolved into stable organization rather than collapse — i.e. the two
   headline chips (§1.1 and §1.6) plus the rupture-threshold event (§1.5) should visibly converge at
   the endgame screen, so a player who has been reading the top bar all game already knows which way
   the story is going.

---

## 4. Map implications — what the default political lens should emphasize

- **Default lens = Φ choropleth** (extraction vs. accumulation), not population, not GDP, not raw
  military strength. This is the single map read Amin's whole argument is built to support: "the
  underdeveloped countries are so because they are super-exploited and not because they are
  backward" (Amin, Ch.4 §2) — a GDP-per-capita or "development" lens would visually reproduce the
  exact ideology (backwardness vs. modernity) the book is refuting. A Φ lens instead shows *who is
  extracting from whom right now*.
- **Borders rendered as an overlay on the extraction substrate, not the substrate itself** — consistent
  with the project's own Constitution (spatial substrate immutable, political claims are overlays)
  and with Amin's point that the international division of labor, not the map, is the primary
  object (Amin, Ch.4 §1). County/state lines should visibly "sit on top of" the hex/extraction layer
  and be able to dissolve or redraw without the underlying material data changing, which is exactly
  the intended mechanic (revolution redraws claims, not geography).
- **A secondary "wage hierarchy" lens** (§1.2) should be one toggle away, so a player can flip between
  "where is value being extracted" (territory-level, Φ) and "who benefits from that extraction"
  (class-level, W_c/V_c) — these are Amin's and Cope's two complementary levels of analysis
  (world-systemic flow vs. domestic class stratification) and the map should let the player hold
  both without conflating them.
- **Trade/tribute edges rendered with the unequal-exchange gradient** (§1.3) as an optional overlay
  on top of either lens, since this is the mechanism connecting the two — it shows *how* Φ moves
  from periphery to core, not just that it does.
- **Avoid a raw "wealth" or "GDP" lens as the default or even as a prominent option** — both books
  spend substantial space refuting the idea that aggregate wealth/output figures are neutral
  (Marx's wealth/value distinction, Amin Ch.4 §6.3: "Marx does not confound 'value' with 'wealth,' as
  do all the vulgar economists"). If a wealth lens exists at all it should be clearly secondary and
  ideally always shown *split* by its Φ-attributable and locally-produced components, never as a
  single undifferentiated number.

---

## 5. Notes on scope and what was not read in full

Given the "which quantities deserve prominence" framing, this pass prioritized the chapters that
directly quantify or formally define the transfer mechanisms (Amin Ch.4; Cope Pt.II §11.2, Pt.II
"Exploitation and Superexploitation," Pt.III intro; mt10's two accounting essays) over chapters that
are primarily historical-narrative (Cope Pt.I "Historical Capitalism," Pt.IV "Marxism or
Euro-Marxism?" country case studies; Amin's "Ground Rent" and "Interest, Money and the State"
chapters) or primarily polemical/organizational (mt10's Black Panther Party and Du Bois reviews,
letters section). Those chapters support and contextualize the priorities above but were sampled
rather than read cover-to-cover; if a future pass wants historical grounding for specific *narrative*
content (wire/event copy) rather than *UI quantity* prioritization, they are the next place to look.
