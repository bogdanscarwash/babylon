# History Sweep — grounding the Epistemic Horizon + Doctrine Tree in revolutionary history

**Owner directive (2026-07-15):** after ratifying both program proposals
(`project/research/epistemic-horizon-program-proposal.md`,
`project/research/doctrine-tree-program-proposal.md`), Percy directed a broad subagent sweep
(≤50 Sonnet agents) over the local mirrors — marxists.org US history, Bolshevik history,
Chinese revolutions, the history of revolutions generally, plus the ProleWiki export — "to
really inform this mechanic."

**Corpus:** `/media/user/data/old-hdd/old-hdd/www.marxists.org` (history/, subject/, archive/,
incl. the 3.3G EROL anti-revisionism encyclopedia) and
`/media/user/data/old-hdd/old-hdd/prolewiki/Exports` (807 plain-text articles + Library).
All source citations in these documents are exact local file paths into those mirrors.

**Method:** workflow `wf_daa1826f-9db` — 6 region scouts → 31 deep-readers (structured
findings with verbatim ≤40-word quotes + exact paths) → 7 theme synthesizers → 1 completeness
critic (45 agents, 0 errors); then 5 targeted Round-2 agents on the critic's gaps (50 total).
**Stats:** 727 cited findings from 443 documents read (425 unique; 15 skipped with reasons);
**374 findings carry a historical number or timescale** — candidate GameDefines coefficients
with documented derivations.

## The briefs

| File | One line |
|---|---|
| `brief-eh-mass-line.md` | The presence-vs-relationship thesis grounded in Mao's Red/White-area leakage comparison; regime-switching M_r growth (Hunan 4-5× threshold jumps); Water gated on Mao's conjunctive base-area preconditions; 14 derived coefficients. |
| `brief-eh-infiltration.md` | Two informant tiers (1-4 tick shallow vs 150+ tick deep-plant); Desert as *active falsification* (Kronstadt troop rumors, Batista's Moncada casualty fictions, COINTELPRO forgeries); 10-12 member cell cap (CPUSA 1935 manual); young-org vulnerability window 13-39 ticks. |
| `brief-eh-receptivity.md` | Decay asymmetry: rupture-collapse ~0.70-0.75/tick (Kronstadt, 9-13 days) vs passive erosion ≤0.5%/tick; two-stage post-repression recovery (2 ticks propaganda / ~47 ticks full parity, CPUSA post-Palmer); cadre-tier multipliers 1:10 / 1:175 / zero (CPUSA 1930 Language Bureau). |
| `brief-dt-trunks.md` | Congress cadence is a doctrine property, not a constant (104 ticks pre-power / 260 post-power / ×1.5 repression slippage); deadlock-triggered congresses; concrete vote thresholds (2/3 trunk-shift supermajority); trap attrition 0.85 severe / 0.5 partial. |
| `brief-dt-traps.md` | Trap durations bifurcate by trunk (Adventurism 17-52 ticks; Liquidationism/Bureaucratic 208-546); the Adventurism trigger couples DT to EH (militancy high ∧ M_r < 0.2); three-tier Congress outcomes with a documented failure case each; rectification base success 0.02. |
| `brief-dt-patsoc.md` | Five-node pipeline mapped to named history (DAP 1918 → Hamburg fusion → Garvey elite capture → KPD-Nazi fraternization → Stennes flip, 1-tick resolution); AND-structured flip trigger; safe-nationalism band ~0.44 (Baku Congress) to protect legitimate anti-imperialism. |
| `brief-dt-theoretical-labor.md` | TL multiplicative form confirmed from primary sources; 1 FTE per ~230 members (UCP 1920 payroll); press split 73:27 agitation:doctrine; Iskra ≥2-issues/month press-liveness gate; coherence-dilution shock at >10× single-tick recruitment (Lenin Levy). |

`findings.json` — all 727 structured findings (mechanic / theme / case / quote / source path /
implication / parameter), plus the read + skipped document lists. This is the raw corpus the
briefs synthesize; regenerable but expensive (~8M subagent tokens).

## Cross-cutting results (what multiple briefs converged on independently)

1. **1 tick = 1 week validated twice over** — Comintern weekly cell-reporting theses and IWW
   weekly job-branch bulletins land on the same cadence. (Confirm against engine tick length
   before promoting any coefficient — every brief flags this.)
2. **Growth is regime-switching, decay is asymmetric.** Mass receptivity moves flat-then-jump
   (4-5× over 4-6 ticks after a threshold crisis), and rupture-collapse (1-2 tick floor,
   Kronstadt) must use a different constant than passive attrition (~0.1-0.5%/tick). One
   smooth curve fits nothing in the record.
3. **Desert-state falsification is bidirectional.** States manufacture normal-looking data
   (Kronstadt, Batista, COINTELPRO) — but factions also falsify their *own* ledgers upward
   (CPA/CLP membership padding to Moscow, KPD March Action overclaiming). Both briefs that
   found this independently recommend the falsification roll apply to the faction's own
   TL/membership self-reports, not just State surveillance.
4. **Infiltration is discrete and latency-gated, not continuous erosion** — 6-18 month
   build-up, catastrophic one-tick yield, ~3-tick re-insertion cooldown after a sweep;
   two informant tiers with different tenure and yield.
5. **The Party Congress must be fallible.** Real congresses reaffirmed errors and got captured
   by the tendency under review; a three-tier outcome (cosmetic/partial/full) with an explicit
   failure mode beats binary pass/fail. Verdicts are historically revisable-with-lag
   (Peng Dehuai: 22 years).
6. **PatSoc entry is cheap everywhere in the record; escape is nearly unevidenced.** The
   zero-TL drift chain is confirmed as a design law; the single reversal case (PRRWO, ~82
   ticks) required theoretical-labor investment AND a co-occurring organizational crisis.
7. **Candidate NEW sub-mechanic: the self-inflicted purge/witch-hunt** (paranoia as a
   trap-adjacent state distinct from Desert decay) — raised independently by three briefs.
   ⚠ Its anchor case (Minsaengdan, ~250× overkill) is single-source regime-memoir; Round 2
   verification below.

## Terminology note (post-sweep rename, 2026-07-15)

After this sweep ran, the owner renamed the reactionary Doctrine-Tree tag
**`NATIONALISM` → `NATIONAL_CHAUVINISM`** (it is a stand-in for oppressor-nation chauvinism /
white supremacy, not the national question in general; national *liberation* is a distinct
progressive axis whose dialectical relation is `NATIONAL_CHAUVINISM ⟷ INTERNATIONALISM`). The
authoritative design docs (`doctrine-tree.yaml`, `-mvp.yaml`, the proposal, `brief-dt-patsoc.md`)
use the new name. **`findings.json` is left unchanged** as a point-in-time research corpus
(immutability of history): where its `case`/`implication`/`theme` fields say "NATIONALISM" or
"patsoc-nationalism," read `NATIONAL_CHAUVINISM`. See the proposal's "Tag rename" section.

## Corpus caveats (apply to every brief)

- **ProleWiki Library stub rate ~21%** (71 of 338 Library files are empty bibliographic
  stubs) — ProleWiki-sourced claims carry elevated single-source risk.
- **Minsaengdan figures** rest solely on Kim Il-sung's memoir; **Garvey salary ratio (7.0)**
  is single-source. Both under Round-2 verification.
- The corpus skews US/Russia/China 1917-1980; anti-colonial theaters are thin.
- Encyclopedia-tier (EROL narrative, ProleWiki) vs primary-source evidence is distinguished
  inside each brief per the sweep's rules.

## Completeness critique → Round 2 (5 targeted agents, complete)

The critic's full verdict is preserved in `critique.md`. Its material gaps, the Round-2
addenda commissioned against them, and what each found:

| Gap | Addendum | Result |
|---|---|---|
| Indonesia/PKI 1965-66 unread | `addendum-indonesia-pki.md` | Desert-unpreparedness CONFIRMED from the PKI's own 1967 self-audit ("did not say a single word on... armed struggle" across 14 peaceful-road years); 3.5M+ members aboveground → ~30,000 arrests in ~10 days; **M_r must be decoupled from survival value** (near-ceiling receptivity, zero defensive worth); proposes a hidden per-tick "unpreparedness debt" under a peaceful-road doctrine flag — a new EH↔DT coupling. |
| POUM / Commune / Yugoslavia / Greek | `addendum-collapse-cases.md` | Trotsky's verbatim Dissociation diagnosis of POUM + the fabrication line; Michel on the Versailles failure; Yugoslavia = a *resisted* trunk-shift (different table than collapses); Greek repression pulse: 20,000 arrests/~8 weeks. Comparative collapse-speed table across six cases. |
| EH rulings 3 + 5 formally open | `addendum-eh-rulings.md` | **EH-5: KEEP the spec's 0.20/0.05/0.01** (Desert half-life 3.1 wk ≈ DoJ reinsertion; Mud 13.5 wk ≈ informant tenure) **+ add event-gated `RUPTURE_DECAY_RATE ≈ 0.776/tick`** (solved from Kronstadt's 9-13 days; no existing constant can produce it). **EH-3: per-(org, territory)** — Cannon's 1931 furriers case + the spec's own org-dependent `I_a` term is incoherent under a shared scalar; implement by materializing only `I_a` per pair. |
| DT rulings 3-5 formally open | `addendum-dt-rulings.md` | **DT-3: upkeep-decay wins** (Yan'an: 1.2M-member dilution cured only by the 1942-45 education movement) — ≈0.55%/tick coherence decay toward the *parent* node at study_allocation = 0 (λ = ln(10)/416 ticks, labeled inferred), reversible via ~156-tick rectification. **DT-4: honestly non-derivable** — corpus gives durations, never fractions; playtest default 0.15-0.25, labeled inference. **DT-5: seeded RNG, tag-deltas as weights** — Lushan resolved under total press blackout, Gang of Four fell in a 4-week discontinuity, only Yugoslavia looks deterministic; `P(purge) = f(tag_delta)` feeding a weighted seeded draw, contingency never zero. |
| Foco tension; Minsaengdan/Garvey single-source; PUWP derivation conflict | `addendum-verification.md` | **Foco: NO exception** — Guevara conditions the foco on popular support ("a prelude to inevitable disaster" without it); foco-without-base = the existing Adventurism trap. **Minsaengdan 250× → flavor** (regime-memoir-only; the self-purge *event* keeps independent grounding: Polish CP 1938, PRRWO 1976). **Garvey 7.0 → placeholder** (the $3k denominator was an inference). **PUWP: both briefs right, two quantities** — adopt the geometric ~0.001/tick (matches the spec's multiplicative form). |

Round-2 corpus corrections (extend the caveats above): POUM holds ~15 files, not the ~102
the critic estimated (ETOL series parts 2-8 missing); Marx's *Civil War in France* text is
ABSENT from the mirror (2014 MECW copyright takedown — only a news appendix survives); the
PKI's 1966 Self-Criticism (the key primary source) is a dead link recovered only via a
secondhand Peking Review quote; several Peking Review / PKI Plenum PDFs are image-only scans
with no text layer. One of the five Round-2 agents split its work with a self-spawned helper
that cross-verified the DT citations before standing down.

## How this feeds the programs

- Each brief's §2 is a coefficient table in GameDefines shape: historical value → weekly-tick
  conversion → recommended start, with **NEEDS PLAYTEST** explicitly marking every number
  history did not supply (Constitution III.12 discipline — no invented calibrations).
- The open-questions sections across briefs + the critic's ruling-gap items constitute the
  **owner ruling queue** for Phase-0 promotion (task #85); they map onto the numbered rulings
  already enumerated in the two proposal docs.
- The EH↔DT coupling findings (Adventurism trigger reads M_r; congress interruption by raids;
  faction self-falsification hitting the TL ledger) are new *cross-program* surface not in
  either proposal — flag for the Phase-0 ADRs.
