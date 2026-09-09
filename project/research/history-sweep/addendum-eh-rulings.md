# Addendum: Epistemic Horizon — Rulings EH-5 and EH-3

Source spec: `/home/user/projects/game/babylon/ai/epochs/epoch3/fog-of-war.yaml` (872 lines,
read in full). Decay constants live in `vision_states.desert.mechanics.decay_rate` (line 312),
`vision_states.mud.mechanics.decay_rate` (line 357), `vision_states.water.mechanics.decay_rate`
(line 403), consolidated in the `intelligence_decay` section (lines 605–637) with the governing
formula at line 615: `I_c(t+1) = I_c(t) × (1 - decay_rate)`.

Briefs consulted: `brief-eh-mass-line.md`, `brief-eh-infiltration.md`, `brief-eh-receptivity.md`
(all in the scratchpad `history-sweep/` directory).

---

## RULING EH-5: Decay re-derivation against the weekly tick

### Method and baseline arithmetic

All three briefs independently converge on **1 tick ≈ 1 week** (Comintern Third Congress weekly
cell reporting, IWW weekly job-branch bulletins) — this conversion is not itself in dispute and is
adopted here. The spec's formula is multiplicative: `I_c(t+1) = I_c(t) × (1 - r)`, so the natural
comparison metric is **half-life** `t½ = ln(0.5) / ln(1 - r)`.

| Tier | `r` (spec) | Half-life (ticks) | Half-life (weeks) |
|---|---|---|---|
| Desert | 0.20 | ln(0.5)/ln(0.8) = 3.106 | ≈3.1 weeks (~22 days) |
| Mud | 0.05 | ln(0.5)/ln(0.95) = 13.51 | ≈13.5 weeks (~3.1 months) |
| Water | 0.01 | ln(0.5)/ln(0.99) = 68.97 | ≈69 weeks (~1.3 years) |

### Desert (r = 0.20, half-life 3.1 weeks) — VERDICT: KEEP

Two independent corpus timescales bracket this well. `brief-eh-infiltration.md` §2.1 gives
`agent_reinsertion_lag_ticks ≈ 3 ticks`, derived from Herman Bernhard's insertion into the Buffalo
CP local **20 days** after the Jan. 2, 1920 Palmer Raids
(`government/dept-justice/1920/0524-doj-provocateurs.pdf`) — 20/7 = 2.86 ticks, almost exactly one
Desert half-life (3.1 ticks). Independently, Ferdinand Petersen's DoJ tenure (Sept–Nov 1919, "~3
months," same source) gives ~13 ticks — about 4 Desert half-lives, i.e. a low-value Desert-zone
HUMINT asset's shelf life is roughly "4 half-lives of intel staleness," a plausible ratio. Desert's
3.1-week half-life is historically well-anchored and should be **kept**.

### Mud (r = 0.05, half-life 13.5 weeks) — VERDICT: KEEP, strongly corroborated

The Petersen informant tenure above (~3 months ≈ 13 ticks) lands almost exactly on the Mud
half-life (13.51 ticks) — a striking independent cross-check, though the same data point used twice
(once as a Desert-asset shelf-life ceiling, once as a near-exact Mud half-life match) should be
read as one corroborating case, not two. The CPUSA underground-reconstitution window — "~2 months"
(8–9 ticks) to minimal function, "~11 months" (47 ticks) to full parity
(`parties/cpusa/1940/0000-mgolos-johnsoninterview.pdf`, `parties/cpusa/1921/0102-ucp-conventionminutes.pdf`)
— brackets 13.5 ticks comfortably between its floor and ceiling. **Keep** 0.05/tick.

### Water (r = 0.01, half-life 69 weeks) — VERDICT: KEEP, weakly anchored

No corpus citation gives a clean direct analog for "how fast does tactical intel go stale in a
fully-trusted base area." The nearest comparanda run *slower*: deep-cover asset tenure (Burns/Iron
Workers 5–6 years ≈ 260–312 ticks; O'Neal/BPP ~5 years ≈ 260 ticks) and the Polish UWP long-run
erosion case (§ below, half-life ≈714 ticks by the geometric method) are both 4×–10× longer than
69 ticks. If anything this suggests 69 weeks *understates* durability in a genuine base area — but
those comparanda measure cover-longevity and demographic drift, not tactical-intel staleness, so
they only bound the constant loosely. **Keep, flagged as the weakest-anchored of the three** —
recommend no change absent playtest data specifically targeting Water-tier freshness.

### The consistency gap: Polish UWP (verified against source)

I re-opened `/media/user/data/old-hdd/old-hdd/www.marxists.org/history/erol/ncm-6/costello-poland.htm`
directly. Line 85 confirms both briefs cite the same real sentence: **"Working-class membership in
the Party fell from a high of 61% in 1949 to 45% in 1955."** Neither brief fabricated or
misattributed the figure — the "gap" is a **method difference**, not a citation error:

- `brief-eh-mass-line.md` §2.3 computes a **linear** rate: 16 percentage points ÷ 6 years = 2.67
  points/year ÷ 52.18 weeks/year ≈ **0.0511 points/week**, reported as "≈0.05%/week."
- `brief-eh-receptivity.md` §2.1 computes a **geometric/proportional** rate: ratio 45/61 = 0.7377,
  `ln(0.7377)/313 weeks ≈ -0.000972` → **0.0972%/tick**, reported as "~0.1%/tick."

Both arithmetic operations are correct on their own terms; they measure different things. Percentage
*points* lost per week (arithmetic slope against the original 1949 base) is not the same operation
as a proportional decay constant compounding against the *current*, shrinking value — and the
spec's own decay formula (`I_c(t+1) = I_c(t) × (1 - r)`) is explicitly the latter. **The
mathematically consistent derivation for this spec is `brief-eh-receptivity`'s ≈0.0972%/tick
(≈0.001 as a decay_rate), not `brief-eh-mass-line`'s linear 0.05%/week.** The ~2× gap between them
is exactly the gap you'd expect between a linear-slope approximation and a geometric rate over a
26%-total decline — not a sign either brief misread the source.

A second honesty note: this Polish figure measures **party membership class-composition drift**
(a demographic/political trend), not Intel Confidence staleness or even Mass Receptivity itself. Its
real value here is as an order-of-magnitude floor: even measured the "fast" (geometric) way, its
half-life is `ln(0.5)/0.000972 ≈ 713 ticks (~13.7 years)` — an order of magnitude slower than even
Water's 69-tick half-life. This confirms that all three existing decay constants are, correctly,
"fast" tactical-freshness rates relative to slow structural/ideological drift; it does not by itself
argue for changing any of the three.

### The SPLIT question: does Kronstadt require a fourth constant?

Yes. Kronstadt is the strongest single case in the corpus, and it **cannot be produced by any of
the three existing decay constants operating continuously**. The garrison's own bulletin: "finding
the party ... completely bureaucratized and absolutely separated from the masses, we are leaving
its ranks" (`history/ussr/events/kronstadt/izvestia/08.htm`, verified — the file lists 130+ named
departures and explicitly uses this "separation from the masses" framing). Collapse from
institutional loyalty to open armed rupture ran **9–13 days ≈ 1.3–1.9 ticks**
(`izvestia/01.htm`, `/08.htm`, `/12.htm`). Even Desert's fastest rate (0.20/tick, half-life 3.1
ticks) cannot halve trust in under 2 ticks, let alone collapse it to near-zero. Solving
`(1-k)² ≈ 0.05` (5% remaining after 2 ticks) gives `k = 1 - √0.05 ≈ 0.776` — `brief-eh-receptivity`
rounds this to a recommended 0.70–0.75/tick, a reasonable conservative rounding of the exact 0.776
solve. **This is not a stretch of the Desert constant; it is a qualitatively different, discrete,
event-triggered process** (a scripted "Dissociation"/rupture trigger — leadership betrayal, felt
"separation from the masses" — not ordinary repression intensity, per `brief-eh-receptivity`'s own
Tension #1). The spec's current three-state structure has no slot for this: it only expresses
smooth per-tick decay conditioned on the *current* vision-state, not a discontinuous collapse *of*
the state itself.

### RECOMMENDATION (EH-5)

1. **KEEP** `decay_rate = 0.20` (Desert), `0.05` (Mud), `0.01` (Water) unchanged — all three
   survive re-derivation against the weekly tick; Desert and Mud are strongly corroborated, Water
   is directionally consistent but weakly anchored (flag for playtest, not for revision).
2. **ADOPT** `≈0.001` (0.1%/tick) as the correct geometric reading of the Polish UWP case if/when a
   long-horizon "passive structural erosion absent any rupture" constant is added — supersede
   `brief-eh-mass-line`'s linear 0.05%/week figure, which is a valid computation of a different
   quantity (points/week, not a multiplicative rate) and understates the proportional rate by ~2×.
3. **SPLIT — add a fourth constant**, `RUPTURE_DECAY_RATE ≈ 0.75/tick`, applied only for 1–2 ticks
   immediately following a scripted rupture-trigger event (leadership betrayal / felt
   "separation from the masses" — Kronstadt-class), never as a response to ordinary repression
   intensity. This is additive to, not a replacement for, the three existing per-vision-state
   constants — the three-state Desert/Mud/Water structure correctly captures *ambient* staleness;
   it structurally cannot and should not be stretched to also cover discontinuous collapse.

---

## RULING EH-3: Scope of M_r — per-territory scalar vs per-(org, territory)

### What the corpus actually shows

Searching `/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/parties/cpusa/anti-trotsky/`
and `.../parties/lovestone/` turned up mostly national-level factional-dispute documents (Comintern
open letters, Gitlow's ILD polemics, "On the Road to Bolshevization") rather than shop-floor
trust comparisons. The single directly on-point piece is James P. Cannon's **"Furriers' Unity"**
(*The Militant*, Oct. 17 1931, `archive/cannon/works/1931/oct/furriers.htm`, read in full) — Cannon
was the expelled CP leader who founded American Trotskyism (Communist League of America) in 1928.

The article describes the **NYC needle-trades/furriers' union**, where the CP-led "Industrial
Union" (Left wing/dual union) had been driven out of the AFL-affiliated union by "police clubs and
economic pressure," while an AFL Joint Council bloc — with, in Cannon's words, "the Lovestoneites
acting in their now fully established role of butlers for them" — regained institutional control
and then proposed a "unity" merger. Cannon's own assessment of the resulting rank-and-file trust
split, quoted verbatim (39 words): *"even those furriers who have been driven back into the A.F. of
L. union by police clubs and economic pressure have not forgotten the traitors and do not trust
them... the soul of the furriers belongs to the Left wing."*

This is genuine evidence of **differential rank-and-file trust for the identical workforce,
independent of which organization held institutional/physical control** — exactly the mechanism
EH-3 asks about. But it must be characterized honestly: this is a **two-pole**, not a three-pole,
comparison. The poles are (a) the CP-led Industrial Union and (b) an AFL-right-wing bloc with the
Lovestoneites as its *subordinate ally* ("butlers"), not an independently organizing third force
with its own separately-measurable trust footprint. Cannon/the Trotskyists appear here as external
critics of CP strategy, not as a third pole competing for the same rank-and-file trust at this
specific 1931 juncture — the Communist League of America had no comparable organized base in the
NYC furriers' shops to measure against. A related but weaker data point: `progressives.htm` (Apr.
1931, same archive) references a Lovestoneite "deal with Levy (read: Sigman) in the I.L.G.W.U." —
again factional maneuvering at the leadership level, not a rank-and-file trust measurement.

So: **the corpus does not contain the clean three-way CPUSA-vs-Lovestone-vs-Trotsky comparative
trust case** the ruling's premise describes. It does contain a directly on-point **two-pole** case
proving the underlying mechanism is real and historically attested — organizational control and
rank-and-file trust can and did diverge sharply for the identical population in the identical
locals.

### The design-grounds argument (independent of the historical case)

The spec's own M_r formula (line 160: `M_r = (1 - P(S|A)) × I_a × C_f`) is **already internally
org-dependent in one of its three factors**. The `ideological_alignment` component (line 182-193)
is explicitly defined as derived from "Average class_consciousness in territory, **Organization's
reputation in territory**, Historical actions (did you help or harm them?)" — this is a per-org
quantity by the spec's own words; the furriers case (CP retains trust, AFL/Lovestone-bloc does not,
identical population) is exactly what "organization's reputation in territory" is meant to encode
when more than one organization is active in a territory. `P(S|A)` (desperation) and `C_f` (class
factor) are properties of the *territory's population*, not of any organization, and are correctly
org-independent. But if `M_r` is stored as a single per-territory scalar, `I_a` can only be
evaluated for one organization at a time — the formula cannot represent two organizations
simultaneously holding different reputations in the same territory, which is precisely what
historically happened. **This is an internal formula/storage mismatch that exists independent of
whether the historical case is "clean," and it gets worse, not better, once the engine has more
than one active organization** (which the Babylon setting already assumes — rival factions,
fascist organizations, competing revolutionary tendencies).

**State-size tradeoff.** Per-(org, territory) scales as `N_orgs × N_territories` versus
`N_territories` for a scalar. This is a modest multiplicative factor (2–10× for a handful of
active organizations), not combinatorial explosion, and it is *cheaper* than it looks: `P(S|A)` and
`C_f` are shared, territory-level lookups computed once and reused across all organizations: only
the `I_a` term (and its "reputation" sub-lookup) needs an org-specific store. The marginal storage
cost is one scalar (`I_a`, or the resulting `M_r`) per (org, territory) pair, not a full
re-derivation of the formula per org.

### RECOMMENDATION (EH-3)

**Adopt per-(org, territory) scope for `M_r`.** Two independent lines support it:

1. **Historical**, qualified: the Cannon furriers' case (`archive/cannon/works/1931/oct/furriers.htm`)
   is genuine, on-point evidence that rank-and-file trust and institutional control can diverge
   sharply for an identical workforce in identical locals — but it is a two-pole case (CP vs.
   AFL-right-wing-with-Lovestone-junior-partner), not the three-way CPUSA/Lovestone/Trotsky split
   the ruling's premise names. State this distinction explicitly if the case is cited going
   forward — do not round it up to "three factions" in design docs.
2. **Design-grounds**, unqualified: the spec's own `ideological_alignment` factor is already
   specified as org-dependent ("organization's reputation in territory"), which is incoherent under
   a per-territory scalar `M_r` the moment two organizations are simultaneously active — this holds
   regardless of the historical case above, and is the stronger of the two arguments.

Tradeoff to record: implement as `M_r[org][territory]`, but only materialize `I_a` per-(org,
territory) — `P(S|A)` and `C_f` remain per-territory shared lookups, keeping the marginal storage
cost to one scalar per pair rather than a full formula re-evaluation.
