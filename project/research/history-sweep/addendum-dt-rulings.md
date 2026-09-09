# Addendum: Doctrine Tree Owner Rulings DT-3, DT-4, DT-5

Follow-up to the 45-agent history sweep (`brief-dt-theoretical-labor.md`, `brief-dt-trunks.md`).
Closes the three rulings the completeness critic found unanswered. All ticks = 1 week per the
program's `tick/520` convention (per `brief-dt-trunks.md` §Tick assumption). All verbatim quotes
below were verified against the local corpus, including visual inspection of the two scanned
(no-text-layer) Peking Review PDFs cited in DT-5.

---

## RULING DT-3: Maintenance/Decay (upkeep-decay vs. acquired-forever)

**Question:** does history support per-node TL upkeep with degeneration toward the parent
doctrine absent continued study, or is "acquired forever" (the MVP's current behavior) the
better model?

### (a) CPUSA post-Browder — an attrition number, but a different decay mechanism

The local mirror holds only two 1944 CPUSA primary documents
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/parties/cpusa/1944/05/0520-cpusa-convminutes.pdf`,
`.../0522-cpa-constitution.pdf`), both already used in the trunks brief for the top-down Browder
dissolution vote; neither covers 1941-43 cadre being pulled into war work or military service.
**No primary evidence links wartime cadre attrition to the line drift.** The closest attrition
figure is party-history tier (Foster's *History*, ProleWiki export): the mid-1944 Gannett report
counted "63,044 members, or 88 percent of those on the rolls of the C.P. (not counting 15,000 in
the armed services)"
(`/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Library/Library_History of the Communist Party of the United States.txt`,
line 2937) — roughly 15,000 cadre in uniform during the drift period, but the source blames
Browder's leadership, not attrition: a number without a causal link.

What the same source *does* establish is a leadership-level curtailment of internal theoretical
practice as the proximate cause of the Teheran-line drift:

> "the violation of democratic centralism went to the opposite, but related extreme, in the
> drastic curtailment of real political discussion, the virtual abolition of self-criticism"
> (same file, line 2873)

> "The basic reason for this error was the inadequate Marxist-Leninist development of the Party
> and its leaders." (same file, line 2867)

That is functionally the claim the ruling is after — coherence requires *ongoing* critical
practice, not a one-time purchase — with a different trigger than hypothesized. The drift
accumulated over roughly eight years (Browder's "follow Roosevelt" line dates to 1936-37; the
Teheran thesis lands January 1944) before the 14-month correction (May 1944 dissolution →
July 1945 reconstitution, per the trunks brief).

### (b) erol corpus — study-contingent line-holding, with one hard timescale

`basoc-20-years.htm` does **not** contain the hypothesized "held only as long as X kept
studying/teaching it" claim — its content is left/right error diagnosis, not decay mechanics.
But four other erol files state the decay claim directly.

`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/erol/ncm-6/lessons.htm` gives the
maintenance condition in general form: "Only a high level of communist consciousness and high
degree of theoretical and political unity among all cadre can begin to guarantee the correct
practice of democratic centralism" — conditional language: coherent practice depends on
*current*, not historical, theoretical unity. The CLP's 3rd-Congress self-assessment is a
native, in-period theory of upkeep-decay: "the theoretical work in this regard has just about
collapsed and it is only a matter of time before we will begin making serious mistakes"
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/erol/ncm-2/clp-3rd/section-f.htm`,
line 271) — lapsed theory is expected to produce *future* line errors, not a static gap. The
OCIC purge post-mortem names the reversion *target*: theoretically neglected cadre "revert to
the guilt ridden liberalism of their class," with "the gross neglect of theoretical development
of the O.C.I.C" judged "to be its undoing" (`.../erol/ncm-7/mrl-ocic.htm`, line 106) — decayed
doctrine slides toward a prior, ambient class position, not toward zero. And Ignatin's POC
memoir supplies the one concrete duration-to-drift figure in the whole sample: absent from his
branch June 1962 to early 1963, on return "Two of the three areas of mass work ... had totally
collapsed," the sectarian drift "directly related to two new theoretical innovations which had
been introduced in those six months" (`.../erol/1956-1960/ignatin01.htm`, line 96) —
**~6 months (≈26 weekly ticks) of lapsed oversight/study sufficed for measurable branch-level
doctrinal drift.**

One counter-case bounds the model: the LRS insists its 1980s drift was *not* a study-hours
lapse — "contrary to the perception that the League ceased to study, most new members have gone
through study groups" (`.../erol/ncm-7/lrs-last-congress-2/appendix.htm`, line 52); the theory
had become "not relevant to our politics and practice." Study *quantity* alone does not
guarantee coherence.

### (c) Yan'an Rectification — the cleanest primary evidence

Mao's 1945 "On Some Important Problems of the Party's Present Policy" states outright that
coherence was diluted by growth and required an active multi-year campaign to restore — doctrine
did not self-repair:

> "of the members who joined the Party before 1937, only a few tens of thousands are left, and
> most of our present membership of 1,200,000 come from the peasantry"
> (`/media/user/data/old-hdd/old-hdd/www.marxists.org/reference/archive/mao/selected-works/volume-3/mswv3_27.htm`)

> "Could we have advanced smoothly if we had not started a widespread movement of Marxist
> education, that is, the rectification movement? Obviously not." (same file)

Structurally this is the Lenin-Levy dilution case (theoretical-labor brief §1.6) with the remedy
made explicit: it took the 1942-45 Rectification Movement — roughly three years of sustained
campaign — to restore ideological unity. No source gives a smooth per-week decay curve; all
evidence is qualitative or discrete before/after. A Soviet party-school analogue (line drift
when school programs lapsed) was searched for and **not found** in the local mirror.

### Verdict and derivation

**History supports upkeep-decay, not acquired-forever** — but only qualitatively; no source
states a per-tick rate. Three concrete timescales exist: (0) ~6 months (26 ticks) from lapsed
branch-level study to first visible drift (Ignatin); (1) ~8 years (416 ticks) of unaddressed
leadership-level drift to full doctrinal capitulation (Browder/Teheran); (2) ~3 years (156
ticks) of sustained active rectification to restore coherence after a dilution shock (Yan'an).
Treating (1) as the point where unmaintained coherence reaches near-total reversal (~90% loss,
by analogy to the Lenin-Levy 99:1 dilution already in the brief) and solving
`coherence(t) = coherence(0) × exp(−λt)` for `exp(−416λ) = 0.1`:

```
λ = ln(10) / 416 ≈ 2.303 / 416 ≈ 0.00554 per tick   (≈0.55%/week)
```

**RECOMMENDATION (DT-3).** Adopt upkeep-decay. While `study_allocation` on a node is zero: no
decay for the first **26 ticks** (`NODE_DECAY_GRACE_TICKS`, the Ignatin lapse-to-drift latency —
the only directly observed onset figure), then ≈**0.5-0.6% per tick** coherence decay **toward
the parent node's value, not toward zero** (OCIC: reversion targets the prior, ambient
position). The rate is inferred by anchoring a decay curve to two discrete historical
timescales, not read off a source — flag it playtest-calibrated. Yan'an justifies the recovery
side: a `RECTIFICATION` action restores decayed coherence over ~156 ticks of sustained
`study_allocation > 0`, not instantly — reversible but expensive, matching the brief's existing
`doctrine_node_revision_cooldown` sticky-doctrine pattern. Gate decay on
`study_allocation == 0`, not merely low; and carry the LRS caveat in design notes — continued
but practice-irrelevant study coexisting with drift is real history that this simple model
deliberately does not capture.

---

## RULING DT-4: Study_Allocation Fraction

**Question:** what concrete historical fraction of cadre time went to study vs. other party work?

### What exists locally

`find .../cpusa/1935/07/organisers-manual/ -type f` confirms **only `ch03.htm` exists in this
local mirror** — no ch01, ch02, ch04+ (ch04, "Party Membership and Cadres," is linked from
ch03's footer but absent from disk). "Read ALL chapters" is impossible here; ch03 was re-read in
full (1,483 lines, not just the excerpt the original sweep used).

### The one real time-budget in the source

Ch03's "HOW SHOULD A UNIT AGENDA (ORDER OF BUSINESS) BE DRAWN UP?" section is the only place
with minutes/hours attached to agenda items:

> "A well-organized, well-prepared discussion should not last longer than from one to one and a
> half hours."
> (`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/parties/cpusa/1935/07/organisers-manual/ch03.htm`,
> line 801)

> "If the points on the agenda are well prepared, and the proposals are concrete, a Unit meeting
> could easily be finished in no more than two and a half hours." (same file, line 827 — the
> `base_unit_weekly_tl_cap` figure already in the theoretical-labor brief)

That yields a **1.0-1.5 hr "political discussion" segment of a ≤2.5 hr meeting — 40-60% of
meeting time**, the only true agenda-fraction in the document. But the segment is
agitation/current-events analysis, not theoretical study: the worked example is a sales-tax
fight ("The city administration wants to put through a sales tax," line 801), the reporter
proposing "how to mobilize the workers" against it. Actual theoretical study is located
**outside** the meeting, in institutions the manual names but never time-budgets: "To organize
open forums, lectures, study circles, workers' schools" and "training schools for functionaries
and study circles for members" (both under "WHAT ARE THE TASKS OF THE AGIT-PROP COMMISSION?").
Personal study is framed as *competing* with organizational work, recoverable only by
redistributing burdens: "leaving more time for study, reading, making friends, and carrying on
personal agitation" (line 889). No duration, cadence, or fraction is ever attached to study
circles, workers' schools, or personal reading anywhere in ch03.

### Comintern and Lenin School cross-checks

The Comintern 3rd-Congress organizational theses
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/international/comintern/3rd-congress/organisation/note.htm`)
contain no agenda/hour breakdown. The Lenin School narrative
(`/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Library/Library_Black Bolshevik.txt`, ch. 7)
confirms cohort size and course length ("a full three-year course and a short course of one
year"; "sixty to seventy qualified students") but no weekly class-hours vs. other-duties split —
the only schedule detail is monthly Party Bureau meetings, an organizational figure, not study
time.

### RECOMMENDATION (DT-4)

**The corpus specifies meeting and course *durations* but never a study-time *fraction* — the
ruling's fallback finding is confirmed.** Do not adopt a number and label it historical. The
closest proxy — the 40-60% political-discussion share of the weekly Unit meeting — must be
labeled agitation/orientation time, not theoretical study, since the manual keeps the two in
institutionally separate buckets (Unit meeting vs. Agit-Prop-run study circles) and never
quantifies the latter. Recommend a playtest-calibrated default `Study_Allocation ≈ 0.15-0.25`
of total cadre time, reasoned from institutional structure (study circles are a smaller,
separately-run add-on to a weekly meeting already fully consumed by discussion plus business) —
explicitly flagged as an inference from institutional structure, not a historical measurement.

---

## RULING DT-5: Party Congress Determinism (tag-vector-deterministic vs. seeded RNG)

**Question:** should the Congress "purge of opposed elements" outcome be (a) a deterministic
function of tag-vector deltas, or (b) routed through the seeded tick RNG?

### (1) Peking Review 1959 — the Lushan Plenum was not reported at all

Every locally available 1959 issue around and after the Lushan Plenum (2-16 August 1959) was
checked: PR1959-33 (18 Aug), PR1959-34 (25 Aug — cover: industry-aids-agriculture, drought
relief, an acrobatic troupe's Latin America tour, poetry; verified by direct visual inspection),
PR1959-36a.htm (8 Sept — Soviet nuclear-test diplomacy), the PR1959-39 Supplement (Liu Shao-chi's
10th-anniversary speech — pure unity/achievement rhetoric), PR1959-45 (10 Nov). **None mention
Peng Dehuai, the Plenum's line struggle, or the anti-Right-deviation campaign that followed**
(all under `/media/user/data/old-hdd/old-hdd/www.marxists.org/subject/china/peking-review/1959/`).
A complete contemporaneous blackout: the decisive process (Mao's reaction to Peng's private
letter, closed-door lobbying) was invisible to any observer — and the verdict was reversed only
in 1981, the 22-year gap already noted in `brief-dt-trunks.md`.

### (2) Peking Review 1976 — the Gang of Four's fall as a fast, contingent event

Mao died 9 September 1976. By PR1976-45 (5 Nov 1976,
`.../peking-review/1976/PR1976-45.pdf`, verified by direct visual inspection) the resolution is
an accomplished, unanimous fact — the contents page announces "Mammoth rallies in all provinces
... warmly hail Comrade Hua Kuo-feng as leader ... and angrily denounce crimes of the 'gang of
four' anti-Party clique," and the Shanghai report credits the Central Committee headed by Hua
with "shattering the scheme of the Wang-Chang-Chiang-Yao anti-Party clique to usurp Party and
state power" (p. 3). The Gang held major institutional power weeks earlier; the reversal was a
sudden arrest operation (6 October 1976, ~4 weeks after Mao's death). The press record shows no
antecedent trend — a discontinuous jump from power to denunciation.

### (3) Yugoslavia 1948 — the one case that looks genuinely deterministic

The Tito-Stalin break has documented, years-long material divergence predating the 1948
Cominform expulsion: Tito "wanted Yugoslavia to accept the Marshall Plan package which Stalin
rejected," and wanted "to incorporate Bulgaria and Albania into a Yugoslav federation"
(`/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Main/Josip Broz Tito.txt`, line 12) —
concrete, observable policy deltas years before the break. Corroborating color (retrospective,
non-primary): the 1922-27 CPSU struggle is described as decided by open, material contestation
over five years — Trotsky "had his day in court and finally lost because his whole position flew
in the face of Soviet and world realities"
(`prolewiki/Exports/Library/Library_Black Bolshevik.txt`, line 2172).

### Synthesis

The trunks brief already showed participants could not foresee the 1903 rupture's axis (Lenin:
"the surmise that a rupture may take place ... is conjectural in the highest degree,"
`archive/lenin/works/1903/2ndcong/9.htm`). This addendum sharpens that into two patterns:
(i) resolutions invisible to any observable state before a discontinuous outcome (Lushan — total
blackout; Gang of Four — a four-week coup); (ii) one case with measurable material deltas
predating the break by years (Yugoslavia). "Foreseeable in hindsight" holds for all three;
"determined by measurable prior conditions" holds clearly only for Yugoslavia — in the two
Chinese cases the decisive information was never part of any observable state at all.

### RECOMMENDATION (DT-5)

**Pick (b): route the Congress purge outcome through the seeded tick RNG, with tag-vector deltas
as a bias on the roll, not a determining function.** Two of three sampled cases resolved through
information (personal reactions, security-apparatus timing, private correspondence) that no
tag-vector state could encode; the third shows large sustained deltas do predict direction.
Implement `P(purge_succeeds) = f(tag_vector_delta)` feeding a **weighted seeded-RNG draw**:
large Yugoslavia-style deltas shift the odds heavily, but a nonzero contingent term stays live
even at extreme deltas — because historically a dominant position (Peng's pre-Lushan standing;
the Gang's pre-October institutional control) could be, and was, reversed by a single contingent
event the visible state did not encode. Routing through the seeded tick RNG preserves the
Constitution's determinism mandate (III.7): same seed, same history; different seed, a genuinely
different Congress.
