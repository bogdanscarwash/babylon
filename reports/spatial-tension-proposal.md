# Per-County TENSION — Proposal for the Nationwide Choropleth

**To:** the Director · **Motion class:** W-𝔇 (ADR109) · **Status:** proposal, nothing wired
**Constitutional posture:** no new sort, no new primitive, no new constructor family. Everything below is a `BoundOpposition` row + a coupling row + a level placement (`level_name="county"`), i.e. C/G/P over the existing generator. NORTH_STAR §0/§3 "closed for v1.0" is not touched.

### Notation reconciliation (read first)

The Director's directive writes the primitive as `D = (A, Ā, w, T, σ)`. The code writes `D = (A, Ā, w, T, s)` (CONSTITUTION I.19), where `s` is the **sublation predicate** and `σ` was freed in v2.8.0 for the I.2a spectrum coordinate. Throughout this document:

| directive | code | meaning here |
|---|---|---|
| A, Ā | `pole_a`, `pole_b` | the two named material aspects |
| w | `GapReading.balance ∈ [−1,1]` | the **witness** — which aspect leads, and by how much |
| T | `OppositionRegistry.step()` + a named material channel | the **transport** — how the opposition moves between ticks, and along what edge |
| σ | `GapMeasure → GapReading.gap ∈ [0,1]` | the **measure** — the magnitude rendered on the map |

`s` (sublation) is inherited free from `classify_regime` against `spatial_lattice_for_counties()`; no candidate below needs new lattice code (`level_index_for("county") == 1`).

**Shared measure kernel.** All three candidates reuse `calculate_wealth_asymmetry_gap` / `..._balance` verbatim (`src/babylon/formulas/contradiction.py:20-45`) — for any pole pair (a, b):

```
σ = |b − a| / (a + b)      clamped [0,1]    (0.0 when a+b ≤ 1e-9 — honest absence, III.11)
w = (b − a) / (a + b)      clamped [−1,1]
```

Scale-free by construction (any k>0 rescaling cancels), so no numeraire drift into the choropleth. Nothing is reimplemented; nothing accumulates across ticks (Amendment K / VIII.11).

---

## Candidate 1 — `county_extraction` · the Φ-direction opposition

> **A** = *value produced by living labor in this county* · **Ā** = *the wage claim this county commands*

**Grounding.** This is the Fundamental Theorem localized. `value_form.py` already computes `phi_class = (W_c − V_c)/V_c` as the counit defect of the wage-form — the engine's own law of motion. The reader corpus converges on the *direction of flow*, not the level of misery, as the assigner of tension:

- Marx, Capital I ch25:4240-4242 — "Ireland is at present only an agricultural district of England, marked off by a wide channel from the country to which it yields corn, wool, cattle, industrial and military recruits"; and the reversal at :4488-4492 — "in England, an industrial country, the industrial reserve recruits itself from the country districts, whilst in Ireland, an agricultural country, the agricultural reserve recruits itself from the towns."
- Lenin, `imp-hsc/pref02.htm:289-315` — "out of such enormous superprofits… it is possible to bribe the labour leaders and the upper stratum of the labour aristocracy… This stratum of workers-turned-bourgeois… is the principal social… prop of the bourgeoisie."
- Lenin, `imp-hsc/ch03.htm:441-446` — finance capital "levies tribute upon the whole of society for the benefit of monopolists."
- MIM Theory #10:2571-2574 (Lenin, quoted) — "the imperialist super-profits of the advanced countries enabled and enable them to bribe the upper strata of the proletariat, to throw them crumbs of these super-profits drawn from the colonies and from the financial exploitation of weak countries."
- MIM Theory #10:5399-5407 — Mexican manufacturing compensation at "15% of the U.S. level, down from 22% in 1980."

**Measure σ.** Let `θ` be the reference wage share — **a ratio of sums, never a mean of county wage shares**:

```
θ = Σ_j v_sum_j / Σ_j (v_sum_j + s_sum_j)
```

Per county i, the two poles are both extensive magnitudes in the same unit:

```
a_i = θ · (v_sum_i + s_sum_i)     # wage entitlement at the reference norm
b_i = v_sum_i                     # wage actually commanded

w_i = (b_i − a_i) / (a_i + b_i)   # witness: >0 net Φ-RECIPIENT, <0 net Φ-SOURCE
σ_i = |w_i|                       # magnitude of the extraction defect

TENSION_i = σ_i · (1 − w_i)/2   ≡  |w_i|(1 − w_i)/2      ∈ [0,1]
```

The `(1−w)/2` factor is the Lenin asymmetry made explicit and auditable: a defect of equal magnitude reads **hot** on the bled pole and **damped** on the bribed pole. It is one line, it is not a free parameter, and it is not a hidden coefficient.

**Witness w.** Signed, and the sign is the whole political content: which side of the wage-form counit defect this county sits on.

**Transport T.** Constitutionally, `registry.step()` — `rate = gap_t − gap_{t−1}`, fresh, never accumulated; principal scoring `score = gap·(1 + 10·|rate|)` inherits free. Materially, the transport channel is the WAGES/TRIBUTE edge set: Φ moves along the wage-form, which is where `_write_edge_tensions` already reads.

**Aggregation discipline.** Vacuously satisfied at county grain — `v_sum`, `s_sum` are extensive sums, and no intensive is ever averaged. The one national quantity, `θ`, is a **ratio of sums** in the `_county_money_ratios` / `_mean_asymmetry` family (`contradiction.py:659-687`, `catalog.py:359-393`). `hex_count` is **not** used and must never be used as a weight here: spatial extent is not a material carrier, and weighting by it would let Nye County, NV swing the national reading as hard as Wayne County, MI — the named intensive-aggregation variance error in its purest form.

---

## Candidate 2 — `county_closure` · the surplus-population / absorptive-margin opposition

> **A** = *population rendered structurally surplus to local capital's self-expansion* · **Ā** = *the remaining absorptive margin (frontier, biocapacity headroom)*

**Grounding.**

- Marx, ch25:1034 — the reserve army "has always three forms, the floating, the latent, the stagnant"; ch25:1084-1099 on the latent form — "the constant flow towards the towns pre-supposes, in the country itself, a constant latent surplus population, the extent of which becomes evident only when its channels of outlet open to exceptional width."
- **Lenin, `dcr8viii/viii8v.htm:271-292` (fn. 4) — the load-bearing passage of this candidate:** "The development of capitalism in depth in the old, long-inhabited territories is retarded because of the colonisation of the outer regions. The solution of the contradictions inherent in, and produced by, capitalism is temporarily postponed because of the fact that capitalism can easily develop in breadth… the possibility (for the peasant) of moving to new territory, mitigates the acuteness of this contradiction and delays its solution."
- Engels, `condition-working-class/ch05.htm:313-334` — "English manufacture must have, at all times save the brief periods of highest prosperity, an unemployed reserve army of workers… In every great town a multitude of such people may be found"; measured at :404-413 by poor-rate as a share of rent and relief rolls as a share of town population.
- MIM #10:8239-8260 — "the extreme poverty in some areas of the countryside is necessary for the limited industrial development of the 'booming' areas… provides the flow of desperate, starving potential workers for the new industries."
- **Binding constraint** — Stalin, `09.htm:823-864, 866-913`: geographical environment "cannot be the chief cause, the determining cause of social development," and the same negative result is repeated verbatim for population density (China vs. USA). Biocapacity and population may **accelerate or retard**; they may never determine.

**Measure σ.** Reference norms, both ratios of sums:

```
w̄ = Σ_j v_sum_j / Σ_j population_j          # wage per head, national
b̄ = Σ_j biocapacity_sum_j / Σ_j population_j # biocapacity per head, national
```

Per county i:

```
E_i = v_sum_i / w̄                            # wage-absorbed population equivalent
P_i = population_i                            # (fallback if population absent: use
                                              #  k_sum-weighted labor demand — see Open Q5)

w_i = (P_i − E_i)/(P_i + E_i)                 # >0 surplus-population DONOR, <0 net ABSORBER
σ_i = |w_i|

κ_i = (P_i · b̄) / (P_i · b̄ + biocapacity_sum_i)   ∈ [0,1]   # frontier CLOSURE
                                              # κ→1 : valve shut, no headroom
                                              # κ→0 : frontier open, contradiction deferred

TENSION_i = σ_i · (1 − ν·(1 − κ_i))           ν ∈ [0,1], a GameDefines coefficient
```

`κ` is deliberately built in the same asymmetry shape (metabolic demand vs. biocapacity supply, both extensive), and it enters as a **multiplier bounded away from zero by ν**, never as an additive competitor and never as a veto. This is the Stalin ruling encoded: `ν < 1` guarantees geography can damp but cannot extinguish the class term. MIM #10:8920-8934 (LaDuke; per-capita energy consumption vs. dam projects, testing, clear-cutting on Indigenous land) says the same thing from the other side — ecological deficit is *correlated with* the extraction axis, so it belongs coupled to it, not summed beside it.

**Transport T.** `step()` for the rate; materially, the transport channel is migration/dispossession — the DISPOSSESSION and ReserveArmy channels. Marx's England/Ireland reversal (:4488-4492) is precisely a claim about the *sign of this transport*, which `w_i` carries.

**Aggregation discipline.** `w̄`, `b̄` are ratios of sums. `E_i`, `P_i`, `biocapacity_sum_i` are extensive and summed, never averaged. Again `hex_count` is excluded as a weight.

---

## Candidate 3 — `county_seam` · the town/country (uneven-development) opposition

> **A** = *this county's mode of production (composition of capital)* · **Ā** = *the mode prevailing in its adjacent counties*

**Grounding.**

- **Mao, On Contradiction §III/IV synthesis (:1017-1019) — the doctrinal anchor:** "The distinctive character or particularity of these two facets of contradiction represents the unevenness of the forces that are in contradiction. Nothing in this world develops absolutely evenly; we must oppose the theory of even development or the theory of equilibrium."
- Mao, §VI (:1349-1356) — "Economically, the contradiction between town and country is an extremely antagonistic one both in capitalist society, where under the rule of the bourgeoisie the towns ruthlessly plunder the countryside…"
- Mao, `mswv1_12.htm:693-703` — "a few modern industrial and commercial cities coexist with a vast stagnant countryside" — his own paradigm case is a *pair of adjacent, differently-developed places*, not a place.
- Lenin, `dcr8iii/iii8ii.htm:130-134` — mixed modes on the same ground: "the combination of such dissimilar and even opposite systems of economy leads in practice to a whole number of most profound and complicated conflicts and contradictions."
- Lenin, `imp-hsc/pref02.htm:103-104` — "The uneven distribution of the railways, their uneven development—sums up, as it were, modern monopolist capitalism on a world-wide scale."
- Engels, `housing-question/ch02.htm:443-462` — the antithesis of town and country "has been brought to an extreme point by present-day capitalist society. Far from being able to abolish this antithesis, capitalist society on the contrary is compelled to intensify it day by day."
- **Stalin, `09.htm:107-115` — the methodological constraint this candidate uniquely satisfies:** "no phenomenon in nature can be understood if taken by itself, isolated from surrounding phenomena." A tension scalar computed from a county's own ledger row alone is, by this criterion, not dialectical.
- MIM #10:5399-5407, 5342-5345 — the steepest gradient sits at a seam (the border), not diffusely.

**Measure σ.** County composition (a claim-to-living-labor ratio):

```
q_i = (c_sum_i + k_sum_i) / (v_sum_i + s_sum_i)
```

Neighborhood composition over the county ADJACENCY relation N(i), **weighted by the extensive carrier that makes q meaningful — living labor (v+s), never a bare mean of neighbor ratios**:

```
q_N(i) = Σ_{j∈N(i)} (v_sum_j + s_sum_j)·q_j  /  Σ_{j∈N(i)} (v_sum_j + s_sum_j)
       ≡ Σ_{j∈N(i)} (c_sum_j + k_sum_j)  /  Σ_{j∈N(i)} (v_sum_j + s_sum_j)     # telescopes to a ratio of sums

w_i = (q_i − q_N(i)) / (q_i + q_N(i))      # >0 the TOWN pole, <0 the COUNTRY pole
σ_i = |w_i|

TENSION_i = σ_i
```

The telescoping identity is the same one `_mean_asymmetry` documents (`catalog.py:359-393`) and is the reason this form is safe: it *is* a ratio of sums, so a thin neighbor cannot swing the seam reading.

Note that **both** sides of a seam read hot. That is materially correct and is Mao's own construction — the antithesis is a property of the pair, not of the town.

**Transport T.** `step()` for the rate; materially, the transport is the ADJACENCY relation itself — this is the only candidate whose T is genuinely *spatial*, and the only one satisfying Stalin's interconnection criterion natively.

**Computability caveat (declared, not hidden).** The named ledger columns do **not** include an adjacency relation. `NodeType.HEX` is declared vocabulary only and hex is never a graph node (Amendment U, `CONSTITUTION.md:583`), so county adjacency must come from the county-grain ADJACENCY edges or from a hex-rolled adjacency built through `ScaleAdjunction` (a W-G motion, `ai/wiring-doctrine.md:58-63`) — never by stamping the finer sort onto the engine graph. This is a real prerequisite, and it is why Candidate 3 cannot ship first.

---

## Recommendation

**Ship Candidate 1 (`county_extraction`) as the principal aspect of spatial tension. Register Candidates 2 and 3 `shadow=True` as its secondary aspects.**

Mao, §IV (:853-858): "if in any process there are a number of contradictions, one of them must be the principal contradiction playing the leading and decisive role, while the rest occupy a secondary and subordinate position." The choropleth is exactly such a forced ranking — one scalar, one color ramp. The dialectical argument for Φ-direction as principal:

1. **Stalin's negative result eliminates Candidate 2 from principal status outright.** Its determining term is population against biocapacity, and `09.htm:823-913` rules twice — once for geographical environment, once for population density — that these "cannot be the chief cause, the determining cause of social development"; the determining term is "the mode of production of material values," i.e. the relations of production. Candidate 2 is a genuine, textually-mandated *condition*: an accelerant/retardant, encoded as the bounded multiplier `(1 − ν(1−κ))`. It is not a rival for the lead.

2. **Candidate 3 is the more dialectical form but the less determinate content.** It satisfies Stalin's interconnection criterion better than Candidate 1 and it is Mao's own paradigm case. But composition-dissimilarity is a *difference*, not a *direction*: two adjacent counties of divergent organic composition with no surplus flowing between them light up a seam that is materially inert. Marx's Ireland argument (:4240-4242, :4488-4492) is not that Ireland *differs* from England — it is that Ireland *yields to* England and that its reserve army runs the wrong way. Direction is the content; difference is only its shadow. Candidate 1 carries the direction natively in `w`; Candidate 3 does not carry it at all.

3. **Candidate 1 is the opposition the engine already adjudicates.** `phi_class = (W_c − V_c)/V_c` is the counit defect of the wage-form in `value_form.py`; `compute_fundamental_theorem()` is already stashed by `ContradictionSystem`. A choropleth that renders a *different* quantity than the one the rupture condition tests would lie to the player at exactly the moment the player needs the truth. Mao's own methodological injunction in *Analysis of the Classes* (:33-41) — "Who are our enemies? Who are our friends? … we must make a general analysis of the economic [status of the various classes]" — is a demand that the map answer the question the engine will act on.

4. **Mao's unevenness doctrine is honored by the recommendation, not violated by it.** "Nothing in this world develops absolutely evenly; we must oppose the theory of even development or the theory of equilibrium" (:1017-1019) forbids a *flat* national reading — and Candidate 1's `θ` is precisely a national reference against which every county's deviation is measured. The map is a map of deviation from the norm; equilibrium is what it renders as zero. Unevenness is the figure, not the ground.

5. **Promotion path.** Per the ADR077 ladder: all three land `shadow=True` (measured every tick, excluded from principal scoring, routed to `shadow_opposition_states`), proving byte-inertness against `qa:regression` before any promotion ceremony. The composed target form, when promoted, is:

```
TENSION = gap( D_extraction ⊕ D_seam )   with κ modulating D_extraction's measure
        = σ_φ + σ_seam − σ_φ·σ_seam
```

`⊕` (`composition.py`, `gap(⊕) ≥ max`) is correct for the two *class-relational* oppositions — either extraction or unevenness suffices to make a place antagonistic. `⊗` (`gap(⊗) ≤ min`) is **wrong** for closure, because it would let an open frontier veto real extraction — geography determining by negation, which is the Stalin ruling violated through the back door. Hence closure stays a bounded multiplier inside the measure, never a composed conjunct. Coupling rows (`catalog.py` `_DEFAULT_COUPLINGS`) at promotion: `county_extraction --feeds--> imperial`, `county_seam --constrains--> county_extraction` — each citing the code that makes the edge true, since a coupling is a claim about the code, not about the theory.

---

## What Candidate 1 predicts on the US map

`TENSION_i = |w_i|(1−w_i)/2` runs **hot** where `v_sum` is thin against locally-produced new value `(v_sum + s_sum)` — high local rate of surplus value relative to `θ` — and runs **cold** where the wage claim exceeds what local living labor produces.

**Hot (net Φ-source):** Gulf Coast petrochemical and refining counties; Permian Basin extraction counties; Appalachian coal counties; Central Valley agricultural counties (Fresno, Kern, Tulare); the Mississippi Delta; Rio Grande Valley border counties; warehouse/logistics counties in the Inland Empire and the I-81 corridor; reservation counties (Oglala Lakota, Apache). Enormous surplus produced against a thin wage bill; the surplus is realized and consumed elsewhere.

**Cold (net Φ-recipient):** the District of Columbia and the Northern Virginia ring; Manhattan and Westchester; Santa Clara and San Mateo; Suffolk/Middlesex MA; the affluent suburban rings generally. The wage bill — professional and managerial salaries, state redistribution, finance — vastly exceeds locally produced new value.

**This is the materially correct answer, and it is deliberately not a poverty map.** Lenin, `pref02.htm:289-315`: the stratum living on the crumbs is "the principal social… prop of the bourgeoisie… the real agents of the bourgeoisie in the working-class movement. In the civil war between the proletariat and the bourgeoisie they inevitably… take the side of the bourgeoisie." A tension map that lit up affluent metros because they contain many people, or lit up poor metros because they contain poor people, would be reproducing exactly the bourgeois-sociological reading MLM-TW exists to refute. Candidate 1 lights up the *donors of surplus*, and it deliberately damps the *recipients* — that is the labor-aristocracy geography rendered honestly.

The Ireland-analogue counties (Appalachia, the Delta, the Rio Grande Valley, reservation counties) run hot on Candidate 1 *and* would run hot on MIM's national-oppression overlay. That coincidence is a prediction the map makes and a correspondence the Director can check. It is also the source of Open Question 2 below, because Candidate 1 **does not carry** the overlay: MIM #10:3156-3159 explicitly rejects the compact-territory thesis ("MIM holds that there is no one compact territory of the Black nation right now, but Blacks are a nation"), so the overlay is non-contiguous and is **not derivable from c/v/s**.

**Cross-check against Candidate 3, if promoted:** the seam term would additionally light the Appalachian coalfield/Piedmont boundary, the Central Valley/Bay Area edge, and the Delta/Memphis edge — Mao's "modern industrial and commercial cities coexist with a vast stagnant countryside" rendered as a literal border in the choropleth. **Cross-check against Candidate 2:** `κ` would sharpen the arid West (Permian, Central Valley groundwater, Colorado River basin) and damp counties with genuine biocapacity headroom — Lenin's safety valve, visible as the map's coolest extractive counties.

---

## Open questions reserved for the Director's line

These are ideological-line questions (Amendment AD / IX.5), not engineering choices. I have not resolved any of them.

1. **The reference frame of `θ`.** Computed over US counties, `θ` renders the *intra-core distribution of imperial rent* — how the tribute is shared downward inside the imperial core. Computed against a world wage share, nearly the entire US map goes cold, because per MIM the US is a net Φ-recipient in its entirety. Both are defensible; they are different claims about what a *nationwide* map of a *core* country is for. **This is the single highest-stakes ruling in the document.**

2. **The national-oppression overlay.** MIM #10:3156-3159 makes it non-contiguous and non-derivable from economic magnitudes; MIM #10:5375-5382 (the $6,928 Black/white median wage gap) makes it quantitatively real. Does the choropleth carry it as a second registered opposition, as a multiplier on `county_extraction`, or not at all? A map that omits it is theoretically incomplete; a map that derives it from c/v/s alone is theoretically false.

3. **Rendering the damping.** Should the bought-off metros render as *cold* (one ramp, `TENSION ∈ [0,1]`, the uncomfortable and honest answer), or should `w` get its own diverging channel (recipient/source, gold↔crimson) so the player sees the *direction* and not only the magnitude? The one-ramp version risks being misread as a poverty map; the diverging version puts the labor-aristocracy thesis on screen unmissably.

4. **Magnitude versus distance-to-the-nodal-point.** Mao, §VI:1293-1300: "Before it explodes, a bomb is a single entity in which opposites coexist in given conditions. The explosion takes place only when a new condition, ignition, is present"; and Lenin quoted at :1358-1359: "Antagonism and contradiction are not at all one and the same." Stalin, `09.htm:156-169, 244-253`, demands nodal-point structure. `_maybe_rupture` already fires only on `gap > threshold AND rate > 0` — level *and* motion. Should the choropleth render the continuous gap, or the county's *proximity to its own nodal point* (which would foreground the counties about to cross rather than the counties already deep)? These produce visibly different maps.

5. **Absence of `population`.** If `population` is not a shipped column, Candidate 2's `E_i`/`P_i` pair has no honest form and must read `GapReading(0.0, 0.0)` per III.11 rather than be proxied from `k_sum`. Confirm whether a proxy is sanctioned or whether Candidate 2 waits for the column.

6. **Whether biocapacity may ever be determining.** Stalin's ruling (`09.htm:823-864`) is unambiguous and I have encoded it as a bounded multiplier with `ν < 1`. If the Director's line on the metabolic lane differs — if ecological limit is to be treated as a determining term in the American case specifically — that is an amendment-scale divergence from the cited text and needs to be stated as one, not slipped in as a coefficient.

7. **The `(1 − w)/2` damping factor.** It is the one place where I have written a political claim (Lenin's asymmetry: bribery damps, extraction sharpens) directly into arithmetic rather than into a `GameDefines` coefficient. That was deliberate — it is line, not tuning, and should not be moddable — but it is the Director's call whether it stays hard-coded, becomes a define, or is replaced by the diverging-channel rendering of Open Question 3.
