# Program Proposal — Doctrine Tree, Theoretical Labor & the Four Trunks

**Status: RATIFIED by owner 2026-07-15** (drafted 2026-07-15 from a full corpus
mine; audit Wave 6's "single largest coherent MISSING cluster"). **Historical grounding:**
50-agent Marxists.org/ProleWiki sweep, owner-directed at ratification —
`project/research/history-sweep/` (`brief-dt-{trunks,traps,patsoc,theoretical-labor}.md` +
`addendum-dt-rulings.md` closes rulings 3-5 with recommendations; coefficient tables inside). Sources:
`ai/epochs/epoch3/doctrine-tree.yaml` (1200 lines, PLANNED — the full spec),
`ai/epochs/epoch3/doctrine-tree-mvp.yaml` (481 lines, SPEC_COMPLETE — 3-trunk/3-tag/15-node
subset), `resource-sinks.yaml:240-280` (TL sinks/decay), `vanguard-economy.yaml` (Reading
Group Trap), `reactionary-subject.yaml:576-583` (class-composition gating),
`reports/epochs-vision-gap-audit.md` §3b/§5 Wave 6 + §6 item 8. Never-authored spec slot:
spec-080 (dependencies 070/071 exist on disk; 072 does not).

## Thesis (the corpus's)

"Your ideology is not neutral infrastructure. It determines what you CAN do and what you
WILL BECOME." A DAG of doctrine nodes with mutually-exclusive splits, purchased with
**Theoretical Labor** (`TL = Cadre_Count × Study_Allocation × Coherence_Factor` — an
opportunity-cost draw on cadre time, not a free currency), summing to a **tag vector**
(CLASS_ANALYSIS vs NATIONAL_CHAUVINISM is the load-bearing pair: "As long as CLASS_ANALYSIS >
NATIONAL_CHAUVINISM, you are safe"), organized into **four trunks** (Reformist / Insurrectionist /
Autonomist / Scientific), each with a mechanically-triggered trap ending (Liquidationism /
Adventurism / Dissociation / Bureaucratic degeneration), plus the **PatSoc Pipeline**: a
five-node drift chain ending in Strasserism with `resource_cost: 0` ("Falls into this,
doesn't choose it") and a faction flip — "You are the fascist who thought he was a
communist." Correction mechanism: the **Party Congress** (500+ TL; trunk_shift /
rectification / theoretical_offensive / self_criticism). "THE TREE MUST HAVE NO 'OPTIMAL
PATH.'"

## Verified engine reality (2026-07-15)

**Nothing exists.** Zero `DoctrineNode`/`DoctrineTree`/`TheoreticalLabor`/trunk-enum code
anywhere in `src/babylon` or the frontend (all rg hits are false friends: `shock_doctrine`
causal detector, "owner's mock doctrine" prose). The audit's §6 item 8 ruling stands: the
resource-balance traps (Reading Group/Influencer) are SUPERSEDED by `trap_detection.py`'s
Liberal/Rightist axes — fold flavor in, build nothing parallel. The ideological TRUNK traps
are a distinct axis and remain in scope.

## Owner rulings required BEFORE spec authoring

1. **MVP vs full**: the corpus itself recommends the 3-trunk/3-tag/15-node MVP first with a
   4-phase upgrade path (`doctrine-tree-mvp.yaml:449-481`). Note the standing
   no-MVP-scoping directive (memory: "full vision is the MVP") — BUT here the MVP is the
   corpus's OWN specced staging, not an external de-scoping; needs your explicit call.
2. **Faction flip on Strasserism**: runtime org-allegiance reassignment ("The State may now
   RECRUIT you") has no engine analog — an org changing sides mid-game is an
   architecture-level ruling (organization model + OODA + endgame interactions).
3. **Maintenance/decay**: full spec has per-node TL upkeep with degeneration-toward-parent;
   MVP strips it ("Once acquired, a doctrine stays forever"). Pick one; don't blend.
4. **`Study_Allocation`**: a cadre time-allocation split that exists nowhere in the Vanguard
   Economy — genuinely new state on the org model.
5. **Party Congress determinism**: "purge of opposed elements" must be a deterministic
   function of tag deltas or route through the seeded tick RNG — spell out which.
6. **Trap escape**: MVP deliberately has none ("teaches the lesson hard"); full spec sells
   self_criticism at 300 TL. Which ships?

## Tag rename + the Lawverian chauvinism⟷internationalism opposition (owner, 2026-07-15)

The reactionary tag `NATIONALISM` was renamed to **`NATIONAL_CHAUVINISM`** across the spec
(`doctrine-tree.yaml`, `-mvp.yaml`, this doc, `history-sweep/brief-dt-patsoc.md`) — docs-only,
no runtime code exists yet. Rationale (owner): the tag is a stand-in for oppressor-nation
chauvinism / white supremacy (Lenin's *great-nation chauvinism*; the CPUSA's own 1930s *white
chauvinism* campaigns), **not** the national question in general. National **liberation** of
oppressed nations is a distinct, progressive phenomenon this tag must never be confused with —
the sweep's African Blood Brotherhood findings draw exactly this line ("bourgeois
[left-]nationalism will betray the real interests of the Black and Latin toiling masses").
Renaming also forced a coherence fix: the old tag's "moderate" effect sold it as *"useful in
anti-imperial struggle"* — the precise conflation now removed (chauvinism is reactionary across
its whole range).

**Owner ruling (2026-07-15): Option C — the Divergence Channel (ADR072).** The tag pair
`NATIONAL_CHAUVINISM ⟷ INTERNATIONALISM` is modelled as **instance #2 of the Divergence
Channel** — the general "authored pole checked against a material pole, shadow-first"
facility ADR072 generalizes from P19's `read_poles` surface (owner directive: "abstract it
enough to apply to whatever", scoped by the follow-on ruling "design general, build
concrete"). It is a **shadow (`shadow=True`) asymmetric opposition** `chauvinism_internationalism`:
read by `read_poles`, **never stepped**, so byte-identical-safe by construction.

- **A (authored, off-graph)** = `normalize(CLASS_ANALYSIS − NATIONAL_CHAUVINISM)` read from a
  `doctrine_tags` Mapping on `GraphInputs`, sourced from `DoctrineTree.calculate_tags()` — the
  Doctrine-DAG's **own structure**, never a BabylonGraph morphism (II.9/VIII.9). `calculate_tags()`
  stays **unchanged**: the authored accrual still drives the tag number, so the III.12 PatSoc
  golden trace (`CLASS_ANALYSIS 3→1→0→−3`, `NATIONAL_CHAUVINISM 0→2→5→7→9`) stays pinnable. The
  divergence rides *alongside*, reading the tags, never mutating them.
- **Ā (material, from graph)** = a new `_chauvinism_internationalism_poles` pole_measure over
  live SOLIDARITY-edge incidence per org (`SolidaritySystem` output — already computed).
  **Honesty caveat (Aleksandrov):** raw SOLIDARITY is class-solidarity-*in-general*, **not** the
  *inter*-national character (a reactionary national bloc has SOLIDARITY edges too). Phase 1
  ships this proxy explicitly labelled **"solidarity proxy, internationalism-UNPROVEN"**, and the
  divergence sentinel's *first* job is to measure whether the proxy tracks internationalism
  before any promotion. Cross-border-restricted SOLIDARITY (only edges crossing a
  sovereign/national boundary — genuinely inter-national) is the eventual target, dormant until
  multi-sovereign / nationwide scenarios exist.
- **The divergence** = a purpose-named `DivergenceReading(gap = |σ_authored − σ|/2,
  claim = σ_authored, claim_side)` on `PoleReading` (**not** an overloaded `GapReading` —
  ADR072 fix #3). The lie the mechanic surfaces: `claim_side == "b"` (the tag vector reads
  "safe"/internationalist) while the material `side == "a"` (the base has collapsed), with a large
  `gap`. Written per-org as `sigma_internationalism` / `divergence_chauvinism` behind a
  **net-new** `ORGANIZATION_COMPUTED_FIELDS` gate (ADR072 fix #5).
- **Amendment S, honestly.** A shadow binding never steps, so it has **no motion**; the divergence
  is an *observational static* that adjudicates nothing — it is NOT claimed to be "a reading of
  opposition motion" (ADR072 fix #2). The material σ stays the primitive; motion is untouched.
- **σ as an authored channel** is proposed as **Amendment T** (Article VIII, observes-only) for
  BD ratification at Phase-0 review — see ADR072.

**Phasing (mirrors P19).** **Phase 0** = this ratification (ADR072, no code). **Phase 1** =
shadow: `read_poles` writes the divergence to `pole_readings`; `qa:regression` byte-identical with
UNMODIFIED baselines (conditional on P19 landing + the org gate built + the harness still
excluding `pole_readings` — DoD requires an actual run); the divergence check advisory across the
5 scenarios + wayne_county; PatSoc trace pinned unchanged. **Phase 2+** = promote to gating only on
stable-divergence evidence with an R-PROOF doc. The `CLASS_ANALYSIS`-vs-`NATIONAL_CHAUVINISM`
safety condition becomes the *documentary* interpretation of the reading (the opposition's
`unity`/`w`); the fascism "wormhole" transition (`CLASS_ANALYSIS ≤ 0 AND NATIONAL_CHAUVINISM ≥ 5`)
is the **named future promotion point** where the material proxy would corroborate the authored
tag — until then the authored accrual alone drives it (III.12). **Deferred until a 2nd real
authored↔material instance exists (rule of three):** the machine-checked `GATING_LEDGER`, the
generalized `sentinels/divergence/` seventh sentinel, and the SHADOW→GATING lifecycle FSM.
**Sequencing:** P19 must fold into the spine first (this extends its surface); `calculate_tags()`
is unbuilt today (the DoctrineTree program must build it before the authored pole is real).

## Constitutional guardrails

- II.9/VIII.9: the Doctrine DAG should be its OWN small structure with its own owner
  (II.11), NOT edges on BabylonGraph — never overload the five canonical morphisms with
  ideology relations.
- II.5 + ADR034: trap conditions are boolean formulas over engine state, engine-evaluated,
  AI-narrated. No LLM ever grades theory, generates line-struggle adjudication, or decides
  NATIONAL_CHAUVINISM > CLASS_ANALYSIS.
- III.12: the PatSoc Pipeline's fully-numeric 5-phase trace (CLASS_ANALYSIS 3→1→0→−3,
  NATIONAL_CHAUVINISM 0→2→5→7→9) is a ready-made golden baseline — seed `tests/baselines/` from it
  directly.
- III.7: all corpus formulas are pure functions of committed state; keep it that way.

## Proposed phasing (post-ratification)

- **Phase 0** — ADR + rulings 1-6 + promotion to `project/programs/`; data file
  (`doctrine_tree.json`/TOML per the Paradox data-driven pattern) authored from the corpus
  node tables; DAG-validity check (the corpus's own error-recovery spec names circular
  dependency as the guard case).
- **Phase 1** — models + TL resource + tag summation, engine system acquiring nodes via a
  new verb or Congress action; shadow attrs + seam rows; goldens seeded from the PatSoc
  trace. qa:regression byte-identical (new system additive; no existing-system inputs
  change — verify, don't assume).
- **Phase 2** — trunk traps + PatSoc drift + (per ruling 2) the faction-flip mechanism.
- **Phase 3** — the 5th takeover: the tree canvas (the corpus even ships an ASCII mockup —
  `[●]` acquired / `[○]` locked / `[!]` trap + live tag bars); DESIGN_BIBLE §9b/§11 bind.
- Explicit dependency note: NATIONAL_CHAUVINISM-tag mechanics interact with spec-071
  (reactionary-subject, on disk) and colonial_stance (spec-070, on disk) — both
  prerequisites already merged per the 05-catalog record.
