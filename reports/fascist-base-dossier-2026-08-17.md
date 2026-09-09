# Who Joins Fascism — a theory dossier for the FascistFaction port

**Status: DESIGN INPUT, NOT A RULING.** Nothing here decides anything. This is the dossier ADR208
cites as design input to the FascistFaction port and to R1's parameter rows on `FASCIST_VEHICLE`;
every parameterization choice below returns to the Director as an option space.

**Revision note (2026-08-17, second pass).** The first draft
(`ai/scratch/2026-08-17-fascist-base-dossier.md`, left in place unaltered) was a
Sakai/MIM/Zetkin/Dutt literature synthesis written *before* the Director shared her own essay on
fascism. This version integrates that essay as the **authoritative frame**. Under Constitution IX.5
the Director holds the ideological line, so the essay outranks every other source here: Sakai and
MIM become corroborating literature read *through* the essay's criterion, Zetkin and Dutt stay
subordinate, and Dimitrov stays excluded — now on the essay's own argument rather than on a bare
directive. The source ordering is therefore: **essay → Sakai/MIM → Zetkin/Dutt → (Dimitrov: out)**.

Every claim about the literature carries its citation; every claim about the simulation names a
file, system, attribute, or line. Where the sources diverge — and the essay, Sakai, and MIM do
diverge, on the declassed, on the color line, and on gender — the divergence is recorded, not
smoothed. Where the essay's own simulation claims are *sharper than the tree warrants*, that is
recorded too (§4.3, §9 Q11–Q12): the honest finding is that two of the four extraction channels
have partial, unwired machinery rather than none at all.

Essay citations below point at the digest, `reports/director-essay-fascism-digest-2026-08-17.md`,
which quotes the full text the Director pasted in-session; the essay itself is
*The Topology of Dialectics*, "What Is Fascism?" (Persephone Raskova).

---

## 1. The Director's frame governs

### 1.1 Fascism is a process, not a crystallized definition

The essay's method is the first thing it establishes, and it is a method claim before it is a
content claim: fascism cannot be captured by a fixed definition because its character fluctuates.
Following George Jackson (*Blood in My Eye*), fascism is **a dialectical process in constant
fluctuation between three phases** (digest §Jackson frame, lines 19-24):

1. **insecure, out of power**
2. **insecure and in power**
3. **secure and in power**

The load-bearing dynamic is the third phase: **secure-in-power fascism can permit mass dissent**,
because it holds ideological hegemony rather than needing to win each confrontation by force. The
essay's example is early Mussolini Italy — press freedom and the PCI in parliament, for years
(digest lines 26-29). A regime that represses *less visibly* is not less fascist; it is more
secure.

That has a direct simulation reading, which the digest states as design input, not ruling: the
phase couples to **legitimation/hegemony strength, not repression intensity** (digest lines 28-30).
It is the reason a Jackson-phase dimension on R1's mutable capture-state flag is posed below as
Q13, and the reason repression magnitude alone cannot serve as a fascization measure (§9 Q5).

### 1.2 Why Dimitrov is excluded — the essay's own argument

The first draft excluded Dimitrov on directive. The essay supplies the reasoning, and it is a
**category error about quantification**, not a matter of line. Against Dimitrov's "open terrorist
dictatorship of the most reactionary, most chauvinistic and most imperialist elements of finance
capital", the essay asks:

> "Is there such a thing as a not-reactionary finance capital? … What unit of measurement are we
> using to measure chauvinism? I was not aware such a thing could be quantified! I could have
> sworn it was rather, a qualitative shift!"
> (digest §The Dimitrov critique, lines 13-17)

Two consequences for this document. First, the exclusion is now *principled* and portable: a
definition built from superlatives ("most reactionary", "most chauvinistic") names no measurable
relation and therefore fails the Aleksandrov Test as a design input — it cannot be traced to a
material relation because it names a degree of a quality without a quantity. Second, the critique
cuts at our own engineering: any parameter we introduce that scores "chauvinism" as a magnitude
inherits exactly the objection the essay makes. That is worth flagging where the tree already does
this — `_STANCE_CHAUVINISM_SCORE` in the Contradiction system, whose restraint levels ADR208 R4
**reopened** rather than confirmed, and `chauvinism` accrued on org→labor-aristocracy `MEMBERSHIP`
edges (`_accrue_chauvinism`, `src/babylon/engine/systems/reactionary.py:293-312`). The essay does
not rule on either; it does supply the reason the question is live.

### 1.3 The materialist oppression criterion

The essay's definitional move replaces the superlatives with one measurable relation:

> "A social relationship can be said to be a form of oppression if, and only if, there is an
> extractive transfer of value from the oppressed to the oppressor class."
> (digest §The materialist oppression criterion, lines 34-35)

This is an *iff*, and it is a quantity. It is the single most consequential sentence in this
dossier for the port, because it converts "who joins fascism" from a demographic question into a
**value-flow question** — the same shape as the Survival Calculus, which is also a comparison of
two computed quantities rather than a stipulated form.

### 1.4 The Four Horsemen — each mode named by what it extracts and what it acts on

The essay names four modes of oppression, each with its extraction channel and its substrate
(digest §The Four Horsemen, lines 37-56):

| Horseman | What it extracts | What it acts on |
| --- | --- | --- |
| **Misogyny** | **Reproductive labor** — sexual reproduction, domestic housework, child rearing, education — from women-as-a-class to the bourgeoisie (and to a lesser extent men) | Reproductive labor |
| **Eugenics** | An **attempt** (emphatically pseudoscientific — "doesn't work") to **standardize the labor-power commodity** by eliminating "defective" workers; capital wants predictable, standardized labor-power so it can plan maximal exploitation | The **quality distribution** of labor-power |
| **White supremacy** | Du Bois's **wages of whiteness** — a psychic wage plus sometimes a real wage differential — pacifying white labor via racial hierarchy while pacifying Black/Indigenous/non-white labor "through brutality, policing, and genocide", yielding a docile labor pool | **LABOR** |
| **Settler colonialism** | "brazen and bloodthirsty theft of land"; white workers and white bourgeoisie "cross the class divide and collaborate over divvying up the spoils of looted Native lands" | **LAND** |

The **labor/land axis split between horsemen 3 and 4 is explicit in the essay** (digest lines
54-56) and is the single most directly implementable structural claim in it: white supremacy's
mechanism lives on class nodes and wage/exploitation edges, settler colonialism's lives on
territory/hex tenure, and the engine already separates those substrates (digest §Simulation
consequences, lines 69-72). A fascist bloc's offer is therefore **a sum over distinct substrate
channels, not one scalar**.

### 1.5 The join calculus

The essay's account of *who joins* follows from §1.3 by arithmetic rather than by a separate
theory. A stratum integrates into the bloc when what it receives from the bloc's extraction
exceeds what the bloc's own internal hierarchies extract from it. The essay states it twice, in
the course of explaining women's participation in fascist projects:

> "the value that they could extract from integration and assimilation into the war machine …
> outweighed the value extracted from them from patriarchy"

> "the share of the spoils of exploitation that they participate in outweigh what is extracted
> from them"
> (both: digest §The Four Horsemen, lines 42-45; the named instances are Auschwitz guards and IDF
> women — "a stolen house built on stolen Palestinian land")

Three properties of that calculus matter for the port:

- **It is a net expected transfer, not a fixed demographic weight** (digest lines 60-64). Nothing
  about a stratum's identity settles its eligibility; the arithmetic does.
- **Cross-class capture is expected, not anomalous** (digest lines 73-75). Settler cross-class
  collaboration is the paradigm case, so eligibility generalizes to any stratum with positive net
  transfer — which is also where the essay lands closest to Sakai's warning that fascism recruits
  across lines.
- **`entitlement` gets a derivation.** The scalar `reactionary.py` reads is, on this frame, "the
  stored expectation of bloc-side transfers (wages of whiteness + settler spoils + patriarchal
  position) — eventually computable from the value-flow estate rather than seeded" (digest lines
  65-68).

The rest of this dossier is written under that frame: §2 states what the code does, §3 states the
two load-bearing code findings, §4 re-derives the parameter mapping as a net-transfer computation,
§§5-7 read the corroborating literature through it, §8 carries the stratum-by-stratum table, and §9
puts thirteen reserved questions to the Director as option spaces.

---

## 2. The question, and what the code answers today

`FascistFactionSystem` (`src/babylon/engine/systems/reactionary.py`, position 17.4) answers one
question every tick: **which class nodes drift toward fascism, and which of them get captured?**
Today it answers with four inputs and one gate:

- **eligibility gate** — `_ENTITLED_ROLES` (`reactionary.py:54-56`) is
  `{SocialRole.LABOR_ARISTOCRACY, SocialRole.COMPRADOR_BOURGEOISIE}`; every other
  `SocialRole` (`src/babylon/models/enums/social.py:35-43`) is skipped at `reactionary.py:123-125`
  before any dynamics run.
- **agitation** — read from the node's nested `ideology.agitation`
  (`reactionary.py:127`, `_agitation_of` at `:350-354`), written the same tick by
  ConsciousnessSystem @17.
- **entitlement** — a per-class scalar (`reactionary.py:128`), declared on the model at
  `src/babylon/models/entities/social_class.py:450` with role defaults at `:41-46`
  (`LABOR_ARISTOCRACY: 0.8`, `COMPRADOR_BOURGEOISIE: 0.7`,
  `PERIPHERY_PROLETARIAT: 0.2`, `LUMPENPROLETARIAT: 0.0`) mirrored in
  `src/babylon/config/defines/reactionary.py:102-112`.
- **incident SOLIDARITY** — the strongest incident `EdgeType.SOLIDARITY` strength
  (`_incident_solidarity`, `reactionary.py:193-201`).

They combine in `calculate_fascist_pull` (`src/babylon/formulas/reactionary.py:33-67`):
`pull = agitation × (entitlement / (solidarity + ε))`, thresholded by
`fascist_pull_threshold` into a `fascist_alignment` drift step, and at
`fascist_recruitment_threshold` into capture by the lowest-id fascist faction
(`reactionary.py:137-177`).

The port is the moment to decide whether that set of inputs is the right **parameterization of
recruitment**. Four pressures make the question live now:

1. **The faction enum is minted and parameterized** (ADR208 R1): the 5-member list
   RESTORATIONIST / SOCIAL_DEMOCRATIC / DECOLONIAL / LIBERAL_IMPERIAL / **FASCIST_VEHICLE**, each
   member carrying a declared parameter row (capturable?, spoiler pole, capture-vehicle flag), with
   a mutable capture-state flag layering beside the static enum. The standing doctrine is recorded:
   **fascism is parameter values on shared faction machinery, never a special-cased code path.**
   The ruling also kills the four substring-match token homes at mint — including
   `_FASCIST_IDEOLOGY_TOKENS` (`reactionary.py:71`) and its identical twin in
   `src/babylon/engine/systems/allegiance.py:77`, which match `"settler"` inside
   `"anti-settler abolitionism"`.
2. **The veteran question was ruled nation-conditioned** (ADR208 R10): settler-nation veteran share
   raises fascist-recruitment propensity through **entitlement seeding**; oppressed-nation veteran
   share does not. That ruling makes `entitlement` the declared seam for population-level
   recruitment inputs and makes nation-conditioning a first-class parameter, not a special case.
3. **One term of the existing formula family is inert** (§3, finding b).
4. **The Director's frame gives `entitlement` a derivation** (§1.5), which means the port is
   choosing between a seeded scalar and a computed sum — and can choose the seeded scalar
   deliberately, with the derivation recorded as the thing it approximates.

---

## 3. Two load-bearing code findings

These two are the findings that survive any framing choice. They are stated here, early and
unhedged, because each of them is a place where the code silently contradicts a theory the
Director has named as binding.

### 3.1 Finding (a): Sakai's core stratum is structurally unrecruitable today

`SocialRole.LUMPENPROLETARIAT` carries `entitlement` default **`0.0`**
(`src/babylon/models/entities/social_class.py:45`, inside `_ENTITLEMENT_DEFAULTS` at `:41-46`,
mirrored in `src/babylon/config/defines/reactionary.py:102-112`). The pull formula is
multiplicative — `return agitation * (entitlement / (solidarity + epsilon))`
(`src/babylon/formulas/reactionary.py:67`) — so a node with `entitlement = 0.0` has pull **exactly
zero at any agitation, forever**. It is also outside `_ENTITLED_ROLES` and so never reaches the
formula at all (`reactionary.py:54-56, 123-125`). The declassed are instead routed to *undirected
disorder*: `calculate_spontaneous_riot_risk(volatility, discipline)`
(`formulas/reactionary.py:94-117`), consumed only by `StruggleSystem`
(`src/babylon/engine/systems/struggle.py:485-505`), with `volatility` defaulting to `0.8` for
lumpen (`social_class.py:47-49`).

Sakai's base is "middle class and declassed men" and "the long-term unemployed or declassed out of
the working class" (`kersplebedeb.com/posts/shock/`). **In the code as written, Sakai's core
stratum can never join fascism; it can only riot.**

**What changes under the essay's frame:** this stops being only a code gap and becomes a
**theoretical tension that has to be surfaced**. The essay's criterion is a *net transfer*
computed over channels (§1.4-1.5), and the channels are separable: a declassed settler who has
lost the wage may still hold the land-channel claim (tenure, MIM's home-ownership threshold) and
still occupy the favored side of the racial labor-market differential. Under a
**difference-of-sums** the land term survives the wage term's collapse; under the frozen
**product**, a zero in one factor annihilates everything the other channels say. So the essay's
calculus would give lumpen and declassed strata a **nonzero bloc-side offer in some
configurations** — precisely the configurations the current formula cannot express.

Two honesty caveats. The essay does *not* name the declassed as a stratum; the sentence above is a
derivation from its criterion, not a quotation, and it is put to the Director as Q2 rather than
asserted. And the frozen behavior is not a defect to repair mid-port: ADR183 sequences frozen
transcription first, machinery generalization after.

### 3.2 Finding (b): the threatened-stake amplifier is inert, with no production consumer

`calculate_entitlement_effective` (`src/babylon/formulas/reactionary.py:120-144`) implements
`effective = clamp(base + threat_gain × threat × (1 − base), 0, 1)` — "a threatened stake reacts
harder", with `entitlement_threat_gain` already declared in defines
(`src/babylon/config/defines/reactionary.py:121-125`). Its only references outside its own module
are the `formulas/__init__.py` re-export (`:141`, `:268`) and
`tests/unit/formulas/test_reactionary.py` (`:20`, `:91`, `:97-98`) — **verified 2026-08-17 by
direct grep over `src/` and `tests/`; there is no production consumer.**

The inertness is *declared*, not accidental: ADR054 records it as a deliberate deferral — the
formula and coefficient "ship DEFINED and DOCTESTED — satisfying FR-015 (the formula must EXIST) —
but are NOT wired into the drift math. Wiring them would change canonical dynamics and break the
byte-identical baseline … Do NOT delete; the formula/coefficient are the ready substrate for the
threat-amplification wiring" (`ai/decisions/ADR054_spec_071_reactionary_subject.yaml:113`).

This matters because the literature is unanimous that the **threat to the stake**, not the stake
alone, is the driver (§5.2). What the formula lacks is a **`threat` producer**. Under the essay's
frame the gap is sharper still: `threat` is the *expected change* in the net transfer, so a threat
term is not an add-on to the entitlement scalar but the derivative of the very quantity
`entitlement` approximates. The port either wires it (with a named producer and a §6.5 ceremony) or
declares it dead — §9 Q3 holds the option space.

---

## 4. Re-deriving the parameter mapping as a net expected transfer

### 4.1 The quantity

Under §1.3's criterion and §1.5's calculus, fascist-capture propensity for a stratum *s* offered
integration into a bloc *B* is a **net expected transfer**:

```
NET(s, B) = Σ_c  T_in(s, B, c)          # the share of bloc-extracted value flowing TO s,
                                        # summed over distinct substrate channels c
          − Σ_h  T_out(s, h)            # what the bloc's own internal hierarchies extract FROM s
```

*s* integrates when `NET(s, B) > 0` — "the share of the spoils of exploitation that they
participate in outweigh what is extracted from them" (digest lines 43-45). Three properties are
load-bearing and all three are structural, not numerical:

- **It is a difference, not a product.** Channels add; the bloc's internal extraction subtracts.
- **It is a sum over substrates.** Because the horsemen act on *different* substrates — LABOR vs
  LAND vs reproductive labor vs the quality distribution of labor-power — the terms are not
  commensurable by accident; they have to be brought to a common unit deliberately (value, per the
  criterion). The digest is explicit that the offer is "a SUM over distinct substrate channels, not
  one scalar" (lines 69-72).
- **It is an expectation.** `T_in` is *expected* transfer, which is why a threat to the stake (§3.2)
  is a change in the quantity itself rather than a separate mechanism, and why the essay's
  criterion is about the *offer* rather than the current wage.

### 4.2 The channels, and where each one lives in the substrate

| Channel (horseman) | Acts on | `T_in` — what the bloc offers *s* | Substrate home today |
| --- | --- | --- | --- |
| **Wages of whiteness** (white supremacy) | **LABOR** | The wage differential a settler stratum holds over the racially subordinated pool, plus the psychic wage | **Partial.** `social-class/wages-received` and `social-class/wage-balance` — the per-class Fundamental-Theorem verdict written by `p4-wage-balance` in `rust/crates/babylon-tick/content/rules/consciousness.bsl` (write at `:236`; `wages-received` bindings at `:247`, `:324`); `super_wage_bonus` on `EdgeType.WAGES` (`reactionary.py:314-323`); per-class Φ via `social-class/imperial-rent` and ImperialRentSystem. **The differential itself is a cross-class comparison and no field holds it.** The psychic wage has no quantity at all. |
| **Settler spoils** (settler colonialism) | **LAND** | Tenure share of looted land and the rent stream from it | **Partial.** `HexTenureComposition` (`src/babylon/domain/economics/substrate/types.py:88`; ADR208 R19 adds an agricultural share, sum-to-1.0 validator updated); `territory/rent-level-x1e6` and `territory/under-eviction` in `rust/crates/babylon-tick/content/rules/territory.bsl` (`:95-96`, `:105-106`); `TerritoryType/RESERVATION` / `PENAL_COLONY` / `CONCENTRATION_CAMP` (`territory.bsl:119-121`); the dispossession channel (`src/babylon/engine/systems/dispossession_events.py`, `content/rules/dispossession.bsl`). MIM's membership threshold names exactly this channel: "You can't be a settler without a house" (`where.buffalo.roam`:13-14). **No transfer term joins the tenure share to a class's expected receipts.** |
| **Patriarchal position** (misogyny) | Reproductive labor | The share of extracted reproductive labor accruing to the stratum ("and to a lesser extent men", digest lines 39-41) | **Cost side only — no transfer.** `community_cost_modifier` (`social_class.py:442-446`) is computed by `compute_community_cost_modifier` (`src/babylon/formulas/community.py:144-174`) as the product of each membership's `reproduction_cost_modifier`, written by CommunitySystem (`src/babylon/engine/systems/community.py:617-621`); the memberships exist — `CommunityType.PATRIARCHAL` (hegemonic) and `WOMEN` ("reproductive labor allocation", marginalized) / `TRANS` at `src/babylon/models/enums/community.py:22-24, 40, 45-46`. The `ReproductionRequirements` tensor carries a reproductive-labor-hours term but its loader is a **stub returning `NoDataSentinel`** — "CEX data source pending constitutional amendment (US4 deferred loader)" (`src/babylon/domain/economics/tensor_hierarchy/reproduction.py:27`, `DefaultReproductionSource` at `:123-145`) — with gendered ATUS seed data present and unwired (`src/babylon/data/atus/seed_data.yaml`, gender codes at `:27-30`). |
| **Labor-power standardization** (eugenics) | The **quality distribution** of labor-power | Not an offer to a stratum — an act *by* the bloc on the distribution | **Absent.** The nearest surfaces are `CommunityType.DISABLED` ("Built environment assumes able-bodiedness", `community.py:27`, `:48`) — a membership, not a dispersion — and per-class wealth/vitality scalars. No within-class quality distribution exists anywhere in the tree. |

`T_out` — what the bloc extracts back from *s* — has one clear home and one interpretive one. The
clear one is the same channel arithmetic run in reverse: a stratum inside the bloc that is *also*
on the extracted side of a channel (the essay's IDF women, its Auschwitz guards) has a nonzero
`T_out`, and for the patriarchal channel the tree's only representation of it is the raised
reproduction cost above. The interpretive one is discussed next.

### 4.3 The frozen pull formula is the approximation of that computation

The port does not have to compute `NET` to be faithful. It has to know what the frozen formula is
approximating, because that is what makes the parameter rows legible. Term by term:

| Frozen term (`formulas/reactionary.py:33-67`) | What it approximates under §4.1 | Fidelity of the approximation |
| --- | --- | --- |
| `entitlement` | `Σ_c T_in` — the stored expectation of bloc-side transfers across all channels (the digest says exactly this, lines 65-68) | **Collapses the channel sum into one hand-set scalar per role** (`social_class.py:41-46`: 0.8 / 0.7 / 0.2 / 0.0). Structure-destroying but sign-preserving: the ordering LA > comprador > periphery > lumpen is the ordering the channel sum would produce for the labor channel alone. |
| `solidarity` (denominator) | The recognition of `Σ_h T_out` | **Weakest link, and it is a proxy for recognition rather than for magnitude.** Incident SOLIDARITY (`reactionary.py:193-201`) measures a stratum's alignment *with the strata the bloc extracts from*; it does not measure what the bloc extracts from the stratum. Read charitably it encodes "the extracted side is legible to me, so the offer does not net positive". Read strictly it is a different quantity in the same slot. Recorded as an interpretation, not a claim about intent. |
| `agitation` | The gate on *acting* on the expectation | The essay's calculus is about a standing offer; a stratum moves when the offer is disturbed. Agitation (written by ConsciousnessSystem @17, read at `reactionary.py:127`) stands in for "the transfer is in question this tick". Its crisis-gating property is exactly right in shape: zero agitation → zero pull → hegemony holds. |
| `ε` (`solidarity_pull_epsilon`) | Nothing in the calculus | A numerical guard that also sets the maximal unsuppressed pull. |
| the **product** form | a **difference of sums** | **The one structural mismatch.** A product annihilates on any zero factor; a difference of sums does not. This is finding (a) (§3.1) in its general form, and it is the reason the entitlement-0.0 lumpen row is a theory question and not only a defaults question. |

Nothing in this table argues for changing the formula. ADR183 sequences frozen transcription first;
ADR173's ruling 5 forbids *imposed* functional forms, which cuts against replacing one stipulated
shape with another stipulated shape as much as it cuts against the sigmoid. What the table gives
the port is the ability to write the parameter rows knowing what they mean.

### 4.4 How R1's parameter rows carry per-stratum transfer terms

ADR208 R1 gives each faction-enum member "a declared parameter row (capturable?, spoiler pole,
capture-vehicle flag)". The net-transfer frame extends that row for `FASCIST_VEHICLE` in a way that
keeps the generalized-factions doctrine intact — **fascism as parameter values, never special-cased
code**:

- **Per-stratum rows, not per-stratum code.** For each capturable stratum, a declared row of
  **channel weights** — a labor-differential term, a land-spoil term, a patriarchal-position term —
  plus a bloc-side extraction term standing for `Σ_h T_out`. Every other faction member gets the
  same row shape with its own values, which is what makes the machinery general: DECOLONIAL's row
  resolves NEVER-CAPTURABLE (R1's row-9 constraint) as a *value*, not a branch.
- **The veteran ruling becomes one additive term.** ADR208 R10's nation-conditioned veteran wiring
  stops being a special case and becomes a **seed contribution to the labor channel**:
  settler-nation veteran share raises the bloc-side expectation, oppressed-nation share does not.
  The seam the ruling already names — per-class entitlement seeding, which
  `calculate_fascist_pull` reads at `reactionary.py:127-135` — is the labor-channel seed slot in
  this shape. The nation side is `SocialClass.community_memberships` (`social_class.py:438`) fed by
  the ADR171 Phase 0 per-county × pole incidence artifact (which ADR208 R29 keeps **open**: #334's
  chartered artifact exists nowhere).
- **Attributed membership is the declared carrier for a per-(stratum, community) term.** Amendment
  AG / ADR189 makes the `(member, hyperedge)` pair a first-class payload-carrying element kind, so
  a per-membership transfer payload is expressible without minting a primitive. That is the only
  currently-legal home for a patriarchal-position term keyed to `CommunityType.WOMEN` /
  `PATRIARCHAL` (§9 Q11c).
- **Row minting is hash-bearing.** Declaration order is hash-bearing under ADR195, and any
  parameter that enters the tick moves baselines, so every landing here is a declared §6.5 baseline
  ceremony with its drift table (`tools/generate_ceremony_message.py`).
- **Wiring is a typed motion.** Connecting any of these to a producer is an ADR109 typed motion
  (W-C dataflow for a channel read, W-P projection for a seed) and owes its sentinel row.

---

## 5. The corroborating literature: MIM and Sakai

Read through §1: MIM supplies the empirical content of the **labor and land channels** in the U.S.
case, and Sakai supplies the account of what happens to a stratum whose channel receipts are
falling. Neither is the frame; both are evidence for it.

### 5.1 The stake, not the wound

MIM's account inverts the liberal one at the root — and it is the same inversion the essay's
criterion performs, arrived at from the other direction. Fascist recruits are not marginal deviants
and white workers are not victims of white supremacy: they are its beneficiaries. MIM says so about
the very literature that says otherwise —

> "these efforts tend to look at the masses of white supremacists as alienated deviants,
> manipulated and duped by greater powers. According to this romantic (and common) view, working
> class whites don't benefit from white supremacy, but are themselves victims of it."
> (`/media/user/data/mim/etext/bookstore/whitenation.html`, lines 36-39)

and faults the book under review for missing "the roots of racism, national oppression and the
material basis for fascism in Amerika" (`whitenation.html`, lines 30-33). The recruits are
ordinary employed workers: MIM endorses the warning of "the increasing tendency toward openly
fascist organization among white workers, most of them originally 'normal' people, not freaks"
(`whitenation.html`, lines 48-51). The recruit's own account is a defense of a possession:

> "We're just common people, working class people, everyday all-American people … and we've
> realized that the only thing we've got to thank for the position we're in is our white culture,
> and we're not going to let it be destroyed by any sub-human trash."
> (`whitenation.html`, lines 54-58 — a Nazi tool-and-die worker in a Michigan auto plant)

The organizational base is named without hedging. A former Klan leader: converts "will come from
the working class, and that's where our strength is even today. When we had 2,000 members of the
Klan in Michigan back in 1970, the bulk of our people came out of the auto factories … that's not
the upper class, that's the working class" (`whitenation.html`, lines 61-66). Electorally, Wallace
"was 'pro-labor' for white people, and the Southern white working class supported him almost
entirely. He won 77% of all working class votes in Birmingham, Alabama in the 1968 election"
(`whitenation.html`, lines 73-76), while organized labor itself supplied the terror: the 1956
Montgomery carpenters' union gallows bore the sign "Built by Organized Labor"
(`whitenation.html`, lines 82-84).

The base is a **nation**, not a fringe: "it is not just the Amerikan ruling elites which benefits
from their oppression of indigenous nations. Rather, throughout the history of Amerika, it has
been the vast majority of Amerikan citizens, often led by the white working class, who have
vehemently fought to run indigenous nations out of existence"
(`/media/user/data/mim/etext/MIM.essays/indigenous.unity`, lines 53-58); the genealogy starts with
poor whites, not elites (`500.years.white.unity`, lines 19-25). This is the essay's **cross-class
settler collaboration over "the spoils of looted Native lands"** in MIM's own vocabulary — the land
channel, with the same actors. Within that nation MIM still names an internal hierarchy of stake —
"managers and labor-aristocrats who can afford to purchase cooperatives and condominiums"
(`/media/user/data/mim/etext/MIM.essays/where.buffalo.roam`, lines 43-46) — and a material
threshold for membership: "The key to full rights as an Amerikan citizen is home-ownership. You
can't be a settler without a house." (`where.buffalo.roam`, lines 13-14). On the net-transfer
reading, that threshold is a **land-channel gate on `T_in`**.

### 5.2 Triggers: threatened or insufficient parasitism

The named triggers are all *threats to the stake*, not immiseration — i.e. all of them are
disturbances to expected `T_in`, which is why finding (b) (§3.2) is load-bearing:

- **Labor-market competition.** "the white labor movement leads a massive fight to keep their jobs
  at hundreds of times the wages of Third World workers, and keep immigrant competition away. They
  clamor for war and conquest abroad, and for the repression of revolutionary movements among the
  oppressed in the ghettos, barrios and fields within Amerika."
  (`500.years.white.unity`, lines 79-84)
- **A racial labor settlement broken from above.** Southern union failure was "largely due to the
  national leadership's shift toward integrationism" (`whitenation.html`, lines 73-78).
- **Squeezed superprofits.** MIM's paradigm case is Germany 1931-4: "the profit rate was actually
  negative … it drove the Germans to the most desperate outlook, because it lacked colonial
  exploitation to fall back on … the labor aristocracy gladly paid the taxes that boosted military
  production profits … the labor aristocracy rejected the communist way out"
  (`/media/user/data/mim/etext/mt/imp97/imp97c1.html`). The contemporary form: demagogues
  "fanning the arrogance of the labor aristocracy and when those chauvinist workers realize they
  need the colonies they will support us in all-out war" (same file).
- **Globalization eroding the national bribe.** "The more capital forms one interpenetrating whole
  … the more the spokespeople for the labor aristocracy protest — Le Pen, Jospin, fascists,
  AFL-CIO leaders etc." (`/media/user/data/mim/etext/mt/imp97/imp97c6.html`).

Two MIM claims bear directly on how a sim should *shape* the mechanism. First, class structure —
not tactics — decides the valence of a mobilization: "The same strategy and tactics applied where
the class structure is one thing results in one kind of communist movement. In another situation
with a different class structure, the same strategy and tactics unleash fascism."
(`imp97c6.html`). Second, fascism is a **continuum, not a category**: escalating state-repressive
collaboration is "in other words movements toward fascism, even if in a less openly violent guise"
(`whitenation.html`, lines 493-496), and falling membership counts prove nothing — "In the
mid-1920s there were 3-4 million Klan members. Now there are less. But is white supremacy any
weaker? Ask Rodney King." (`whitenation.html`, lines 94-96). **This is the closest MIM comes to the
Jackson frame, and it is not the same claim**: MIM's continuum is a scalar degree of fascization,
Jackson's is a three-phase cycle with a phase that *reduces* visible repression. Q5 and Q13 hold
that difference open.

MIM also names two strata a simulation would otherwise miss. Repressive-apparatus employment is a
labor-aristocratic sinecure with fascist indoctrination bundled in: prison labor "provides jobs to
the labor aristocrats who guard them even as it indoctrinates them with fascist ideology"
(`/media/user/data/mim/etext/mt/mt11labor.html`), with "the reactionary middle class and labor
aristocracy, clamoring for more and more punishment" as its constituency (same file) — on the
net-transfer reading a **labor-channel `T_in` paid directly out of the repression budget**. And the
funding/leadership layer is analytically distinct from the base: Ford as "main publicist" of
conspiracy material, whom Hitler called "the leader of the growing fascist movement in America"
(`whitenation.html`, lines 40-45) — with MIM explicitly faulting over-credit to that layer at the
base's expense (lines 40-51).

Finally, MIM draws an exclusion line the eligibility gate has to respect one way or the other:
immigrant Third World workers inside imperialist countries cannot be folded into the base, because
"Even the bought-off immigrant worker knows that his/her position is more fragile than that of the
oppressor nation workers" (`/media/user/data/mim/etext/mt/imp97/imp97b4.html`). Note the form of
that argument: it is a claim about the *magnitude and reliability* of the offer, which is a
net-transfer claim, not a claim about identity.

### 5.3 Sakai: declassing, status loss, and payback

Sakai shares the settler frame but locates the engine elsewhere — in **dislocation out of
productive life**:

> "Fascism is a revolutionary movement of the right against both the bourgeoisie and the left, of
> middle class and declassed men, that arises in zones of protracted crisis."
> (`kersplebedeb.com/posts/shock/`, J. Sakai, "The Shock of Recognition")

> "It is the classes dislocated out of productive life, the humiliated layers of middle class men
> who are angry and frightened, who feel they have nowhere to turn to restore their
> status…except towards fascism." (same)

What fascism offers is not relief but restoration: "To the increasing mass of rootless men fallen
or ripped out of productive classes – whether it be the peasantry or the salariat – it offers not
mere working class jobs but the vision of payback" (same). His recruitment list is explicit —
"urban small traders and businessmen, craftsmen and foremen, junior military officers, significant
parts of the peasantry (small farming landowners), petty government civil servants, the long-term
unemployed or declassed out of the working class, the police and criminals" (same) — and the
politics is "anti-bourgeois but not anti-capitalist. Because it is based on fundamentally
pro-capitalist classes" that want "basic capitalism, o.g. capitalism" restored (same). The Nazi
cadre data he cites points the same way: party cadres came "almost exclusively from the lowest of
the middle classes (office workers, petty civil servants, self-employed craftsmen and traders),
not from either the main middle classes or industrial workers" (same).

Where Sakai converges hardest with MIM is on the **contracting labor aristocracy** — a falling
labor-channel `T_in`:

> "Timothy McVeigh can't be the real white man his father was, because the lifelong, high paying,
> industrial labor aristocracy of the steel mills and auto plants is shrinking not expanding. …
> Men who are robbed of having a place and as a class can't go forward and can't go backward. Who
> are at an end." (`kersplebedeb.com/posts/shock/`)

`Settlers` dates the material turn: "Real wages in the U.S. began to stagnate in 1967, when
imperialism ran aground on the Vietnamese Revolution" (`readsettlers.org/ch13.html`), with the
first response adaptive rather than radical — the two-wage-earner family, "75% of the U.S.
families with incomes over $25,000 per year had two wage-earners" by 1978 (same). Reaction is
pre-organized, not built from scratch: "Tens of millions of settlers are organized into special
reactionary groupings of the most diverse kinds … the AFL craft unions, 'ethnic' organizations …
the NRA … thousands of neighborhood 'Improvement Associations'" (same), on a substrate of
privatized consciousness — "The most significant fact about the real consciousness of the
Euro-Amerikan masses is how anti-communal and private it is" (same). And mobilization fires on
visible threat: "In April, 1968, Martin Luther King, Jr. was assassinated in Memphis. Detroit blew
up - and settler Detroit armed up." (`readsettlers.org/ch14.html`).

---

## 6. Where the essay, Sakai, and MIM diverge — stated honestly

Three genuine divergences, all of which land on parameters. Each now carries the essay's position
where the essay takes one — and a note where it does not.

### 6.1 The declassed: core for Sakai, out of scope for MIM, *arithmetic* for the essay

Sakai's base is "middle class and declassed men," "the long-term unemployed or declassed out of the
working class" (`shock/`). MIM's base is defined by *possession* of the bribe — the whole-nation
thesis and the "bought-off white working class"
(`/media/user/data/mim/etext/MIM.essays/american.leninism`, lines 69-73). In our schema the two
accounts point at different node sets: MIM at `LABOR_ARISTOCRACY` with high `entitlement`; Sakai
additionally at `LUMPENPROLETARIAT`, `INTERNAL_PROLETARIAT`, and `PETTY_BOURGEOISIE`, whose
`entitlement` defaults are `0.0` or absent (`social_class.py:41-46`). Under the current
multiplicative formula, an `entitlement = 0.0` node has pull `0.0` at any agitation: **Sakai's core
stratum is structurally unrecruitable in the code as written** (§3.1).

**The essay's position is a third one, and it is neither side's.** Its criterion is agnostic to a
stratum's current class position and asks only about the offer: `NET(s, B) > 0`. Because the
channels are separable (§4.2), a stratum can lose the labor channel entirely and retain the land
channel — tenure, MIM's own home-ownership threshold — and the calculus then returns a positive
offer for a declassed settler. So the essay **licenses a nonzero bloc-side offer to declassed
strata in some configurations**, which is Sakai's conclusion reached by MIM's mechanism.

**Recorded rather than resolved:** the essay never names the declassed, so this is a derivation
from its criterion and not an authority for a code change. It is put to the Director as Q2. And it
does not resolve MIM's *scope* claim — MIM is arguing about who the base *is*, empirically, in the
U.S.; the essay is arguing about what makes anyone joinable. Both can be true, and the sim has to
choose which one the eligibility gate encodes.

### 6.2 The color line: the essay sides with Sakai on form, with MIM on content

Sakai: "there is no reason to view fascism as necessarily white just because there are white
supremacist fascists … fascist potentials exist throughout the global capitalist system," and
concretely, U.S. white fascist bands found "political brethren in the Muslim world. Politics is
thicker than blood." (`shock/`). MIM's exclusion of immigrant Third World workers from the base
(`imp97b4.html`) is a claim about the *bribe*, not about blood.

**The essay's position:** formally with Sakai — capture propensity is "a NET EXPECTED TRANSFER …
not a fixed demographic weight", and eligibility "generalizes to any stratum with positive net
transfer" (digest lines 60-64, 73-75), which is exactly Sakai's denial of race-fixity expressed as
a formula. Substantively with MIM — what makes settler strata net-positive *is* the wages of
whiteness and the settler spoils (digest lines 50-56). The essay therefore dissolves the
disagreement into **one formula with different inputs** rather than picking a side.

**Recorded rather than resolved:** the essay's own instances of cross-line participation
(Auschwitz guards, IDF women) are strata oppressed on *one* axis joining on the strength of
*another* — which is Sakai's structural point, and which is also the hardest case for a hard
eligibility block. MIM's fragility claim survives inside the essay's frame as a claim about the
**magnitude and reliability of the offer**, i.e. as a penalty on the seed rather than a categorical
exclusion. Q6's option space carries both readings; R1's DECOLONIAL-never-capturable row is a
constraint on the *faction* side, not on this question.

### 6.3 Gender: the essay decides it, against Sakai's compositional claim

Sakai treats fascism as constitutively male: "It is a male movement, both in its composition and
most importantly in its inner worldview … Fascism exalts this, and makes of women a semi-slave
resource of the State" (`shock/`). MIM instead folds white women into the same chauvinist base,
citing the suffragist claim that "the White women's vote would give supremacy to the white race"
(`500.years.white.unity`, lines 66-69).

**The essay's position — and this is the one place it rules on a divergence rather than reframing
it.** Women's participation is the essay's *worked example* of the join calculus: "the value that
they could extract from integration and assimilation into the war machine … outweighed the value
extracted from them from patriarchy" (digest lines 42-44), with Auschwitz guards and IDF women as
the named instances. So the essay sides **with MIM on composition** — women join, and the reason is
arithmetic — while keeping Sakai's point about what fascism *does* to women, since misogyny is one
of the four horsemen and its extraction channel (reproductive labor) is a standing feature of the
bloc, not an incidental one.

**What this changes for the sim.** Q8 in the first draft was "which literature does the engine
follow, given no gender surface". Under the essay it is no longer a choice between authorities; it
is a question about **whether the patriarchal channel gets a representation at all** — and the
honest finding (§4.2) is that the tree has *partial, cost-side, unwired* machinery, not nothing:
`CommunityType.PATRIARCHAL` / `WOMEN` / `TRANS` memberships, `community_cost_modifier`, and a
stubbed `ReproductionRequirements` reproductive-labor term. That is a **correction to the digest's
"no current substrate home"** (digest lines 80-83), and Q8 is re-posed alongside the new Q11
accordingly.

---

## 7. Classical-Marxist corroboration (subordinate)

Kept deliberately subordinate to §§1, 5-6, and free of Dimitrov and popular-front framing — for
the reason §1.2 gives, not merely by directive. Note that the reader corpus flagged the same
exclusion: MIM's own definitional theses quote **Dutt**, and the finding explicitly excludes the
Dimitrov citation sitting in the same thesis
(`/media/user/data/mim/etext/wim/cong/fascismdef.html`, thesis 4 note).

**Zetkin (1923)** names two material roots. Economically, war-declassing: "Large numbers of the
former middle classes have become proletarians, having entirely lost their economic security.
Their ranks were joined by large masses of ex-officers, who are now unemployed. It was among these
elements that Fascism recruited quite a considerable contingent."
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/zetkin/1923/08/fascism.htm`, line 34)
— the shock hitting "the petty bourgeoisie, the small peasantry and the intellectuals" alongside
the proletariat (same line). Politically, disillusion: masses "have now lost their faith not only
in the reformist leaders, but in socialism as a whole … joined by large circles of the
proletariat, of workers who have given up their faith not only in socialism, but also in their own
class. Fascism has become a sort of refuge for the politically shelterless." (same file, line 36).
She assigns the Communist parties part of the blame — "our actions at times failed to stir the
masses profoundly enough" (line 36) — which makes the base contestable rather than fixed: "we must
endeavour either to win over or to neutralise those wide masses who are still in the Fascist camp"
(line 48). Once in power, fascism's own betrayal of that base (rent control abolished, tax burden
shifted onto small peasants) reopens the antagonism inside its ranks
(`…/prolewiki/Exports/Library/Library_Fighting Fascism_…_Fascism s contradictions.txt`,
lines 47, 49) — which under §4.1 is a `T_in` the bloc promised and did not deliver. **Sourcing
caveat recorded by the reader:** the enumerated recruitment account is absent from the three
prolewiki chapter exports (only 5 of 18 chapters exist there); the verbatim passage was read in
Zetkin's own August 1923 Labour Monthly translation at the marxists.org path cited above.

**Dutt** supplies the routing law. Crisis produces the raw material — the "new dispossessed and
ruined middle-class elements break out as an extremely unstable, violent force potentially
revolutionary or, alternately, ultra-reactionary, without clear social basis or consciousness, but
recklessly seeking any line of immediate action" (`Library_Fascism and Social
Revolution.txt:6105-6119`), with inflation as an explicit mechanism ("a direct robbery of all
small owners … the partial expropriation of the petit-bourgeoisie", `:15615-15620`) and
professional overcrowding as another (45,000 unemployed graduates, `:6099-6103`). Structurally
these strata can only be auxiliary: "Either finance-capital … can seek to make the middle class
its auxiliary … (fascist militia, police-officer class, fascist bureaucracy). Or the proletariat
… can at last give full scope to all the useful trained and technical abilities … These are the
only two alternatives" (`:6129-6140`). Which way they tip is decided by the workers' movement:
"Where the working-class movement is strong, follows a revolutionary line … the mass of the
petit-bourgeoisie is swept in the wake of the working class … But where the working-class movement
… follows the leadership of Reformism and thus surrenders to large capital … the discontented
petit-bourgeois elements and declassed proletarian elements begin to look elsewhere for their
leadership. On this basis Fascism is able to win its hold." (`:6166-6182`; his 1925 formula,
"Fascism is the child of Reformism", `:6254-6260`). Recruitment is selective within the working
class: "All the politically backward discontented elements … petit-bourgeois, declassed elements
and backward workers, were swept into the National Socialist net. The class-conscious workers who
became disillusioned with Social Democracy passed to Communism." (`:8062-8069`). Fascism is never
spontaneous — it is "financed and directed by finance-capital" (`:5873-5878`) — and its pre-mass
nucleus is narrowly military: ex-officers and Reichswehr-linked organizers who "could not provide
the main basis, as it had no mass support" (`:2496-2503`), pensioned by the very Republic they
meant to overthrow (`:7808-7853`). His four general conditions: crisis-driven class-struggle
intensification, disillusionment with parliamentarism, the *existence* of a wide
petit-bourgeoisie/slum-proletariat/capital-influenced worker population, and the absence of
independent class-conscious proletarian leadership (`:14888-14895`).

**What this adds, and what it does not.** It corroborates §5.3's declassing channel (Sakai's, more
than MIM's) and supplies two mechanisms MIM's parasitism frame does not: **demobilized soldiers as
a named recruitment pool** (Zetkin line 34; Dutt `:2496-2503`) — the classical shadow of ADR208
R10's nation-conditioned veteran seam — and **reformist betrayal as the routing switch** (Dutt
`:6166-6182`, `:15357-15361`). It does *not* carry the imperial-rent content: Zetkin and Dutt read
the ruined petty bourgeoisie as materially *falling*, whereas MIM reads the fascist base as
materially *bribed*.

Under the essay's frame there is now a **principled** reason they stay subordinate rather than a
merely ordinal one. Zetkin's and Dutt's mechanism is entirely a change on the `T_out`/loss side —
what is being taken *from* the stratum — with no account of the bloc-side `T_in` that makes the
fascist offer preferable to the communist one. That makes their account a **partial evaluation of
the join calculus**: necessary for explaining why a stratum is in motion, insufficient for
explaining why the motion goes right rather than left. Dutt half-recognizes this and answers it
with the *absence* of proletarian leadership (`:14888-14895`) rather than with a positive offer.
Where the frames conflict, §1 governs, then Sakai and MIM.

---

## 8. Stratum-by-stratum mapping to simulation parameters

The generalization target (ADR208 R1): eligibility and entitlement dynamics become **declared
values on a faction classification**, replacing the hardcoded `_ENTITLED_ROLES` frozenset
(`reactionary.py:54-56`) and the substring probe `_FASCIST_IDEOLOGY_TOKENS` (`:71`, duplicated at
`allegiance.py:77`), which dies at mint. The table reads: stratum → what the eligibility
generalization would have to say about it → **the net-transfer reading (which channels carry its
`T_in`, and what the bloc extracts back)** → the hook that exists today. Every file/line citation
is carried unchanged from the first draft; only the third column's framing is re-derived.

| Stratum (source) | Eligibility under the R1 generalization | Net-transfer reading — channels and `T_out` | Existing sim hook |
| --- | --- | --- | --- |
| Settler labor aristocracy — MIM's white working class / "whole nation" (`whitenation.html`:61-66, `indigenous.unity`:53-58); Sakai's contracting industrial aristocracy (`shock/`, McVeigh) | **Eligible today** (`SocialRole.LABOR_ARISTOCRACY`, `reactionary.py:55`) | Both **labor** (wages-of-whiteness differential, Φ share) and **land** (tenure, home-ownership) channels positive; `T_out` near zero. The literature's triggers are all *falling `T_in`*: wage stagnation, Φ decline, immigrant competition | `entitlement` default `0.8` (`social_class.py:43`, `defines/reactionary.py:105`); `pull` at `formulas/reactionary.py:33-67`; chauvinist pressure from positive `social-class/wage-balance` × `consciousness/chauvinist-pressure-scale` in `rust/crates/babylon-tick/content/rules/consciousness.bsl` (`p4-wage-balance`, `p6-route`) |
| Comprador bourgeoisie (MIM's collaborating intermediary layer) | **Eligible today** (`reactionary.py:55`) | Labor-channel `T_in` via the imperial circuit; sits *inside* the bloc's internal hierarchy, so a nonzero `T_out` is expected and unrepresented | `entitlement` default `0.7` (`social_class.py:44`) |
| Labor-aristocratic union membership — organized labor as chauvinist agent (`whitenation.html`:82-84; `500.years.white.unity`:79-84) | Eligible via the org channel, not the class channel | The org is the *vehicle* through which the labor-channel `T_in` is defended (the "Built by Organized Labor" gallows); discipline is the counter-pressure | `chauvinism` on org→LA `EdgeType.MEMBERSHIP` (`_accrue_chauvinism`, `reactionary.py:293-312`), `super_wage_bonus` on `EdgeType.WAGES` (`:314-323`), `calculate_defection_probability` (`formulas/reactionary.py:70-91`), `RED_BROWN_COUP` at `red_brown_coup_fraction` |
| Carceral enforcers / police — MIM's indoctrinated guard sinecure (`mt11labor.html`); Dutt's "police-officer class" (`:6129-6140`); Sakai's "police and criminals" (`shock/`) | **Not eligible today** — `SocialRole.CARCERAL_ENFORCER` absent from `_ENTITLED_ROLES`, and absent from `_ENTITLEMENT_DEFAULTS` (`social_class.py:41-46`) | A labor-channel `T_in` paid straight out of the repression budget — the wage *is* the bloc's transfer; carceral expansion raises it | Produced by `DecompositionSystem` (`engine/systems/decomposition.py:89-108`, position 11.0, on `SUPERWAGE_CRISIS`); split coefficients `carceral.enforcer_fraction` / `proletariat_fraction` in defines (note: the module docstring says 30/70, canonical defines carry 0.15/0.85 — reconcile at the port, Q10) |
| Internal proletariat — Zetkin's "proletarians, having entirely lost their economic security" without proletarian consciousness (`fascism.htm`:34) | **Not eligible today** (`SocialRole.INTERNAL_PROLETARIAT` absent from both frozensets) | Labor channel collapsed by the declassing event; land channel possibly retained. Zetkin/Dutt describe only the `T_out` side, which §7 flags as a partial evaluation | The 70/85% leg of `DecompositionSystem`; `CLASS_DECOMPOSITION` event; solidarity absence via `social-class/solidarity-inbox` (`consciousness.bsl` `p6-route`, `eff-sol`) and incident SOLIDARITY (`reactionary.py:193-201`) |
| Lumpenproletariat / declassed men — **Sakai's core** ("middle class and declassed men", "long-term unemployed or declassed out of the working class", `shock/`) | **Structurally unrecruitable today**: not in `_ENTITLED_ROLES`, and `entitlement` default `0.0` (`social_class.py:45`) zeroes the multiplicative pull at any agitation | The **critical row for the frame**: a difference-of-sums with a surviving land term returns a positive offer where the frozen product returns zero (§3.1, §6.1). "Payback" is the promise of restored `T_in` | Currently routed to *undirected disorder* instead: `calculate_spontaneous_riot_risk(volatility, organization)` consumed only by StruggleSystem (`engine/systems/struggle.py:485-505`), `volatility` default `0.8` (`social_class.py:47-49`) |
| Ruined petty bourgeoisie / small traders, craftsmen, professionals (Sakai's list, `shock/`; Dutt `:5559-5566`, `:6099-6103`) | **Not eligible today** — `SocialRole.PETTY_BOURGEOISIE` in neither frozenset nor `_ENTITLEMENT_DEFAULTS` | Property-side `T_in` (small ownership) eroding; Dutt's inflation channel is a `T_out` by expropriation | `wealth` / `subsistence_threshold` (`social_class.py`, `EconomicComponent`); `politics.petty_bourgeois_liquidation_threshold` in defines; dispossession channel (`engine/systems/dispossession_events.py`, `rust/…/content/rules/dispossession.bsl`) |
| Demobilized soldiers / veterans — Zetkin (`fascism.htm`:34), Dutt (`:2496-2503`, `:7808-7853`); **ruled nation-conditioned** (ADR208 R10) | Settler-nation veterans **raise** propensity via entitlement seeding; oppressed-nation veterans **do not** — a per-class seed multiplier, not a new formula term | One **additive term in the labor-channel seed** under §4.4, conditioned on nation membership; the ruling's own grounding is that settler-nation veterans hold a labor-aristocratic position while oppressed-nation enlistment is economic coercion | Seam is `entitlement` seeding (`social_class.py:450` + role defaults `:41-46`); nation side is `SocialClass.community_memberships` (`social_class.py:438`) fed by the ADR171 Phase 0 per-county × pole incidence artifact (**still unbuilt** — ADR208 R29 keeps #334 open); **zero veteran references exist in `src/babylon/` today** (verified 2026-08-17) |
| Oppressed-nation strata — MIM excludes them from the base (`imp97b4.html`); Sakai denies race-fixity ("politics is thicker than blood", `shock/`) | Reserved: the gate can key on nation membership or on the stake (§9, Q6) | The essay's formal answer is a **computed** offer, so exclusion becomes a magnitude claim (a penalized seed) unless the faction row forbids it outright (§6.2) | `community_memberships` + ADR171 B+C+I named-nations partition; `IdeologicalProfile.national_identity` (`social_class.py:92`); ADR171 OQ9 requires the oppressed-nation-bourgeois-nationalism guard before any consciousness wiring |
| Monopoly-capital financing / leadership layer — Ford (`whitenation.html`:40-45), Dutt's finance-capital (`:5873-5878`); explicitly distinct from the base (`whitenation.html`:40-51) | Not a recruitment stratum — a separate funding/leadership channel | Not a `T_in` recipient but the **source of the extraction** the bloc redistributes; over-crediting it is the error MIM names | State-AI defines `state_ai.fascist_finance_ceiling`, `fascist_security_threshold`, `fascist_settler_ci_threshold`; `organization.tendency_modifier_fascist` |
| State repressive apparatus as such — MIM's continuum: reforms that deepen repressive collaboration are "movements toward fascism" (`whitenation.html`:493-496) | Not a class node — a *degree*, not a category (§9, Q5/Q13) | Under the Jackson frame the *degree* is not monotone in repression: a secure-in-power phase represses less visibly while extracting the same (digest lines 26-30) | `social-class/repression-faced` (`consciousness.bsl`), `consciousness.repression_level_sensitivity` / `repression_backfire` defines, `_find_fascist_faction`'s `is_settler_formation` + `colonial_stance == "uphold"` predicate (`reactionary.py:215-227`) |
| Disillusioned reformist masses — Zetkin's "politically shelterless" (`fascism.htm`:36); Dutt's routing switch (`:6166-6182`) | Not a role — a *history* on existing nodes | A `T_in` promised by the reformist channel and not delivered; §7 records that this is a loss-side account with no bloc-side offer of its own | DoctrineSystem @14.7 reformist trunk `PracticeVariable`s + liquidationism absorbing state; AllegianceSystem @17.42 valve; ElectoralSystem @17.45 T-7 disillusion routing (`politics.disillusion_window_ticks`, `disillusion_conversion_boost`) |

**Where a declassing / falling-`T_in` signal would come from.** Three existing producers, none of
them currently read by `FascistFactionSystem`:

- **`DecompositionSystem`** (`engine/systems/decomposition.py`, position 11.0) — the discrete
  declassing *event*: on `SUPERWAGE_CRISIS` the labor aristocracy splits into
  `CARCERAL_ENFORCER` + `INTERNAL_PROLETARIAT` with proportional wealth transfer and a
  `CLASS_DECOMPOSITION` event. It runs in the material base, well before 17.4, so its event and
  the new nodes are visible to the reactionary system the same tick.
- **`ReserveArmySystem`** (`engine/systems/reserve_army.py`, position 5.0) — the continuous
  labor-market pressure: territory `reserve_ratio` → `wage_pressure` → `median_wage`, published as
  `RESERVE_ARMY_PRESSURE`. This is the closest existing analogue to MIM's "immigrant competition"
  and Sakai's 1967 wage stagnation. (Its border-valve wage throttle owes a dedicated ruling —
  ADR208 R2.)
- **The wage systems** — `social-class/wage-balance` (written by `consciousness.bsl`
  `p4-wage-balance` from `wages-paid` vs `value-produced`) already encodes the Fundamental Theorem
  verdict per class, and its *positive* branch is already the chauvinist-pressure term in
  `p6-route`. Its **derivative** (falling bribe) is the literature's actual trigger and has no
  representation; `p7-persist-baselines` stores `previous-wages` / `previous-wealth`, which is the
  material for one.

The inert `calculate_entitlement_effective` (`formulas/reactionary.py:120-144`) is the declared
shape for "a threatened stake reacts harder" — `effective = base + threat_gain × threat ×
(1 − base)`, with `entitlement_threat_gain` already in defines
(`config/defines/reactionary.py:121-125`). What it lacks is a **`threat` producer**; the three
bullets above are the candidates.

---

## 9. Open Director questions (option spaces, no recommendations)

Each item is reserved line. Options are stated with engineering consequence only; no lean is
offered, and none of the readers' or the workforce's leans are carried in. Q1-Q10 are the first
draft's ten, kept and re-framed where the essay reshapes the option space without settling it;
Q11-Q13 are new, exposed by the essay.

**Q1 — What is the eligibility gate keyed on?** Today: role membership in a hardcoded frozenset
(`reactionary.py:54-56, 123-125`).
(a) Keep a closed role list, declared on the faction classification instead of hardcoded.
(b) Widen the list to declared per-role `capturable` values covering `PETTY_BOURGEOISIE`,
`CARCERAL_ENFORCER`, `INTERNAL_PROLETARIAT`, `LUMPENPROLETARIAT`.
(c) Drop the role gate for a stake gate — any class with `entitlement` above a declared floor,
which makes eligibility emergent from seeding rather than enumerated.
(d) Two declared channels with distinct formulas: an *entitled* channel (MIM) and a *declassed*
channel (Sakai), each with its own driver and threshold.
(e) A net-transfer gate — eligibility is `NET(s, B) > 0` computed from declared channel weights
(§4.4), which is the essay's own form and the most expensive: it needs the channel terms of §4.2
to exist as data.
*Essay-frame note:* (e) is the shape the essay's criterion implies; the essay does not rule on
whether the port should pay for it.

**Q2 — Are the declassed recruitable at all, and by what driver?** Sakai's core stratum has
`entitlement = 0.0` and therefore zero pull under the multiplicative form
(`social_class.py:45`, `formulas/reactionary.py:67`); today it produces riots instead
(`struggle.py:485-505`). §3.1/§6.1 record that the essay's calculus would give it a nonzero offer
in configurations where the land channel survives.
(a) Recruitment-neutral — the declassed riot, they do not join; Sakai's claim is carried as
narrative only.
(b) A second driver term (status loss / declassing recency) so a zero-entitlement node can
develop pull.
(c) History-dependent entitlement — a *lost* stake keeps a memory value, so decomposed nodes
inherit entitlement rather than resetting it (needs a new stored field on the class surface).
(d) Route declassed pull through the org/defection channel rather than the class channel.
(e) Additive channel terms so a retained land-channel claim survives a collapsed labor channel —
the difference-of-sums reading (§4.3); largest baseline impact of the five, since it changes the
formula's form and not only its inputs.

**Q3 — Does the threatened-stake amplifier wire, and what is `threat`?**
`calculate_entitlement_effective` is inert with no production consumer (§3.2), by declared ADR054
deferral.
(a) Leave it inert and declare it dead at the port.
(b) `threat` = Φ / imperial-rent decline (`social-class/imperial-rent`).
(c) `threat` = the negative derivative of `wage-balance` (needs `previous-wages` differencing,
`consciousness.bsl` `p7`).
(d) `threat` = reserve-army pressure (`RESERVE_ARMY_PRESSURE`, `reserve_army.py`).
(e) `threat` = a discrete decomposition/dispossession event flag.
(f) `threat` = the expected change in `NET` itself, i.e. the derivative of the quantity
`entitlement` approximates (§4.3) — coherent with the frame, and dependent on the channel terms
existing.
Any of (b)-(f) moves baselines and therefore owes a §6.5 ceremony.

**Q4 — Where does nation-conditioning live?** ADR208 R10 makes it a parameter; ADR171 OQ5 holds
`chauvinist_pressure` untouched and calls any national input into consciousness a Director
escalation.
(a) Per-class nation membership (`community_memberships`, `social_class.py:438`) conditions the
entitlement seed — keeps the conditioning inside seeding, as R10's evidence notes.
(b) A county-level incidence overlay (ADR171 Phase 0 artifact) conditions seeds spatially — note
ADR208 R29 keeps #334 **open**: that artifact does not exist yet, so this option carries a
blocking dependency.
(c) Faction-level: the classification declares which nations it can capture from.
(d) Some combination, with the overlap policy ADR171 declared for the B+C+I partition.

**Q5 — Category or continuum — and if a continuum, of what shape?** MIM treats mainstream
repression and organized fascism as points on one continuum
(`whitenation.html`:493-496, 94-96); the code has a discrete `fascist_recruitment_threshold`
capture. The essay's Jackson frame supplies a *third* shape: not a scalar degree but a three-phase
cycle in which the securest phase represses least visibly (§1.1).
(a) Keep the discrete threshold; the continuum stays interpretive.
(b) Keep capture but expose a continuous fascization measure on institutions/orgs as well as
classes.
(c) Replace capture with a continuous measure entirely (largest baseline and endgame impact —
`FASCIST_CONSOLIDATION` detection reads alignment today).
(d) Keep capture and add the Jackson phase as the structure of the continuum (see Q13, with which
this option is coupled).

**Q6 — Cross-line capturability.** Sakai: not necessarily white, "politics is thicker than blood"
(`shock/`). MIM: immigrant Third World workers are not part of the bribed bloc (`imp97b4.html`).
The essay is formally with Sakai and substantively with MIM (§6.2).
(a) Hard block — oppressed-nation classes are never capturable (pairs with the
`FAC_DECOLONIAL`-never-capturable constraint in ADR208 R1).
(b) Capturable with a declared penalty on the seed — the reading under which MIM's fragility claim
becomes a magnitude, not a category.
(c) Capturable only through a declared bourgeois-nationalism path (ADR171 OQ9's guard ships
first).
(d) Capturable iff the computed net transfer is positive, with no identity term at all — the
essay's own form, and the one that admits the Auschwitz-guard / IDF-women cases by construction.

**Q7 — Is elite financing a distinct mechanism?** MIM distinguishes the Ford layer from the base
and warns against over-crediting it (`whitenation.html`:40-51). Under §4.1 the financing layer is
the *source* of the value the bloc redistributes, not a `T_in` recipient.
(a) A coefficient on drift (cheapest; loses the distinction).
(b) A state/bourgeois-organization action that subsidizes a fascist vehicle (existing
`state_ai.fascist_finance_ceiling` surface).
(c) Not modeled at the port.

**Q8 — Gender: composition.** Sakai calls fascism constitutively male (`shock/`); MIM folds white
women into the same base (`500.years.white.unity`:66-69); **the essay sides with MIM on
composition, by the net-transfer calculus** (§6.3, digest lines 39-45). `SocialClass` carries no
gender surface, though `CommunityType.WOMEN` / `PATRIARCHAL` / `TRANS` memberships exist
(`models/enums/community.py:22-24, 40, 45-46`).
(a) Not modeled — composition stays literature-only; the essay's ruling is carried as narrative.
(b) A demographic surface in the lifecycle estate feeds a seed modifier.
(c) Community-membership-derived: a stratum's `PATRIARCHAL` / `WOMEN` membership mix conditions the
seed, reusing the existing community machinery rather than minting a demographic surface.
(d) Declined explicitly, with the reasoning recorded so it is not re-litigated.

**Q9 — Does reformist betrayal route into fascist pull?** Zetkin's "refuge for the politically
shelterless" (`fascism.htm`:36) and Dutt's "child of Reformism" (`:6254-6260`) name it as *the*
switch; the machinery exists (Doctrine reformist trunk, Allegiance valve, Electoral T-7
disillusion routing) and is not connected to `FascistFactionSystem`. §7 records that this is a
loss-side account with no bloc-side offer, which is why the essay's frame does not promote it.
(a) No coupling — the classical claim stays subordinate, as the source ordering has it.
(b) Disillusion events feed agitation only (already partly true via consciousness).
(c) Disillusion feeds the fascist pole directly (a new declared dataflow, ADR109 typed motion +
sentinel row).

**Q10 — Reconcile the decomposition split coefficients.** `decomposition.py`'s docstring says
30% enforcer / 70% internal proletariat; canonical defines carry
`carceral.enforcer_fraction = 0.15` / `proletariat_fraction = 0.85`. This is an engineering
discrepancy with an ideological reading (how large the carceral turn is), so it is flagged here
rather than silently normalized at the port.

**Q11 — NEW. Reproductive labor has no transfer channel: does misogyny's extraction get a
substrate home?** The essay's first horseman extracts reproductive labor from women-as-a-class
(digest lines 39-45), and the digest records the gap as "reproductive labor is not modeled"
(lines 80-83). **The honest finding is partial machinery, not none** (§4.2): a cost-side proxy
exists — `community_cost_modifier` (`social_class.py:442-446`) from
`compute_community_cost_modifier` (`formulas/community.py:144-174`), written by CommunitySystem
(`engine/systems/community.py:617-621`) over `CommunityType.PATRIARCHAL` / `WOMEN` / `TRANS`
memberships (`models/enums/community.py:22-24, 40, 45-46`) — but nothing *receives* what the cost
represents.
(a) Not modeled — misogyny stays a narrative and doctrine-level fact, with no engine term.
(b) Cost side only — the raised reproduction cost is the sole representation; no receiving side is
named, and the channel contributes nothing to `T_in`.
(c) A paired transfer — the `WOMEN`-side cost becomes a `PATRIARCHAL`-side credit, which requires
naming the receiving entity; Amendment AG / ADR189's attributed membership (the payload-carrying
`(member, hyperedge)` pair) is the declared carrier, so this needs no new primitive.
(d) Wire the `ReproductionRequirements` reproductive-labor-hours term, which is blocked: its
loader is a stub returning `NoDataSentinel` — "CEX data source pending constitutional amendment
(US4 deferred loader)" (`domain/economics/tensor_hierarchy/reproduction.py:27`) — with gendered
ATUS seed data present but unwired (`src/babylon/data/atus/seed_data.yaml`). This option therefore
carries an amendment/ruling dependency of its own.

**Q12 — NEW. Labor-power quality dispersion (eugenics' target) has no representation: does it get
one?** The essay's second horseman acts on the **quality distribution** of labor-power — an
attempt to standardize the commodity by eliminating "defective" workers (digest lines 46-49). The
tree has no within-class quality distribution anywhere; the nearest surfaces are
`CommunityType.DISABLED` membership (`models/enums/community.py:27, 48`) and per-class
wealth/vitality scalars.
(a) Not modeled — eugenics stays literature and doctrine only.
(b) A quality axis on the within-class dispersion machinery that ADR173 already mandates for
P(S|A) (the S-curve must emerge from within-class *wealth* dispersion) — reuses the arriving
machinery on a second axis, and inherits its no-imposed-forms constraint.
(c) Membership only — `DISABLED` membership stands in for the target population; a membership is
not a dispersion, and the essay's mechanism (narrowing a distribution) cannot be expressed.
(d) Represent eugenics as an **act of the bloc** rather than as a distribution: a policy/repression
effect on a marginalized community, which keeps it inside existing machinery and drops the
distributional claim.
*Note:* the essay stresses the attempt is pseudoscientific and "doesn't work" (digest lines 46-47),
so any representation has to distinguish the bloc's *intent* from its *effect* — an engineering
consequence, not a lean.

**Q13 — NEW. Does the R1 capture-state flag carry a Jackson-phase dimension coupled to
LEGITIMATION strength?** ADR208 R1 layers a mutable capture-state flag beside the static enum. The
essay's Jackson frame makes phase — insecure-out / insecure-in / secure-in — the fluctuating
character of fascism, with the secure-in-power phase **permitting visible dissent** because it
holds hegemony (§1.1; digest lines 19-30, 76-79).
(a) No phase — the mutable flag stays as ruled, and phase remains interpretive.
(b) Three declared phases on the flag, with capture and repression behavior keyed to phase; phase
transitions declared rather than derived.
(c) Phase **derived** from legitimation strength — ElectoralSystem @17.45 carries the legitimation
machinery (turnout, government formation, legitimation, L-SUSPEND) and PolicySystem @17.47 the
reform ceiling, so the phase would be a read rather than a stored flag.
(d) Both — a declared phase whose *behavior* is legitimation-coupled, so a secure-in-power bloc
tolerates visible dissent while staying extractive (the essay's early-Mussolini case).
Engineering consequences common to (b)-(d): any phase term entering the tick is hash-bearing and
owes a §6.5 ceremony; `EndgameDetector`'s `FASCIST_CONSOLIDATION` reads `fascist_alignment` today,
so a phase dimension interacts with terminal-outcome detection; and (c)/(d) create a new
consequence-phase dataflow (ADR109 typed motion + sentinel row) from the electoral estate into the
reactionary estate.

---

## Summary

The Director's essay, not the literature synthesis, is the frame. Fascism is a **Jackson-phase
dynamic system** rather than a crystallized definition — three phases, with the securest one
*permitting* visible dissent because it holds hegemony — and oppression has exactly one criterion:
"an extractive transfer of value from the oppressed to the oppressor class" (digest lines 34-35).
The Four Horsemen name four channels of that transfer with their substrates: misogyny on
reproductive labor, eugenics on the quality distribution of labor-power, white supremacy on
**LABOR**, settler colonialism on **LAND**. Who joins follows by arithmetic: a stratum integrates
when "the share of the spoils of exploitation that they participate in outweigh what is extracted
from them" (digest lines 43-45). Dimitrov is out on the essay's own argument — a definition built
from unquantifiable superlatives is a category error, not a line dispute.

Re-derived under that frame, fascist-capture propensity is a **net expected transfer**: a sum of
per-channel bloc-side receipts minus what the bloc's internal hierarchies take back. The frozen
`pull = agitation × (entitlement / (solidarity + ε))` (`formulas/reactionary.py:33-67`) is that
computation's **approximation**: `entitlement` collapses the channel sum into one hand-set scalar
per role, `solidarity` in the denominator proxies the *recognition* of the extracted side rather
than its magnitude, `agitation` gates action on a disturbed expectation — and the **product form is
the one structural mismatch**, because the calculus is a difference of sums. R1's `FASCIST_VEHICLE`
row generalizes cleanly: per-stratum channel weights (labor differential, land spoils, patriarchal
position) plus a bloc-side extraction term, with ADR208 R10's nation-conditioned veteran wiring
becoming one additive labor-channel seed term instead of a special case, all of it hash-bearing
(ADR195) and each wiring a typed ADR109 motion.

Both load-bearing code findings survive, verified fresh: `LUMPENPROLETARIAT` entitlement defaults
`0.0` (`social_class.py:45`) so **Sakai's core stratum is structurally unrecruitable today** — and
under the essay's frame that is now a *theoretical* tension, since a difference-of-sums with a
surviving land term would give declassed settler strata a nonzero offer — and
`calculate_entitlement_effective` (`formulas/reactionary.py:120-144`) remains **inert with no
production consumer** (only the `formulas/__init__.py` re-export and its unit test), by ADR054's
declared deferral, still lacking a `threat` producer.

Three tensions are **recorded rather than resolved**. (1) The essay never names the declassed, so
the claim that its calculus admits them is a *derivation* from its criterion, not an authority —
and it does not settle MIM's separate, empirical claim about who the U.S. base actually is. (2) The
`solidarity` denominator and the `T_out` term are different quantities occupying the same slot;
reading the frozen formula as an approximation of the calculus requires charity about that, which
is stated rather than smoothed. (3) The digest's claim that reproductive labor "has no current
substrate home" is **too strong**: the tree carries `CommunityType.PATRIARCHAL`/`WOMEN`/`TRANS`
memberships, a live `community_cost_modifier` cost-side proxy, and a stubbed
`ReproductionRequirements` reproductive-labor term blocked behind the US4 deferred loader — so the
real gap is a *transfer* term with a named receiving side, not the absence of all machinery. Same
correction, weaker, for eugenics: `CommunityType.DISABLED` exists as a membership, but no quality
dispersion does. Thirteen questions go to the Director as option spaces, with no recommendation on
any of them.
