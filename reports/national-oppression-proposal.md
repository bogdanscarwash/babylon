# The National Question — Proposal for the Domestic Oppression Axis

**To:** the Director · **Motion class:** W-P projection (Phases 0–1), W-𝔇 opposition (Phase 2, conditionally blocked)
**Status:** proposal, nothing wired · **Revision 2**, after three adversarial reviews (fidelity / constitutional-engineering / data-honesty). Every finding is disposed of in Appendix B.
**Filed against:** `origin/dev` @ `8da23398` (PR #327 — ADR170 + the tension/fog lens producers, **merged**), M5 Maps contract `docs/superpowers/specs/2026-07-28-m5-maps-contracts.md` as amended by that commit.
**Constitutional posture:** no new sort, no new primitive, no new constructor family. **No amendment is required to build anything recommended.** The contingent Amendment AE offered in Revision 1 is **withdrawn** (Appendix A) — it amended a restriction that does not exist.

---

## §0 — Provenance, notation, and what is declared absent

**Provenance repair (the review's most serious procedural finding).** Revision 1 cited MIM Theory #10 ~15 times as `full.txt:NNNN`. **No file named `full.txt` exists at any path on either data drive.** The line numbers are nonetheless exactly reproducible; they were produced by:

```
pdftotext /media/user/data/mim/mt10.pdf /tmp/mt10.txt     # NO -layout; 9,671 lines
```

Every MIM citation below is re-keyed to **`mt10.txt:NNNN`** under that declared extraction and was re-verified line-by-line in this pass. Archive root for all other citations: `/media/user/data/old-hdd/old-hdd/www.marxists.org`. ERoL paths are under `history/erol/…` (Revision 1 wrote `archive/erol/…`; corrected throughout).

**Tree provenance repair.** Revision 1 leaned on ADR170 and `reports/spatial-tension-proposal.md` while both sat unmerged. Both are now on `origin/dev` (`043a2c4c` PR #326; `8da23398` PR #327). **But the substance of the criticism stands and is honored:** ADR170's own consequence line reads *"the national-oppression overlay seed (**Director-gated, needs a real reference dataset**)"* — the ADR charters this question **open**, and no citation of it may treat the dataset question as closed. It is cited below as precedent for the **projection/engine split only**.

**Notation** follows `reports/spatial-tension-proposal.md`: `A/Ā` = `pole_a`/`pole_b`; `w` = `GapReading.balance ∈ [−1,1]`; `T` = transport; `σ` = `GapMeasure → GapReading.gap ∈ [0,1]`; `s` inherited from `classify_regime`.

**Declared textual gaps (III.11).** Stalin's *Marxism and the National Question* is absent from the mirror except its table of contents (`archive/stalin/works/1913/03.htm`; `03a.htm` does not exist). The four-criteria definition reaches this document **only through its quoters** — CAP at `history/erol/ncm-8/cap-black-nation.htm:40`, MLP,USA at `history/erol/ncm-8/mlp-bnq.htm:80-97`. Also absent: Stalin's *The National Question and Leninism* (1929); Lenin's *Critical Remarks* — **so the "two nations in every nation" formula is not sourceable here and is used nowhere**; *Self-Determination* chs. 1–2, 5–10.

**Declared modelling gaps (new in Revision 2 — a §0/III.11 failure in Revision 1).** Claim (5) names **police terror** (`mt10.txt:296-300`) and **prison labor** (`mt10.txt:9063-9068`) as constitutive of the reproduction floor. **Neither is modelled by anything in this proposal, and the chosen measure structurally excludes the carceral population** — see §2.1's denominator declaration. This is a named gap, not an oversight.

**Declared data gaps.** No AIANNH↔county bridge exists (`tl_2025_us_aiannh.shp`, 867 records, consumed only by `tools/build_geo_assets.mjs`). No HUD colonias source in `data-catalog.yaml`. `compute_seed_influences.py:190-196` admits its AIANNH constants are "Fixture-grade."

---

## §1 — The Root: what national oppression IS materially

Seven claims, each quote-grounded.

**(1) It is a relation between nations, not an attribute of persons — and the duty it imposes is DOUBLE.** Lenin, `archive/lenin/works/1914/self-det/ch04.htm:136-140`: "the Great Russians in Russia are an **oppressor nation**, and opportunism in the national question will of course find expression among oppressed nations otherwise than among oppressor nations." Revision 1 asserted this rule had "no oppressed-nation mirror." **That was false, and refuted 15 lines from the citation.** `ch04.htm:151-153`: the proletariat "values above all and places foremost the alliance of the proletarians of all nations, and **assesses any national demand, any national separation, from the angle of the workers' class struggle**." And `:174-178`: "**Insofar as** the bourgeoisie of the oppressed nation fights the oppressor, we are always, in every case… in favour… **But insofar as** the bourgeoisie of the oppressed nation stands for its own bourgeois nationalism, we stand against."

**Mechanical consequence, which the recommendation must carry:** a model with only a "national oppression" channel can render national mobilization *only* as progressive. That is bourgeois nationalism with the guard removed. **The guard is a hard requirement on Phase 3, not a nicety** (§4, OQ9).

**(2) The asymmetry is constitutive.** Lenin, `archive/lenin/works/1916/mar/25.htm:306-323`: "imperialism is precisely the epoch in which **the division of nations into oppressors and oppressed is the essential and typical division**." The sincerity test is likewise asymmetric (`:296-301`). **The measure must be signed and its sign must be politically meaningful.**

**(3) The question is POWER, not culture.** Stalin, `1921/05/02.htm`: Springer and Bauer "converted the right to self-determination into the right of the oppressed nations of Europe to **cultural** autonomy… while all *political* (and economic) power was to *remain* in the hands of the dominant nation… and the question of secession was excluded." His positive programme is territorial (`1917/03/25.htm`). **This is real textual pressure toward territory-attachment; §2.0 explains what the code can and cannot settle about it.**

**(4) Its incidence is a historical-political fact and is NOT derivable from economic magnitudes.** `mt10.txt:3158-3165`: "**MIM holds that there is no one compact territory of the Black nation right now, but Blacks are a nation and should not be addressed as a 'race' within U.S. borders.**" Gallegos, `history/erol/ncm-7/ncm-chicanos.htm:48`: "the starting point… was not a somewhat obscure text from the Soviet Union, but **the historical fact of annexation**." Puerto Rico: `history/erol/ncm-1a/pr-puerto-rico-1.htm:38`, "Puerto Rico is a colony of the U.S. since 1898."

Note precisely what this forbids: **deriving the partition**. It does not forbid measuring the differential the partition sustains — MIM does exactly that with BLS numbers. **The elegant principle: the partition is historical and enters as declared data; the oppression is a measured asymmetry across it.**

**The review's severest finding lands here and is accepted.** MIM's *second clause* — "should not be addressed as a 'race'" — prohibits the operationalization Revision 1 proposed. `T − H` is a **Census racial residual**, not a historically-instituted partition; conquest, enslavement and annexation appear nowhere in it. Revision 1 quoted the prohibition on the same line it violated, then treated the problem as disclosure. **It is not a disclosure problem. It is the ruling in OQ1, and it is now blocking**: no lens ships under a partition the Director has not ruled. What §2.1 publishes below is therefore explicitly labelled a **measurement instrument exercised under a stated, unsanctioned partition** — evidence that the arithmetic works, not a claim that the partition is right.

**(5) The material content is an extensive MASS — and MIM measures it in the OPPRESSOR's direction.** Revision 1 got the referent and the direction backwards. MIM's $52.8bn is the Black/white median *wage* gap × Black workers, framed as capitalist saving: "the capitalists saved $52,812,144,000… which is 21.2% of the $249.1 billion the capitalists claimed in profits" (`mt10.txt:5375-5382`). The load-bearing figure is the adjacent one: **using the Black worker's wage as the demonstrated reproduction floor**, "If the white workers had been paid $21,750 each instead of $28,678… the white workers would have been paid a total of **$414.1 billion less**… **$414.1 billion in the pockets of the labor aristocracy, courtesy of the capitalists who want their allegiance**" (`:5389-5400`). The subsidy sum is `:5454-5469` ("**$324.6 billion in subsidies for being white**… This is still more than the capitalists kept for themselves"), with MIM's own honesty flag at `:5470-5479` ("This is not a final measure of the non-exploitation of the white working class… we would like to calculate reparations").

**MIM measures the oppressor's non-exploitation against the oppressed's demonstrated floor.** A measure of the oppressed's deprivation against the oppressor's norm is the *dual* quantity and is not the MLM-TW thesis. §2.1 now ships **both directions**, computed from the same rows at zero extra data cost.

**(6) The FORM is time-indexed and mutable — and the territory question is LIVE, not settled.** MLP,USA, `mlp-bnq.htm:151-153`: "**The historic trend**… is presently against the crystallization of a black nation"; `:133-138`: "by the 1960's one can certainly no longer speak of a black nation in the deep South"; `:154-159`: "there is no longer a basis for raising the slogan of… the right of the people in a definite territory to secede. **At the same time, the black people remain an oppressed nationality**." And `:128-131`: "The black people were **forced off the land in the rural South and were driven into ghettoes in the major urban centers** throughout the country"; `:141`: "most have **migrated out of the former territory** of the black nation to the big cities."

**Against which stands a position Revision 1 suppressed and this revision restores.** CAP, `cap-black-nation.htm:41`, defends exactly what MLP and MIM deny: the *"Black Belt being the national territory of the Black Nation… and Blacks elsewhere being a national minority"* is "not only made by leading American communists, but **in 1928 and 1930 the Communist International passed resolutions firmly stating the definite existence of the Black Nation**." CAP names `:42` "**common territory and common economic life**" as precisely the two grounds under attack, and `:47` insists the northern ghetto is "almost exact duplication of the conditions inside of the Black Nation." Haywood's territorial claim stands at `history/erol/ncm-8/rcp-haywood.htm:76` ("a stable community of over five million"), the RCP rebuttal at `:77`.

**Taking MIM+MLP over CAP+Haywood+Comintern is the ideological line under Amendment AD §IX.5.** Revision 1 made that choice silently and omitted it from the escalation list. It is now **OQ0 — the first question, and the one that governs all others.**

**(7) Point-in-time poverty is a contested proxy, and MIM says so on the page.** `mt10.txt:5544-5550`: "there is a lot of movement in and out of poverty. **Only half of all poverty by this definition lasts four months or more**, while 13% lasts two years or more. Whites make up 70% of all those in poverty, but **only 56% of those in long-term poverty**… Still, in an average month Blacks are three-times more likely to be poor." MIM raises the statistic to **discount durationless counts**, and B17001 is exactly that statistic.

**Direction of the resulting bias, stated so the Director can price it.** Because settler poverty is disproportionately transient, a point-in-time `p̄` **overstates** the settler nation's durable reproduction failure, so the counterfactual `a_i = u_o·p̄` is too high and the measured deficit is **too low**. The bias is *conservative* — it understates the differential. That is the acceptable direction for a shipped claim, but it must be printed on the lens, not buried.

**The root, in one sentence.** National oppression is an *asymmetric, historically-instituted relation between an oppressor nation and an oppressed nation, whose incidence is declared from history rather than derived from value magnitudes, whose content is a measurable extensive quantity best read as the oppressor's non-exploitation against the oppressed's demonstrated reproduction floor, whose political stake is state power and secession, whose form is a time-indexed material claim that can be won or lost — and whose partisan defence carries a double duty: against the oppressor nation's privileges, and against the oppressed nation's own bourgeois nationalism.*

---

## §2 — The Formal Candidates

### §2.0 Structure: what the code settles, and what it does not

`ScaleAdjunction` (`instances/scale.py:56-177`) validates `mapping` as a **total function child → parent** with per-parent shares summing to 1 (verified this pass). A nation over counties is not that shape in either direction. **Therefore the national structure cannot be a lattice rung.** It is a *span*: an incidence matrix `N[county][pole]` with two projections, neither a partition of the other.

**Correction to Revision 1's over-claim.** That validator proves a **type fact** about lattice membership. It proves **nothing** about whether territory is constitutive of nationhood — a live corpus dispute (claim 6). Revision 1 wrote "**Ruling implied by the code plus the corpus: people-attached, county-measured**" and treated a reserved line question as settled by a Pydantic validator. **That sentence is withdrawn.** The correct statement is:

> *The national structure is not a lattice rung (code fact, settled). Whether the oppressed nation is people-attached or territory-attached is a line ruling (OQ0, escalated). The span form can carry either answer — under a territorial ruling the incidence matrix is dominated by a declared county set; under a people-attached ruling it is a population weighting. **The engineering does not force the ruling.***

**Consequences that survive the correction:** the measure aggregates by ratio of sums (`hex_count` structurally unusable as a weight); Constitution I.20 is satisfied without effort (an overlay, never a substrate mutation); `level_name` is `"community"` (`LEVEL_INDEX["community"]=1`, `instances/levels.py:113-122`) or unplaced `""`, never `"nation"` (a homonym collision with the *spatial* top rung).

**Consequence that does NOT survive: the `PoleBinding.community_id` showpiece is withdrawn.** `community_id: str` (`core/opposition.py:185-188`) is **one** XGI hyperedge id. Binding pole B to three communities needs either a multi-community field — **primitive invention, amendment-scale** — or a collapsed label string, which is the VIII.9 reduction the field exists to forbid. Worse, and decisively: the community hypergraph is **empty in production**. `sentinels/seam/registry.py:2170-2192` classifies `community_memberships` as `LivenessClass.STRUCTURALLY_IMPOSSIBLE` — *"no scenario builder assigns `SocialClass.community_memberships` anywhere in production… no runtime condition can light it — only a code/data change."* A pole bound to `CommunityType.NEW_AFRIKAN` today binds to a hyperedge that never exists.

**This is the single most useful thing the review surfaced, because it names the prerequisite.** See §3 step 4 and §4 Phase 2.

### §2.1 Candidate N1 — `national_oppression` · the reproduction-floor differential (**recommended, with the partition ruling as a hard gate**)

> **A** = *the terms of social reproduction the settler nation commands* · **Ā** = *the terms the oppressed nations are held to*

**Grounding.** Claims (1), (2), (5), (7).

**Three channels, not one.** Let `u_o(i), b_o(i)` be the oppressed-nation universe and below-floor counts in county *i*; `u_s(i), b_s(i)` the settler ones; `U(i)` the **ACS poverty universe** (see the denominator declaration below).

```
p̄ = Σ b_s / Σ u_s                         # oppressor-nation reproduction-failure norm
q̄ = Σ b_o / Σ u_o                         # oppressed-nation DEMONSTRATED reproduction floor (claim 5)

w_i = (b_o − u_o·p̄)/(b_o + u_o·p̄) ∈[−1,1]  # witness: signed side of the relation
σ_i = |w_i| · damp(u_o)                    # damped — see the small-count law
E_i = b_o(i) − u_o(i)·p̄                    # DEPRIVATION mass (persons)   — the dual quantity
Ω_i = u_s(i)·q̄ − b_s(i)                    # BRIBE mass (persons)          — MIM's own direction
Λ_i = E_i / U(i)   ·   Ω̂_i = Ω_i / U(i)    # per-capita intensities of each mass
```

`w` reuses `calculate_wealth_asymmetry_gap`/`_balance` verbatim (`formulas/contradiction.py:20-45`). **`Ω` is new in Revision 2 and answers the fidelity review's economism finding**: it uses the oppressed nation's demonstrated rate as the floor and measures the settler nation's excess *above* it — MIM's referent and MIM's direction, from the same rows.

**Data grounding — recomputed in this pass, one universe, one reference.**

`fact_census_poverty` (26,576,550 rows, `data-artifacts.yaml:726-736`) × `dim_race` × `dim_county` × `dim_poverty_category` (`category_id=1` = `B17001_001`, `category_id=2` = `B17001_002`), `time_id=23` (2019).

| quantity | value |
|---|---|
| settler universe `Σu_s` / below `Σb_s` | 192,609,332 / 18,525,178 |
| **`p̄`** (oppressor norm) | **0.096180065** |
| oppressed universe `Σu_o` / below `Σb_o` | 124,083,302 / 23,975,901 |
| **`q̄`** (demonstrated floor) | **0.193224234** |
| **national deprivation mass `ΣE`** | **12,041,561 persons** (Λ = 0.0380229) |
| **national bribe mass `ΣΩ`** | **18,691,613 persons** (Ω̂ = 0.0590213) |
| ratio bribe : deprivation | **1.552** |

The ratio is structurally the relation MIM reports in wage terms — the premium exceeds the deficit ("$414.1 billion less… **That's even bigger bucks**", `mt10.txt:5395`). That the same ordering falls out of a headcount measure on independent data is the strongest evidence in this document that the instrument is measuring MIM's quantity.

**Universe and absence — corrected from "1 honest absence" (understated ~14×).**

- Engine territory artifact `us_county_territories.json` = **3,153** counties; **3,139** carry 2019 rows; **14 absent**.
- `scopes.py:_load_national_fips` (`WHERE substr(fips,1,2)<'60' AND substr(fips,3,3)!='999'`) = **3,156**; **3,140** readable; **16 absent**. The two universes differ by three retired FIPS the resolver admits and the artifact does not (`02261` Valdez-Cordova, `02270` Wade Hampton, `46113` Shannon) — **a vintage-drift trap in its own right; `02261` still carries 2019 rows.**
- `p̄` = 0.096180065 on the 3,140; 0.096181349 on the 3,139. The lens must **declare which universe it renders**; recommendation is the artifact's 3,153, and the 1.3×10⁻⁶ difference is disclosed rather than hidden.

**The 14 absences, Pine Ridge named first, because the absence is CORRELATED with the measured thing:**

| FIPS | county | why absent |
|---|---|---|
| **46102** | **Oglala Lakota County SD (Pine Ridge)** | **zero rows at every `time_id` in the table.** Predecessor `46113` (Shannon) carries rows only 2010–2014 |
| 02063 / 02066 / 02158 | Chugach, Copper River, **Kusilvak** AK | post-2019 Alaska reorganizations; Kusilvak is ~92% Alaska Native |
| 09110…09190 (9) | CT planning regions | 2022 county-equivalent replacement; `09001…09015` have 2019 rows |
| 51515 | Bedford city VA | reverted to town 2013 |

**Four of fourteen are majority-Indigenous.** Revision 1 reported Bennett County SD parenthesized as "(Pine Ridge)" — Bennett is *adjacent to* Pine Ridge, not Pine Ridge. **The single most load-bearing county in the internal-colony argument is a hole in the data and was reported as a hit.** Thirteen absences are recoverable only by a **FIPS vintage crosswalk that does not exist and is now budgeted in §4 Phase 0.** No county fails the way Revision 1 described (`u_s==0` or `u_T==0` never occurs).

**The denominator, renamed and its bias declared.** `U(i)` is **not** "the county total." `category_id=1` is `B17001_001`, *the population for whom poverty status is determined* — it **excludes institutionalized group quarters, barracks and dorms**. Measured against the engine's own population column: median ratio **0.865**, minimum **0.382** (Kalawao HI), and — in the other direction, from vintage mismatch — up to **5.50** (Aleutians East AK); national sum ratio 0.982. Cross-county comparison is therefore **not on a common basis**, and — worst for this axis specifically — **the carceral population is deleted from the oppressed-nation count in exactly the Black Belt counties the map lights**, while claim (5) cites MIM on prison labor as constitutive. This is a declared structural bias of the instrument (§0, OQ7).

**The map, published once, under one stated reference (national `p̄`, `T−H`, scopes universe n=3,140).** Revision 1's table was rendered in the **per-county** reference the same section called "the intensive-aggregation variance error wearing a different hat," with a footnote calling the difference "reorders slightly." Six of fourteen rows changed. The corrected table:

| FIPS | County | Λ | w | oppressed share | poverty universe |
|---|---|---|---|---|---|
| 46121 | Todd SD (Rosebud) | +0.4613 | +0.7227 | 0.920 | 10,070 |
| 46095 | Mellette SD | +0.3879 | +0.7693 | 0.605 | 1,973 |
| 46031 | Corson SD (Standing Rock) | +0.3552 | +0.7225 | 0.709 | 4,119 |
| 22035 | East Carroll Parish LA | +0.3190 | +0.7156 | 0.659 | 4,422 |
| 28051 | Holmes MS | +0.3169 | +0.6575 | 0.858 | 17,005 |
| 46137 | Ziebach SD (Cheyenne River) | +0.3157 | +0.6825 | 0.763 | 2,778 |
| 46071 | Jackson SD | +0.3154 | +0.7240 | 0.625 | 3,190 |
| 28021 | Claiborne MS | +0.3144 | +0.6490 | 0.884 | 8,329 |
| 48377 | Presidio TX | +0.2959 | +0.6385 | 0.871 | 6,970 |
| 46017 | Buffalo SD (Crow Creek) | +0.2956 | +0.6402 | 0.864 | 2,002 |
| 48047 | Brooks TX | +0.2925 | +0.6208 | 0.929 | 6,573 |
| 38085 | Sioux ND (Standing Rock) | +0.2853 | +0.6273 | 0.881 | 4,294 |
| 01063 | Greene AL | +0.2780 | +0.6356 | 0.829 | 8,242 |
| 28083 | Leflore MS (Greenwood) | +0.2758 | +0.6494 | 0.774 | 27,917 |
| 13061 | Clay GA | +0.2742 | +0.6877 | 0.647 | 2,898 |

**And the re-territorialization finding, which is correct and changes the rendering ruling.** Λ divides by a *county* denominator, so dispersed urban oppressed populations are diluted and small rural counties dominate. Under Λ the map lights the **former** territory — the Black Belt and the reservations — while MLP says the nation was "driven into ghettoes in the major urban centers" (`mlp-bnq.htm:128-131`). **Having declined to rule the nation territorial, Revision 1 restored territorial nationhood through the rendering channel.**

The extensive mass `E` — MIM's actual quantity — says the opposite, and this is the load-bearing result of Revision 2:

| FIPS | County | `E` (excess persons) | poverty universe |
|---|---|---|---|
| 06037 | Los Angeles CA | 525,425 | 9,928,773 |
| 48201 | Harris TX | 319,232 | 4,601,170 |
| 17031 | Cook IL | 288,756 | 5,112,701 |
| 36005 | Bronx NY | 252,320 | 1,400,341 |
| 42101 | Philadelphia PA | 206,509 | 1,535,277 |
| 12086 | Miami-Dade FL | 199,255 | 2,661,642 |
| 04013 | Maricopa AZ | 199,107 | 4,272,832 |
| 36047 | Kings NY | 197,804 | 2,564,539 |
| 26163 | Wayne MI (Detroit) | 183,582 | 1,736,055 |
| 48215 | Hidalgo TX | 169,268 | 844,950 |

Top 15 counties carry **26.4%** of the national mass; top 100 carry **58.3%**. **`E` is where the nation is; `Λ` is where the nation was.** Both are true, and they answer different questions. **Rendering ruling requested: `E` ships as the primary (proportional/dot-density) channel with `Λ` as the secondary per-capita channel, and the legend states which question each answers.** A Λ-only map teaches a thesis MLP explicitly retracted.

**And the bribe channel `Ω̂`, which is the one no liberal choropleth can produce.** Top: Loving TX +0.1636, Morgan UT +0.1628, Banner NE +0.1577, Daggett UT +0.1569, Campbell SD +0.1504. Bottom: **Leslie KY −0.1815, Clay KY −0.1634, Harlan KY −0.1547, Lee KY −0.1522, Bell KY −0.1462.** Santa Clara +0.0442, Montgomery MD +0.0695, Hidalgo TX +0.0055. Settler-majority Appalachian counties read **negative** — settler-nation populations *not* receiving the bribe. That is the labor-aristocracy thesis rendered with its own internal differentiation intact, and it is the guard against the map reading as "all whites are bribed."

**The two "found by running it" claims, corrected.**

- **Sherburne County MN does not reproduce.** Under the per-county reference it ranks **40 of 3,140** (`w=+0.7153`). Rank 1 is **King County TX** and **Loving County TX**, both at `w = +1.0000` — *saturating the measure outright* on oppressed universes of 78 and 15 persons; then Sharkey MS +0.9480, Banner NE +0.9173. The phenomenon is real and **worse** than described; the named exemplar was invented.
- **The small-count pathology is NOT confined to the per-county variant.** New finding this pass: under the *recommended national* reference, the top of the `w` ranking is **Elliott County KY +0.8245 on an oppressed universe of 31 persons**, tied with Loving TX (15 persons). **`σ = |w|` is undamped as `u_o → 0` in both variants.** A small-count damping law and an ACS-suppression policy are therefore **hard requirements**, not polish (§4 Phase 0; Todd County's `D` cohort reads 84.1% on a trivial base).
- **The Hidalgo `H`-over-`A` argument does not survive as stated.** Revision 1 compared oppressed-pole `T−A` against race code **`B`** — a denominator the recommended construction never uses. Recomputed properly: under `T−H`, settler 0.1029 vs oppressed 0.3095 (gap 0.2066); under `T−A`, settler 0.2887 vs oppressed 0.3502 (gap 0.0615). **The sign does not invert — the signal collapses by 70%**, because RGV "White alone" is overwhelmingly Hispanic-identifying and 87% of Hidalgo's population lands in the settler pole. The reference-code decision is still right; the argument for it is now the true one.

**Transport `T` — the review's correct verdict, and the answer.** Every input above is a frozen ACS 2019 constant. Registered as a `BoundOpposition`, `gap` would be constant, `rate` identically 0 for all 520 ticks, and `_score()` (`opposition.py:576`) constant. **A constant is not a dialectic**, and `D = (A, Ā, w, T, σ)` cannot absorb one. Aleksandrov is satisfied for the construct and violated for its motion.

**What makes it move:** the **incidence** is declared and static (that is claim 4 and must never move); the **reproduction levels** must be engine-produced. The channel is per-COLONIAL_AXIS-community aggregates over the XGI hypergraph (`systems/community.py:57-112`) — member-summed wealth against subsistence, recomputed every tick, with the census incidence supplying only the membership *weights*. **That channel is blocked on exactly one thing: `SocialClass.community_memberships` has no production writer** (`sentinels/seam/registry.py:2170-2192`, `STRUCTURALLY_IMPOSSIBLE`). **Phase 0's incidence artifact is precisely the data program that seam has been waiting for.** This is a blocking-dependency citation per ADR109, and it is now §4's critical path rather than an unstated assumption.

**`GapMeasure` body.** `BoundOpposition.measure` is `GraphInputs → GapReading` — **one** reading. Revision 1 specified only per-county `w_i` and never wrote the national `(a,b)` pair. It is now written: `a = Σ_i u_o(i)·p̄`, `b = Σ_i b_o(i)`, ratio-of-sums over the declared universe. **Never `mean(|w_i|)`** — that is the variance error §2.1 exists to avoid.

**`PoleMeasure` / `PoleSample`.** `PoleSample.entity_id` is documented "**Graph node id** this sample reads" (`opposition.py:129`); communities are hyperedges, never nodes (Amendment U). A per-*community* `PoleSample` is a type violation, and `IdeologySystem` iterates `social_class` nodes with no determinate community→class function. **Resolution: samples are emitted per `social_class` node, weighted by that node's `community_memberships`** — which is the same unblocking as the transport, and the same seam.

### §2.2 Candidate N2 — `settler_bribe` · the σ-tier attribution mirror

> **A** = *the Φ retained inside the imperial core* · **Ā** = *its distribution across the core's own national poles*

**Grounding.** `mt10.txt:1064-1065` ("the settler Euro-Amerikan nation and the state it controls"), `:7880-7884` ("the Euro-Amerikan working class is not a proletariat, but instead a labor aristocracy"), `:5454-5469`.

**Measure.** Structural mirror of `sigma/attribution.py:89-101` (`raw = tier_weight · gap^e · trade`): `raw_i = nation_weight(pole) · Ω̂_i^e · L_i`, normalized to shares — with `nation_weight` playing `_NODE_TIER`'s role: **settler = 0** by the argument that gives `eu`/`canada` weight 0 (`postgres_initialization.py:509-517`), oppressed nations = 1, and a damped middle `w_dom`.

**Grounding status, corrected.** Revision 1 said `w_dom` would be "derived from data and never invented, exactly as `derive_w_semi`." `derive_w_semi` (`sigma/attribution.py:104-129`) consumes **two explicit sequences of measured OUTFLOW-%GDP**. **No domestic analogue series is named here and none exists in `data-catalog.yaml`.** "Derived on the `derive_w_semi` pattern" was a promise of a derivation, not one. **`w_dom` is therefore an open data question (OQ3), not a scheduled derivation.**

**Scope change from Revision 1.** N2 was deferred to "Phase 4, chartered not scheduled" — which meant *the disparity map shipped and the imperialism didn't*. **Ω̂ moves into N1's shipping scope** (§2.1) so the axis carries the bribe from day one; what remains in N2 is only the **share attribution**, which genuinely needs `w_dom`.

### §2.3 Candidate N3 — `self_determination` · claim versus jurisdiction

> **A** = *the oppressed nation's claim on territory and self-government* · **Ā** = *the settler state's jurisdiction over it*

**Grounding.** Claim (3) plus Lenin's secession test (`ch04.htm:181-192`), the 1928/1930 Comintern resolutions as quoted at `cap-black-nation.htm:41` and `rcp-haywood.htm:70`, LRS at `history/erol/ncm-8/lrs-chicano-uf.htm:45` ("Without the right to self-determination, that is **political control of their territory**"). spec-070 has this half-built: `ColonialStance` UPHOLD/IGNORE/ABOLISH, `is_settler_formation`, FR-031a, RED_OGV.

**Grounding status: BLOCKED.** No county↔AIANNH bridge; the one AIANNH number in the engine is admitted fixture-grade. Needs a real spatial join against `tl_2025_us_aiannh.shp`.

**Elevated in Revision 2.** N3 is the *only* candidate that carries **power, jurisdiction and secession** — claim (3)'s entire content. With N3 blocked, the shipped axis adjudicates none of it. Revision 1 rejected N0 as liquidationist for making nat-op a modifier of the class magnitude, then shipped N1 `shadow=True` with `feeds → county_extraction` — measuring, adjudicating nothing, feeding the class axis. **The critic is right that the distinction is thinner than claimed.** The honest defence is not that shadow-plus-coupling *is* non-reduction; it is that a **registered opposition with its own poles, witness, and transport is a construct that CAN be promoted**, whereas a coefficient never can. That is a real difference, and it is a difference of *trajectory*, not of current state — which is why §5 asks for the promotion rule up front.

### §2.4 Candidate N0 — a bounded multiplier on `county_extraction`

`TENSION_i = |w_i| · (1 + μ·Λ_i)` — one coefficient, no new row, no new lens. Included for completeness; argued against in §3. Its virtue is that it cannot drift `qa:regression`.

---

## §3 — Recommendation

**Ship Phase 0 (the incidence artifact) unconditionally. Ship Phase 1 as an explicitly-declared STATIC REFERENCE OVERLAY, re-pinned into the M5 contract by amendment, and gated on the Director's OQ0/OQ1 rulings. Hold Phase 2 as BLOCKED with a named prerequisite. Withdraw Phase 3 as an engineering item and re-file it as an escalation. Reject N0.**

**1. N0 is the liquidation form, and the corpus names it.** Folding national oppression into a multiplier on an economic scalar makes it a *modifier of the class magnitude* — the move MIM diagnoses as liquidationist: "the main weakness of this and many other publications… is that it **has no distinction between oppressed and oppressor nations**" (`mt10.txt:9117-9121`), and Avakian's "in the end, all nationalism is bourgeois" (`:6499-6501`). Stalin, `1925/06/30.htm`: "Whoever regards the national question as a component part of the general question of the proletarian revolution **cannot reduce it to a constitutional issue**" — and a coefficient is exactly a side-condition on someone else's law of motion.

**2. It is genuinely subordinate — but the quote Revision 1 used to say so is transposed across the seizure of power, and is withdrawn.** `1923/04/17.htm:220` ("the basis of all our work lies in strengthening the power of the workers, and **only after that**…") is Stalin arguing against Bukharin **inside the USSR**, about "the right of the working class **that has come to power** to consolidate its power," speaking of "the **formerly** oppressed nations." Transposing post-revolutionary intra-socialist subordination onto a **pre-revolutionary imperialist core** contradicts Lenin 1916 (`25.htm:306-323`) and Constitution I.1. **The correct support for a bounded subordination existed all along in the chapter Revision 1 mis-read**: Lenin's mirror duty, `ch04.htm:151-153` — every national demand assessed "**from the angle of the workers' class struggle**." That is a *double-sided* subordination that also licenses the guard in claim (1). `shadow=True` is now justified by Lenin's assessment rule and by the transport blocker, **not** by transposed Stalin.

**3. The non-derivability claim holds for N1 and is NOT a defect claim against `chauvinist_pressure`.** Revision 1 called `max(0, class_wage_balance) · scale` (`ideology.py:245-256`) "the theoretically-false derivation, live in production" and named it "the strongest engineering argument in the document." **That is a category error and is retracted.** The in-code comment names the mechanism: *"only a POSITIVE balance (**the imperial bribe**) biases routing toward the fascist pole."* That is `W_c > V_c` — the Fundamental Theorem — and the ratified bifurcation rule. It derives **pressure** from the bribe; constraint (a) forbids deriving **the partition** from c/v/s. Nothing there names a nation. Changing it is an **Amendment AD / IX.5 escalation**, not a repair (OQ5). The genuine, smaller claim survives: `chauvinist_pressure` has no *national* input, and N1 could supply one — which is an enhancement request to the Director, priced as baseline-moving.

**4. What N1 uniquely offers is not a lens but a data program that unblocks a dead seam.** Phase 0's per-county × pole incidence table is the only credible seeder for `SocialClass.community_memberships` — `STRUCTURALLY_IMPOSSIBLE` since audit Wave 4. Unblocking it lights the community hypergraph, makes `PoleBinding.community_id` usable *for real*, gives `PoleSample` a legitimate node-level route, and supplies the per-tick reproduction aggregate that is the only honest transport. **That chain, not the choropleth, is the engineering case.**

**5. What ships must carry the bribe, or it is a poverty map.** With `Ω̂` in scope, the lens renders the settler premium (18.7M persons held above the demonstrated floor, 1.55× the deprivation mass) alongside the deficit, and the Appalachian negatives keep the labor-aristocracy claim from flattening into a racial one. **Without `Ω̂`, what reaches the player is a disparity choropleth legible to any liberal.** That is the economism failure in institutional form, and it was Revision 1's real shape.

**6. It makes the domestic and world scales one estate** — `Ω̂` is the same arithmetic as P26's σ-composition with `nation_weight` where `tier_weight` stands, which is what `project/programs/10-spectrum-of-unequal-exchange.md:46-47` already asserts as design intent.

---

## §4 — The Wiring Path

**Phase 0 — the incidence artifact (M5-adjacent, no engine contact).** A `sources`-producing loader emits per-county × national-pole incidence + reproduction-floor rows, registered in `data-catalog.yaml`/`data-artifacts.yaml` with `material_relation` and vintage. **Costs the Director must see, all newly budgeted:**

- **CI subset.** `tools/make_reference_subset.py:405` is `"fact_census_poverty": TablePolicy("skip", _UNREFERENCED_REASON)` — **no guard built on this table can execute in CI today.** Flipping to `full` means 26.5M rows; the ci-data re-cut is owner-gated per ADR169. **Director-visible ingestion work.**
- **Vintage crosswalk.** 13 of 14 absences are geography-vintage mismatches recoverable only by a FIPS crosswalk that **does not exist**. Plus the three retired FIPS the `scopes.py` resolver admits and the artifact does not.
- **Deleted extractor.** `data-catalog.yaml` records `extractor: deleted @ 4ce7c96a^ (src/babylon/data/census/loader_3nf.py)`, `disposition: fill`. **OQ11's vintage question cannot be re-cut without reconstructing it.**
- **ADR098 circularity.** `loader_to_sources.py`'s invariant is that sources come from *upstream raw*. An artifact derived from the built DB is a product-derived source feeding the product that produced it. **Either re-derive from raw ACS, or declare the artifact a second-order product with its own disposition — not silently both.**

**Guards (mutation-validated):** the `T − pole` exactness law; the ratio-of-sums law (no county-rate averaging anywhere); the **honest-absence law budgeted at 14/16 cells, not 1**; the **small-count damping law** (`σ` damped as `u_o → 0`; Loving/King/Elliott are the fixtures); an **ACS-suppression policy** distinguishing suppressed from genuinely-zero cells; and a **III.11 zero-denominator law** — `calculate_wealth_asymmetry_gap` returns `0.0`, which on a diverging ramp renders *at the settler norm*, indistinguishable from real parity. **That is a fabricated data point.** Absence must reach the envelope as absence.

**Phase 1 — the M5 overlay (projection-time, W-P) — re-scoped.** Revision 1 proposed "a second registered lens." The contract fixes `lens` to a **closed enum** `"value"|"tension"|"fog"`, cycles `l` value→tension→fog→value with a recorded *"match-arm-order trap, third time"*, requires a **frame-content golden per lens**, and already ships `"overlay_absent": "national oppression overlay chartered, not derivable from c/v/s"`. A fourth lens **deletes a declared-absence envelope key mid-milestone** against a pinned contract. *"`qa:regression` cannot drift"* is true and insufficient — the contract, the Rust match arms, the per-lens goldens and the parity harness all move.

**Therefore:** ship as a **declared static reference overlay**, entered by a **contract amendment** in the same shape as ADR170's, not smuggled in as a lens. And declare it honestly under Amendment V / II.8: `tension.py` folds engine-stamped county values; **this folds a census artifact and is identical at tick 1 and tick 520.** A layer that never moves must be labelled a reference overlay, or §3's pedagogy claim teaches that one axis is inert.

**Phase 2 — shadow registration (engine train, W-𝔇) — BLOCKED, three named prerequisites.**

1. **A production writer for `SocialClass.community_memberships`** — without it the transport is a constant and the poles bind to empty hyperedges (`seam/registry.py:2170-2192`).
2. **`county_extraction` registered as a `BoundOpposition`.** Verified: it appears nowhere in `src/` except `projection/topology/tension.py` as a *producer*; `CouplingGraph.__init__` (`core/coupling.py:120-129`) **raises `KeyError` on an unregistered endpoint**, so the coupling row `national_oppression --feeds--> county_extraction` **throws at import today.** ADR170 chartered that registration as separate engine-train work.
3. **The OQ0/OQ1 rulings**, since the pole shape (one binary opposition vs. three per-nation sibling rows each binding one `community_id`) is *determined* by them.

Proof obligation when unblocked: byte-identical `qa:regression` **plus** `qa:vault-regression-ci`.

**Phase 3 — `chauvinist_pressure` — WITHDRAWN as an engineering item, re-filed as OQ5.** It is baseline-moving, it rewires the ratified fascist-bifurcation route, and the county-scalar → per-class-delta join is unspecified. If the Director authorizes it, it is a `test(baselines):` ceremony with a `Baselines: blessed(<slug>)` trailer. **Prerequisite in either case: the claim-(1) guard.** No national channel may enter consciousness routing until bourgeois nationalism *of the oppressed nation* has its own channel, or the model can render national mobilization only as progressive.

**Phase 4 — N2 share attribution + N3 unblocking (chartered).** `w_dom` needs a named domestic outflow analogue (OQ3); N3 needs the AIANNH bridge.

---

## §5 — Open Questions reserved for the Director's line

Amendment AD / IX.5 questions. **I have resolved none of them.** OQ0 is new and governs the rest.

**0. Which side of the territory dispute is the project's line?** MIM + MLP (non-contiguous; nation dispersed; `mt10.txt:3158-3165`, `mlp-bnq.htm:133-159`) **against** CAP + Haywood + the 1928/1930 Comintern (Black Belt as national territory, Blacks elsewhere a national minority; `cap-black-nation.htm:41-42`, `rcp-haywood.htm:76`). Revision 1 took the first side silently. **Every other ruling depends on this one**, including whether the map's primary channel is `E` (people, urban) or `Λ` (territory, Black Belt).

**1. Which partition is the oppressed-nation pole?** **(a) `T − H`** — exact, zero double-count, verified working, **but it is a Census racial residual, which MIM's own sentence forbids, and it lumps Asian-American and multiracial populations into "oppressed nations," which no document in the corpus authorizes.** **(b) `B + C + I` with a declared overlap policy** — matches MIM's and ERoL's named nations and the live `CommunityType` COLONIAL_AXIS exactly, but double-counts Hispanic-Black and Hispanic-AIAN persons. **(c) A declared historical incidence** — county sets from conquest/enslavement/annexation, which is what claim (4) actually demands and what neither (a) nor (b) is. **All §2.1 numbers were computed under (a); the Director should read them as instrument validation, not as a sanctioned result.**

**2. Is the oppressor reference `H`?** `H` preserves the signal (`T−A` collapses the RGV differential 0.2066 → 0.0615). But `H` is Census self-identification standing in for what MIM calls the settler nation, and MIM flags the compromise itself (`mt10.txt:2810-2812`). Sanctioning it needs a disclosure of the same class as P26's `latin_america`→Non-OECD proxy (`ADR167:33-39`).

**3. Is there a middle tier, and what data derives its weight?** ERoL's nation / national-minority / oppressed-people taxonomy maps onto CORE/SEMI/PERIPHERY — but **no domestic outflow series exists**, so `w_dom` is a *data acquisition* question before it is a line question. The alternative is a flat 0/1, which is Lenin-faithful and needs no new data.

**4. Puerto Rico.** PR municipios are present with full rows and read as the highest-Λ geography in the estate (Maricao +0.5460, Lajas +0.5326 — **omitted from Revision 1's list of three** — Guánica +0.5369, Adjuntas +0.5221). **But the settler universes are 54, 133 and 53 persons, and `p̄` carries no cost-of-living or PPP deflator**, so the readings are structurally degenerate as well as excluded by `scopes.py`'s `substr(fips,1,2) < '60'`. Options: (a) declare the absence with its cause named; (b) widen the scope (touches the 3,153-county artifact, the CZ crosswalk's PR gap, every downstream baseline); (c) carry PR as a separate declared surface outside the county lattice, matching its status as an external rather than internal colony. My reading favours (c) — **and under any option the small-count and deflator problems must be fixed before a PR number is shown.**

**5. May a national channel enter `chauvinist_pressure` at all?** The existing derivation is ratified MLM-TW mechanism, not a defect. Replacing its input is baseline-moving and rewires the fascist-bifurcation route. **Prerequisite: OQ9's guard.**

**6. May `national_oppression` ever become the principal contradiction?** Constitution I.1 says "imperialism vs oppressed nations, NOT capital vs labor." **Revision 1's mechanical justification was false and is retracted:** `_principal_key` (`opposition.py:582-601`) filters on `shadow_keys` and `governance` only — **`level_name` plays no part in principal selection anywhere**; `level_index_for` has one production caller (`systems/contradiction.py:1075`, field-lattice placement, not scoring), and `political_form` is CANONICAL *and* unplaced. **The sole bar is `shadow=True`.** Un-shadowing plus a §6.5 ceremony achieves the whole effect. The remaining question is genuinely a line one: *should* it outrank `capital_labor` domestically, and under what rule?

**7. Does the point-in-time / carceral-exclusion bias disqualify B17001 as the reproduction proxy?** MIM discounts durationless counts on the page (`mt10.txt:5544-5550`), and `B17001_001` deletes institutionalized group quarters — **the prison population, in the counties the map lights, on an axis whose own §1 cites prison labor as constitutive**. The bias direction is conservative (§1 claim 7), which is the acceptable direction, but the Director should rule whether it is acceptable *at all* or whether a persistence-adjusted or non-poverty floor is required.

**8. Is `SETTLER` a `CommunityType` the map may name on screen?** MIM's vocabulary (`mt10.txt:1064-1065`) and the project's ratified line, and the single most legible political claim the interface will make.

**9. Does the oppressed-nation-nationalism guard ship before or with the axis?** Lenin's second duty (`ch04.htm:174-178`). Without it the model renders national mobilization as unconditionally progressive.

**10. Rendering: `E` or `Λ` primary?** `E` puts the nation where MLP says it is (Los Angeles, Harris, Cook, Bronx, Wayne); `Λ` puts it where CAP says it is (the Black Belt, the reservations). **This is OQ0 rendered in pixels.**

**11. Vintage — rule it as determinism, not line.** A tick-year-tracking vintage puts a reference-DB read on the per-tick path and moves the norm under the player; a pinned vintage makes claim (6) partly false on screen. **And the deleted extractor means re-cutting a vintage is itself unbudgeted work.**

---

## Appendix A — Amendment AE: WITHDRAWN

Revision 1 drafted a contingent Amendment AE making a COLONIAL_AXIS-bound opposition "principal-eligible at every level rung." **`level_name` is not a bar on principal selection anywhere in the code** (`_principal_key`, verified). AE amended a restriction that does not exist. **Withdrawn in full.** If the Director rules OQ6 affirmatively, the operative change is: **remove `national_oppression` from `shadow_keys`** via a declared §6.5 ceremony, with the incidence-provenance and ratio-of-sums obligations recorded in the promotion ADR. No amendment, no new primitive, no substrate mutation. Consequence if ruled negatively: Phases 0–1 ship unchanged; the axis simply never adjudicates.

---

## Appendix B — Critique disposition

**Fidelity review (11).**
1. *`T−H` violates MIM's "not a 'race'" clause* — **ACCEPTED, blocking.** Elevated to OQ1(c); all §2.1 numbers relabelled instrument-validation under an unsanctioned partition.
2. *A reserved territory ruling was settled by a Pydantic validator; CAP/Haywood/Comintern suppressed* — **ACCEPTED.** §2.0's ruling sentence withdrawn; CAP restored verbatim; new OQ0.
3. *Λ's county denominator re-territorializes and inverts MLP* — **ACCEPTED and extended.** Raw `E` computed: top 15 counties are LA/Harris/Cook/Bronx/Philadelphia; `E` proposed as primary channel; new OQ10.
4. *N1 measures deprivation, not MIM's bribe; imperialism deferred* — **ACCEPTED, scope-changing.** `Ω` added to shipping scope: 18,691,613 persons, 1.55× the deficit, with Appalachian negatives.
5. *"No oppressed-nation mirror" is false* — **ACCEPTED.** `ch04.htm:151-153`, `:174-178` restored; guard made a Phase-3 prerequisite; new OQ9.
6. *Stalin 1923 subordination is transposed across the seizure of power* — **ACCEPTED.** Quote withdrawn; subordination re-grounded on Lenin's assessment rule.
7. *MIM discredits point-in-time poverty counts* — **ACCEPTED with a direction finding.** Declared as claim (7); bias shown to be conservative; new OQ7.
8. *Stalin's four criteria declared then unused; numbers computed on an unauthorized partition* — **ACCEPTED.** "Emerges from the data alone" struck; §2.1 relabelled.
9. *N3 blocked ⇒ the shipped axis carries no power; N1-with-`feeds` ≈ N0* — **PARTLY ACCEPTED.** The thinness is conceded; defence narrowed to promotability (a difference of trajectory), and the promotion rule moved up front.
10. *Police terror / prison labor cited then unmodelled and undeclared* — **ACCEPTED.** Declared in §0 and tied to the carceral-exclusion finding.
11. *`full.txt` does not exist* — **ACCEPTED.** Re-keyed to `mt10.txt` under a declared `pdftotext` invocation; all 13 spot-checked line ranges reproduce exactly.

**Constitutional-engineering review (13).**
1. *No transport — a constant dressed as a dialectic* — **ACCEPTED, decisive.** Phase 2 declared BLOCKED; the moving quantity named (per-community reproduction aggregates) and its blocker cited (`seam/registry.py:2170-2192`).
2. *`GapMeasure` never specified* — **ACCEPTED.** National `(a,b)` written as a ratio of sums, with `mean(|w_i|)` explicitly forbidden.
3. *Coupling row raises `KeyError` at import* — **ACCEPTED.** Verified (`coupling.py:120-129`); listed as Phase-2 prerequisite #2 with the ADR109 blocking citation.
4. *`community_id` holds one community; the headline claim doesn't survive* — **ACCEPTED, twice over.** Verified single-`str`, and the hyperedge is `STRUCTURALLY_IMPOSSIBLE` in production. Headline withdrawn; the seam repurposed as the engineering case.
5. *OQ6/AE rest on a false mechanical claim* — **ACCEPTED.** `_principal_key` verified to ignore `level_name`; AE withdrawn; OQ6 re-posed as pure line.
6. *The `chauvinist_pressure` "repair" is a category error* — **ACCEPTED.** Retracted as a defect claim and the "strongest argument" billing; re-filed as OQ5 escalation.
7. *Per-community `PoleSample` is a type violation* — **ACCEPTED.** Samples re-specified per `social_class` node, membership-weighted.
8. *Absence measured against the wrong universe* — **ACCEPTED.** Both universes published (14 of 3,153; 16 of 3,156) plus the three-retired-FIPS drift.
9. *Phase 1 conflicts with the pinned M5 contract* — **ACCEPTED.** Re-scoped to a contract-amendment-entered static reference overlay; goldens, match arms, parity harness and the `overlay_absent` key named as movers.
10. *N1 is a static basemap, not engine state* — **ACCEPTED.** Declared as a reference overlay under Amendment V / II.8.
11. *No small-count / suppression policy* — **ACCEPTED and extended.** Shown to bite the *national*-reference variant too (Elliott KY, `u_o=31`); damping and suppression made Phase-0 hard requirements.
12. *OQ8 vintage is determinism, not line* — **ACCEPTED.** Re-posed as OQ11, determinism-first.
13. *Cited artifacts live only on an unmerged worktree* — **ACCEPTED with a factual update.** Both merged (`043a2c4c`, `8da23398`); the document is re-filed against `origin/dev` and ADR170 is cited only for the projection/engine split.

**Data-honesty review (13).**
1. *Pine Ridge absent at every vintage; Bennett mislabelled* — **ACCEPTED.** Verified `46102` has zero rows at every `time_id`; `46113` only 2010–2014. Named first in the absence table; parenthetical struck.
2. *Absence understated 14×, non-randomly Indigenous* — **ACCEPTED.** All 14/16 tabulated; the correlation-with-the-measured-thing point stated as a bias, not thinning.
3. *Headline table computed on the rejected arithmetic* — **ACCEPTED.** One table, one stated reference; six rows changed; "reorders slightly" struck.
4. *Sherburne does not reproduce* — **ACCEPTED.** Rank 40/3,140; King/Loving TX saturate at `w=+1.0000`; claim rewritten with the true exemplars.
5. *The `H`-over-`A` argument substitutes a denominator* — **ACCEPTED.** `T−A` does not invert; recomputed as a 70% signal collapse.
6. *`u_T` misnamed; prison population excluded* — **ACCEPTED.** Renamed to the ACS poverty universe; median ratio 0.865 (0.382–5.50) published; carceral exclusion tied to claim (5) and OQ7.
7. *`fact_census_poverty` is `skip`-scoped out of CI* — **ACCEPTED.** Verified `make_reference_subset.py:405`; added to Phase 0 as Director-visible ingestion cost.
8. *ADR098 path circular; upstream extractor deleted* — **ACCEPTED.** Both added to Phase 0 costs; the source-derivation must be re-derived from raw or declared second-order.
9. *ADR170 unmerged and itself charters the dataset as open* — **PARTLY ACCEPTED.** Merge status updated (now on `origin/dev`); the substantive half accepted in full — the "needs a real reference dataset" consequence line is quoted in §0 and the "ruled/pinned" framing struck.
10. *The `chauvinist_pressure` claim rests on an equivocation; join unspecified* — **ACCEPTED.** Concurs with engineering #6; retracted, and the county-scalar → per-class join named as unspecified.
11. *`w_dom` "derived from data" has no data* — **ACCEPTED.** Downgraded from scheduled derivation to open data question (OQ3).
12. *III.11 papered over at the zero denominator* — **ACCEPTED.** `0.0` on a diverging ramp named as a fabricated data point; an absence path made a Phase-0 guard.
13. *PR numbers degenerate and undeflated; Lajas dropped* — **ACCEPTED.** Settler universes of 54/133/53 published, deflator absence declared, Lajas restored, and OQ4 gated on fixing both before any PR number renders.

**Rejected: none in substance.** Two findings are narrowed rather than accepted whole — fidelity #9 (the N1≈N0 charge: the promotability difference is real, though thinner than Revision 1 claimed) and data-honesty #9 (the merge-status half is now factually superseded, while its substantive half is accepted in full).
