# Addendum: Verification & Tension-Resolution Sweep

**Scope:** four items flagged by the completeness critic over the 45-agent history sweep. Method: primary sources re-opened and read directly; all quotes verbatim, ≤40 words; all paths local to the corpus; primary evidence distinguished from encyclopedia/tertiary summaries throughout. Read-only except this file.

---

## 1. The Foco Tension — does foco theory break the no-scouting law?

**The flagged problem.** `brief-eh-mass-line.md` (§3) and `brief-eh-infiltration.md` (§3) both cite Castro's Moncada trial speech (`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/cuba/archive/castro/1953/10/16.htm`) as the one strong counter-example to the design law "you cannot SCOUT your way to intelligence" — a case where a small secretive cadre staged armed action and expected mass support to *follow*. Neither brief read Guevara's actual foco-theory writings. This sweep did.

**What the corpus contains.** The Guevara holdings live at `/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/guevara/1967/che-reader/che-reader.doc` (extracted via `catdoc`; 12,517 lines), which includes the full text of **"Guerrilla Warfare: A Method" (September 1963)** — Guevara's own mature theoretical statement of the foco. Confirmed *absent* from the corpus: the standalone 1960 book *Guerrilla Warfare*, the Bolivian Diary, any Régis Debray text (`find -iname "*debray*"` returns nothing), and any primary Bolivia-campaign material (the Che Reader's only Bolivia content is three editorial footnotes of the form "written before he left for Bolivia... 1966").

**Guevara's three theses, in his own words.** The essay states the famous triad: popular forces can defeat the army; the countryside is the principal terrain; and — the thesis the briefs could misread as licensing action-without-base — *"it is not always necessary to wait for all conditions favorable to revolution to be present; the insurrection itself can create them"* (che-reader.doc, "Guerrilla Warfare: A Method").

**But Guevara conditions the entire method on pre-existing popular relationship.** The same essay, repeatedly and unambiguously:

- *"an attempt to carry out this type of war without the population's support is a prelude to inevitable disaster"*
- *"Without these prerequisites, guerrilla warfare is not possible"*
- On what a founding nucleus should actually do first: work *"toward becoming acquainted with the terrain and its surroundings while establishing connections with the population and fortifying the places that will eventually be converted into bases"* — the foco's opening move is mass-line work, not combat.
- The named cost of skipping it: *"the terrorized peasants will in some cases give them away to the repressive troops in order to save themselves."* Betrayal by an unorganized population is the explicit, expected failure mode — precisely the mechanic's Desert state.
- On the peasantry needing leadership built through relationship: *"it cannot launch the struggle and achieve victory alone."*

So thesis 2 ("the insurrection can create conditions") refers to the *insurrectionary situation* — the political crisis — not to intelligence or mass receptivity, which Guevara treats as prerequisites whose absence makes the war "not possible."

**Moncada re-examined.** Castro's own speech does not claim the raid would conjure support ex nihilo. He asks *"Why were we sure of the people's support?"* and answers with pre-existing structural grievance — enumerating 600,000 unemployed, 500,000 farm laborers, and other classes already primed. The briefs' framing ("mass support to follow a spectacular armed action") overstates the source. Additionally, Moncada (1953) predates the formulation of foco theory (1959–63, from the Sierra Maestra experience) by a decade; citing it as "the foco exception" conflates two different things.

**Bolivia — the natural experiment.** The corpus contains **no primary Bolivia-campaign material**, so the "foco without a base" case cannot be characterized from corpus evidence. Flagged as a coverage gap rather than filled. (*Background knowledge, not from this corpus:* the 1966–67 Ñancahuazú campaign is widely documented as having failed to build local peasant relationships — consistent with Guevara's own 1963 prerequisites, but unverifiable here.)

**VERDICT: NO EXCEPTION.** Guevara's own theory makes population support a *precondition* of the foco, not a product of armed action — "without these prerequisites, guerrilla warfare is not possible." The no-scouting law holds. Foco-without-base is exactly Lenin's Adventurism ("politics without the masses are adventurist politics," `archive/lenin/works/1902/dec/00.htm`, per brief-dt-trunks) and the Insurrectionist trunk's existing Adventurism trap covers it; no new M_r growth path is warranted. The two EH briefs' Moncada citation should be corrected: it neither is a foco action nor claims support-from-spectacle. A "propaganda of the deed" action, if ever added, should sit *inside* the Adventurism trap, tagged illegitimate/unstable — as brief-eh-infiltration already recommended.

---

## 2. Minsaengdan — is the 250x overkill ratio corroborated?

**The flagged problem.** The ~2,000-purged-vs-7–8-real-infiltrators figure (≈250:1) is load-bearing in three briefs (`paranoia_overkill_ratio`, the Self-Inflicted Purge event, the paranoia-meter open question) and sourced solely to Kim Il-sung's memoir.

**Source located and quoted.** The figures are at `/media/user/data/old-hdd/old-hdd/prolewiki/Exports/Library/Library_Reminscences With the Century_ The Anti-Japanese Revolution Volume 4.txt` (line 902; note the corpus splits Volumes 3 and 4 into separate files and misspells "Reminscences" — the briefs' combined "Volume 3 & 4" citation is imprecise): *"There is a record in an enemy document stating there were only seven or eight 'Minsaengdan' members. In order to ferret out those seven or eight, the 'purge' campaign had massacred more than two thousand friends..."*

**Internal attribution nuance the briefs missed.** Kim's memoir does not present the 2,000 figure on his own authority: *"Zhou Bao-zhong... testified in his reminiscences that 2,000 people had been killed"* (Vol. 4, line 368). Zhou Baozhong was the senior *Chinese* commander of the Northeast Anti-Japanese United Army — a materially different chain of authorship than pure DPRK self-narration. The 7–8 figure is separately attributed to an unnamed Japanese "enemy document." Both are citations-within-the-memoir; neither underlying source exists in this corpus to check.

**Corroboration search.** `rg -il "Minsaengdan|Minshengtuan|Minsheng"` across the full `/media/user/data/old-hdd/old-hdd/prolewiki/Exports/` and `/media/user/data/old-hdd/old-hdd/www.marxists.org/` trees (plus supplementary `Jiandao`/`East Manchuria`+purge terms) produced exactly two additional hits — **both still Kim Il-sung as author**: his December 1955 corrective speech (`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/kim-il-sung/1955/12/28.htm`) and its verbatim reprint (`prolewiki/Exports/Library/Library_On eliminating dogmatism and formalism...txt`). Neither gives numbers — only "many people lost their lives." No CCP historiography, no Comintern document, no independent scholarly treatment of the incident exists anywhere in the corpus. Zhou Baozhong's own reminiscences are not in the corpus.

**VERDICT: UNCORROBORATED-REGIME-MEMOIR-ONLY.** Every corpus occurrence of the incident traces to Kim Il-sung as author. The `paranoia_overkill_ratio ≈ 250x` coefficient must be **demoted from historically-derived to flavor/narrative-only**. The *qualitative* pattern (self-inflicted purges exceeding the real threat by orders of magnitude) survives independently — the 1938 Polish CP dissolution (`history/erol/ncm-6/costello-poland.htm`, tertiary) and PRRWO 1976 (`history/erol/ncm-3/wing-history.htm`, near-primary participant retrospective) both stand on non-DPRK sources — so the Self-Inflicted Purge *event* keeps its historical grounding; only the specific 250:1 magnitude is regime-memoir-only. Briefs should carry a note that Kim's text internally attributes the death toll to Zhou Baozhong (a named, checkable third party), which is a lead for future corroboration outside this corpus, not corroboration itself.

---

## 3. Garvey Ratio — is PATSOC_ELITE_CAPTURE_RATIO = 7.0 corroborated?

**Primary source re-read.** `/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/groups/abb/1921/1210-lorenzo-negrolib.pdf` (Lorenzo, "The Negro Liberation Movement," *The Toiler*, Dec. 10, 1921 — an ABB-aligned Workers Party organ). Verbatim: *"Salaries range from $3,000 to $12,000 per year. Garvey gets $12,000 a year as President-General of the UNIA, and $10,000 more as President of the Black Star Line, Inc."* The $22k total is confirmed as quoted. **Two credibility problems:** (a) this is a hostile source — the same article boasts that "several thousand members have left the UNIA" for the ABB, i.e., it was published by a rival mid-poach; (b) the brief's denominator is an inference, not a quote — the text says salaries *"range from $3,000 to $12,000"* and never states that $3,000 is "ordinary officer pay." The 7.33x ratio divides the leader's total by the *bottom of a stated range*.

**Second-source hunt.** `/media/user/data/old-hdd/old-hdd/www.marxists.org/subject/africa/` contains no Garvey/UNIA material (its holdings are later African liberation figures — ANC, Cabral, Nkrumah, Nyerere, Fanon). `rg -il "Garvey"` over `subject/` and `history/usa/` finds nothing with financial content beyond the ABB piece and bibliography stubs. No Comintern 4th-Congress "Negro Question" theses in corpus. Haywood's 1930 polemic (`history/usa/parties/cpusa/1930-haywood-againstbourgeoisliberaldistortions.pdf`) treats Garvey only ideologically. ProleWiki's `Main/Garveyism.txt` is a six-line stub (tertiary, no figures). The one partial corroboration: William Z. Foster's *History of the Communist Party of the United States* (`prolewiki/Exports/Library/Library_History of the Communist Party of the United States.txt`, ~line 1503) independently confirms the *pattern* of UNIA financial malfeasance — *"About $500,000 was collected"* for the Black Star Line, the line never materialized, and Garvey was *"convicted, and sent to Atlanta federal penitentiary"* — but supplies no salary figures.

**VERDICT: SINGLE-SOURCE-FLAVOR (partial corroboration of pattern, none of magnitude).** `PATSOC_ELITE_CAPTURE_RATIO = 7.0` must be demoted from "historically derived" to **unverified single-source placeholder**: the figure rests on one hostile rival's article, and its denominator is the brief's own inference from a salary range. The demotion is narrow, though — the *qualitative* elite-capture trigger in the PatSoc pipeline (node 3, `brief-dt-patsoc.md` §2.1) keeps two independent legs: the contemporaneous ABB salary claim and Foster's independent (CPUSA, non-ABB) account of the $500k Black Star Line collapse plus the 1923 fraud conviction. Keep the node; flag the 7.0 threshold as playtest-calibrated with flavor-text citation only.

---

## 4. PUWP Consistency — 0.05%/week vs 0.1%/tick

**Source verified.** Both briefs cite the same file and quote it correctly: `/media/user/data/old-hdd/old-hdd/www.marxists.org/history/erol/ncm-6/costello-poland.htm` (Paul Costello, "Class Struggles in Poland," *Theoretical Review* No. 19, Nov–Dec 1980 — a tertiary/retrospective EROL analysis, not a party document), line 85: *"Working-class membership in the Party fell from a high of 61% in 1949 to 45% in 1955."* Same source, same figures, same 6-year span in both briefs. Two caveats: the metric is **party-composition share** (working-class fraction of PUWP membership), a *proxy* for mass-base erosion, not M_r itself; and the same passage contains a second interval (45%→40%, 1956–59, ≈1.67 pts/yr vs 2.67 pts/yr) showing the rate is not constant — neither brief uses it.

**Arithmetic redone.**

- *Linear (mass-line brief §2.3):* (61−45)/6 = 2.667 pts/yr ÷ 52 = **0.0513 percentage-points/week** ≈ "0.05%/week." Correct for what it computes.
- *Exponential (receptivity brief §2.1):* ln(45/61) = ln(0.7377) = −0.3040; ÷313 weeks = **−0.000971/week ≈ 0.097%/week** ≈ "0.1%/tick." Correct for what it computes.

**Reconciliation.** These are not contradictory numbers — they are two parametrizations of the *same data point*. The linear figure is absolute share-point attrition per week; the exponential figure is the fraction of the *currently remaining* value lost per week. Normalizing the linear rate by the moving base recovers the exponential one: 0.0513/61 ≈ 0.084%/wk at the start, 0.0513/45 ≈ 0.114%/wk at the end, averaging ≈0.097%/wk — exactly the exponential constant. The apparent ~2x discrepancy is purely the functional-form difference.

**Which one the engine needs.** The EH briefs' decay pattern is multiplicative (`M_r0·(1−k)^n`, per `RUPTURE_DECAY_RATE`), so the **exponential form is the structurally correct one**: `LONG_HORIZON_EROSION_RATE ≈ 0.001/tick` (i.e., per-tick multiplier ≈ 0.999) should be canonical. The mass-line brief's 0.05%/week is right arithmetic in the wrong functional form for a multiplicative per-tick coefficient — it should be reformulated, not treated as an independent corroborating data point (it is the same source, once).

**VERDICT: BOTH CORRECT — TWO DIFFERENT QUANTITIES, ONE SOURCE.** (1) 0.05 pp/week = linear percentage-point attrition of PUWP working-class composition share; (2) ~0.097%/week ≈ 0.1%/tick = exponential per-week decay constant of the same ratio. Adopt the exponential (~0.001/tick) for the engine's multiplicative decay; mark the linear figure as a re-derivation of the same single data point, and note the metric is a composition proxy resting on one tertiary source with a demonstrably non-constant rate.

---

## Cross-cutting note

Three of four flags resolved *against* the briefs' more dramatic readings: the foco "exception" dissolves on contact with Guevara's own text, and both headline coefficients (250x, 7.0) demote to flavor. This is the expected direction for single-source numbers in a corpus of partisan primary documents — the corrective is not to delete the mechanics they inspired (the Self-Inflicted Purge event and elite-capture node both keep independent qualitative grounding) but to strip the false precision. Only the PUWP flag was a non-problem: correct arithmetic twice over, needing a functional-form ruling rather than a source correction.
