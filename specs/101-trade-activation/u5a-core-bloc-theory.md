# U5a — Core-bloc Φ-treatment theory research

**Program**: 26 International Trade, Unit U5a (`project/programs/26-international-trade.md`
§4, feeding U5). **Status**: RESEARCH ONLY — no code, no wiring. Precedes the σ-composition
core-bloc treatment inside the U5 engine train, per ADR165 Q2.
**Ruling record**: `ai/decisions/ADR165_p26_director_rulings_trade_slate.yaml:23-33` — Q2 is a
*research directive, not a binary*: ground the core-bloc question in Samir Amin (unequal
exchange / imperial rent) + Immanuel Wallerstein (world-systems core/semi-periphery/periphery)
+ MIM internal-colonies theory, and the status-quo 48.5%-of-drain-to-core figure
(`specs/101-trade-activation/u4-phi-attribution-options.md:96-106`, §2.3 table) is explicitly
**not re-affirmed**.

______________________________________________________________________

## 1. Theory synthesis

### 1.1 Samir Amin — imperial rent / unequal exchange

Amin's framework (as already cited in-repo: `ADR055:35` records "unequal-exchange rent scales
with trade volume (Amin, Hickel, Cope)"; Program 10 `project/programs/
10-spectrum-of-unequal-exchange.md` is the ratified in-repo formalization of the
Amin/Emmanuel/Wallerstein synthesis this game already runs on) treats imperial rent as the
value gap that opens when labor-power is priced globally but wages are set nationally —
capital in the center pays a Southern worker for output priced far below its
socially-necessary labor time, and the difference (Φ) is transferred North on every unit of
trade. The mechanism is asymmetric by construction: it requires a wage/price differential
between a *high-organic-composition, high-wage* pole and a *low-organic-composition, low-wage*
pole (Program 10 §1, the σ-gradient this program already encodes,
`specs/107-sigma-gradient/spec.md:24-29`). Two mutually-comparable high-wage economies do not
generate this gap between each other by the mechanism's own construction — Amin's transfer is
a function of the wage/price differential, and that differential is close to zero between core
formations. **Consequence for Q2**: under Amin, EU/Canada/Russia's-Europe-mapping sourcing rent
*into* the US requires a US-ward wage/price differential these blocs do not have relative to
the US; the theory does not predict core-to-core drain as a first-order term. It does not rule
out *intra-core* redistribution through finance, seigniorage, or monopoly rent (Amin's later
work on "generalized monopoly rent," outside this program's scope), but that is a distinct
mechanism from the ERDI/Hickel drain series Babylon has loaded
(`u4-phi-attribution-options.md:64` — `fact_hickel_erdi_annual`, itself an Amin/Hickel-style
unequal-exchange construction, not a finance/seigniorage series).

### 1.2 Immanuel Wallerstein — world-systems positions

Wallerstein's core/semi-periphery/periphery schema (already the vocabulary
`dim_country.world_system_tier` uses in-repo — `src/babylon/reference/schema.py:308-317`,
constrained to exactly `{core, semi_periphery, periphery}`) treats the tripartite division as
a structural position in the world division of labor, not a static country list: core
economies specialize in capital-intensive, high-wage production and *appropriate* surplus from
peripheral, labor-intensive production via the world market's price mechanism; semi-periphery
occupies an intermediate position, exploited by the core but itself exploiting the periphery —
Wallerstein's own worked example is that semi-peripheral states can be net *senders* of value
to the core in some relations while being net *receivers* from the periphery in others
(*The Modern World-System*, Vol. I, the core-appropriation mechanism; general treatment also in
*World-Systems Analysis: An Introduction*, ch. 2). The theory's direction-of-flow claim is
unambiguous at the core/periphery ends — value moves periphery → core, and semi-periphery is
where the direction can locally invert. It says nothing that would predict a *net* core →
core → US transfer; the core pole is definitionally where accumulation terminates, not a
transit or source zone relative to another core state.

**China's contested position** is exactly the case Wallerstein's semi-periphery concept was
built to hold: an economy with rapidly rising organic composition of capital and export
sophistication, exploited historically by the core via unequal exchange, now itself extracting
rent from lower-wage peripheries through its own global value chains (Belt-and-Road financing,
manufacturing FDI into Southeast Asia and Africa) while remaining a *net* labor-value exporter
to the US-EU core on the aggregate trade balance. Babylon's own data agrees with this
classification empirically (§2 below): the Ricci dataset used here classifies China as
`SEMI_PERIPHERY`, not `CORE`, and China is a 100%-OUTFLOW region across every observed year.

**Russia's position** is the one genuinely live tension in the sources. `russia_csi`'s current
crosswalk target is `"Europe"` (a Census bloc, `u4-phi-attribution-options.md:69`) — an
implicit CORE classification by proxy. But `dim_country.world_system_tier` for the *Ricci*
region "Russia and CSI" is `semi_periphery` (verified,
`u4-phi-attribution-options.md:271-272`), and the Ricci CSV rows show it as a 100%-OUTFLOW
region in every observed year (1995, 2000) — see §2. Both Amin and Wallerstein have a slot for
this: Russia is a raw-materials/energy exporter with a historically distorted organic
composition of capital, structurally closer to semi-periphery than to the EU/North-America
core, and the in-repo tier data already says so. The current `russia_csi → "Europe"` crosswalk
is therefore not just a granularity problem (L3 in the U4 paper) — it is a **tier
misclassification** the theory synthesis flags independently of the granularity issue.

### 1.3 MIM — internal-colonies / internal semi-colonies theory

The MIM text (`project/notes/percy/mim-internal-colonies.txt` — not present in this worktree;
read from the sibling checkout `/home/user/projects/game/babylon/project/notes/percy/
mim-internal-colonies.txt`, full 634 lines, "On the internal class structures of the internal
semi-colonies," MC5 1998/MC45 1999) is **not about the international bloc question at all** —
its subject is domestic (§4 below covers the mapping). Two passages are relevant to Q2's
core-bloc question by extension, though, because they bear on how much of the imperial pie is
attributable to trade-partner geography versus the receiving country's own internal class
structure:

- The text's central "pie" argument (lines 559–599) is that essentially the *entire* annual
  increment of US wealth is explainable by "transfer of surplus-value from the productive
  sector in the Third World to the unproductive sector in the [first] world" — the mark-up on
  goods delivered by Third World labor is "sufficient to explain all the new wealth of the
  imperialists every year." This is a *periphery-sourced* origin claim for the drain, stated as
  strongly as the source states anything, and it leaves no room in the accounting for a
  core-bloc-sourced component: if the Third World's surplus alone accounts for 100% of the
  increment, EU/Canada/Russia contribute ~0% by construction of the argument, not by omission.
- MIM never applies "internal colony"/"internal semi-colony" terminology to inter-imperialist
  relations (US↔EU, US↔Canada) — the term is reserved throughout for populations *inside* an
  imperialist country's own borders (Black, First Nations, Chicano/Aztlan, Puerto Rican
  nationalities). The absence is itself informative: MIM's entire analytical apparatus for
  "internal" extraction is domestic, and offers no vocabulary or mechanism for one core state
  extracting from another core state.

**Net MIM contribution to Q2**: no direct support for or against core-to-core drain (the text
doesn't address it), but strong indirect support for periphery-sourced-only accounting via the
"pie" argument, and zero borrowed vocabulary that would license treating EU/Canada as anything
but non-sources.

### 1.4 Convergent reading

All three sources point the same direction on the narrow Q2 question:

| Source | Predicts core→US drain? | Mechanism cited |
|---|---|---|
| Amin (unequal exchange) | No, to first order | Requires a wage/price differential; core-core differential ≈ 0 |
| Wallerstein (world-systems) | No | Core is accumulation's terminus, not a source pole; only semi-periphery can locally invert |
| MIM (internal colonies) | No (indirect) | "Pie" argument attributes ~100% of first-world wealth increment to Third World surplus; no core-core vocabulary exists in the text |

None of the three sources offers a mechanism by which EU, Canada, or a genuinely core-tier
Russia mapping would source drain into the United States. The status-quo 48.5% core share
(§2.3 of the U4 paper) has **no support in any of the three ruled sources** — it is an artifact
of the trade-volume proxy (Option A), not a theory-grounded finding.

______________________________________________________________________

## 2. Empirical cross-check — the Ricci dataset

The Director named `babylon_ricci_final.csv` as the canonical world-scale anchoring source
(ADR165 D2/D3, re-ingestion ruled). It is also, independently, the one dataset in the repo that
carries a *signed, region-typed* transfer direction — exactly the empirical check Q2 asks for.
Full read of `src/babylon/data/reference/babylon_ricci_final.csv` (51 rows, columns `year,
region_name, region_type, flow_direction, transfer_type, value_usd_billions, value_pct_gdp,
signed_value, gvc_share_of_total, source_table, source_priority, region_granularity, edge_id`):

**Region → tier assignment in the data** (13 distinct `region_name` values):

| region_type | regions |
|---|---|
| CORE | Advanced Economies, EMU, North America, OECD, Western Europe |
| SEMI_PERIPHERY | China, Emerging and Developing Economies, Russia and CSI |
| PERIPHERY | India, Non-OECD, South Asia, Southeast Asia, Sub-Saharan Africa |

**Flow direction by tier** (all 51 rows, both `GVC` and `TOTAL` transfer types):

| region_type | INFLOW rows | OUTFLOW rows |
|---|---:|---:|
| CORE | 17 | **0** |
| SEMI_PERIPHERY | 0 | 8 |
| PERIPHERY | 0 | 26 |

**Finding, stated precisely**: every CORE-tier row across all four observed years (1995, 2000,
2007, 2009) is `flow_direction = INFLOW` (`signed_value` positive). There is not one CORE
OUTFLOW row in the dataset. Every OUTFLOW row (34 of 34, including all SEMI_PERIPHERY rows) is
PERIPHERY or SEMI_PERIPHERY. **Core-to-core drain does not exist empirically in the one dataset
the Director ruled canonical for world-scale anchoring** — the dataset shows a strict two-sided
partition (CORE always receives; SEMI_PERIPHERY and PERIPHERY always send), not a continuous
gradient with core-to-core cross-terms. This directly falsifies the status-quo's implicit claim
(48.5% of drain sourced from `eu`+`canada`+`russia_csi`→"Europe") for the `eu`/`canada` portion
of that share on the data the Director himself designated as ground truth; it does **not**
speak to `russia_csi`, because Ricci classifies "Russia and CSI" as `SEMI_PERIPHERY` — an
OUTFLOW-only region in every year observed (1995: −$47.0B GVC / −$163.0B TOTAL; 2000: no
Russia-and-CSI row) — meaning `russia_csi`'s current `→"Europe"` (CORE) crosswalk target is
itself empirically wrong on this dataset: Russia and CSI should be a *source*, not booked
against a receiving bloc at all.

China is also SEMI_PERIPHERY throughout (100% OUTFLOW, all four years, rising in magnitude:
1995 −$125.7B TOTAL → 2007 −$382.5B TOTAL), consistent with §1.2's Wallerstein reading — the
data does not classify China as CORE.

______________________________________________________________________

## 3. Proposed treatment rule

Stated precisely enough for the U5 σ-composition crosswalk to implement. This rule traces every
element to §1 (theory) or §2 (Ricci data); nothing here is a new invented coefficient.

**Rule (three-tier, not binary):**

1. **CORE-classified nodes receive Φ-share weight 0.** Per Ricci `region_type = CORE`
   (§2) — no CORE row is ever OUTFLOW in any observed year, and no ruled source (Amin,
   Wallerstein, MIM) predicts a core-to-core-into-US drain mechanism (§1.4). This directly
   answers Q2: `eu` (Ricci "Western Europe"/"EMU", both CORE) and `canada` (Ricci "North
   America", CORE) get weight 0 — the U4 paper's disclosed L1 defect (48.5% of Φ credited to
   core, `u4-phi-attribution-options.md:96-106`) is resolved by zeroing, not re-affirmed.
2. **`russia_csi` is re-mapped off the CORE "Europe" crosswalk target onto its Ricci-native
   SEMI_PERIPHERY classification**, receiving positive weight (rule 3, not rule 1). This is a
   correction the empirical check in §2 forces independently of Q2's core-bloc question — the
   current `_NODE_TO_BLOC` mapping (`postgres_initialization.py:371-379` per the U4 paper) was
   never theoretically grounded for this node (flagged "weak" in the source comment itself,
   `u4-phi-attribution-options.md:122`), and Ricci settles it: Russia and CSI is
   `SEMI_PERIPHERY`, 100% OUTFLOW.
3. **SEMI_PERIPHERY nodes (`russia_csi`, `china`) receive a positive but *damped* Φ-share
   weight relative to PERIPHERY nodes**, reflecting Wallerstein's structural claim that
   semi-periphery both sends to the core *and* itself extracts from the periphery (§1.2) — a
   semi-periphery node's *net* outward transfer to the US core is real but structurally smaller
   per unit of trade than a periphery node's, because part of what a semi-periphery economy
   generates is retained as its own extraction from peripheries beneath it. This is the one
   element of the rule that is a *qualitative* theory reading translated into a *quantitative*
   damping coefficient — flagged explicitly in §5 as needing a declared value, not left
   implicit.
4. **PERIPHERY nodes (`india`, `southeast_asia`, `sub_saharan_africa`, and the constructed
   `latin_america` once ADR165 Q3's membership ruling is executed) split the remainder of the
   $8,625B in proportion to `σ-distance × trade volume`** — i.e., the ruled Option C mechanism
   (ADR165 Q1, `specs/107-sigma-gradient/spec.md`) is retained *unmodified* for the set of
   nodes it was designed for; this treatment rule only changes which nodes are eligible to
   receive a positive share, not the σ-gradient math itself. `σ-distance` is `σ_US − σ_i`
   (`u4-phi-attribution-options.md:341`), and trade volume is the existing disjoint-taxonomy
   bilateral trade figure (ADR165 Q4).
5. **Renormalization is mandatory and unchanged**: whatever weights rules 1–4 produce, the
   final shares renormalize to Σ = 1.0 before multiplication by national Φ, preserving the
   conservation invariant (`u4-phi-attribution-options.md:189-196`, constraint 1). Zeroing CORE
   nodes does not reduce total Φ — per the U4 paper's own framing, it *concentrates* the whole
   $8,625B onto SEMI_PERIPHERY + PERIPHERY nodes, which is itself the theory claim this rule
   makes explicitly rather than silently.

**Summary table (illustrative structure, not final coefficients — those are for the U5 σ
weighting pass to fix per ADR165's D1 delegation, `specs/107-sigma-gradient/spec.md:104-125`):**

| Tier (Ricci `region_type`) | Engine nodes (post Q3/Q4 rulings) | Φ-share weight rule |
|---|---|---|
| CORE | `eu`, `canada` | **0** |
| SEMI_PERIPHERY | `russia_csi` (re-mapped), `china` | `w_semi × σ-distance × trade_volume`, `w_semi < 1` (damped) |
| PERIPHERY | `india`, `southeast_asia`, `sub_saharan_africa`, `latin_america` (pending Q3) | `σ-distance × trade_volume` (undamped, the ruled Option C form) |

______________________________________________________________________

## 4. Internal-colonies seed (future unit, not U5 scope)

MIM's internal colonies/internal semi-colonies are explicitly **domestic** — populations
*inside* US borders occupying a structurally distinct position from the settler/oppressor
nation despite formal citizenship (`mim-internal-colonies.txt`, throughout; the term is applied
to Black, First Nations, Chicano/Aztlan, and Puerto Rican nationalities, never to another
country). This is orthogonal to the Q2 core-bloc question (§1–§3, which is about *international*
bloc nodes), but the Director's ruling explicitly named MIM as a source to "incorporate," and
Babylon already has domestic machinery this maps onto:

- **`SocialRole.INTERNAL_PROLETARIAT`** (`src/babylon/models/enums/social.py:26,42`) — "Core
  workers outside LA (precariat, unemployed, incarcerated)." This is the closest existing class
  position to MIM's internal-semi-colony proletariat, though MIM's own text draws a national
  (racialized) line, not merely an income-stratum line — MIM's "internal semi-colonies" are
  nationality-defined populations that MIM itself says are "labor aristocracy or higher" for
  the *employed* majority (`mim-internal-colonies.txt:547-557`) with the internal-colony
  proletariat concentrated specifically in the undocumented and lumpen strata
  (`:75-76`, `:216-217`, `:601-613`). Babylon's `INTERNAL_PROLETARIAT` currently keys off
  income/employment status only (precariat/unemployed/incarcerated), with no nationality axis —
  a gap between the model and the source theory worth naming for a future unit.
- **`equilibrium_w3` / `equilibrium_w4`** (`src/babylon/data/defines.yaml:463-464`) — the
  labor-aristocracy (p50-90) and single folded proletariat (bottom-50, "migrant farm workers,
  the $7.25 stratum") wealth-share targets. The `equilibrium_w4` comment already gestures at
  MIM's point (owner ruling 2026-07-16 folded "internal + periphery proletariats and the lumpen
  into one bracket") — a future internal-attribution unit would need to *un-fold* this bracket
  along a nationality/settler axis to make MIM's internal-semi-colony claim representable,
  rather than collapsing it into one undifferentiated bottom stratum.
- **`ColonialStance` / `RED_SETTLER_TRAP`** (`src/babylon/models/enums/balkanization.py:37-46`,
  `src/babylon/engine/systems/faction_influence.py:160-189`, `EventType.RED_SETTLER_TRAP_DETECTED`
  `src/babylon/models/enums/events.py:154`) — the existing UPHOLD/ABOLISH settler-sovereignty
  axis and its `class_reduction` threshold detector are the nearest live analogue to MIM's
  settler/internal-colony political split, but they operate at the *faction* level (an
  organization's stance toward settler sovereignty), not as a per-node Φ-sourcing attribution.
  A future internal-attribution unit would need to connect `INTERNAL_PROLETARIAT` node
  population *as a Φ-receiving/sourcing category* to this existing settler-stance machinery,
  which today only fires a diagnostic event.
- **`NodeType.SETTLER` / territory classification** (`src/babylon/models/enums/
  community.py:39`, `src/babylon/models/enums/territory.py:57-90`) — the settler-colonial
  territorial hierarchy already exists as a topology concept; a future unit would need to ask
  whether internal-colony Φ-sourcing should be territory-resolved (e.g., Wayne County as an
  internal semi-colony per the existing Detroit case-study framing,
  `docs/concepts/unified-class-system.rst:178`, `specs/026-tri-county-economic-substrate/
  spec.md:14`) rather than class-stratum-resolved.

**What a future internal-attribution unit would look like (seed, not spec):** a companion to
the international σ-crosswalk (§3) that asks, symmetrically, whether the national Φ figure
itself needs an *internal* redistribution pass — i.e., whether the labor-aristocracy stratum
(`equilibrium_w3`) is credited with "receiving" imperial rent that MIM's internal-colonies
argument says should instead route disproportionately away from `INTERNAL_PROLETARIAT` nodes
inside internal-semi-colony territories. This is squarely a **new Director ruling** (a
nationality/settler axis inside domestic wealth distribution is exactly the kind of
ideological-line question IX.5 reserves), chartered as its own unit — not scope creep into U5,
which is bounded to the *international* bloc crosswalk.

______________________________________________________________________

## 5. Flagged theory calls (Director-level, not resolved here)

1. **The SEMI_PERIPHERY damping coefficient (`w_semi` in §3 rule 3) has no numeric value
   assigned by any of the three sources.** Wallerstein's qualitative claim (semi-periphery
   both sends to core and extracts from periphery, so its *net* outward transfer is smaller
   per unit than a pure periphery's) is well-supported textually, but none of Amin,
   Wallerstein, or MIM supplies a number. The Ricci CSV's own magnitudes could ground an
   empirical damping factor (e.g., compare China's/Russia's OUTFLOW value_usd_billions per
   unit of GDP against periphery regions' equivalent figures), but that derivation is an
   engineering step this research pass does not perform — flagged for the U5 implementer to
   propose and the Director to confirm alongside the σ weights (ADR165 already delegated the
   analogous σ-weight decision, D1; whether this damping factor is similarly delegable or
   needs explicit ruling is itself a question for the Director).
2. **Russia's re-mapping off `"Europe"` (CORE) onto SEMI_PERIPHERY (§3 rule 2) is presented
   here as forced by the Ricci data**, but it is also a *change in kind* from what ADR165 Q2
   asked — Q2 named `russia_csi`'s "Europe" mapping as one of the three blocs under scrutiny,
   implicitly treating it as a CORE candidate to be confirmed or zeroed. This paper's finding
   (Russia and CSI is empirically SEMI_PERIPHERY, not CORE, and should be re-crosswalked
   entirely rather than zeroed) goes beyond a yes/no on "does it source rent into the US" —
   it changes the crosswalk target. Flagged explicitly so the Director can confirm this reading
   rather than have it land as an unremarked side effect of the σ-crosswalk implementation.
3. **Amin's later "generalized monopoly rent" work** (finance, intellectual property,
   seigniorage-based inter-imperialist extraction, not covered by his classical unequal-exchange
   apparatus or by the Hickel/Ricci drain series Babylon has loaded) is a distinct mechanism by
   which a channel for intra-core transfer *could* exist in Amin's broader corpus, but it is
   untested against any data in this repo and was not requested by ADR165's citation of Amin
   (which names "unequal exchange/imperial rent" specifically). Flagged as out of scope for
   this research pass, not resolved as "no" — if the Director wants intra-core financial rent
   modeled at some future point, that is a distinct theoretical construct from the Φ/Hickel
   drain series this program attributes.
4. **The three sources were not in disagreement on the narrow Q2 question** (§1.4) — this is
   itself worth flagging as a finding, since the Director's ruling framed Q2 as requiring
   synthesis across sources that could plausibly have conflicted. They converge. The one
   substantive disagreement surfaced in this research pass is not between the three ruled
   sources but between the *status quo* (Option A, trade-volume proxy) and all three of them.
5. **MIM's internal-colonies material (§4) does not bear on the international Q2 question at
   all** — it was read in full per the Director's directive but contributes nothing to the
   core-bloc treatment rule in §3; its entire contribution is the seed material in §4. Flagged
   so this is legible as "read and found inapplicable to Q2," not "skipped."
