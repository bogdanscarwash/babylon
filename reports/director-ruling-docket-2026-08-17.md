# Director Ruling Docket — 2026-08-17

> **STATUS: RULED IN FULL — same day.** The sitting ran interactively 2026-08-17 in five
> question batches; all thirty rulings are recorded in
> `ai/decisions/ADR208_docket_sitting_2026_08_17.yaml` (per-item outcomes there), ruling
> comments posted on #589/#564/#546/#590/#563/#557/#572/#522/#594/#529/#527/#379/#382,
> #566 and #335 closed, #599 (Bevy reference-surface re-charter) and #600 (narration-ladder
> tracker) filed, #334 explicitly kept open. This document is now a historical record of the
> option spaces as presented.

**Purpose.** One batched sitting to clear the accumulated Director-gate pileup. Pattern: the
ADR202 curves session (2026-08-14, #561), which discharged eight register rows in a single
sitting and unblocked four port trains at once. Same shape here — reserved-line items first
while attention is freshest, then mixed, then engineering batches you can rubber-stamp or
override in a sentence, then the confirmation-closes.

**Coverage.** Seven readers swept: register #564 (all 24 rows, comments + cross-issues);
faction-enum gate #589 (+ ADR204 W9, ADR206, worldview-ternary design doc); Wave A #566 and
intrinsic-host #576; data gates #545/#546 (+ the 2026-08-12 data-gap audit report, with
source-file verification of the named enums/symbols); the §11 Standard gate family
#376/#377/#378/#379/#380/#381/#382/#336; a full open-issue gate-shaped census (everything
else gate/charter/seed-shaped); and a dedicated WS3-due / Checkpoint-A adjudication check
against state.yaml, ADR198, #502, #557, #578 and the ported-rules filesystem.

**Gaps and uncertainty (honest line).** The synthesis pass flagged six unverified lines; a
follow-up verification pass (same day) closed **all six**, and the items below carry the
verified facts inline. Two verifications materially changed the docket: (d) register row 13's
cited `#536` input turned out to be a **miscitation** — #536 is a hyperedge-lane charter-rider
issue that never mentions floors, consciousness, nationality, or ADR171; the real floor
defaults live in frozen source with declared provenance (see D-02); and (e) #334 turned out
**not stale but live** — its chartered artifact does not exist anywhere and the 2026-08-12
audit names its absence an Aleksandrov failure (C-12 is flipped to DO-NOT-CLOSE accordingly).
The other four: (a) D-17's five dead outputs re-verified by name against survey §5c row 24;
(b) #576's eight gated systems enumerated from survey §4.4/4.5 — Decomposition and ControlRatio
are NOT among them (C-03's open thread resolved); (c) D-11 verified against
`FascistFactionSystem` source — zero veteran references exist in `src/babylon/`; (f) EXIOBASE's
second blocker (L-MAT-6 / OQ-5 letter collision) stands as recorded at D-21.

Four genuine conflicts *in the record* are recorded rather than smoothed: D-07 (an inline
"adjudicator: yes" against an unchecked gate), D-09 (a closed port issue against a
still-unchecked frozen-side ask), D-15 (three documents disagreeing about what "Checkpoint A"
means), and D-02 (register row 13 citing #536 for content #536 does not contain — a register
hygiene fix is owed alongside whatever ruling lands).

---

## A. IDEOLOGICAL / RESERVED LINE — options and consequences only

> Constitution IX.5: the ideological/theoretical line is the Director's sole authority. Every
> item in this section is presented as an option space with engineering consequences. Where a
> reader supplied a lean, it has been stripped — noted per item.

### D-01 — Faction-classification enum: rule the closed member list
**DECISION:** Rule the member list of the faction-classification enum (which values, how many,
and which current free-text ideology strings map onto each) so the ADR195/196 `defenum`
ceremony can mint it.
**SOURCE:** #589 (the gate; 0 comments) + #564 row 14 + ADR204 W9 + ADR206 (executed sibling
ceremony, PR #586) + worldview-ternary design doc §4.1/§4.4/§5.
**BLOCKED-WORK:** the vocabulary-ceremony PR that mints the enum; downstream the
FascistFaction / Allegiance / Electoral port trains that ADR202 R6 already scheduled; and
row 9's FAC_DECOLONIAL-never-capturable conformance vector, which needs a stable declared
property to pin against. #589 sits `blocked:owner`, agent spend ~0 until ruled.
**EVIDENCE:** W9 already ruled the *mechanism* (one content-declared closed enum, not a token
substring probe). Membership is explicitly Director-only (design §5, "taxonomy membership").
Four token homes die on mint: `reactionary.py:71` and `allegiance.py:77` (identical
`("fascist","reaction","revanch","settler")`, substring-matched against free-text `ideology`
— this is row 9's bug: `"settler"` matches inside `"anti-settler abolitionism"`);
`electoral.py:139-144` `_FACTION_BY_IDEOLOGY`; `electoral.py:159-164` `_SPOILER_POLES`;
`migration.py:32-37` `_IDEOLOGY_TENDENCY`. Seed content (`seed_factions.json`) carries four
free-text ideologies and **no born-fascist row** (W3: fascism is the capture terminus), while
`PoliticalFaction` *does* carry a standing `"fascist"` key — an asymmetry any single shared
enum must reconcile. The enum needs a field home on **two** entity types
(`BalkanizationFaction` and `PoliticalFaction`), so the `deffield` work doubles under every
option.
**OPTIONS:**
- **A — reuse the 3-pole `WorldView` enum** (factions carry `world_view`). No new ceremony, a
  `deffield` on the already-minted enum, ordinal law fixed by ADR206. Costs: collapses
  `liberal_imperial` and `social_democratic` into one pole, discarding the left/right split
  `_SPOILER_POLES` runs on; and a faction seeded FASCIST at tick 0 contradicts W3's
  capture-terminus ruling.
- **B — a wider 5-ish enum** (RESTORATIONIST / SOCIAL_DEMOCRATIC / DECOLONIAL /
  LIBERAL_IMPERIAL / FASCIST_VEHICLE). Full ceremony; capturability, spoiler poles and
  fascist-vehicle all become one declared lookup — kills the substring-bug class by
  construction. Raises: is FASCIST a state a faction *transitions into* (mutable, which
  ADR195/196 static-content enums do not model) or a born label? A mutable target needs a
  separate flag layered on the enum, an added ceremony surface.
- **C — charter separately with a minimal capturability-focused list**, deferring the electoral
  map consolidation to the port trains per ADR202 R6. Smallest ceremony; reproduces the
  "one classification, two homes" duplication W9 promised to end.
**Constraint binding on all three:** row 9 (ruled 2026-08-12) requires FAC_DECOLONIAL's mapped
member resolve to never-capturable.
**DIRECTOR SIGNAL (2026-08-17, in-session):** "a lot of that can be abstracted into
generalized factions with specific parameters for fascist" — generalized faction machinery,
fascism as parameter values rather than special-cased code. This is the option-B shape taken
seriously: one declared classification whose members each carry a parameter row (capturable?,
spoiler pole, capture-terminus/vehicle flag), so `_find_fascist_faction`-style special-casing
and the four token homes die into declared content, and any faction whose parameters make it a
capture target behaves as one. Port-as-is discipline (ADR183) still sequences it: the frozen
FascistFactionSystem behavior transcribes first; the generalization lives NOW in the enum +
parameter table (declared content, no port violation) and the machinery generalization rides a
post-port D-record.

### D-02 — Community `SUBSTRATE_FLOOR_DEFAULTS`: consciousness floors by oppressed nationality
**DECISION:** Transcribe the floor defaults verbatim under existing ADR171 authority, or treat
them as a new declaration needing its own sign-off?
**SOURCE:** #564 row 13 (cites #536 as input; unchecked, no ruling comment).
**BLOCKED-WORK:** Community System port train's floor-default transcription (#536 rider).
**EVIDENCE (verified 2026-08-17):** Floors keyed by oppressed-nationality status sit directly
on the National Question line (ADR171, MIM+MLP framing, bribe:deprivation 1.55). **Row 13's
`#536` citation is WRONG**: #536 is the Community train's hyperedge-lane charter-rider issue
(shape-verb collect-then-apply, membership heads) and never mentions floors, consciousness,
nationality, or ADR171. The actual defaults live in frozen source:
`models/entities/consciousness.py:356` `SUBSTRATE_FLOOR_DEFAULTS`, keyed by `CommunityType`
(NEW_AFRIKAN 0.12, FIRST_NATIONS 0.12, INCARCERATED 0.18, CHICANO 0.08, …), each row carrying
declared provenance (`data_sources`, `computation_method` — Vera incarceration rates, Chetty
mobility atlas), consumed at `engine/systems/community.py:453`. A register hygiene fix
(re-point row 13's citation) is owed alongside whatever ruling lands.
**OPTIONS:** (a) transcribe under ADR171 — fast, consistent with the ratified ruling, but
assumes ADR171 already covers this specific floor mechanism; (b) fresh ruling distinct from
ADR171 — slower, avoids extending a ruling's scope by inference. The frozen rows' declared
provenance fields make (a) more defensible than it looked before verification: the values are
sourced, not invented.

### D-03 — ReserveArmy border-valve throttle: "the settler-wing wage bargain"
**DECISION:** Same ADR171-scope question for the border-valve wage throttle.
**SOURCE:** #564 row 20 (unchecked, no ruling comment).
**BLOCKED-WORK:** ReserveArmy port train's border-valve mechanism (also a curve-bearing system
under ADR202/T4, so it is already queued for porting).
**EVIDENCE:** Named in the register as riding the ADR171 surface — the same authority row 13
cites, which is why it is docketed adjacent to D-02 and can be ruled with one motion.
**OPTIONS:** transcribe under ADR171 authority, or require its own ruling. Ruling D-02 and
D-03 together with a single scope statement ("ADR171 does / does not cover downstream
mechanism transcription") disposes of both.

### D-04 — FactionInfluence: `colonial_stance` as the principal-contradiction axis
**DECISION:** Confirm port-as-is, or reopen the axis choice.
**SOURCE:** #564 row 16 (unchecked, no ruling comment).
**BLOCKED-WORK:** FactionInfluence @14.5 port train.
**EVIDENCE:** Principal-contradiction axis selection is core MLM-TW framing. Memory note
`doctrine-tag-national-chauvinism` records the axis as already grounded in `colonial_stance`,
which is what port-as-is would lock into Rust/BSL.
**OPTIONS:** (a) confirm port-as-is — locks `colonial_stance` as THE principal axis going
forward, matching frozen precedent; (b) reopen — delays the port, requires fresh theoretical
justification before transcription can start.

### D-05 — Contradiction: national opposition + `_STANCE_CHAUVINISM_SCORE` restraint level
**DECISION:** Confirm the frozen restraint level, or set a different one now.
**SOURCE:** #564 row 15 (unchecked, no ruling comment).
**BLOCKED-WORK:** Contradiction System port train's national-opposition axis.
**EVIDENCE:** The scoring *magnitudes* are themselves the reserved content — there is no
engineering-only path here, which is why the row asks for a confirmation rather than a design.
**OPTIONS:** (a) confirm as-is; (b) tighten; (c) loosen. Any change away from as-is drifts the
goldens and owes a §6.5 baseline ceremony.

### D-06 — Struggle: George Jackson bifurcation + SocialRole taxonomy
**DECISION:** Confirm port-as-is, or revisit the taxonomy.
**SOURCE:** #564 row 17 (unchecked, no ruling comment).
**BLOCKED-WORK:** Struggle System port train.
**EVIDENCE:** A named theoretical lineage (the George Jackson bifurcation model) plus a
class/role taxonomy.
**OPTIONS:** (a) port-as-is — preserves the named attribution unchanged; (b) revisit —
reopens a settled theoretical attribution and delays the Struggle port.

### D-07 — Doctrine tree: is transcribing the 14-node tree into `.bscn` defconsts Director-gated?
**DECISION:** Rule whether BSL transcription of the 14-node Doctrine tree requires an explicit
Director act.
**SOURCE:** #564 row 10.
**BLOCKED-WORK:** Doctrine System @14.7 port train (ADR073/ADR137 machinery) cannot begin
transcription.
**EVIDENCE / CONFLICT IN THE RECORD:** the row text contains an inline self-answer
("adjudicator: yes") but the checkbox is unchecked and **no ruling comment exists anywhere**.
Recorded as a conflict, not resolved by the workforce.
**OPTIONS:** (a) treat as gated, require an explicit ruling comment before transcription —
matches the row's own annotation, blocks Doctrine until scheduled; (b) accept the inline
annotation as authorization — faster, but sets a precedent that inline notes substitute for a
ruling comment, weakening the gate's evidentiary trail. *(Reader lean stripped: engineering
framing only; line is reserved.)*

### D-08 — Sovereignty `_STANCE_TO_POLICY`: hash-bearing declaration order on a reserved axis
**DECISION:** Preserve the frozen declaration order verbatim, or re-derive a principled one.
**SOURCE:** #564 row 11 (unchecked, no ruling comment).
**BLOCKED-WORK:** Sovereignty System port train — the stance→policy map and its
metabolic-impact coupling cannot be transcribed until order is ruled (declaration order is
hash-bearing under ADR195).
**EVIDENCE:** This is simultaneously a determinism decision and a reserved-axis ordering
decision, which is why it cannot be settled as pure engineering.
**OPTIONS:** (a) port-as-is — byte-parity with the frozen reference, no re-litigation, but
freezes whatever order legacy Python happened to declare; (b) re-derive now — clarity gain,
but breaks golden-hash parity, owes a declared §6.5 baseline ceremony, and opens a reserved
ordering question. *(Reader lean stripped; line is reserved.)*

### D-09 — Consciousness `national_identity: 0.5` absent-data default (OPEN-NARROW)
**DECISION:** Close row 19 on the strength of the port, or require an explicit re-declaration
covering the frozen sites.
**SOURCE:** #564 row 19 + its W10 comment (#588 / ADR207, issue CLOSED).
**BLOCKED-WORK:** Nothing engineering-side — the ported class surface already kills the
fabrication. Only the frozen-side declarative ask is live.
**EVIDENCE / CONFLICT:** the port comment states the disease "dies by law" — an unpositioned
class carries no ternary fields, a raw read errors loud (III.11), and absence is observed only
through declared `:optional`/`:default 0.0p` gated on `(> (+ r (+ l f)) 0)`, never a fabricated
0.5. Full frozen-disease register in `docs/concepts/consciousness-taxonomy.rst` (L-ABS
section) with exact sites `ideology.py:74/82/89` (consumed at `:359`/`:411`),
`struggle.py:100/107/128/139/164/171`, `solidarity.py:66/73`, `social_class.py:99`,
`bridge.py:93-100`, `node_access.py:15-37`. **But the checkbox is still unchecked** and the
comment explicitly scopes itself to port-side evidence only, leaving the frozen-side ask to
the Director.
**OPTIONS:** (a) close now — treat the frozen Python sites as inert/reference-only, no further
action; (b) require an explicit re-declaration statement so the historical record shows the
"a class with no ideology record is half-nationalist" fabrication was *named and rejected*
rather than silently superseded by a port.

### D-10 — Charter a rentier / absentee-landlord class or organization node?
**DECISION:** Does the game model a distinct rentier fraction of the bourgeoisie, and if so as
what?
**SOURCE:** #546 ruling item 3; data-gap audit §8.7 #3.
**BLOCKED-WORK:** `compute_ground_rent`'s extraction has no destination
(`ground_rent.py:65` books `rent_from_v`/`rent_from_s` from variable capital and surplus value
with no receiving entity); `dim_wealth_class` has no rentier row
(`data-artifacts.yaml:534-541`); and the USDA 2024 TOTAL survey (2.0M landowners renting
348M acres, 79% non-operating, $34.1bn rental income vs $12.0bn expenses) cannot be acquired
under the Director's own wire-before-buy rule until a consumer exists. T2 (#540, ground rent)
will surface the gap immediately once agricultural rent is grounded.
**OPTIONS:** (a) a discrete rentier/landlord class or organization node distinct from the four
Fed-DFA wealth-class rows — touches `dim_wealth_class` schema, likely a new NodeType or
membership category; most invasive, theoretically cleanest, and the audit rates the USDA survey
"worth its weight on the pedagogy criterion alone"; (b) rentier income as a sub-fraction of the
existing bourgeoisie — cheaper, no new NodeType, but the booking still needs *some* receiving
entity so this may relocate rather than close the gap; (c) decline — leaves
`rent_from_v`/`rent_from_s` permanently unreceived and kills USDA TOTAL acquisition.

### D-11 — Does county veteran population wire into `FascistFactionSystem` recruitment?
**DECISION:** Who fascism recruits.
**SOURCE:** #546 ruling item 5 (audit §6.3, §10).
**BLOCKED-WORK:** the wiring decision only — the loader itself is a trivial ACS pull
("wiring, not acquisition"). #546 is explicit that this "must be ruled, not assumed by whoever
writes the loader."
**EVIDENCE (verified 2026-08-17 against source):** zero veteran/military references exist
anywhere in `src/babylon/` (rg sweep). Recruitment is the drift→capture path in
`engine/systems/reactionary.py:109-177`: pull = `calculate_fascist_pull(agitation,
entitlement, incident-SOLIDARITY, epsilon)` (`:127-135`), eligibility gated on `SocialRole ∈
_ENTITLED_ROLES` (`:123-125`), capture fires at `fascist_recruitment_threshold` (`:159-165`).
The natural seam for a veteran input is **`entitlement` seeding** — already a general per-class
scalar the pull formula reads — not a new formula term; that keeps the wiring inside the
generalized-parameters direction the Director signaled at D-01.
**OPTIONS:** (a) wire as a positive input to fascist recruitment probability — trivial loader,
existing system inputs, no acquisition; (b) treat veteran status as recruitment-neutral — the
ACS field sits unconsumed by this mechanic; (c) route the same loader to a different consumer
(Consciousness or Struggle rather than fascist recruitment) — same cost, different valence.

---

## B. MIXED — ideological half reserved, engineering half leaned

### D-12 — County-varying subsistence threshold into `base_subsistence`?
**DECISION:** Should ERS food-access data spatially modify the subsistence threshold inside
the Survival Calculus's P(S|A)?
**SOURCE:** #546 ruling item 6 (audit §6.3, §10).
**BLOCKED-WORK:** acquisition of the ERS food atlases is gated on this — the data is only
worth buying if the mechanic is chartered.
**EVIDENCE:** `base_subsistence` is a live, actively-consumed coefficient, verified in
`config/defines/economy_basic.py`, `domain/economics/lifecycle/cohort_dynamics.py`,
`engine/hydration/reference.py`, `engine/optimization/bayesian.py`,
`engine/systems/vitality.py`, `engine/systems/economic.py` — not a hypothetical hook. #546
already anticipates the ADR173 constraint correctly ("never a bolted-on food subsystem").
**ENGINEERING LEAN:** if chartered, the entry point is `base_subsistence` per-county and
nothing else — that path is ADR173-clean (a genuine input to the emergent S-curve, no imposed
form) and reuses existing machinery.
**RESERVED HALF (no recommendation):** whether modeling geographic food-access disparity as a
material input to rupture probability is the correct theoretical framing at all. Options:
(a) charter as a `base_subsistence` modifier; (b) decline — ERS stays on the defer list,
subsistence stays spatially flat; (c) a broader food-security subsystem — which #546's own
framing pre-emptively rejects as the wrong shape.

### D-13 — Charter the per-county (r, l, f) seeding data spec (ADR043's named replacement)
**DECISION:** Approve the ACS-correlates + 2010–2020 election logit-model construction as the
spec before the build train starts.
**SOURCE:** #590 (ADR204 W7; worldview-ternary design doc §4.6).
**BLOCKED-WORK:** T7 (#545) and the synthetic community-defaults calibration (spec 034
SC-007) both wait on approval; until then ADR043's uniform placeholder (0.05, 0.50, 0.45)
governs. Deliverable is the spec (model + lineage + validation), ~3 Mtok / 2 windows; the
pipeline is a separate train after.
**ENGINEERING LEAN:** no objection to chartering the spec-drafting work — ACS+election logit is
methodologically standard and spec-first matches the declared-not-derived provenance rule.
**RESERVED HALF (no recommendation):** the model embeds a theoretical commitment about how
objective political disposition correlates with demographic/electoral covariates — the same
class the Director reserved for #510 and ADR171. Options: (a) approve as specified — cheap,
ships soon, risks encoding liberal-electoralist assumptions about affiliation into a
materialist simulation; (b) demand a more materially-grounded covariate set (the #510
SCF-microdata analogue) — slower, more defensible; (c) bless as provisional/gameplay-only
pending revisit, matching #510's "not v1.0-blocking" posture.

### D-14 — TickDynamics reserved-line pack (bifurcation score, ClassDistribution, cascade milestones)
**DECISION:** Two halves. Engineering: proceed with #563's dormancy re-read and
ServicesProtocol charter. Reserved: the bifurcation directional score, the five-share
ClassDistribution (Feature-016) and `dispossession_cascade_milestones`.
**SOURCE:** #564 row 21 (unchecked) + #563 (OPEN, umbrella #557) + #578 critical-path map.
**BLOCKED-WORK:** TickDynamics port train's reserved-line section; #563 carries the explicit
RESERVED-LINE section alongside its engineering-only work (per-scenario dormancy re-derivation
across six county-bearing canonical scenarios, ~28-field ServicesProtocol boundary,
round-half-even, FIPS encoding).
**ENGINEERING LEAN:** run #563's dormancy re-read and ServicesProtocol charter now — no
reserved-line entanglement, well-evidenced.
**RESERVED HALF (no recommendation):** port-as-is for the bifurcation score / ClassDistribution
/ milestones vs. re-derive. Note the sequencing: the reserved question cannot be *posed
precisely* until #563's charter output exists, so a "proceed on the engineering half, hold the
reserved half for the next sitting" ruling is a coherent disposition here.

### D-15 — ★ WS3-due adjudication: what does "Checkpoint A" mean? (PROMINENT)
**DECISION:** Is WS3 (type-surface completion + en-masse workaround retirement) now due — i.e.
does "Checkpoint A / MATERIAL BASE COMPLETE" mean "the four named gate-trains landed" or "all
13 Material Base systems are ported in Rust"?
**SOURCE:** `ai/state.yaml:3660`; ADR198 R8; #502 (WS1–WS4 definitions and sequencing); #557
(umbrella, OPEN); #578 (critical-path map); `reports/port-estate-survey-2026-08-12.md` §4.3;
the P29 design doc; direct filesystem check of `rust/crates/babylon-tick/content/rules/*.bsl`.
**BLOCKED-WORK:** whether to launch the WS3 sweep (mint the missing Real-lane declared-domain
op, then retire every bare-Int/scaled-Int workaround across packs — Dispossession D-2/D-4,
Metabolism D-1) now or hold it.
**EVIDENCE — the conflict, recorded not smoothed:**
- ADR198 R8 names Checkpoint A's gates as exactly T2/T3/T4/T6 — four infra/design trains, and
  all four are now cleared as *design/infra deliverables*: T2 landed (#575/ADR201), T3 landed
  (PRs #582+#585, ADR203+ADR205), T4 "ruled" (ADR202's own consequences line says the gate is
  **satisfiable**, not that the curve-bearing systems are ported), T6 delivered as dormancy
  memos + a ServicesProtocol charter verdict.
- But the Material Base in the engine's own terms is **6/13 ported** — Vitality, Territory,
  Production, Lifecycle, Dispossession, Metabolism exist as `.bsl`; TickDynamics, ReserveArmy,
  Community, Solidarity, ImperialRent, Decomposition, ControlRatio do not. Those seven are
  precisely what T2/T3/T4/T6 were chartered to *unblock*. Clearing a blocker makes a system
  portable, not ported.
- #557's own closure bar is stronger than the four gates: "all six trains **and waves A-D**
  merged with evidence." T1 and T5 unaddressed; even Wave A is half done (#566 still open).
- #578 schedules the actual porting of Solidarity/ReserveArmy/ImperialRent/TickDynamics/
  Community under **Phase 1 → Checkpoint B**, i.e. *after* the phase whose own title calls its
  trigger "MATERIAL BASE COMPLETE." The map is internally inconsistent with itself.
**ENGINEERING LEAN (the process half):** hold WS3. Two reasons: #557's closure text is the more
specific and more recent statement, and the survey's "at minimum / unreachable until" language
supports necessary-not-sufficient; and WS3 can only retire workarounds that exist in landed
code, so running it now catches 6 systems' workarounds and misses 7 — forcing either a second
pass or an incomplete sweep. Recommend also correcting `ai/state.yaml:3660` from "P29
Checkpoint A completion" to "Program 29's T2/T3/T4/T6 gate-trains complete (Checkpoint A's
gates cleared; Checkpoint A itself not yet reached)" so the ledger stops misdirecting the next
agent.
**RESERVED-ADJACENT:** if the Director's intent when ratifying ADR198 R8 genuinely was "the
four named gates ARE Checkpoint A, full stop" — a legitimate reading of R8's literal text —
that is a one-line ruling the other way, and it changes the landing order of systems the
Director has stakes in (Solidarity, ImperialRent).

---

## C. ENGINEERING — batched by theme, each with a LEAN

### Theme E1 — Register #564 port-methodology residue

**D-16 — Honest-partial packs: does Territory's no-sliver precedent bind?**
SOURCE #564 row 23 (unchecked). BLOCKED-WORK: ContradictionField+FieldDerivative, Policy,
MarketScissors port trains — full port vs. accepted partial.
EVIDENCE: no ruling comment; the row weighs Territory's established no-sliver precedent
against a proposed "carrier pilot" partial, but contains no concrete cost/benefit case for the
pilot.
LEAN: **no-sliver binds; no exception now.** Consistent with ADR183 port-as-is discipline and
the post-port-refactor norm. Revisit only if a carrier pilot is proposed separately *with*
evidence.

**D-17 — Shadow/dead outputs: port verbatim or retire on the WS4 ledger?**
SOURCE #564 row 24 (unchecked). BLOCKED-WORK: whether `field_registry` (never wired) and five
verified-dead outputs get transcribed into BSL/Rust.
EVIDENCE (verified 2026-08-17 vs survey §5c row 24): the five = `transport_demand_signal`,
`effective_controller_by_territory`, `sigma_capital_labor`/`derived_class_cell`,
`wealth_share`, MarketScissors' `_swell_reserve_army` — all "verified dead outputs with zero
readers"; `field_registry` is tracked separately as "never wired in production"
(ContradictionField Computation 1, FieldDerivative Phase 0) and classed NOT-A-PACK. The
survey states the zero-readers conclusion without restating its method.
LEAN: **retire on the WS4 ledger, not port verbatim.** WS4 is the ratified home for exactly
this class of dead/unwired disposition, and porting verified-dead code forward has no cited
benefit. Names now confirmed; the WS4 ledger entry should carry a fresh zero-readers grep as
its own evidence row since the survey doesn't restate one.

**D-18 — Slice-4 first consumer: which half do Allegiance/Electoral need?**
SOURCE #564 row 22 (unchecked). BLOCKED-WORK: Allegiance/Electoral Slice-4 wiring.
EVIDENCE: mostly already decided — the dyadic half is RULED (ADR198 R1–R3, #560); any
hyperedge-membership half rides R4/#536. Residue is a narrow post-T3 confirmation.
LEAN: **defer by construction** — the row's own text says it can only be confirmed "after T3
lands." Schedule as a fast follow-up check immediately after #560 merges; do not spend a
ruling slot on it today.

### Theme E2 — Data schema and license (#545 / #546)

**D-19 — Extend `BiocapacityType` (and/or `TerrainType`) for cropland/soil**
SOURCE #545 + #546 §8.7 #1; audit §8.7 #1 (599-608), §0.3.4.
BLOCKED-WORK: **the entire T7 B-construction train** (CDL / Annual NLCD land cover, gSSURGO
NCCPI soil fertility, ERS Major Land Uses control totals, NASS Land Values), plus Annual NLCD
(which also carries the #379 multi-res consumer gate).
EVIDENCE verified in source: `models/enums/territory.py:117-152` — `TerrainType` =
LAND/WATER/RESOURCE only; `BiocapacityType` = FRESHWATER/FISHERY/SHIPPING_ACCESS/MINERAL/
TIMBER/HYDROELECTRIC, all keyed off non-LAND terrain. All cropland lives on LAND hexes, which
therefore carry zero biocapacity stock — B is flat 100.0 in every scenario, leaving
ECOLOGICAL_COLLAPSE ungrounded on the agricultural side. Enum invention is
Constitution-protected, hence the gate. Raster doctrine is already settled and non-blocking
(one-time zonal-statistics fixture; raster never a runtime dependency) — this enum ruling is
T7's sole blocker.
LEAN: **extend, following the existing pattern** — add a `BiocapacityType` member (analogous to
WATER→FRESHWATER/FISHERY and RESOURCE→MINERAL/TIMBER) rather than inventing a fourth
`TerrainType`; terrain is physical classification, biocapacity is stock. Zero existing
construction sites depend on the current shape, so migration cost is nil.

**D-20 — Add an agricultural share to `HexTenureComposition`**
SOURCE #546 §8.7 #2; audit §8.7 #2, §0.3.3.
BLOCKED-WORK: nothing today; prerequisite alongside D-19 for any agricultural tenure modeling
(ground rent, dispossession consumers).
EVIDENCE verified in source: `domain/economics/substrate/types.py:88-108` — frozen model with
exactly 7 shares under a strict sum-to-1.0 validator, no agricultural/farmland category.
LEAN: **approve, and rule it now rather than batching** — strictly cheaper than D-19 (zero
consumers to migrate), purely additive (new share + validator update). Grouped with D-19 only
because both surfaced from the same audit pass, not because they are coupled.

**D-21 — EXIOBASE 3's CC-BY-SA 4.0 share-alike**
SOURCE #546 ruling item 4; audit §10 (line 656).
BLOCKED-WORK: EXIOBASE 3 acquisition (international ecological-exchange lane). Not fully
stalled — the in-DB ERDI series is the honest bootstrap meanwhile.
EVIDENCE: the audit names **two** blockers — (a) the share-alike virality, the only viral
license in the slate, which would propagate into sha-pinned build products; and (b) L-MAT-6
ratification, which lives in an inbox document with an unresolved OQ-5 letter collision.
**#546 surfaces only (a)** — flagging (b) as a possibly-missed second blocker.
LEAN: **do not admit EXIOBASE as a build-product dependency** unless it can be quarantined as a
non-redistributed, locally-computed-only input so the virality cannot reach the parquet/DB
products. Consistent with ADR098's redistribution-rights-first hard filter and with how
ZTRAX/ATTOM/CoreLogic/Enverus were killed in §9. ERDI removes any urgency.

### Theme E3 — BSL / content surface

**D-22 — #572: disposition of the `the` accessor (carrier-anchored non-carrier-subject read)**
BLOCKED-WORK: **four named first-consumers** — Allegiance, Electoral, Policy,
WealthDistribution — blocked because T1 §6.2's carrier-anchored subject-side pattern doesn't
cover this read shape.
EVIDENCE: T2's charter row (ADR198/#559) scopes T2 to dyadic edge reads; `the` is tagged
slice-2 in UNSERVED_EXPRESSION_HEADS and ADR197 but is a carrier read, not an edge read. Three
options offered (fold into T2 / own micro-train / ride T3). Umbrella #557.
LEAN: **fold into T2 as an explicit scope widening.** Hash-free and read-only like the rest of
slice 2, and T2's evaluator machinery makes it cheap there; a separate micro-train adds
coordination overhead for four already-blocked consumers; T3 is the weakest fit by the issue's
own admission (storage keystone vs. a read-side need).

**D-23 — #522 / D93: is the `.bscn` scenario `deffield` positional, or does it move to §2.9's keyworded form?**
BLOCKED-WORK: Tier-1 drafting (normative scenario-dialect chapter, `bsl.ebnf` productions,
sync-guard rows), which gates Tier-2 (format-version marker, content-compat policy — needed
"before mods are invited") and Tier-3 (v1.0 modding boundary: validator CLI, tree-sitter).
EVIDENCE: the real spec today is `scenario.rs` itself — "the exact contract implied by one
implementation our own standards forbid." scenario/node/edge top-forms appear nowhere in
`bsl.ebnf`; 8 scattered `.bscn` mentions in `bsl-language.rst` with no chapter; load-bearing
rules (declaration-order-is-id-order — hash-relevant — local-names-don't-survive-load,
EnumRef-only seeding) live only in module docstrings.
LEAN: **charter the chapter rather than rule a bare rename** — the chapter resolves D93
naturally and simultaneously discharges the docstring-only documentation debt. No new
formalism is minted, so no amendment needed; this is a visibility ruling only.

**D-24 — #594: f→r ε-gate wiring as its own D-record micro-train**
BLOCKED-WORK: the W10 consciousness-routing surface. `apply_fr_gate` has no frozen production
caller (only a unit test), and the gate's conjunction depends on a PROLETARIANIZATION signal
with no ported definition yet — which must be defined from the ported value-flow estate first.
EVIDENCE: `formulas/consciousness_routing.py:474-511` is minted-but-unwired in the frozen
estate; the define `fr_gate_epsilon` exists (`config/defines/reactionary.py:129`,
`defines.yaml:945`) but production never calls it. Port archaeology digest A.5a corrects #588's
"preserved verbatim" framing. Mechanics are expressible in current BSL (`when` +
`exists(neighbors … SOLIDARITY)` + two update-node effects).
LEAN: **approve as scoped — its own D-record micro-train.** This is textbook Wiring Doctrine
(ADR109): connecting a built-but-dormant construct is a typed motion owed its own sentinel row,
never bundled into an unrelated transcription claim. No counter-argument found.

### Theme E4 — Persistence gates and two dropped investigations (#379 / #382)

**D-25 — P-E: multi-res persistence storage shape + res-7 refinement coverage budget**
SOURCE #379 (Standard §11 item 27), 2026-07-30 ruling comment. BLOCKED-WORK: Phase 4
multi-res/hex persistence schema design (Rust side).
EVIDENCE: explicitly held open in the 2026-07-29 batch — "P-E … needs the full-shape reference
run first"; every other #379 item (Q1, Q9, Q11a/d, Q11b, P-C, P27 Task 8) was ruled. No later
comment reports the reference run complete. Budget stakes: ~2.3 GB vs 10 GB+/campaign; mixed-
resolution vs per-resolution table; mid-campaign refinement as fresh checkpoint vs deltas.
LEAN: **hold — sequencing is already correct.** Run the full-shape reference campaign, then
decide from measured coverage rather than pre-committing.

**D-26 — P-A: hex-delta inertness (an apparently DROPPED item)**
SOURCE #382 (Standard §11 item 21); #379's ruling comment calls P-A "a workforce investigation
item." BLOCKED-WORK: explicitly "blocks Phase 4 hex persistence."
EVIDENCE: #382's 2026-07-29 ruling comment covers P-J/P-B/P-D/P-F/P-H/P-I/W-I and flags P-G
open — **and never mentions P-A at all**, ruled or open. The 2026-08-11 status comment lists
only P-D and W-I as remaining. This reads as a dropped item, not a closed one.
LEAN: **charter the investigation explicitly** — is the write path inert (a bug) or is 500+
ticks of a genuinely static hex layer expected at current pacing? No Director ruling is
actionable until that is answered; the action needed today is scheduling, not adjudicating.

**D-27 — P-G: sparse-table exact-tick trace read — defect or perf?**
SOURCE #382 (Standard §11 item 28), 2026-07-30 ruling comment.
EVIDENCE: the ruling deferred itself — "P-G is a workforce investigation first; defect-vs-perf
ruling follows evidence." No later comment reports the investigation run.
LEAN: **run the investigation first** — specifically, does `view_runtime_trace_emission`'s
exact-tick read silently miss rows the way the documented `dynamic_hex_state` sparsity gotcha
predicts (`MAX(tick)` ≠ last committed tick)? Classify after.

### Theme E5 — Hook / gate friction (both blocking real pushes today)

**D-28 — #529 (+#580): repair-or-ratchet for four pre-existing radon-mi C-ranked files**
BLOCKED-WORK: **any push touching `src/babylon` Python**, unless `SKIP=radon-mi` is used as an
undocumented stopgap.
EVIDENCE: four C-ranked files — `topology/graph.py` (5.96), `engine/systems/electoral.py`
(8.16), `sentinels/_ast.py` (0.00), `domain/economics/tick/system/__init__.py` (3.42).
Deterministic across 5/5 runs, reproduced in two checkouts (one unexplained early "Passed"
anomaly noted, not evidence of fresh debt). Gate installed 2026-08-11 per the #525 ruling.
LEAN: **ratchet first, burn down opportunistically** — a cited/dated allowlist of the four
offenders, precedented by the existing `EXTRA_STAMPABLE_ATTRIBUTES` / `ATTRIBUTE_EXEMPTIONS`
exemption-governance pattern. All four sit in active-program estates where a drive-by refactor
risks scope creep and conflicts with in-flight port trains; blocking real work on an
unscheduled refactor is the worse failure mode. The issue reaches the same conclusion.

**D-29 — #527: deliberate-change path for `uv.lock` bumps + the `UV_FROZEN` / `uv lock --check` conflict**
BLOCKED-WORK: every legitimate dependency-lock bump (e.g. the Dependabot security fix in
PR #526) currently needs `SKIP=worktree-contract` plus a manual workaround.
EVIDENCE: `check_lock_unmodified` (`tools/check_worktree_contract.py:76-85`) refuses any
uv.lock-vs-HEAD diff unconditionally; separately the standard commit prefix maps `UV_FROZEN=1`
to `--check-exists`, which conflicts with the hook's own `uv lock --check` and exits 2. Both
surfaced in PR #526's commit body as declared workarounds.
LEAN: **approve both one-line fixes** — an explicit override env (e.g. `LOCK_BUMP=1`)
downgrading `check_lock_unmodified` to a warning while keeping the default accidental-relock
protection, and `env -u UV_FROZEN uv lock --check` in the hook (hook-side fix preferred; the
prefix-drop alternative is more fragile). Bundle into the next lock-touching commit.

---

## D. RESOLVED / SUSPECTED-MOOT — one-line confirmation-closes

| ID | Item | Status to confirm |
|----|------|-------------------|
| C-01 | #564 rows 1–8 (all curve reformulations) | DISCHARGED 2026-08-14 via ADR202 R1–R9 at the #561 session (CLOSED). Stale — do not reopen. |
| C-02 | #564 row 9 (FascistFaction FAC_DECOLONIAL mis-selection) | RULED 2026-08-12 — repair-at-the-port as a declared property, conformance vector pins never-capturable, D-record at the port train, golden divergence declared per ADR183. Becomes structurally permanent once **D-01** lands. |
| C-03 | #564 row 12 / #566 Wave A gate | RULED 2026-08-12 — joint Decomposition+ControlRatio train stands, port-as-is, revolution-vs-genocide branch verbatim under a P19-cutover-pending D-record; P19's LAST-ordering governs the cutover, not the transcription. #566 carries the "gate discharged" comment yet is **still open** — bookkeeping close/relabel, not a decision. Open thread RESOLVED (2026-08-17, survey §4.4/4.5): #576's eight gated systems are Doctrine, Struggle, OODA, Survival, Consciousness, Community, MarketScissors, ImperialRent — Decomposition/ControlRatio NOT among them; the two trains are fully independent. |
| C-04 | #576 intrinsic-host train (RNG binding + exp/log + sqrt dispatch) | **No ruling needed** — survey §4.5 ranks it buildable-today, no lane dependencies. Its one soft dependency (exp/log call-site shape) already resolved favorably: #561 closed, rows 6–7 discharged as ADR202 R8/R9. Proceed without Director involvement. |
| C-05 | #376 endings & verdicts (Q3/Q7/W-G) | All ruled 2026-07-29 — sixth ending "THE LONG CONTAINMENT"; RED_OGV named plainly (code's UNRESOLVED routing is the acknowledged defect); all three crisis sovereigns, each from its own trigger. PR #386 landed the CHARTER rows. Open as an implementation tracker only. |
| C-06 | #377 doctrine surface (Q2/Q6/Q8/W-F) | All ruled — fourth trunk Autonomist, player-facing trunk names kept, Q8 mechanics/origin/failure, MILITANCY both faces, is_goal demoted to projection label, tag namespace opens by AE rider. Implementation only. |
| C-07 | #378 strike & verb algebra (Q10/W-A..W-E) | All ruled; Article V 3×3 RATIFIED 2026-08-10 (ADR187), sole reservation OQ-5 ruled same day (ADR190). All eight OQ items disposed. Implementation only. |
| C-08 | #380 pacing/density/long-wave (Q4/Q5/W-J) | All ruled and largely implemented (PRs #386, #394–#397, #404). The one consequence — chartering the restoration-channels design train — was sequenced by the 2026-08-11 scheduling ruling (#381/ADR194 R4): B2 tick loop + port lane first, then charter. A scheduled dependency, not an open ruling. |
| C-09 | #381 narrator (Q12) | CLOSED. Register/wrong-line/theory-voice all ruled and shipped (PRs #394, #405). **Hygiene gap:** it cross-references #27 as the sole remaining tracker for the four-tier narration ladder, but #27 is closed under an unrelated title — the ladder likely needs a fresh tracking issue. |
| C-10 | #336 verb-algebra research seed | SUSPECTED-MOOT — strike superseded by #378's W-A..W-E; clandestinity discharged via #408 (CLOSED, ADR180 R15–R20); funding via #407 (CLOSED, ADR180 R8–R14); "metabolic absence" very likely satisfied by OQ-5/ADR190, though #336 never cites it — **inferred, worth the one-line confirmation.** Recommend closing with those three pointers. |
| C-11 | #335 wiki content architecture (Glossary/State/Flavor namespaces) | RESOLVED-BY-SUPERSESSION, pending confirmation — its stated authority was the `babylon-md`/WikiView estate, deleted outright by Amendment AF's (iii) ceremony (ADR186). Close with an ADR186 pointer, or re-charter fresh against `babylon-client` if the namespace design should be re-homed onto Bevy. A stale gate pollutes the docket. |
| C-12 | #334 Phase 0 national incidence artifact (ADR171) | **FLIPPED — DO NOT CLOSE (deep-read 2026-08-17).** ADR171 ruled the line, but #334's chartered DATA deliverable — the per-county × pole incidence table + reproduction-floor rows, registered in data-artifacts.yaml — exists nowhere; the 2026-08-12 audit calls its absence "the Aleksandrov failure the Constitution forbids" (audit report line 142) and `community_memberships` is still `LivenessClass.STRUCTURALLY_IMPOSSIBLE` (`sentinels/seam/registry.py:2171`). Live build work remains (~6 Mtok / 3 windows per its own body). |
| C-13 | #382 P-D (checkpoint-only hydration) + W-I (retire and re-mint all five dead edge types under BSL) + P-B (forward-only) | All RULED; all three still unshipped code per the 2026-08-11 audit comment. Implementation, no Director action. |
| C-14 | Standing observation (audit §1.1 #8) — no decision pending | "Data-on-the-shelf, mechanic-on-fiat is the estate's DOMINANT failure mode, bigger than the acquisition gap": `fact_census_rent_burden` (450k rows), `fact_bls_unemployment_decomposition`, `fact_census_worker_class` (900k), `fact_coercive_infrastructure` all exist unconsumed or mis-consumed while TerritorySystem's heat/eviction pipeline runs on pure defines. Clearest illustration: `fact_eviction_lab_filing` has 6,570 rows but SUM(filings/executions/filing_rate/renter_households) all return 0. Surfaced once because the Director asked to see it; it is a prioritization observation, not a gate. |

---

## Unblock map — what starts moving the moment each is ruled

| ID | Unblocks immediately |
|----|----------------------|
| **D-01** | The `defenum` ceremony PR; FascistFaction / Allegiance / Electoral port trains (ADR202 R6); row 9's conformance vector gets a stable property; four duplicated token homes die. |
| **D-02** | Community port train's floor-default transcription (#536 rider). |
| **D-03** | ReserveArmy port train's border-valve mechanism (already curve-queued under T4). |
| **D-04** | FactionInfluence @14.5 port train. |
| **D-05** | Contradiction port train's national-opposition axis. |
| **D-06** | Struggle port train. |
| **D-07** | Doctrine @14.7 port train's 14-node `.bscn` transcription. |
| **D-08** | Sovereignty port train (stance→policy + metabolic coupling). |
| **D-09** | Closes row 19; nothing engineering-side waits. |
| **D-10** | T2 (#540) ground-rent grounding gets a receiving entity; USDA 2024 TOTAL acquisition becomes permissible. |
| **D-11** | A trivial ACS veteran loader gets a declared consumer (or is stood down). |
| **D-12** | ERS food-atlas acquisition; per-county subsistence in P(S\|A). |
| **D-13** | T7 (#545) and spec-034 SC-007 community-defaults calibration; retires ADR043's uniform placeholder. |
| **D-14** | #563's dormancy re-read + ServicesProtocol charter start; TickDynamics reserved section still held. |
| **D-15 ★** | Either launches WS3 now, or formally holds it and corrects `state.yaml:3660` so the next agent isn't misdirected into an early half-sweep. **The WS3-due adjudication the WS3 reader found live.** |
| **D-16** | ContradictionField+FieldDerivative, Policy, MarketScissors port trains get a scoping rule. |
| **D-17** | `field_registry` + five dead outputs get a disposition (port vs. WS4 ledger). |
| **D-18** | Nothing today — a scheduled post-#560 confirmation. |
| **D-19 ★** | The whole T7 B-construction train (CDL/NLCD, gSSURGO, ERS MLU, NASS); Annual NLCD's shared gate. It is T7's **sole** blocker. |
| **D-20** | Agricultural tenure modeling downstream of ground rent/dispossession; cheapest item on the docket. |
| **D-21** | EXIOBASE 3 acquisition (or its formal decline); ERDI continues meanwhile. |
| **D-22 ★** | **Four** blocked first-consumers at once: Allegiance, Electoral, Policy, WealthDistribution. |
| **D-23** | Tier-1 `.bscn` chapter drafting → Tier-2 compat policy → Tier-3 the v1.0 modding boundary (an external modder is already waiting, #531). |
| **D-24** | The f→r ε-gate micro-train (and forces the PROLETARIANIZATION signal definition). |
| **D-25** | Phase 4 multi-res persistence schema (after the reference run). |
| **D-26** | Phase 4 hex persistence — currently stalled on a dropped item nobody owns. |
| **D-27** | Runtime trace-emission consumers; classification follows the investigation. |
| **D-28** | **Every push touching `src/babylon`** stops needing an undocumented SKIP. |
| **D-29** | Every dependency-lock bump, including Dependabot security fixes. |
