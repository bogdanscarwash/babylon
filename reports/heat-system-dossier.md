# The Heat System — A Materialist Reformulation (Research Dossier)

**Status:** research dossier, no code changed. Commissioned by the Director 2026-07-30 after the
current heat formulation was rejected as question-begging.
**Inputs:** five research sweeps — Serge's *What Every Revolutionary Should Know About Repression*
(Okhrana practice); the police/intelligence network-targeting literature (Sparrow, Arquilla &
Ronfeldt, Jones & Libicki); the counterinsurgency/civil-war social science (Kalyvas, Wood, Scott,
RAND); the movement's own record of repression (MIM etext, ProleWiki, ERoL, Lenin); and an
exhaustive map of the `heat` estate in this worktree.
**Binding law:** ADR172 ruling 5 / ADR175 (1) — no imposed functional forms; the Aleksandrov Test;
coefficients in `GameDefines`; determinism.

Sourcing tiers are stated where they matter. **Serge's book is primary-verified in full** — all four
parts plus the preface, the three files missing from the local mirror fetched directly on 2026-07-30
(the Director asked for the entire book; §7 records what that pass changed, including one claim
withdrawn and its design inference retracted). One class of claim remains **thin** (§7): COINTELPRO
action-type proportions.

---

## 0. The indictment — why "illegality raises heat" is ideologically false

### 0.1 What the code actually does today

Three node types independently declare a bare `heat` float in `[0,1]`, default `0.0`, with no shared
model, formula module, or owner:

- `Organization.heat` — `src/babylon/models/entities/organization.py:184`
- `Territory.heat` — `src/babylon/models/entities/territory.py:130`
- `CommunityState.heat` — `src/babylon/models/entities/community.py:274`

Every writer mutates the raw graph attribute directly. The production write paths are:

| Site | Rule | Signal used |
|---|---|---|
| `src/babylon/engine/actions/attack.py:49-56` | `new_heat = min(1.0, heat + attack_self_heat_gain)` (0.1) on the **acting** org | none — flat constant, target-independent |
| `src/babylon/engine/actions/mobilize.py:134-160` | `heat_generated = turnout * _HEAT_PER_DEMONSTRATOR`, ×2.0 above 100 turnout | crowd size |
| `src/babylon/engine/systems/territory.py:107-137` | `HIGH_PROFILE: heat += 0.15` else `heat *= 0.90` | a **static scenario label** (`OperationalProfile`), never mutated by any verb or system in production |
| `src/babylon/engine/systems/territory.py:269-308` | `adjacent.heat += source.heat * 0.05` | unweighted nearest-neighbour diffusion over ADJACENCY |
| `src/babylon/ooda/layer3.py:75-118` | `+repress_heat_delta` (0.15) / `+surveil_heat_delta` (0.05) on the target | a **binary** REPRESS/SURVEIL flag |

And the read paths — what heat mechanically *does*:

- **Capacity suppression.** `src/babylon/models/vanguard_resources.py:96-115`:
  `CL_max = cadre_level*10*(1 - heat*0.5)`, `SL_max = cohesion*territory_count*5*(1 - heat*0.3)`.
  Five bare module literals; there is no `VanguardDefines`.
- **Eviction/carceral routing.** `src/babylon/engine/systems/territory.py:196-267`:
  `heat >= 0.8` flips `under_eviction`.
- **The state's own posture.** `src/babylon/ooda/state_ai/decision.py:222-245` — faction objective
  functions read heat as the state apparatus's urgency; `escalation.py:39-76` scores verb-rank fit
  as `1 - |heat - normalized_rank|`.
- **Target selection.** `src/babylon/ooda/state_ai/decision.py:258-351` — `select_repress_target`
  sorts by `heat * visibility * topology_score`, falling back to `heat * visibility`.
- **Fog.** `heat` is masked as a POLITICAL_FIELD (`src/babylon/projection/fog/filter.py:60`) and is
  one of the fields INVESTIGATE reveals (`src/babylon/projection/verbs/investigate.py:40-44`).

So the operative claim encoded in the engine is: **conspicuous or forceful action by an
organization raises a scalar; the state sorts by that scalar and hits the top.** Crowd size and a
flat per-attack constant are the only real inputs. Nothing in the write path reads the graph.

### 0.2 The record refutes it, in four independent ways

**(a) The most-repressed program in the archive was legal, unarmed, and a service delivery.**
Hoover's 15 May 1969 internal memo on the Chicago BPP: the Breakfast for Children Program
"represents the best and most influential activity going for the BPP and, as such, is potentially
the greatest threat to efforts by authorities … to neutralize the BPP"
(`/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Library/Library_The Assassination of Fred Hampton.txt`;
independently quoted in `/media/user/data/mim/etext/mt/mt11bpp.html`). Agents Piper and Johnson's
own signed instructions ordered them to "destroy the Breakfast for Children Program." Under the
current mechanic a breakfast program generates **zero heat** — it is not a MOBILIZE, not an ATTACK,
and does not change `OperationalProfile`. The single most heavily targeted activity in the
documented record is invisible to the model.

**(b) The state's own stated criterion for LOW attention is absence of structure, not legality.**
FBI informant reports on the contemporaneous white-led women's liberation movement recorded
"women at the meeting state they are not revolutionaries" and "this movement has no leaders, dues
or organizations" — and it was, per MIM's analysis, "allowed … to continue unfettered" because it
"posed no real threat" (`/media/user/data/mim/etext/mn/mn220/threat.txt`). Same era, same bureau,
same legal exposure, opposite response. The discriminating variable is organization, not conduct.

**(c) Serge states the priority order explicitly: know first, repress at the chosen hour.**
"The study of the workings of the Okhrana shows us that the immediate aim of the police is more to
know than to repress. To know in order to repress at the appointed hour, to the extent desired"
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/serge/1926/repression/ch2.htm`,
read verbatim from the local mirror). The Riga liquidation plan is organizational cartography —
Central Committee (4 names) → propaganda committee (2) → Riga Committee → 5 groups → 26 sub-groups,
76 names — completed *before* any arrest, with the punchline "the only thing left to do is to seize
them all in one fell swoop"
(`ch1b.htm`; **primary-verified 2026-07-30** — the liquidation plans and diagrams are in Part 1's
second half, not `ch1a.htm` as an earlier draft of this dossier stated). Informant pay tracked
position in the organization (3–200 roubles/month; Malinovsky's "100 roubles a month, a princely
rate," against a 50-rouble ordinary surveillance agent), and the official directive makes the
map-improving logic explicit: **"A collaborator working in secondary posts in a revolutionary
organisation can be promoted within it by means of the arrest of more important members"**
(`ch1a.htm`, primary-verified). The state arrests in order to *place* its informant higher.
Repression is an instrument for improving the state's map, not a response to an act.

**(d) There is a hard colonial/racial gradient in severity, stated by the movement's own record.**
Same architecture (COINTELPRO), radically different floor:
- white-led non-threatening movement: surveillance only (`mn/mn220/threat.txt`);
- Black-led (BPP): two successive layers of local leadership assassinated, 40 martyrs cited, and
  within the *same* "gang war" cover story "the repression faced by the BPP was much more severe
  than that faced by the US organization" (`/media/user/data/mim/etext/mt/mt11bpp.html`);
- Indigenous (AIM): a comprador paramilitary (the GOONs) doing frontline violence, then a
  fabricated federal prosecution (`/media/user/data/mim/etext/mn/mn193/peltier.txt`);
- Puerto Rican independentistas: 35–90 year sentences for property-only offenses — "many were
  convicted of crimes that would not have been prosecuted if they were white"
  (`/media/user/data/mim/etext/mn/mn194/rico.txt`).

A race-neutral scalar keyed to conduct cannot express any of this. Worse, it launders the state's
own legitimating story — that attention tracks law-breaking — into the engine's causal structure.
The professional literature does not even pretend to hold that story. On the post-9/11 refusal to
distinguish terrorists from those harbouring them, Sparrow writes approvingly: "The sense was not
'we cannot distinguish…'; of course his administration could. Rather it was 'we choose not to
distinguish' … it provides an important opportunity to create an effective scarcity"
(`scratchpad/heat_txt/The_Character_of_Harms___Operational_Cha.txt`). Legality is a discretionary
instrument of enforcement narrative, selected after the target.

**The purpose, stated by the movement in the plainest terms available.** Of the Chicago Black
radical Tibbs, jailed for years on a fabricated automobile-tire-theft charge during the Palmer
Raids: "This continual persecution reduced his political effectiveness, which was as the
authorities intended" (`/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Library/Library_Black
Bolshevik.txt`). That sentence is the objective function. The engine should optimize it, not a
compliance check.

### 0.3 A second indictment: most of the good code is dead

The code map's most load-bearing finding is not that heat is crude — it is that a large estate of
historically-grounded repression mechanics is **fully written, documented, tested, and never
called**:

- `src/babylon/ooda/state_ai/repress_effects.py` — `resolve_infiltrate` / `resolve_raid` /
  `resolve_prosecute` / `resolve_liquidate`, including the COINTELPRO double bind
  (`compute_raid_consciousness_effect`, lines 125-143, radicalizes high-CI territories) — **zero
  production callers**; referenced only from `tests/contract/state_ai/test_repress_contract.py`
  and `tests/integration/test_state_ai_integration.py`.
- `src/babylon/ooda/state_ai/territory_effects.py:228-256,326-348` — a PRESENCE-edge-based,
  per-organization heat model (TE-06) — **unwired from the tick loop**.
- `src/babylon/engine/systems/community.py:230-245` — `designate_community`, the only writer that
  could raise community heat — **zero non-test callers**, so community heat only ever decays
  (`_apply_community_decay`, lines 624-676).
- `src/babylon/ooda/npc_stub.py:459-503` collapses every `StateAction` sub-verb (RAID, LIQUIDATE,
  INFILTRATE, PROSECUTE) to legacy `ActionType.REPRESS` before the heat write, so the differentiated
  resolvers cannot influence heat even if they were called.
- The Sparrow topological targeting *is* correctly plumbed into `select_repress_target`, but is
  inert: the candidate subgraph is built only from org↔org SOLIDARITY edges
  (`npc_stub.py:296-318`), and **no production verb or system ever creates one** — the only
  SOLIDARITY writer, `src/babylon/engine/actions/_mass_work.py:96`, early-returns unless the target
  is a `social_class`. The module says so honestly at `npc_stub.py:277-291`: "honestly a no-op."

This reframes the work. The reformulation is mostly a **wiring and re-grounding** job, not an
invention job — and per ADR109 that makes it a typed motion requiring sentinel rows, not a bare
import-and-call.

---

## 1. What the political police actually do

### 1.1 Serge: mapping precedes repression; the target is position

**The Okhrana drew network diagrams. This is the dossier's foundation stone**, and it is worth
stating before anything from the police-science literature, because it is testimony from the opened
archives by a communist who was repressed — not a police service describing itself
(`ch1a.htm`/`ch1b.htm`, **primary-verified 2026-07-30** by direct fetch).

The signature instrument was a typed, tiered ego network. Serge describes one titled **"Connections
of Boris Savinkov"** — physically "two feet deep and three feet wide" — with the subject at the
centre and colour-coded relations radiating outward: "red circles which represent his 'combat'
connections" (in groups of nine, eight and six), "green circles represent the people with whom he is
or was in direct contact" (37 of them), "nine yellow circles represent his relatives," and "brown
circles represent people connected with his friends and acquaintances." Read structurally, that is
**a two-hop ego network with typed edges and role-differentiated tiers, drawn by hand in the 1910s.**
Organizational diagrams carried timestamps on the arcs: "Ivanov called on Pavel. Another arrow links
him to Marfa, who visited him at mid-day on the 27th."

The state's map could be **better than the movement's own**: the archives held "a Plan of the
Organisation of the Socialist-Revolutionary Party, such as not even the members of its own Central
Committee possessed." Any fog model that assumes the state sees a degraded copy of the movement is
refuted by the primary record — on the structural question the political police were sometimes the
better cartographers.

And the map **is** the strike plan. The "Plan for the liquidation of the Social-Democratic
organisation of Riga" shows "the Central Committee (4 names) and the propaganda committee (2 names);
below, the Riga Committee, linked with 5 groups, with 26 sub-groups under them. **In all, 76 names in
some 30 units. The only thing left to do is to seize them all in one fell swoop.**" Three mechanics
follow directly: repression waits on map completeness; it fires as one simultaneous sweep (denying
the org the chance to regrow around a partial cut); and the unit of the operation is a **structure**,
never an offence. Nobody on that chart was arrested for an act.

The apparatus was materially expensive and Serge prices it: the file on Social-Democratic
organizations for 1912 alone "amounts to 250 thick volumes"; the Paris station chief drew "1,000
francs a month"; a double execution itemized to 76 roubles, of which the hangman took 50 — "All in
all, not very dear." The analytical output was published: a 156-page study of the Zionist movement,
102 pages on Social Democracy during the war, and a manuscript review for the Tsar "appearing twelve
or fifteen times a year," annotated by Nicholas II in his own hand. **The dossier is a produced good
with a labour cost, not a free-floating suspicion meter.**

Two recruitment pools for informants: coercible people ("weak character," "disillusioned with or
aggrieved at the party," impoverished, returned exiles) and ideological adventurers who retain
organizational access after losing conviction. Malinovsky is the limit case: entered Okhrana
service 1907, raised to a "princely" 100 rubles/month by 1910, elected to the Bolshevik Central
Committee at Prague 1912, chair of the Bolshevik Duma faction by 1913. Standard rates were 3–200
rubles/month against a student's 25 rubles/month cost of living — the payment is a life-changing
bribe calibrated to the recruit's material position, and the *premium* is paid for centrality
(`ch1a.htm`, **primary-verified**).

What made a milieu penetrable was not secrecy but **structure**: small conspiratorial cells built
on individual initiative and idealist leadership were penetrable for decades (Okladsky ran 37
years, 1880–1917, inside Narodnaya Volya), whereas Serge argues "the more proletarian the
revolutionary movement, the more clearly and energetically communist it is, the less danger will it
face from agents provocateurs," because "designs of individuals, when they are not in line with the
needs of the party, lose much of their importance." Mass verification, doctrinal schooling and
collective decision are the anti-penetration variables.

The countermeasures Serge prescribes (`ch3.htm`, read verbatim) are information rationing — "a
revolutionary should know only what it is useful for him to know; it is often dangerous to know or
to tell more" — including toward "your closest friend, girlfriend or most trusty comrade." The
cost is never quantified but runs through the whole chapter: every rule that survives penetration
is a friction on organizing speed and openness. **Compartmentalization is a purchase, not a stat.**

The dual-structure evidence in `ch2.htm` (read verbatim) is three paired cases with numbers:
- Yugoslav CP, 120,000 members, legal-only posture — banned 1921 under the State Defence Law,
  "disappeared from the political scene";
- Italian CP, semi-legal — survived 4,000 arrests in the first week of 1923 and grew 10,000 →
  30,000 by 1925;
- German CP, standing illegal apparatus alongside the legal one — banned by von Seeckt 1923, came
  out "with its forces hardly impaired" to win 3.5 million votes in 1924.

And the symmetric failure is attested too: the CPUSA's "disastrous decision to go underground in
the early 1950s" left "American communism … reduced to largely inconsequential proportions"
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/erol/ncm-8/rose-cio.htm`), and the
1978 CPUSA/ML purged an internal faction that fetishized secret work at the expense of open
organizing (`.../history/erol/ncm-7/illegal-party.htm`). **Both ends of the legibility dial are
punished. There is no monotone "hidden = safe."**

### 1.2 COINTELPRO: a catalogue of distinct modes with distinct structural targets

Hoover's 25 Nov 1968 directive ordered field offices to "prevent the rise of a messiah who could
unify and electrify the militant black nationalist movement," naming Carmichael, King and Elijah
Muhammad (`Library_The Assassination of Fred Hampton.txt`). This is a targeting doctrine over
*bridging centrality*, and it was executed as sequential decapitation of the **successor**: Bunchy
Carter and John Huggins assassinated January 1969 → Geronimo Pratt rises to fill the vacuum → Pratt
becomes "the next local Cointelpro target for 'neutralization'" (`mt/mt11bpp.html`). The state
retargets whoever inherits the position.

Distinct modes, each with a citable case:

- **Edge corruption between allies.** Held "devised and released a series of cartoons and forged
  [correspondence] in the names of the Panthers and … United Slaves (US), in which the rival groups
  appeared to be viciously and publicly ridiculing one another" (`mt/mt11bpp.html`) — culminating in
  assassinations. Independently: forged Newton/Cleaver letters split the BPP
  (ProleWiki `COINTELPRO.txt`); the FBI "tried to drive a wedge between gay liberation and Black
  liberation," and a 1969 GLF vote on supporting the BPP precipitated the walkout that founded the
  GAA (`Library_Lavender & Red.txt`). Three unrelated movements, one mechanism.
- **Isolation of a hub from its own base.** "The FBI also took actions to isolate Pratt from the
  rest of the Party, leaving him vulnerable to state attack" (`mt/mt11bpp.html`) — his isolation is
  precisely why almost no member could corroborate his alibi. Isolation is a *precursor*
  multiplier on decapitation, targeting the hub→base edges rather than the hub.
- **Bad-jacketing.** MIM's own definition and mechanism: "a pig tactic that can cause havoc, say by
  accusing a genuine Palestinian of being an I$raeli snitch and then watching while that Palestinian
  patriot is killed by other Palestinian patriots"
  (`/media/user/data/mim/etext/pirao/security/badjacketing072007.html`). The state injects a false
  attribute and the organization's own security apparatus does the damage.
- **Attacking the sympathy perimeter, not the membership.** The Jean Seberg operation — a planted
  false pregnancy story via a "cooperating journalist," aimed at a prominent white *supporter*
  (`mt/mt11bpp.html`).
- **Administrative reclassification with no criminal act at all.** Michigan's Security Threat Group
  policy "automatically restricts family visits … enforces repeated shakedowns," and MDOC's own 1996
  policy named "Black Muslims, Revolutionary Students, and Latinos" —
  "nothing more than a 'Miniature COINTELPRO'"
  (`/media/user/data/mim/etext/aa/articles/micensorship4.txt`,
  `/media/user/data/mim/etext/aa/old/stg.html`). A pure legal-classification channel.
- **Legal theatre.** A Marlin Johnson memo to Hoover records DOJ's Jerris Leonard privately
  confirming "that no indictments of police officers are planned," contingent on local Panther
  charges being dropped *before* surviving Panthers were even subpoenaed
  (`Library_The Assassination of Fred Hampton.txt`). The grand jury was pre-agreed cover.
- **Informant-enabled tactical strike, priced by structural access.** Piper's bonus request for
  William O'Neal cites his "detailed inventory of the weapons and also a detailed floor plan of the
  apartment … not available from any other source" (ibid.). Informant value = positional access.
- **Comprador/proxy violence.** "Comprador tribal chief Dick Wilson's paramilitary Guardians of the
  Oglala Nation (GOONs) were terrorizing the Oglala people"; the June 1975 raid opened with two
  agents' fabricated theft pretext and was backed within a day by "hundreds of M-16-equipped FBI
  agents … at least nine armored personnel carriers and several Huey helicopters"
  (`mn/mn193/peltier.txt`).
- **Co-optation by state-sponsored rival.** The deepest precedent is Zubatovshchina — tsarist police
  sponsoring their own "legal" workers' societies to drag workers "along the line of clerical and
  gendarme 'ideology'"
  (`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/lenin/works/1901/witbd/ch02.htm`).
  Repression as capture of the recruitment pool, with no violence at all.

Serge's Okhrana "connection diagrams" — the Savinkov network chart with combat cells grouped in
nines, eights and sixes — are the direct ancestor of the FBI link chart used against the Panthers,
AIM and SDS; and the Okhrana's book-length monographs cataloguing a party's ideology, membership
and structure are the same genre as the FBI reports characterizing a breakfast program as a
strategic threat. Serge is explicit that "fetishism of legality" is bourgeois ideology and that law
is "invariably enforced along rigorous class lines" — which licenses reading police
self-description as evidence of *capability and practice*, never as neutral norm.

### 1.3 RAND: the modal state response is routine policing and politics

Across 648 groups tracked since 1968, 268 clean endings: **politics 43% (114), policing 40% (107),
military force 7% (20), victory 10% (27)**
(`scratchpad/heat_txt/How_terrorist_groups_end___lessons_for_c.txt`). Two-thirds of all groups have
fewer than 100 members, which is why force is "too blunt an instrument"; conditioned on insurgency
scale (>100 members, territory-holding), force's share rises to 25%, and 70% of its successful uses
were against groups above that size. In high-income states, of the 96 groups that fully ended, **92
ended via policing or politics.** Broad-goal and religious groups resist everything longest — 62%
of all groups have ended against only 32% of religious ones.

The design consequence is blunt: in the regime type Babylon models, spectacular force is the rare
tail. The default state repertoire is collection, prosecution, and co-optation. The current engine
has REPRESS as the workhorse.

---

## 2. What the state can SEE — legibility, information, and its limits

**This is the fog layer, and it is the load-bearing one: the state's model of the movement is not
the movement.**

**Kalyvas's information problem.** Selective violence requires fine-grained local information;
that information is asymmetric and costly; the cheapest and most effective source is **denunciation**
by civilians, not material indices and not violent extraction ("torture during interrogations rarely
yields better information than traditional human intelligence" — Rejali, quoted in Kalyvas; "it is
said that informers [within the IRA] supply over two-thirds of all intelligence")
(`/home/user/Downloads/babylon_books/The Logic of Violence in Civil War (Cambridge Studies in --
Stathis N_ Kalyvas.pdf`). And denunciation is itself endogenous: a civilian denounces only when the
risk of counter-denunciation through the rival is low — a function of the *denouncer's* actor's
control, not the target's guilt. Where information fails, actors do not stop; they switch to
indiscriminate violence, "an informational shortcut." Kalyvas quotes a Filipino observer of the
1900s US Army: a "blind giant, powerful enough to destroy the enemy, but unable to find him." The
codebase's own "Blind Giant" fallback in `decision.py`/`npc_stub.py` names the same thing.

**Sparrow's three constraints on any real criminal-intelligence graph**
(`scratchpad/heat_txt/application_of_network_analysis_to_crimi.txt`, read in full, 1604 lines):
1. **Incompleteness that is systematic, not random.** "The incompleteness in the criminal databases
   will be anything but random … The focus of existing intelligence data is determined more by the
   prior subjective judgments of investigators than by objective reality." He warns that degree-based
   targeting therefore means "the determination of centrality will depend upon who you know most
   about, rather than who is central or pivotal in any structural sense … it may incline an agency
   to pay closest attention to those it already knows most about." **This is a feedback loop the
   engine should reproduce, not correct.**
2. **Fuzzy boundaries.** "There is no obvious criterion by which players can be excluded or
   included" — real organizations interpenetrate.
3. **Dynamism.** "The relationship between any two individuals … has a distribution over time,
   waxing and waning."

**Scott on legibility.** State simplifications are (i) interested — only what serves an official
purpose, (ii) documentary, (iii) static, (iv) aggregate, (v) standardized so that individual variance
is discarded by construction. And they are not passive: "a state cadastral map created to designate
taxable property-holders does not merely describe a system of land tenure; it creates such a system
through its ability to give its categories the force of law"
(`.../Seeing Like a State … James C_ Scott … .pdf`). What escapes: anything resisting
standardization. Scott's operational test — "would an outsider have needed a local guide?" — and his
cases (the Casbah of Algiers, Iranian bazaar politics) establish that **"illegibility … has been and
remains a reliable resource for political autonomy."** And formal order "is always and to some
considerable degree parasitic on informal processes, which the formal scheme does not recognize,
without which it could not exist" — total legibility is self-defeating, which forbids modelling full
suppression as a free win state.

**Legibility is produced, and it is not monotone in "more policing."** Sparrow's *Informing
Enforcement* (1992) shows the generic mechanism across EPA/police/IRS: legibility comes from routine
record-generating interactions (permits, complaints, filings, cross-agency GIS integration) as a
deliberate infrastructure investment — "an 'information craft shop,' and … 'information craftsmen'"
(`scratchpad/heat_txt2/informing_enforcement.txt`). But *Beyond 911* documents the counter-channel:
reform-era centralized dispatch destroyed an older channel — "the old links of foot patrol officers
to their beats were severed. Their familiarity with their turf … and the informal networks of
friends, acquaintances, and informers … diminished or disappeared"
(`scratchpad/heat_txt/Beyond_911___a_new_era_for_policing_--_M.txt`). Two channels, and policing
*mode* trades one against the other.

**One claim WITHDRAWN on primary verification.** An earlier draft of this dossier asserted that
Okhrana provocateurs "often lay low" during revolutionary upsurge and "reappeared as reaction gained
the upper hand," and inferred from it that a heat model rising monotonically with visible unrest "has
the sign wrong on this channel." **Direct verification of `ch1a.htm` and `ch1b.htm` (2026-07-30) does
not support the claim**, and the nearest primary statement runs the other way: the Okhrana's
provocation service "developed to an extraordinary degree after the 1905 revolution" — expansion
*following* upheaval. The claim and the design inference built on it are both withdrawn. Recorded
here rather than deleted silently, because it is a worked example of the hazard: a relayed-tier
quotation had already grown a mechanic conclusion before anyone checked the text.

What the primary record *does* establish about penetrability is structural, not cyclical: "The more
proletarian the revolutionary movement, the more clearly and energetically communist it is, the less
danger will it face from agents provocateurs," because a mass party has "collective thought and work,
strict discipline, and action calculated by the masses" and therefore "designs of individuals, when
they are not in line with the needs of the party, lose much of their importance" — whereas
"handfuls" led by idealists (Carbonari, Blanquists) were far more vulnerable (`ch1a.htm`,
primary-verified). **Penetration resistance is a property of mass character and collective
decision-making, not of concealment** — the same finding the organizational dossier reached from the
clandestinity literature.

---

## 3. What the state COMPUTES — the topological targeting practice

**Read this section in the right order.** The ground truth is §1.1's Okhrana practice: hand-drawn
typed ego networks, complete org-charts, and a liquidation plan enumerating 76 names in 30 units
before a single arrest. That is topological targeting, attested from the opened archives in 1926 by
the man the apparatus hunted. What follows is **corroboration of continuity** — the same operation,
mechanized and given vocabulary eighty years later. It is police literature and is read here as
evidence of capability and practice only; where it narrates the force as a neutral servant of law,
that framing is discarded and the arithmetic kept.

Sparrow evaluates six centrality notions purely by their usefulness for incapacitation and reaches
an unambiguous verdict: "the second, fifth and sixth notions of centrality (Betweenness, Point
Strength, and Business) have greater relevance to the identification of network vulnerabilities than
the others (Degree, Closeness, and Euclidean Centrality)." Operationally:

| Measure | What it is | What it is FOR |
|---|---|---|
| **Betweenness** | share of geodesics through a node | communications/flow chokepoint — interception and coordination denial |
| **Point strength → Set strength** | size of the minimal cutset containing the node | fragmentation: "it is quite practical … to consider larger cutsets … Finding minimal cutsets … that effectively sever communications channels or supply lines is a versatile and useful strategy" (Menger's theorem territory) |
| **Business** | steady-state communication load under a decay/retransmission model | which channels actually carry traffic |
| Degree | link count | criticized: proxies for *what the state already knows* |

Orthogonal to centrality is **replaceability**: "The most valuable targets will be both central and
difficult to replace … If another individual exists, who can take over the same role, already having
the same connections, then the target individual was not well chosen." Target value is
centrality **×** irreplaceability, not centrality alone. Role equivalence is graph-computable via
automorphism orbits, approximated by comparing successively higher-order neighbourhoods —
`N^{i+1}(S) = N(N^i(S))` — i.e. a k-hop degree-signature fingerprint
(`scratchpad/heat_txt2/borgatti_role_similarities.txt`, Everett & Borgatti 1988).

**Which edges to watch.** In cell-structured organizations the priority surveillance targets are the
*weak* ties, not the dense internal ones: "The 'cell' structure of the Irish Republican Army fits
Granovetter's model exceptionally well … The most valuable communications channels to monitor,
therefore, are those which are seldom used and which lie outside the relatively dense clique
structures."

**Template matching — the mechanism that flags a breakfast program.** Sparrow describes an
institutionalized pattern-to-suspicion inference: "ingredients of a criminal network are superimposed
on a model template for particular kinds of deduction … The template is the encapsulation of an
expert investigator's accumulated experience." Structural resemblance to an archetype yields
suspicion with no reference to any act. This is precisely how a mutual-aid program becomes a
strategic threat in an internal report.

**Netwar topology-resistance claims** (`scratchpad/heat_txt/Networks_and_Netwars__The_Future_of_Terr.txt`):
three canonical designs — chain, hub/star, all-channel. The all-channel design is engineered so that
"there is no single, central leadership, command, or headquarters — no precise heart or head that can
be targeted," but "the capacity of this design for effective performance over time may depend on the
existence of shared principles … an overarching doctrine or ideology." **Doctrine substitutes for
command, and that substitution has an upkeep cost.** Redundancy is deliberately over-provisioned
against attrition — "For criminal networks, such costs are greatly outweighed by the benefits of
redundancy in the face of attack and degradation by law enforcement." Jones & Libicki state the
resistance/vulnerability pair precisely: "Unlike a hierarchical organization that can be eliminated
through decapitation of its leadership, a network resists fragmentation because of its dense
interconnectivity … A network is vulnerable, however, at its hubs. If enough hubs are destroyed, the
network breaks down into isolated, noncommunicating islands of nodes."

Netwar also names the **role taxonomy** with distinct removal profiles — organizers, *insulators*
(buffer the core from exposure), communicators, *guardians* (internal security), *extenders*
(recruit/corrupt outward), monitors, *crossovers* (embedded in licit institutions) — and the attack
menu: sever the boundary/crossover, hit the core (uncertain — the network may absorb it), or run an
internal attack: "One option … might be to destroy trust through misinformation … identify some of
the network crossovers and, rather than remove them, use them to feed misinformation into the
network." COINTELPRO's core technique, stated in doctrine-neutral network-science language.

**And decapitation reliably fails against organizations that can promote from within.** Tampa: "the
'King-Pin' theory suggested removal of those at the top … would make it collapse. In fact, the
trafficking organizations were able to adapt, promoting from within, and the King-Pin strategy was
abandoned." What worked instead was fixed, low-redundancy infrastructure — the 61 "dope-holes";
in the French Connection, the scarce "courier-recruiters"
(`scratchpad/heat_txt/The_Character_of_Harms___Operational_Cha.txt`). Corroborated: "for a
decapitation to be successful, good leaders within the terrorist groups must be a scarce resource"
(RAND, *Social Science for Counterterrorism*). And attested in the movement's own record by the
Pratt succession — the state had to kill the position twice.

---

## 4. The repression production function

Four things determine what actually happens. All four are constraints, not curves.

**(1) Effort and allocation are triage under a fixed toolkit.** "Such analyses seek a single, overall
allocation of resources among a finite set of familiar functional approaches … a financial regulator
might rank-order the companies or banks it regulates, prioritizing them for an existing audit
program" (*Character of Harms*). The state pre-commits a budget across instruments, then rank-orders
a much larger target population against it. The structural blind spot is that budget-locked triage
can never surface "we have no tool for this." The COIN force-ratio literature is independent
evidence that planners reason in hard capacity-per-population terms — FM 3-24's ~20–25
counterinsurgents per 1,000 residents, 10:1–20:1 ratios, correlated with success in RAND's
cross-case data (`.../Victory has a thousand fathers … .pdf`).

**(2) Priority must track live behaviour, not accumulate from history.** "Whatever probabilities one
might derive from observations of past experience inevitably carry much higher levels of variance …
simply because of an opponent's decision" (*Character of Harms*). Against a conscious adversary,
accumulating a scalar from past actions is the wrong operation. **Recompute from current structure.**
This single sentence retires the entire additive-heat design.

**(3) Escalation timing is set by map completeness, not by an offence.** Serge: repress "at the
appointed hour, to the extent desired"; the Riga sweep waits for the org-chart. Timing is a state
decision about legibility, and the strike is one coordinated sweep, not a per-incident reaction.

**Serge's own answer to "when is repression effective?" is the production function, and it is now
primary-verified** (`ch4.htm`, direct fetch 2026-07-30). It is not a curve and not a coefficient —
it is a **conditional on whether the state's general policy has materially satisfied the constituency
in question**: "Repression is effective when it completes the effect of efficient measures of general
policy," and when it "acts along the lines of historical development." His worked example is a
controlled comparison across one year and one country: *the same act of arresting land agitators*
inflamed the peasantry under Kerensky, who refused redistribution, and worked after the Bolshevik
land decree had satisfied peasant material interest. Same repression, opposite result, and the
difference is entirely in the material settlement. Serge adds the class condition: repression works
only "in the hands of an energetic class, conscious of what it wants, and serving the interests of
the greatest number."

**This maps onto machinery Babylon already has.** The core-state's "efficient measures of general
policy" toward its own population *is* the imperial-rent bribe (Φ) and the reform ceiling: repression
lands on a bought constituency and fails on a deprived one. The efficacy term therefore composes out
of quantities the engine already computes rather than being tuned — see §6.

Serge also locates repression's place in the order of coercion: "Economic constraint — by means of
hunger — is by far the most important factor, and the only really effective one, while repression
only adds to it what is required to defend capitalist order." Repression is a **top-up on the wage
relation**, not a parallel axis. The inverse holds too, and he states it as the socialist claim: "The
Soviet state, attacking the causes of the evil, has evidently much less need of repression … There
will be much less thieving when no one is hungry any longer."

The Cheka's own targeting doctrine confirms structure-over-act from the other side of the barricade:
it struck "a class through the men belonging to it," weighing "the enemy's social origins, political
attitude, outlook and ability to do damage" rather than examining individual cases — which happens
only "in periods of calm."

**(4) The non-monotonic control/violence relation, stated precisely.** Kalyvas's Proposition 1: "The
higher the level of control exercised by an actor, the higher the rate of collaboration with this
actor — and, inversely, the lower the rate of defection." A Greek villager's wartime alignment was
"largely geographical … chances that were largely geographical," not conviction. Over five control
zones (1 = total incumbent … 5 = total insurgent), selective violence is:

- **near zero at total control** (zones 1 and 5): no defection remains to punish;
- **peaking under dominant-but-incomplete control** (zones 2 and 4): high defection *and* enough
  asymmetric protective capacity that denunciation is safe;
- **collapsing again at parity** (zone 3): defection is rife but mutual retaliation suppresses
  denunciation — "the front line in irregular war is likely to be nonviolent."

This inverts the security-dilemma and "power in jeopardy" intuitions, and Kalyvas's Greek microdata
support the inversion. Note carefully **what produces the hump**: two monotone material quantities
moving in opposite directions — the mass of defection available to punish (falls with state control)
and the safety of denunciation (rises with state control). The peak is a **product**, not a chosen
bell curve. §6 relies on this.

**Serge gives backfire a topological magnitude, not a mood.** The Riga-style sweep propagates through
the neighbourhood of the removed set: "each of the members was in contact with at least ten people.
**Seven hundred people, at least, were suddenly faced with the brutal fact of the seizure.**" Seventy-six
arrests, seven hundred politicized contacts. The quantity that backfires is therefore the *boundary of
the cut* — the neighbours of whoever was removed — which is exactly what the graph can measure and
exactly what an accumulating scalar cannot. It also explains why decapitating a well-connected
organization is self-defeating in a way that decapitating an isolated one is not: the same removal
touches more people. Serge's summary is the mechanism in one line: "Relying on intimidation, the
reactionaries forget that they will cause more indignation, more hatred, more thirst for martyrdom,
than real fear."

His conclusion also names what the apparatus structurally *cannot* reach, which is the honest ceiling
on any heat model: "the revolution was the outcome of economic, psychological and moral causes outside
their reach," and the regime's own rot — "Nothing great is achieved without disinterestedness. And the
autocracy had no disinterested supporters." Perfect legibility bought tactical liquidations and no
strategic outcome. **The state's dossier is not the state's victory.**

**Backfire has exactly two independently sufficient gates, and neither is universal.**
- *Protection* (Kalyvas): indiscriminate violence drives civilians to the rival **only when the
  rival can credibly protect them**; "indiscriminate violence is likely to be effective when there
  is a steep imbalance of power between the two actors." The Kerdilia reprisal (weak insurgents, no
  backfire) against occupied Yugoslavia and Greece 1943–44 (insurgents could protect, severe
  backfire).
- *Interpretation* (Wood, El Salvador): "the Salvadoran guerrillas offered precious little
  protection in the early years … repression forged insurgency because it reinforced the framing of
  the government as a profoundly unjust authority" — running through catechist networks that "built
  up the sense of community." Where that density was absent: "Repression in Santiago had the
  intended consequence of suppressing political mobilization"
  (`.../Insurgent Collective Action and Civil War in El Salvador … .pdf`).

**Repression against an atomized population simply works.** That is the honest, unpleasant finding,
and the movement's own record agrees on the countermeasure: "Power responds to all threats. The
response is repression … the only chance We got to defeat STG is to end Our isolation. We got to
reach outside these … kkkoncentration kkkamps into the communities from which We came … Do Our
people on the outside know Us?" (`/media/user/data/mim/etext/aa/old/stg.html`). Resilience is
bridging connectivity — the exact structural inverse of the state's isolation mode.

Finally, actors **learn**: Kalyvas's Hypothesis 1 — "Political actors are likely to gradually move
from indiscriminate to selective violence" — documented in Nazi-occupied Greece, the US Phoenix
Program, Chechnya, and the CCP's own post-hoc ban on indiscriminate killing.

---

## 5. The proposed reformulation

### 5.1 Heat is not one scalar

The record forces a split into three quantities with different owners, different update rules, and
different observability. Conflating them is what produced the question-begging mechanic: a single
`[0,1]` float has to stand in for what the state *knows*, what the state *spends*, and what the org
*is* — and the only way to write one number for all three is to key it to conduct.

**(A) `L` — the state's DOSSIER on an organization (legibility).**
*What it is:* the state's own, partial, biased model of the org — which members, which edges, which
territories it has resolved. Not a threat score. Not owned by the org.
*Why the record forces it:* Serge's priority order ("more to know than to repress"), the Riga
org-chart-before-arrest sequence, and Sparrow's systematic-incompleteness constraint. Scott's five
properties dictate its *form*: interested, documentary, static, aggregate, standardized — so `L` is
not "a fraction of the truth" but a **projection** that keeps only what the state's categories can
hold.
*Where it lives:* state-side, hidden from the player except through INVESTIGATE-class verbs and
narration. It is the fog layer's other half — today `src/babylon/projection/fog/filter.py` masks
what the *player* sees; nothing models what the *state* sees.
*Update:* grows through named collection channels (§5.3), and — crucially — grows fastest where it
is already high (Sparrow's feedback loop). It **decays** through membership turnover,
compartmentalization, and org restructuring. It is not conduct-keyed.

**(B) `K` — CAPACITY allocated (the caseload).**
*What it is:* the finite state repressive budget, per instrument, and the assignment of it to
targets this tick.
*Why the record forces it:* Sparrow's triage-under-fixed-toolkit; the COIN force-ratio literature's
capacity-per-population constraint; RAND's base rates, which are what a budget distribution across
instruments *looks like* from the outside. Escalation is an allocation decision — Serge's "at the
appointed hour, to the extent desired."
*Where it lives:* on the state apparatus organization/institution nodes, per instrument class.
*Update:* set by fiscal and institutional capacity (this is where the existing
`src/babylon/engine/systems/control_ratio.py` guard:prisoner capacity mechanic generalizes rather
than being rebuilt). Spent and replenished. **Rank-order under a budget is the whole escalation
mechanic** — no threshold constant required.

**(C) `X` — the org's structural EXPOSURE.**
*What it is:* what a strike against this org would *yield*, computed from the graph — centrality ×
irreplaceability, per mode. Derived, never stored as an accumulator.
*Why the record forces it:* Sparrow's measure ranking and his centrality-×-replaceability rule; the
netwar topology-resistance claims; the Tampa King-Pin failure and the Pratt succession — both are
"the removed capability was not scarce."
*Where it lives:* recomputed each tick from the current graph. It is a property of the movement's
own structure, and the player changes it by organizing differently, not by behaving legally.

**And one quantity that already exists and must stay separate.** `SocialClass.repression_faced`
(`src/babylon/models/entities/social_class.py:169,359`) is the material violence a class actually
experiences, and it is the sole P(S|R) denominator consumed at
`src/babylon/engine/systems/survival.py:130,160-163` via
`formulas/survival_calculus.py:46-65`. Today it is coupled to `heat` only by a shared constant at
one call site (`src/babylon/ooda/action_effects.py:392-488`, where the same 0.15/0.05 increment is
applied to `heat` on the org and split across the org's SOLIDARITY-linked classes as
`repression_faced`). **In the reformulation this coupling becomes causal instead of coincidental:**
`repression_faced` is the *output* of an executed repression mode, and `L`/`K`/`X` are what
determine whether, where, and how a mode executes. Serge's ordering is respected — repression "only
adds to it what is required," an additive top-up on the wage relation, never a parallel axis.

**Deliberately NOT a quantity:** a stored "threat" or "heat" score on the org. Sparrow's
conscious-opponent argument forbids it: priority must be recomputed from live structure, not
accumulated from past conduct. The state's ordering over targets is a **composition evaluated at
decision time**, not a field.

### 5.2 The topological terms, computable on BabylonGraph today

The state's targeting graph is `L` — its own biased projection — never the true `BabylonGraph`.
Everything below is computed on that projection.

**Available now, no new mathematics** (`src/babylon/topology/graph_algorithms.py`, rustworkx-native):

| Function | Line | Material reading |
|---|---|---|
| `betweenness_centrality` | :90 | share of coordination paths a node carries — Sparrow's top-ranked measure; the "messiah" doctrine's actual quantity |
| `degree_centrality` | :85 | raw connections — kept only as the *bias* term Sparrow criticizes (what the state already knows), never as target value |
| `articulation_point_set` | :56 | cut vertices — nodes whose removal disconnects the org; Sparrow's Point Strength |
| `min_edge_cut_edges` | :100 | Stoer-Wagner global min cut — the minimal edge set severing the org; **currently unused by any consumer** |
| `component_sets` / `component_count` | :36 / :41 | measures fragmentation *after* a hypothetical removal |
| `density` | :61 | the all-channel/redundancy signature |
| `shortest_path_length_between` | :121 | Dijkstra; distance from a known node to an unresolved one (collection reach) |
| `sparrow.analyze_network` | `src/babylon/ooda/attention/sparrow.py:22` | already assembles centrality rankings, degree-signature equivalence classes (`_compute_equivalence_classes`, :91), betweenness-outlier hubs (`_identify_singletons`, :117), and articulation cutsets (`_compute_cutsets`, :158) |
| `TopologyMonitor` percolation ratio + `check_resilience` | `src/babylon/engine/topology_monitor.py` | giant-component fraction and a randomized-20%-removal survival test — the "Sword of Damocles." Runs **today** as the player's self-assessment only; zero references to `heat`. The same computation from the state's vantage is the fragmentation-value signal. |

**Needs a helper — each a composition of the above, no new mathematics:**

1. **Removal differential** `Δφ(v) = φ(G_L) − φ(G_L − v)`, where `φ` is giant-component fraction
   (`component_sets`). This is Sparrow's Point-Strength→Set-Strength generalization made
   quantitative, and it is exactly what `check_resilience` already does for random removals — the
   helper evaluates it for a *named* candidate. Extend to a candidate *set* for the coordinated
   sweep (the Riga "one fell swoop").
2. **k-hop degree signature.** `_compute_equivalence_classes` today uses 1-hop degree. Extend to
   `N^{i+1}(S) = N(N^i(S))` layering for k=2 or 3 — a BFS, per Everett & Borgatti. The size of a
   node's signature class **is** its replaceability. Class size 1 = irreplaceable specialist.
3. **Bridge-edge set.** Threshold the SOLIDARITY/MEMBERSHIP subgraph by edge weight (the same
   strong-tie thresholding `TopologyMonitor` already performs for "liquidity"), then any edge whose
   endpoints fall in different components of the strong-tie subgraph is a Granovetter weak tie.
   Composition of `component_sets` + a weight filter.
4. **External bridging degree.** Count of an org's or leader's edges to nodes *outside* the org's
   own membership closure — the `stg.html` resilience measure ("Do Our people on the outside know
   Us?"). A set difference over existing edge queries.

**THE BLOCKER, and the fix.** The Sparrow path is inert because it needs org↔org SOLIDARITY edges
and **no production writer creates one** (`src/babylon/engine/actions/_mass_work.py:96` early-returns
unless the target is a `social_class`; the honest admission is at `npc_stub.py:277-291`). Two edge
types with real production writers already induce an org↔org adjacency without inventing anything:

- **shared class base** — `org --MEMBERSHIP--> social_class <--MEMBERSHIP-- org`
  (`EdgeType.MEMBERSHIP`, `src/babylon/models/enums/topology.py:109`);
- **shared territory** — `org --PRESENCE--> territory <--PRESENCE-- org`
  (`EdgeType.PRESENCE`, ibid.:113).

Both projections are materially correct, not conveniences: two orgs organizing the same class in the
same place share an exposure, which is Serge's penetrability argument and Kalyvas's "chances that
were largely geographical" in one construction. **This one change makes the entire existing Sparrow
estate live with no new math and no new edge type**, and it is the highest-leverage item in the
dossier. Per ADR109 it is a W-C dataflow motion and owes a sentinel row.

**Template matching** (§3) is also computable today: compare an org's local subgraph shape and role
composition against stored archetype signatures. It is the mechanism by which a legal breakfast
program registers as a threat — a structural resemblance, no act required. The archetypes are
declared data, and *that* is the design's honesty point: the state's suspicion templates are
authored content the player can eventually read, not a hidden truth about the org.

### 5.3 How repression ACTS — the mode catalogue

Each mode: structural target on `L`, information precondition, engine effect, backfire term. Modes
compete for the same `K`; the state's choice is an argmax of expected damage per unit capacity,
which is what makes escalation an allocation rather than a threshold.

**1. COLLECTION / SURVEIL.**
*Target:* bridge edges and crossover/boundary nodes — Sparrow's weak ties, netwar's crossovers.
*Precondition:* a collection channel. Two, per §2, and they trade off: **formal records** (permits,
payroll, filings, complaint contacts) and **informal embedding** (patrol-mode ties). A policing-mode
choice buys one and destroys the other.
*Effect:* raises `L`. No `repression_faced`. This is why the non-threatening movement got
surveillance and nothing else (`mn/mn220/threat.txt`).
*Backfire:* near zero. Correctly so.

**2. INFILTRATION.**
*Target:* a node with high removal differential and low current `L`. Recruitment pool is
material — Serge's two pools: the coercible (low wealth relative to subsistence, aggrieved, low
cohesion) and the positioned adventurer. Payment scales with the recruit's material position
(3–200 rubles/month against a 25-ruble cost of living).
*Effect:* a large, persistent `L` gain concentrated on the infiltrated node's neighbourhood — this
is the O'Neal floor plan, valued precisely because it "was not available from any other source."
Unlocks two follow-on moves: **engineered promotion** (remove the members above the mole to raise its
centrality — Serge's explicit directive) and **disinformation injection** (§4 below).
*Resistance:* Serge's variable is not secrecy — it is mass base plus doctrinal and disciplinary
cohesion. In engine terms: MEMBERSHIP mass and cohesion, not concealment.
*Backfire:* low while undetected; on detection, the org's security response fires, which is the
opening for mode 5.
*Existing code:* `resolve_infiltrate` in `src/babylon/ooda/state_ai/repress_effects.py` — written,
tested, uncalled.

**3. DISRUPTION (edge corruption between allies).**
*Target:* inter-org bridge edges — computed by helper (3) above. Requires a node adjacent to both
sides, i.e. an infiltrator or crossover.
*Effect:* degrades or inverts the edge, not the nodes. The forged BPP/US cartoons, the forged
Newton/Cleaver letters, the GLF/GAA wedge — three cases, one mechanism.
*Backfire:* small while unattributed; large legitimacy cost on exposure.

**4. DISINFORMATION / BAD-JACKETING.**
*Target:* a loyal, well-positioned member. Injects a false "compromised" attribute.
*Effect:* the **organization's own** security apparatus does the damage — netwar's "destroy trust
through misinformation," MIM's Palestinian example. This is the cheapest mode by state capacity and
the most expensive by movement cost, which is exactly why it is in the record.
*Backfire:* the org's counter-intelligence discipline is the resistance term; Serge's mass
verification and collective decision are the specific countermeasures.
*Design note:* this mode requires the engine to represent a movement purging its own — see §7.

**5. ISOLATION.**
*Target:* the edges connecting a hub to its own base (not the hub). The Pratt operation.
*Effect:* not damage — a **precursor multiplier** on modes 6 and 7 against that node. Isolation is
why nobody could corroborate the alibi.
*Resistance:* external bridging degree, helper (4) — the `stg.html` countermeasure stated by its
targets.

**6. DECAPITATION (raid / prosecute / liquidate).**
*Target:* argmax over `Δφ(v) / |signature_class(v)|` — removal differential divided by
replaceability. Sparrow's rule, verbatim.
*Effect:* node removal, `repression_faced` on the linked class base.
*Efficacy gate:* succession capacity. Tampa's King-Pin failure and the Pratt succession are the
same finding — where the org promotes from within, the state must re-target the *position*, and it
pays `K` again each time. Under `Δφ / |class|`, a redundant org yields near zero and the state's
own allocation rule declines the strike. **The failure of decapitation against distributed
organizations is thus not a coded exception; it falls out of the targeting rule.**
*Backfire:* §5.3's shared backfire term.
*Existing code:* `resolve_raid`, `resolve_prosecute`, `resolve_liquidate` — written, tested,
uncalled. `compute_raid_consciousness_effect` (lines 125-143) already implements the COINTELPRO
double bind: high-CI territories radicalize instead of suppressing.

**7. INDISCRIMINATE (territory-wide sweep, pogrom, mass carceral).**
*Precondition — and this is the inversion:* chosen when `L` is **LOW**. Kalyvas's informational
shortcut; the blind giant. Today the engine escalates when heat is high; it should escalate to
indiscriminate force when *legibility* is low and `X` is believed high.
*Effect:* `repression_faced` across the territory's classes, not one org.
*Backfire — the shared term:* two independently sufficient gates over the affected population,
each a **measure over a partition**, not a coefficient:
  - *protected mass* — the fraction of the affected population inside a rival's protective reach
    (Kalyvas);
  - *interpreted mass* — the fraction inside a community/organizational structure that frames the
    event (Wood's catechists).
  Where both are ~0 (Santiago de María), repression suppresses. Where either is substantial, it
  recruits. Because the two populations are drawn from a partition of the affected mass, the
  combination is a **sum of measures**, which is legitimate composition, not a chosen curve.
*Existing code:* `_resolve_fascist_verb` (`src/babylon/engine/action_effects.py:311-373`) already
routes POGROM/VIGILANTISM to `repression_faced` with `ReactionaryDefines` constants. The natural
home for mode 7's non-state variant.

**8. ADMINISTRATIVE DESIGNATION.**
*Target:* a categorical class of persons, not a node. No violence, no criminal act — the STG
mechanism.
*Effect:* raises collection capacity against the designated category *and* caps the org's verb
affordability (visits, correspondence, assembly). Cheapest mode by `K`, and its price is set by
§5.4's gradient.
*Existing code:* `designate_community` (`src/babylon/engine/systems/community.py:230-245`) is the
unwired writer this mode needs.

**9. CO-OPTATION (state-sponsored rival).**
*Target:* the recruitment pool, not the org. Zubatovshchina; and per RAND the **largest single
end-channel at 43%**.
*Effect:* siphons MEMBERSHIP-edge mass toward a reformist competitor. The engine already has most of
this machinery in the Allegiance @17.42 / Electoral @17.45 / Policy @17.47 lane and the reform
ceiling.
*Backfire:* if the co-opting vehicle is discredited, its base returns radicalized — the T-7
disillusion routing already exists.
*Reserved:* whether this belongs in the repression catalogue at all — see §7.

**10. PROXY / COMPRADOR VIOLENCE.**
*Target:* frontline violence executed by a collaborator stratum *within* the oppressed nation —
the GOONs.
*Effect:* `repression_faced` at low `K` and low legitimacy cost (deniability).
*Precondition:* a collaborator stratum must exist as a modelled population. **This requires the
`community_memberships` seam, which is BLOCKED** — see §5.4.

**11. LEGAL THEATRE.**
*Target:* the state's own legitimacy account, not the org. The pre-agreed grand jury.
*Effect:* recovers legitimacy spent by modes 6/7 without acting on the org at all. It explains why a
state can absorb a visible atrocity — a fact the current model cannot represent.

### 5.4 The colonial/racial gradient as a first-class term

Non-negotiable for MLM-TW, and the record is unambiguous (§0.2d). The question is *how it enters*,
and the wrong answer is a race multiplier on repression intensity — that would stipulate exactly
what must be derived, and it would model the state as having a racial *appetite* rather than facing
a racially differentiated *price*.

**The materialist mechanism is price, not appetite.** Every mode in §5.3 spends two things: `K`
(capacity) and legitimacy. Legitimacy is spent **against a constituency whose consent the state
needs** — and the oppressed nations are, by the very relation that defines them, outside that
constituency. Puerto Rican independentistas drew 35–90 years for property-only offenses because the
sentencing constituency does not count them (`mn/mn194/rico.txt`); the BPP drew assassination where
the white-led movement drew filing cabinets (`mt11bpp.html`, `mn220/threat.txt`). Same appetite,
different price. The gradient is therefore **a term in the mode's cost, not in its yield** — which
is both materially correct and mechanically stronger, because a cost term changes *which mode the
state selects*, producing the qualitative severity gradient the record shows rather than a scaled
version of the same action.

**Where the constituency's size and stake already exist as measured quantities.** ADR171's
instrument validation on real reference data: the bribe mass is **18,691,613 persons = 1.55× the
deprivation mass (12,041,561)**, recovering MIM's wage-term ordering from independent census
headcounts, with **Appalachian settler counties reading negative bribe**
(`ai/decisions/ADR171_national_question_rulings.yaml`). That is the settler premium Ω̂ with its
internal differentiation — the legitimacy constituency, measured, including the fraction of it that
has no material stake. This is the natural source of the price term, and its internal
differentiation is what stops the mechanic from being a flat racial dial.

**Where the state's material stake is highest.** ADR175(4) rules negative Φ **real** for internal
colonies: "Phi can be negative within the US in the event of value being extracted from a Native
reservation through mining, or pollution in internal colonies." County Φ becomes signed in the Rust
engine; a negative value is real **when it carries a named internal-colony extraction attribution**
and noise otherwise, with the PR #370 clamp standing as the frozen-Python boundary guard
(`ai/decisions/ADR175_emergence_extension_logging_phi_sign.yaml`). A county where the state is a net
*extractor* is precisely where suppression has the highest material return and the lowest
legitimacy price. The `phi_hour` outlier events are ADR175's own named candidate feed for
attribution. **This is the gradient's material anchor: the state represses hardest where it extracts
and owes nothing.**

**Incidence is declared, never derived.** ADR171 OQ0/OQ1: the oppressed nations are real and
**non-contiguous**, incidence is **people-attached**, the partition is **B + C + I** (Black +
Chicano + Indigenous) with a declared overlap policy — intersections measured where computable and
disclosed, never silently netted. This matches the live `COLONIAL_AXIS` community types already in
the enum (`NEW_AFRIKAN`, `FIRST_NATIONS`, `CHICANO`, and the hegemonic `SETTLER`;
`src/babylon/models/enums/community.py:39-46`, `HyperedgeCategory.CONTRADICTION_PAIR`). Nothing here
is derived from c/v/s, which is the point: national oppression is "an asymmetric,
historically-instituted relation whose incidence is DECLARED from history."

**HONEST BLOCKER — say it plainly.** `national_oppression` does **not exist in code**. A grep across
this worktree finds it only in `ai/decisions/ADR171_national_question_rulings.yaml` and
`reports/national-oppression-proposal.md` — zero hits under `src/babylon`. ADR171 blocks Phase 2
shadow registration on three named prerequisites: a production writer for
`SocialClass.community_memberships` (the Phase 0 artifact is its designated seeder),
`county_extraction`'s own `BoundOpposition` registration (chartered by ADR170), and a coupling row
that raises `KeyError` today. `county_extraction` exists only as a projection producer
(`src/babylon/projection/topology/tension.py`). **Therefore:** modes 8 and 10 (administrative
designation, comprador proxy) and the full price term cannot be built until the
`community_memberships` seam has a production writer. The dossier's recommendation is to design the
gradient as a first-class term **now** (so nothing else is built in a shape that cannot accept it),
implement the parts that ride on already-live quantities (signed county Φ with attribution, the
declared B+C+I incidence artifact), and hold modes 8/10 behind the seam rather than approximating
them with a race coefficient. Pine Ridge remains a declared permanent data hole until a source
exists (ADR171).

### 5.5 What this replaces in the current code

**Retires outright:**

| Site | What retires |
|---|---|
| `models/entities/organization.py:184`, `territory.py:130`, `community.py:274` | the three independent bare `heat` floats, as the *causal* quantity. A rendered "heat" readout may survive as a narrated projection of `L`+`K` — it must not be an input to anything. |
| `engine/actions/attack.py:49-56` | flat `attack_self_heat_gain` self-heat. Replaced by: the attack changes the graph; `X` is recomputed; nothing accumulates. |
| `engine/actions/mobilize.py:42-50,134-160` | all five module literals (`_TURNOUT_PER_SL`, `_SOLIDARITY_AMP_PER_EDGE`, `_HEAT_PER_DEMONSTRATOR`, `_BACKFIRE_TURNOUT_THRESHOLD`, `_BACKFIRE_HEAT_MULT`) and the crowd-size→heat rule. Mobilization changes MEMBERSHIP/SOLIDARITY mass and public record; collection channels read it. |
| `engine/systems/territory.py:107-137` | `OperationalProfile`-driven accumulate/decay. The label is scenario-static and never mutated in production — it is the purest form of the question-begging: heat from an authored tag. |
| `engine/systems/territory.py:269-308` | unweighted ADJACENCY heat diffusion. Legibility does not diffuse spatially; it propagates along the *collection* channels and the org's own edges. |
| `ooda/layer3.py:75-118` | the binary REPRESS/SURVEIL heat propagation — the site that forces every sub-verb into one delta. |
| `ooda/npc_stub.py:459-503` | the sub-verb → legacy `ActionType.REPRESS` collapse. |
| `formulas/state_ai.py:18-83` | hardcoded 0.5/0.2/0.3/0.1 faction-shift literals (see orphan note below). |
| `models/vanguard_resources.py:96-115` | the `(1 - heat*0.5)` / `(1 - heat*0.3)` capacity penalties and their five bare literals. Capacity suppression is real (Serge's compartmentalization cost) but must come from the *mode* that acted, not a scalar. |
| `engine/systems/territory.py:196-267` | `heat >= 0.8` eviction threshold — a stipulated constant on a quantity that is being deleted. |
| `config/defines/organizations.py:517-541` | `heat_generation_per_demonstrator`, `max_demonstrators_before_backfire`, `backfire_heat_multiplier` — **orphaned defines, read by nothing**; editing `defines.yaml` here silently does nothing today. |
| `config/defines/state_apparatus.py:42-46` | `heat_to_ss_coefficient` — same, never read. |
| `engine/trap_detection.py:200-242` | the `org_heat > 0.5` ultra-left heuristic, reachable only from the legacy `web/` client. |

**Revived (already written, currently uncalled — this is the bulk of the work):**

| Site | Becomes |
|---|---|
| `ooda/state_ai/repress_effects.py` (all four resolvers + the double bind) | modes 2, 6 — with a real dispatch path instead of the REPRESS collapse |
| `ooda/state_ai/territory_effects.py:228-256,326-348` | superseded in *form* (its PRESENCE-edge counting is right, its accumulate/decay is not) — the collection-channel reading of PRESENCE survives |
| `engine/systems/community.py:230-245` (`designate_community`) | mode 8's writer |
| `engine/topology_monitor.py` (`percolation_ratio`, `check_resilience`) | run from the state's vantage on `L` to yield `Δφ`; keeps its player-side role unchanged |
| `ooda/attention/sparrow.py` + `topology/graph_algorithms.py:100` (`min_edge_cut_edges`, unused today) | `X` — made live by the MEMBERSHIP/PRESENCE co-projection of §5.2 |
| `engine/systems/control_ratio.py` | generalized into `K` (finite capacity per population) rather than rebuilt |
| `engine/action_effects.py:311-373` (`_resolve_fascist_verb`) | mode 7's non-state variant |

**Kept and re-grounded:**
- `SocialClass.repression_faced` and `survival.py:130,160-163` / `survival_calculus.py:46-65` —
  unchanged as P(S|R)'s denominator, now fed causally by executed modes rather than by a
  shared-constant convention at `ooda/action_effects.py:392-488`. Note that ADR173 has already
  ruled the logistic P(S|A) form frozen-reference-only; `L`/`K`/`X` must not reintroduce an imposed
  curve on the P(S|R) side.
- The fog estate (`projection/fog/filter.py:60`, `projection/verbs/investigate.py:40-44`) — extended
  from "what the player sees of the world" to include "what the player can learn of `L`." INVESTIGATE
  becomes counter-intelligence: measuring the state's dossier.
- `ooda/state_ai/decision.py:258-351` (`select_repress_target`) — the *shape* survives (an argmax
  over a composed score with an honest degradation path); the `heat * visibility * topology` product
  is replaced by expected damage per unit `K` over `L`.

**Every coefficient introduced goes in `GameDefines` with a category sub-model**, per the standing
rule and the two orphan cases above as the cautionary precedent. And every wiring step is an ADR109
typed motion owing a sentinel row — the estate's existing failure mode is precisely built-but-dormant
constructs, and this reformulation adds many.

---

## 6. Emergence check

ADR172 ruling 5 / ADR175(1): curves appear in outputs, never stipulated in mechanisms. Walking each
proposed quantity.

**`L` (dossier).** A count/measure over resolved nodes and edges, produced by named collection
channels and reduced by turnover. No functional form. **Two emergent behaviours fall out for free:**
(a) Sparrow's self-reinforcing bias — because collection reach is computed by graph distance from
already-resolved nodes (`shortest_path_length_between`), `L` grows fastest where it is high, which is
a saturating trajectory nobody wrote; (b) Scott's remaking effect — because `L` gates which modes are
available, the state's categories acquire force. *Clean.* One caution: "turnover reduces `L`" must
be a measure over departed members, not a decay coefficient. If it becomes `L *= (1-α)` the emergence
claim is void.

**`K` (capacity).** A budget in the same units as other state fiscal quantities, allocated by
rank-order. **The escalation "threshold" is emergent and this is the strongest result in the
dossier:** rank-order-under-a-budget produces sharp, threshold-like behaviour (an org either makes
the cut or does not) with **no threshold constant anywhere**. The current `heat >= 0.8` eviction
trigger is exactly the stipulation this replaces. Serge's "at the appointed hour" also emerges: the
state waits because a partial dossier scores below the cut, and scores above it once the org-chart
completes. *Clean.*

**`X` (exposure).** `Δφ(v) / |signature_class(v)|` — a ratio of two graph measures, both counts.
No curve. **Emergent:** the Tampa/Pratt succession finding. Where the org is redundant, the
denominator is large and the numerator small, the score falls below the budget cut, and the state
declines the strike — decapitation's documented failure against distributed organizations is a
consequence of the targeting rule, not a coded exception. Netwar's topology-resistance ordering
(chain < hub < all-channel) is likewise a consequence of `Δφ` on those degree distributions, not a
classification with attached multipliers. *Clean.*

**The non-monotonic control/violence relation.** Selective violence ∝ (defection mass available to
punish) × (safety of denunciation). The first is a measure over the population with material reason
to defect, and *falls* with state control. The second is a probability that a denouncer escapes
retaliation, and *rises* with state control. Their **product** is zero at both extremes and peaked
in between. Kalyvas's hump is recovered as a product of two monotone material measures — **no bell
curve, no sigmoid, no tuned peak location.** The peak's position is wherever the two measures cross
for the actual population, which will differ per territory. *This is the dossier's best emergence
result, and it is worth protecting: the moment anyone writes a hump-shaped function of a control
ratio, the finding is lost and the mechanic becomes a stipulation.*

**Backfire.** Two masses over a partition of the affected population — protected mass and
interpreted mass — combined as a sum of measures. No multiplier, no threshold. Both are counts of
people in a structural position, which is what Kalyvas and Wood actually claim. *Clean.*
**Flagged:** the record does not say how the two channels interact when they overlap. Wood's case
had interpretation without protection, Kalyvas's had protection without much interpretation. Treating
them as a partition is my construction, defensible because a person is either inside a protective
reach or not, but it is a modelling choice the record does not settle.

**The colonial/racial gradient.** Enters as a *price* — legitimacy spent against a measured
constituency (the ADR171 bribe mass, 18.7M persons, with Appalachian negatives) — and as a *stake*
(signed county Φ with named internal-colony attribution, ADR175). Both are measured extensive
quantities on real reference data. The severity gradient emerges because a different price changes
*which mode wins the argmax*, not because severity is multiplied. **No race coefficient anywhere.**
*Clean in construction* — but see the honest blocker in §5.4: it cannot be built end-to-end today.

**What still smells stipulated — flagged honestly:**

1. **"Expected damage per unit capacity" needs a damage model, and that is where a curve will try to
   sneak in.** `Δφ` measures fragmentation of the graph; it does not measure the org's lost
   *reproductive* capacity. The bridge between them is the danger point. Recommendation: define
   damage as the change in a material quantity the engine already computes (recruitment mass,
   cadre_labor, class base reached), so the composition stays material — and refuse any
   damage-vs-fragmentation transfer function.
2. **Succession capacity** is currently only implicit in `1/|signature_class|`. That understates
   what the record describes: Tampa's traffickers *promoted from within*, which is a process over
   time, not a static class size. Modelling it as a rate risks a stipulated constant. Unresolved.
3. **The legitimacy account** has no live engine representation. `legitimacy_cost` exists in
   `repress_effects.py` but that module is uncalled, and the ADR171 bribe mass is a data artifact,
   not a tick quantity. Until legitimacy is a real account fed by real quantities, the price term is
   a designed-but-unfed slot.
4. **Template matching** is authored data (archetype signatures). That is the correct treatment —
   the state's suspicion templates *are* authored ideology — but it means part of the mechanic is
   content, not derivation, and should be labelled as such rather than presented as emergent.
5. **`K`'s magnitude** must come from fiscal/institutional quantities, or it becomes a tuning dial
   controlling the entire mechanic's aggressiveness. This is the single highest-risk stipulation
   site in the design and deserves its own derivation before implementation.
6. **The collection-channel trade-off** (formal records vs. informal embedding) is well attested
   qualitatively by *Beyond 911* but **the record gives no magnitudes**. Any exchange rate between
   the two channels is presently invention. Say so rather than pick one.

---

## 7. Reserved for the Director

Genuinely reserved — ideological and theoretical, not engineering.

1. **Is co-optation (mode 9) part of the repression catalogue, or is it the Electoral/Allegiance
   lane's business?** RAND puts politics at 43% — the largest single end-channel — and
   Zubatovshchina makes the state-sponsored reformist rival a *police* instrument, not merely a
   political one. Placing it in the repression catalogue asserts that the reform ceiling and the
   electoral valve are instruments of repression. That is a line call, and it is baseline-moving
   for P25's ratified machinery.

2. **May a quantified repression threshold be used as calibration at all?** Serge supplies a rare
   number, now **primary-verified** (`ch4.htm`, direct fetch 2026-07-30): "20,000 Communards, at
   least, were machine-gunned to death, not in battle, but *afterwards*" out of "160,000 fighters"
   — about 1-in-8 — alongside "more than 100,000 victims in Paris," Finland 1918 ("11,000 workers
   were shot" plus "more than 70,000 … interned in the concentration camps"), Munich 1919 ("505
   people were shot in the town, 321 of them without the slightest pretence of justice") and Germany
   1918–21 (~15,000). The verification changes the question's character: the figures are sound, so
   this is purely the **class-of-question call** — may a historical constant anchor an emergent
   quantity at all, or does that stipulate exactly what ADR172(5) forbids? Answering it as a class
   (rather than case by case) would also settle several pending items elsewhere.

3. **Does the settler bribe Ω̂ become a live engine resource?** Making the legitimacy price a
   function of the ADR171 bribe mass means the settler population's consent is a spendable state
   account inside the simulation. Theoretically that is exactly MIM's line, and it is the mechanic
   that makes the racial severity gradient materialist rather than stipulated. But it un-shadows a
   quantity ADR171 OQ6 explicitly reserves ("`national_oppression` stays shadow=True; un-shadowing
   is a §6.5 ceremony reserved to the Director").

4. **Is bad-jacketing (mode 4) representable at all?** It requires the engine to depict a
   revolutionary organization killing or expelling its own loyal cadre on state-planted evidence.
   It is unambiguously in the record — MIM's own security literature names it and gives the
   mechanism — and omitting it would flatter the movement in a way MIM itself refuses. But it puts
   the movement's internal violence on screen, which touches the ideological line on how movements
   are portrayed.

5. **Can the player buy compartmentalization, paying in reach?** Serge's `ch3.htm` prescriptions are
   a real strategic dial with a real cost, and both ends of the legibility dial are punished in the
   record (Yugoslav CP legal-only collapse vs. CPUSA underground irrelevance). But adding a verb of
   this kind borders on the verb-algebra research seed, which is Article V amendment-gated and must
   not be improvised into a milestone.

6. **Are the indiscriminate and comprador modes against internal colonies player-visible or
   narrator-only?** Modes 7 and 10 model pogrom, mass carceral designation, and a collaborator
   paramilitary inside an oppressed nation. Whether these are surfaced as mechanics the player
   observes and counters, or as narrated consequences, is a pedagogy call under the Director's
   standing compass (engaging **and** instilling correct revolutionary theory).

**Sourcing gaps the Director should know about before any of this is cited in an ADR:**

- **Serge: CLOSED. The entire book is now primary-verified** (2026-07-30). `preface.htm`, `ch2.htm`
  and `ch3.htm` were read verbatim from the local mirror
  (`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/serge/1926/repression/`); the three
  files absent from that mirror — `ch1a.htm`, `ch1b.htm`, `ch4.htm`, i.e. Part 1 (The Russian
  Okhrana, both halves) and Part 4 (The Problem of Revolutionary Repression) — were fetched directly
  from marxists.org and extracted section by section against the book's own table of contents. All
  four parts and the preface are therefore attested. Consequences of that pass, applied above:
  the diagram/liquidation material was **re-cited from `ch1a` to `ch1b`** (its actual home); the
  recruitment directive, salary scale and promote-the-mole-by-arresting-his-superiors instruction are
  now quoted verbatim; "when is repression effective?" is quoted in full with its Kerensky/land-decree
  comparison; the Commune ratio and four further casualty figures are verified; and **one claim was
  WITHDRAWN** (provocateurs lying low during upsurge — unsupported by the text, with the nearest
  primary statement running the other way), taking its design inference with it. §2 records the
  withdrawal in place rather than deleting it.
- **No COINTELPRO action-type proportions.** Any claim of the form "X% of COINTELPRO actions
  targeted Black nationalist groups" would need the Church Committee report itself, which is present
  in these corpora only as a citation target. If calibration constants of that kind are wanted, that
  is a separate acquisition task.
- **MIM's foundational police-theory essay is missing from the mirror.** "What is a pig question?"
  (MIM Notes 43) is cited repeatedly across the MIM corpus, but `mn43.txt` is a stub redirect and
  `mn43/eldmn43.html` is a different article. If MIM's own police theory is to govern the doctrine
  writeup rather than being reconstructed from the security primer and the bad-jacketing essay, this
  needs sourcing from prisoncensorship.info.
- **No magnitudes for the compartmentalization/reach trade-off, and none for the formal-vs-informal
  collection channel exchange.** Both trade-offs are qualitatively well attested and quantitatively
  absent. Any numbers there would be invention.
