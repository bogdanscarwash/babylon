# Port-Estate Survey — the 29 adjudicated Phase-1 inventories, consolidated (2026-08-12)

**Scope.** Every adjudicated inventory in `reports/port-inventories/` (29 files, 17,530 lines),
read in full including each file's `## Adjudication (2026-08-12)` section — **the adjudicated
verdict outranks the reader's throughout, and where the two disagree this survey carries the
adjudicator's**. Sequencing law: `reports/territory-port-phase1-inventory-2026-08-11.md` (the
Territory precedent + its DEFER verdict and 2026-08-11 UPDATE), `ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml`
(the four-slice cut), and a parallel scout's BSL surface dossier
(`reports/territory-bsl-surface-facts-2026-08-12.md`, committed alongside this survey; verified against dev @ `4e0faf2`).
Facts not carried from an inventory are my own `file:line` checks against the working tree and
are marked *(own check)*.

**Out of scope by design:** the five systems the goal lane already accounts for — **Vitality**
@1.0, **Territory** @2.0 (inventoried 2026-08-11, not yet ported), **Lifecycle** @7.0,
**Dispossession** @10.0, **Metabolism** @13.0. The four ported systems' packs are landed at
`rust/crates/babylon-tick/content/rules/{vitality,lifecycle,dispossession,metabolism}.bsl`
*(own check; the same directory also carries the `fundamental-theorem` and `organization` probe
packs, which are not system ports)*.

---

## 1. Executive summary

1. **Estate size: 34 systems; 5 out of scope; 29 adjudicated here.** Emission load for WS1:
   **67 distinct `EventType` names across 20 of the 29** *(own check, `rg -o --no-filename
   'EventType\.[A-Z_]+'` over each system module)*; 9 systems emit nothing.
2. **PORTABLE NOW: essentially zero.** Not one adjudicated verdict grades a whole system
   PORTABLE NOW. Every candidate row so graded by a reader was downgraded on adjudication
   (Production's extraction broadcast → D102; Decomposition's LA deactivation → 0/1 encoding;
   CollapseTransition's CLAIMS strip → refused at content **load**; Struggle's peripheral revolt
   → same load gate).
3. **PORTABLE WITH D-RECORDS (whole pack): 2** — Production @3.0, Decomposition @11.0.
   **Split/partial-portable: 5** — Substrate @2.5, Policy @17.47, MarketScissors @17.8,
   FieldDerivative @20.0, ContradictionField @19.0. **NOT-A-PACK: 2** — Transport @9.5,
   EpistemicHorizon @22.0. **BLOCKED: 20.**
4. **The single biggest sequencing fact: the dominant blocker is not a query slice.** It is the
   **`GraphSubstrate` edge-attribute STORAGE gap (D35/D65)** — a Constitution III.7,
   hash-widening decision **unscheduled on all four named slices**. Eleven systems key off it, and
   the adjudicators repeatedly correct their readers on this: *"Slice 2 mints edge references; it
   does not mint edge attributes"* (sovereignty adjudication, correction 1). Landing Slice 2
   clears **Solidarity alone**.
5. Second-largest re-verdict: **§3.6's carrier-node ruling WITHDRAWS the "graph-scope state has
   no BSL home" blocker across ~8 systems** — it is servable on **landed Slice 1**, with no `the`
   and no Slice 2, via a `:ceiling 1` carrier `NodeType` anchored through `subject_type_of`.
6. Third: **D102** (`field-of` refused at LOAD on `:enum-type` fields) bites seven inventories and
   forces an **int-ordinal `SocialRole`** encoding **mutually exclusive** with the ADR195/196
   `defenum` recommendation. One estate-wide ruling is owed before any pack declares
   `social-class/role`.
7. **Checkpoint A (WS3 at MATERIAL BASE COMPLETE) is not reachable on today's surface.** It is
   gated on three lanes that do not exist (Slice 2, the D35/D65 substrate widening, the ADR188
   Row-7 sigmoid-as-measure design) plus TickDynamics @4.0, which needs its own charter.
8. **20 of 29 inventories carry a formal INADEQUATE-COVERAGE label**; TickDynamics @4.0's is the
   only one that says a re-read is *required*. Three carry a non-blocking COVERAGE NOTE; six need
   no re-read.
9. **Director-gate register: 24 rows** — 8 ADR172/173 sigmoid/curve reformulations, 12
   reserved-line surfaces (three of them **unflagged by their own inventory**), the Slice-4
   first-consumer escalation, and three shadow-system dispositions.
10. **Recommended next three trains: Territory → Production @3.0 → Decomposition @11.0 +
    ControlRatio @12.0 (joint).** These are the only three that require no lane that does not
    already exist.

---

## 2. Consolidated verdict table

Verdicts are the adjudicated ones. "Lane" names the blocker by the vocabulary in §3.
Positions are `position: ClassVar[float]` *(own check, `rg -n 'position: ClassVar\[float\]'`)*.
Event counts are distinct `EventType` names per module *(own check)*.

### 2a. Material Base (positions 1–13, + Substrate @2.5)

| Pos | System | Adjudicated verdict | Exact blockers (named lane) | D-record candidates | libm hazards | Events (WS1) | Reserved-line |
|---|---|---|---|---|---|---|---|
| 1.0 | Vitality | **PORTED** — out of scope | — | — | — | 2 | — |
| 2.0 | Territory | **DEFER → UNBLOCKED** (ADR197) | none remaining; D116/Q14 constrains rule layout | enum→int-ordinal (`profile`, `territory_type`); two-clamp inconsistency (`territory.py:137` vs `:315`); `rent_spike_multiplier` scaled-Int (D-1); `displacement_mode` `:const` | none (`floor` landed) | 0 | none filed |
| 2.5 | Substrate | **SPLIT** — depletion PORTABLE W/ D-RECORD; lattice NOT-A-PACK; aggregate publish BLOCKED | *graph-scope multi-key* for the aggregate; `the` slice-2 for C3's own accessor | int 0/1 eligibility discriminator (**not** `bool` — `scenario.rs:919-931` admits no bool); null case dropped-because-unconstructible | none | 0 | **NONE** — but the inventory files no check at all (adjudicator rules genuinely none) |
| 3.0 | Production | **PORTABLE WITH D-RECORDS** — narrowed | none blocking the pack; `la_production` channel BLOCKED (graph-scope + `the`) and omitted | **int-ordinal `role`** (D102 — mutually exclusive with §3/§6's own `defenum` recommendation); `fips_code`/`county_fips` defect transcribed verbatim; D116 multi-rule pack analysis | **none** | 0 | Amin/Wallerstein producer-role routing — described, not touched (the model discipline for the batch) |
| 4.0 | TickDynamics | **BLOCKED** — portable island smaller, canonical coverage larger than reader found | ~28-field ServicesProtocol boundary; graph-scope **live Pydantic objects** (no lane); **no string type** (`county_fips`); **D102**; `round()` half-even absent | int-ordinal `role`; county-identity encoding (undesigned) | `round()` (`accumulation.py:115,121`), the wage-pressure sigmoid (ADR188 Row 7) | 4 | **MISSING from the inventory** — bifurcation directional score, five-share `ClassDistribution`, `crisis.dispossession_cascade_milestones` |
| 5.0 | ReserveArmy | **BLOCKED** — both counts confirmed; dormancy premise **withdrawn** | ADR188 Row 7 (ruled prohibition, design owed); CLAIMS `control_level` = Slice 2 **+ D35/D65**; `policy_overlays` graph-scope | absent-`median_wage` row; dodges D102 (favourable) | wage-pressure sigmoid | 1 | **MISSING from the inventory** — the border-valve throttle is the imperial-bribe / National Question surface (ADR171) |
| 6.0 | Community | **BLOCKED** — harder than filed; **#536 own charter** | `community_memberships` = node-local list-of-structs, served by **NO numbered slice**; dyadic-edge read (Slice 2) + `update-edge` one-`f64` refusal; slice-4 attributed membership | **int-ordinal `role`** (D102 kills the `defenum` route); 5×5 solidarity matrix + 14-row floor table as `defconst`s | `math.log` ×2 (Shannon entropy — a **measure**, not an ADR188 Row-7 stipulated form) | 0 | `SUBSTRATE_FLOOR_DEFAULTS` (`consciousness.py:356-455`) — National Question consciousness floors by oppressed nationality |
| 7.0 | Lifecycle | **PORTED** — out of scope | — | — | — | 4 | — |
| 8.0 | Solidarity | **BLOCKED — Slice 2 ALONE** | Slice 2 (dyadic edge lane) — and **no D35/D65 needed**, the datum fits the implicit `<edge-type>/strength` (D32) | boolean 0/1 for `active`; **collect-then-apply divergence** for multi-inbound-edge targets (ruled, not open); `ideology` flattening | **none** | 2 | **NONE** — independently confirmed |
| 9.0 | ImperialRent | **BLOCKED** — blocker set **widened** | D35/D65 storage on every `value_flow` write **and** Phase-4's `subsidy_cap` read; graph-scope `economy`/`la_production`; Task-12 two-pass rules out the sequential pool accumulator | **int-ordinal `role`** (correction to a "PORTABLE" row); `active` 0/1 on all five phases | acquiescence sigmoid (`survival_calculus.py:43`) | 4 | ADR172/173 acquiescence sigmoid — with `PROHIBITED_INTRINSIC_NAMES=["sigmoid"]` already mechanizing half the gate |
| 9.5 | Transport | **NOT-A-PACK** / blocked on a prerequisite **architecture** ruling | spec-108 D2 puts the corridor mesh outside `BabylonGraph`; default-OFF; `corridor_mesh` seeded by nothing | master gate portable as a **bool `defconst`** (not a bool field); per-link reads need **D35/D65**, not Slice 2 | none | 0 | **NONE** — confirmed |
| 10.0 | Dispossession | **PORTED** — out of scope | — | — | — | 2 | — |
| 11.0 | Decomposition | **PORTABLE WITH D-RECORDS** — upheld, larger D-record set | none blocking; **joint** with ControlRatio (Class-D, `_class_decomposition_tick`) | **int-ordinal `role`** (the best-argued row in the batch; corrects two siblings); boolean 0/1 (read **and** write); node-identity D-record (`add-node` names are rule-local; no literal-id lookup); `persistent_data` → `:field`-anchored carrier | **none** | 2 | **NONE** in this half — inherits ControlRatio's ADR070 flag via the joint train |
| 12.0 | ControlRatio | **BLOCKED on Q6** — singleton-carrier escape route CONFIRMED as the only one on dev | `context.persistent_data` has no `<bind-src>`/write verb; escape = `:field`-anchored `:ceiling 1` carrier (no `the`) | **int-ordinal `role`**; `active` 0/1; the census is portable under Slice 1 + those two | **none** | 2 | **ADR070 / Program 19** — the revolution-vs-genocide branch, explicitly-LAST in the emergent-class-partition cutover |
| 13.0 | Metabolism | **PORTED** — out of scope | — | — | — | 1 | — |

### 2b. Action (@14) and Consequences (14.5–22)

| Pos | System | Adjudicated verdict | Exact blockers (named lane) | D-record candidates | libm hazards | Events (WS1) | Reserved-line |
|---|---|---|---|---|---|---|---|
| 14.0 | OODA | **BLOCKED as one pack — DEFER to a re-cut train split**; dormancy premise **withdrawn** (`org_probe` is canonical + CI-gated) | **Task-12 deferred-shape-verb LOAD gate** (Trains D/E); `update-edge` storage; Slice 2; **RNG intrinsic binding** (kernel landed, binding missing) | RNG conformance = **ensemble envelope, never byte replay** (R8, `rng.rs:30-33`) | none in the live path | 6 | `CLASS_ANALYSIS` theory bonus (`action_effects.py:99-111`); `StateFaction`→`PolicyAxis` LEGISLATE proxy (`npc_stub.py:476-480`) |
| 14.5 | FactionInfluence | **BLOCKED** — language gaps stand; "no oracle exists" leg **WITHDRAWN** (5 goldens seed FACTION/SOVEREIGN today) | **multi-hop BFS has no analog at ANY slice** (`materialize` serves 2 heads; no re-query mid-iteration); no `deffield` **reference type**; Task-12 load gate on `add-edge`; `update-edge`; `persistent_data` handoff | RED_SETTLER_TRAP diagnostic is the one PORTABLE row — and now has a live byte-gated oracle | none | 4 | `colonial_stance` as Constitution I.1's principal-contradiction axis; the four seed factions' stance/`class_reduction` values |
| 14.7 | Doctrine | **BLOCKED** — taxonomy widened, verdict line amended | edge-attribute read/write (Slice 2 **+** storage); acquired-set/study-target storage (no string/list/optional-ref in `deffield`'s 7 names); **no construct iterates a content-derived set**; **RNG binding** | 14-bool-field workaround does **not** recover greedy/trap selection | none | 4 | **THE reserved surface** — 14-node tree, three trunks, two traps, five-stance reformist fork, liquidationism absorbing state. **Inventory files NO reserved-line section** |
| 15.0 | Survival | **BLOCKED** — sustained and hardened; the `:const 1.0` escape hatch **WITHDRAWN** | Slice 2 (solidarity multiplier — **live** on `debs`/`bernie_valve` at 0.4); **`exp` declared but not dispatchable** (`intrinsic_host.rs:59-69` serves `floor` alone) | `EPSILON` import-time freeze as `:const`; the bare `"territory"` compare; the over-broad write surface transcribed verbatim | P(S\|A) logistic | 0 | ADR172/173 — whether P(S\|A) may be transcribed as a stipulated logistic at all |
| 16.0 | Struggle | **BLOCKED** — both blockers restated | **RNG intrinsic binding** (NOT a categorical prohibition — §2.8 sanctions it); the edge lane's **two** gates: `update-edge` storage **+** Task-12 load gate (kills `remove-edge`) | discarded `_update_agitation` return transcribed verbatim (backfire is dead on the agitation channel); `role` enum D-record | spark-roll RNG | 8 | George Jackson bifurcation — comprador insolvency → LA/periphery fork, the `national_identity` axis, the `SocialRole` taxonomy |
| 17.0 | Consciousness | **BLOCKED** — Slice 2 **plus** the `GraphSubstrate` edge-attribute reader it presupposes | Slice 2 + substrate edge reader for `core_wages` (C6) and `solidarity_pressure` (C8) — the two reads that decide bifurcation direction | `opposition_states` provably `0.0` on all 12 scenarios → `:const` (Metabolism-D-2 class); `material_conditions` discard defect transcribed | **chauvinist-pressure Gaussian** (`sustained_exploitation.py:198`) | 0 | the Gaussian's peak/falloff coefficients; **plus, unflagged by the reader:** `"national_identity": 0.5` absent-data default, three copies (`ideology.py:74,82,89`) |
| 17.4 | FascistFaction | **BLOCKED** — narrower, better-named set | Slice 2 + substrate reader (`_incident_solidarity`); Slice 2 **+ storage widening** (`_is_superwaged`/`_accrue_chauvinism`: two named fields on one edge); **event read-back — no lane at all** | `opposition_known` provably `False` on all 12 → `:const false`, so the graph-composite blocker **drops out of the critical path** | imposed defection sigmoid (`formulas/reactionary.py:89-91`) | 7 | (i) ADR172/173 sigmoid; (ii) **`FAC_DECOLONIAL` mis-selection is LIVE on 5 byte-gated goldens** — the anti-colonial front is written as "the fascist faction". Constitution I.1/I.4. **Director escalation, NOT port-as-is** |
| 17.42 | Allegiance | **BLOCKED** — on three separable lanes the report fuses into one | (i) **Slice 4** — `allegiance` open-cardinality map, `membership-field-of`, Director-escalation-gated; (ii) **`sqrt` absent** from `DECLARABLE_INTRINSICS` (a one-line ask); (iii) the reused acquiescence sigmoid | keyed registers decompose onto ordinary nodes (landed Slice 1); only `popular_front`'s singleton half needs `the`; `policy_delivery`/`electoral_disillusion` provably absent on 9/12 | `sqrt` ×2 sites; `exp` acquiescence sigmoid ×2 per (class,party) | 1 | `_FASCIST_IDEOLOGY_TOKENS` — the identical 4-token tuple hardcoded at `allegiance.py:77` **and** `reactionary.py:71`, no `defines.yaml` entry |
| 17.45 | Electoral | **BLOCKED** — no portable subset exists | **Slice 4** (`allegiance`); graph-scope singletons via `the` (Slice 2); Slice 2 **+ substrate reader + storage widening** for C2's two-field TRANSACTIONAL write | `renormalize_faction_balance` is **convergence-guarded, not a fixed 5-loop** — any unroll must carry the guard; keyed registers → ordinary nodes | `round(x,6)` half-even (two sites); **no `sqrt`/`exp` on this path** (verified relief) | 10 | `ColonialStance`/`ExtractionPolicy` (ADR171); `acquired_doctrine_ids`; the two-file ideology-token surface |
| 17.47 | Policy | **PORTABLE WITH D-RECORDS** for the resolver math + 4 per-entity registers; **BLOCKED on Slice 2 for THREE** computations | `_org_bridges` edge read (Slice 2 **+ substrate reader**); `national_financial` singleton via `the`; computation 12's `incumbent_id` read-back | the singleton/per-entity split (this inventory is the batch's reference for it); empty-register guard provably the whole behavior on **9 of 12** | none | 7 | `"entryism"` membership test over open, content-authored doctrine ids — deferred to the doctrine tree's own port |
| 17.5 | Sovereignty | **BLOCKED on a SUBSTRATE storage decision, NOT Slice 2** | D35/D65 — every computation keys off CLAIMS-edge `control_level`; the substrate exposes no edge-attribute reader at all | `persistent_data` → §3.6 carrier (portable **today** on Slice 1, no `the`); `nodes` iteration is ascending **NodeId**, not string id — sorted emission is a content-modeling obligation | none | 1 | **MISSING from the inventory** — `_STANCE_TO_POLICY` (`formulas/balkanization.py:24-28`) is the mechanical output of the National Question line; the three signed `metabolic_impact_*` coefficients encode a theoretical claim. The int-ordinal workaround fixes a **hash-bearing declaration order on a reserved axis** |
| 17.8 | MarketScissors | **BLOCKED** — but the national oscillator core is **PORTABLE WITH D-RECORDS today** | (i) per-county axis — `county_fips` is an **open-domain string**, no `deffield` string type; (ii) `price_divergence`'s **tri-state** has no representation; (iii) nested `national_financial`/`tick_dynamics` reads whose producer @4.0 is unported; (iv) `MARKET_CORRECTION` on the WS1 ledger | graph-scope state is storable **today** on a `:ceiling 1` carrier (both named blockers withdrawn); `:optional`/`:default` replaces `bound?`, with a **seeding obligation** for folded elements; five clamp shapes; `c`-suffix sweep | `log` (declarable, needs a tolerance derivation); `tanh` present but **never called by this system** | 1 | **NONE** — confirmed. (The `tanh` doctrine question belongs to its actual caller, Contradiction @18.0) |
| 18.0 | Contradiction | **BLOCKED** — re-based onto **three** blockers, not four families | (i) **substrate edge-attribute gap on READ and WRITE** across computations 1, 2.8, 9 — the deepest gap in the system; (ii) the **architecture question** — a 19-binding ranked registry, a typed morphism graph, a Lawverian level-lattice/Aufhebung classifier; (iii) `tanh` as an ADR172-ruling-5 escalation | the graph-attribute family (rows 2/3/5/10) is **WITHDRAWN as a blocker** — §3.6 rules it and Slice 1 builds it; survivors are `tick_dynamics`'s live Pydantic objects and row 7's nested-record write | `tanh` = `2·sigmoid(2x)−1` on the **CANONICAL `price_value` opposition**; `exp` shaping `financialization_index` | 2 | the `national` opposition (`_STANCE_CHAUVINISM_SCORE`, `contradiction.py:155-163`; `catalog.py:963-984`) — described, not proposed on. The correct level of restraint |
| 19.0 | ContradictionField | **BLOCKED on ONE lane — the SUBSTRATE, not a slice** | `exploitation`'s per-edge `tension` read needs edge-attribute **storage** | `atomization`, the node-iteration shell, the clamp and the 3-slot history unroll are **PORTABLE WITH D-RECORDS on landed Slice 1** via a `:ceiling 1` carrier; the shift chain must live in **one** rule (D116) | none | 0 | **inventory performs NO check** — adjudicator notes an absent check is not a negative result |
| 20.0 | FieldDerivative | **BLOCKED on Phase 1 only**; Phases 3/4 move to **PORTABLE WITH D-RECORDS** | Phase 1 `field_gradients` — edge-attribute storage on read **and** write; "no verb writes a graph-level attribute" is **WITHDRAWN** (§3.6 + `update-node`) | edge-type enumeration + the **witnessed double-counting divergence** on the default scenario (TRIBUTE and CLIENT_STATE on the same node pair); the Option/`float\|None` representation, a third time | none | 1 | **NONE** — upheld (`1.0 if f == "exploitation"` is a mechanical tiebreak) |
| 20.5 | CollapseTransition | **BLOCKED — SIX gaps, ZERO portable** | **five of seven verbs refused before/at execution** — four shape verbs at content **LOAD** (`check_no_deferred_shape_verbs`), `update-edge` at every path. The reader's one PORTABLE-NOW row does not exist | the two Phase-2 defects (wholesale payload carry; comment says "disputed", code writes `de_facto`); the `emit` payload excludes **references**, not just strings | **none** | 3 | `ColonialStance`→`ExtractionPolicy` — correctly raised, correctly not acted on. **New: `bulk_partition_claims` iterates a `set[str]`** (`topology/graph.py:992`) — a `PYTHONHASHSEED` determinism hazard, dormant on canonical |
| 21.0 | EdgeTransition | **BLOCKED twice over** | (i) Slice 2 + the `GraphSubstrate` storage gap; (ii) `persistent_data` `latent_contradictions` **promoted** from D-record to a second independent BLOCKED gap | `_build_transitions`' import-time `GameDefines()` freeze → `defconst` (favourable); 16 pairs + 1 self-loop = 17 | none | 4 | (via the channel correction) the OODA `resolve_negotiate` CO_OPTIVE stamp feeds `doctrine._practice_env`'s `CO_OPTIVE_SHARE` **same-tick** — a live channel into reserved doctrine content |
| 21.5 | WealthDistribution | **BLOCKED** — narrower, better-attributed pair | (i) **Q6** — whole system is graph-scope; `the` is Slice-2 **and** `(domain :graph)` does not execute at all; (ii) **D102** for `_bracket_resistances`' element-wise role read | Step 5's per-subject role read and `_BRACKET_BY_ROLE` are served **today** by `:field` + `=` (`organization.bsl:26-29` ships it); `**2` → `(mul f f)`; the flat-`class_consciousness` bug transcribed | `**2` (no `pow` intrinsic needed) | 0 | none filed; none found |
| 22.0 | EpistemicHorizon | **NOT-A-PACK — on S-23 alone** | the round-trip-exclusion argument does **not** distinguish fog from transient material state and must not be re-used as a projection-lane test | (i) M_r is **DUAL-HOMED** and hash-bearing via `resolve_investigate`→`investigation_intel` — **the verb lane owes this formula a port**; (ii) the per-tenant role read is D102-refused | none | 0 | the four `class_factor_*` values — correctly raised, correctly not acted on |

---

## 3. Blocker lanes, and what each one gates

Named by the vocabulary the adjudications settled on. Counts are systems whose adjudicated verdict
names the lane as binding.

| Lane | Gates | Status | Note |
|---|---|---|---|
| **D35/D65 — `GraphSubstrate` edge-attribute STORAGE** | **11**: ImperialRent, Sovereignty, Contradiction, ContradictionField, FieldDerivative, FascistFaction, EdgeTransition, Electoral (C2), Doctrine, ReserveArmy, Struggle | **Unscheduled on all four slices.** A Constitution III.7 hash-widening decision needing its own ADR | `substrate.rs:80-248` has *no* edge-attribute accessor; `add_edge`'s own `:strength` has no reader. Refusal text at `structural_verbs.rs:387-398` |
| **ADR188 Row 7 — sigmoid-as-measure design** | **8**: ReserveArmy, TickDynamics, Survival, FascistFaction, ImperialRent, Allegiance, Consciousness, Contradiction | **CLOSED RULING with undone design work** — not an open escalation | `PROHIBITED_INTRINSIC_NAMES=["sigmoid"]` already mechanizes half the gate at load |
| **§3.6 carrier ruling** | ~8 systems' graph-scope blockers | **DISCHARGED — servable on landed Slice 1** | `subject_type_of` (`tick.rs:159-181`) + `graph.nodes(&subject_type)` (`:536-538`); `defvocabulary` is a landed `.bscn` form. **No `the` needed** |
| **D102 — `field-of` on enum fields, refused at LOAD** | **7**: Production, Decomposition, ControlRatio, ImperialRent, Community, TickDynamics, EpistemicHorizon (+ WealthDistribution's fold half) | Workaround exists (int-ordinal), **mutually exclusive with `defenum`** | `typecheck.rs:246-289`, wired at `rule_pipeline.rs:293-301`. Substrate @2.5 and ReserveArmy @5.0 are the two adjudicated as **dodging** it (zero enum discriminants) |
| **`exp`/`log` intrinsic DISPATCH** | **5**: Survival, Consciousness, Community, MarketScissors, ImperialRent | Declarable, **not dispatchable** — `KernelIntrinsicHost` serves `floor` alone | Mechanical: one arm. Short runway |
| **Task-12 deferred-shape-verb LOAD gate** | **4**: CollapseTransition (fully), Struggle, FactionInfluence, OODA | Needs the **placeholder-id design** the refusal message names | `check_no_deferred_shape_verbs`, `rule_pipeline.rs:268`. Refuses all six shape verbs anywhere in a rule form |
| **Slice 2 — dyadic edge lane** | **4 named**, but clears **1 alone** (Solidarity) | SCOPED, NOT BUILT | **Scope correction:** four adjudications independently find Slice 2 must ALSO mint the substrate edge-attribute *read* method. ADR197's cut understates it |
| **RNG intrinsic binding** | **3**: Doctrine, Struggle, OODA | Kernel **landed** (`babylon-kernel/src/rng.rs`); missing = a `DECLARABLE_INTRINSICS` name + a `KernelIntrinsicHost` arm | Conformance = 32-seed ensemble envelope, never byte replay (R8) |
| **Slice 4 — attributed-membership storage** | **2**: Allegiance, Electoral | Director-escalation-gated, "**deferred to its first consumer**" | **The first consumer has arrived.** Morning-report question |
| **No string type in `deffield`** | **3**: TickDynamics, MarketScissors, CollapseTransition | Undesigned. `Str` is `:material-basis`/vector-ids only | `county_fips` grouping needs county carrier nodes + incidence edges — a content redesign |
| **No lane at all** | Community (node-local list-of-structs); FactionInfluence (multi-hop BFS); FascistFaction (event read-back); Doctrine (content-set iteration); TickDynamics (live Pydantic objects); MarketScissors/FieldDerivative (Option tri-state) | Unnamed on any roadmap | Community's is **mislabelled Slice 3 by its own reader** — correct the headline up |
| **Slice 3 — hyperedge + metric lane** | **0** | SCOPED, NOT BUILT | **No adjudicated verdict names it as binding.** Do not pull it forward |
| **D116/Q14 — two rules at one anchor** | Territory, ContradictionField, FieldDerivative, Production, Survival | RECORDED, deferred to its own train | Constrains rule layout, not portability |
| **`round()` half-even** | 2: Electoral, TickDynamics | No `round` intrinsic; `floor(x+0.5)` diverges at ties | |
| **`sqrt`** | 1: Allegiance | A one-line `DECLARABLE_INTRINSICS` amendment | Mechanically separable from the sigmoid question beside it |

---

## 4. Recommended train grouping and order

Grouping rule applied: **fuse only where systems share a fixture *and* a verification lens.**
Where the lens differs — byte-gated `graph_content_hash` vs. behavioral `test_electoral_goldens.py`
vs. hand-built `.bscn` — the trains stay apart, because a fused train has no single acceptance test.

### 4.1 Material Base remainder — schedulable today

**MB-1 · Territory @2.0 — the precedent train. SCHEDULE FIRST.**
Gated on nothing. ADR197 closed the blocking dependency and `query_lane_e2e.rs` ships four working
templates matching Territory's four blocked shapes. Three inputs the train must carry, all from
the scout dossier:
- `bool` is **not** available on the live `.bscn` pipeline (`scenario.rs::load_deffield`) — the
  eviction latch is an `int` 0/1, not a bool.
- D116/Q14 means heat / eviction / spillover / necropolitics cannot be four rules at one anchor.
  Either merge (vitality's precedent) or use distinct anchor positions.
- `territory` is **already a registered system name** in `babylon-tick/src/lib.rs:190-197`, added
  as a placeholder by the query-eval train and explicitly marked "NOT a Territory-port system".
  Whether the real registration reuses it is a Director call, not a port-time one.

**MB-1a · The estate-wide role-encoding ruling — a DECISION, not a train, and it belongs in
Territory's dossier.** D102 forces int-ordinal; ADR195/196 recommends `defenum`. Seven systems
read `role` off a non-subject node. Territory already owes an enum D-record for
`profile`/`territory_type`, so it is the natural and cheapest place to settle the question once.
**Nothing in MB-2 or MB-3 should be scheduled before this lands.**

**MB-2 · Production @3.0 — standalone. SCHEDULE SECOND.**
PORTABLE WITH D-RECORDS. Rides MB-1a's ruling. Ships the `fips_code`/`county_fips` defect verbatim
(provably dead on every scenario including `single_county`) and omits the `la_production`
broadcast with a D-record (graph-scope, and C3's own accessor is slice-2 unserved). Zero libm
hazards, zero events, no edge lane. Verification lens: `graph_content_hash` covers
`TERRITORY.extraction_intensity` and `SOCIAL_CLASS.wealth`, both live. Not fused with anything —
its adjudication owes an explicit D116 multi-rule row that no other train shares.

**MB-3 · Decomposition @11.0 + ControlRatio @12.0 — MANDATORY JOINT. SCHEDULE THIRD.**
The Class-D coupling is exactly one key wide and confirmed from **both** sides
(`decomposition.py:223` writes `_class_decomposition_tick`, `control_ratio.py:128` reads it), and
both halves must co-design the same `:ceiling 1` carrier. They share the fixture (total dormancy on
all 12 scenarios; the conformance fixture must **recalibrate** the trigger, not merely run longer —
`SUPERWAGE_CRISIS` never fires within 150 ticks) and the lens (hand-built `.bscn`). The joint train
inherits **ControlRatio's ADR070 reserved-line flag** even though the Decomposition half raises
none.

**MB-4 · Substrate @2.5 — schedulable any time, INDEPENDENT of MB-1a (dodges D102).**
Honest scope is the per-territory depletion rule plus an int 0/1 eligibility discriminator. But
its Steps 3 and 4 are **unreachable** on the canonical estate — the early return at
`substrate.py:218-219` precedes the lattice build — so the pack ships a rule nothing exercises.
**Whether that is worth a train is a Director scope question**, not an engineering one. Ranked
below MB-3 for that reason.

### 4.2 Material Base remainder — NOT schedulable, with the gate named

| Train | Blocked on | Earliest |
|---|---|---|
| **Solidarity @8.0** | **Slice 2 alone** — the cheapest unblock in the estate (no D35/D65; the datum fits `<edge-type>/strength`) | After Slice 2 |
| **ReserveArmy @5.0** | ADR188 Row-7 design **+** Slice 2 **+** D35/D65 (CLAIMS `control_level`) **+** graph-scope `policy_overlays` | After the sigmoid design and the substrate decision |
| **ImperialRent @9.0** | D35/D65 (write **and** read) **+** graph-scope **+** ADR172/173 **+** Task-12 rules out its pool accumulator | After the substrate decision and a Director ruling |
| **TickDynamics @4.0** | Its own charter — the ServicesProtocol boundary, live Pydantic metadata, string identity, D102, `round()`. The single largest unported Material-Base surface | **Needs a charter, comparable to Community #536.** Its inventory needs a re-read first |
| **Transport @9.5** | **Do not schedule.** NOT-A-PACK; a WS4 adjudication row. It does not get cheaper when slices 2–3 land — which is exactly why the verdict is NOT-A-PACK rather than DEFER | n/a |
| **Community @6.0** | **#536, own charter**, Director-scheduled after the Consequences estate per the goal lane | Charter input: the headline blocker is **not** slice 3 — it is a node-local list-of-structs no numbered slice serves |

### 4.3 Checkpoint A — the honest read

**MATERIAL BASE COMPLETE (the WS3 trigger) is not reachable on today's surface.** Reaching it
requires, at minimum: Slice 2 (Solidarity), the D35/D65 substrate widening (ImperialRent,
ReserveArmy), the ADR188 Row-7 sigmoid-as-measure design (ReserveArmy, TickDynamics), and a
TickDynamics charter. Three of those four do not exist as chartered work today, and one is a
program rather than a train. **A schedule that puts Checkpoint A after MB-4 is not supported by
the evidence.**

### 4.4 The Consequences estate (~20 packs)

**C-1 · The graph-scope carrier pilot — ContradictionField @19.0 (atomization half) +
FieldDerivative @20.0 (Phases 3/4).** Both adjudicated PORTABLE WITH D-RECORDS on **landed
Slice 1** via the `:ceiling 1` carrier. Same lens: their outputs *are* byte-gated, via
`_restamp_field_stack` putting `contradiction_fields`/`field_derivatives` back onto nodes
(`world_state.py:855-867`). Both are downstream of Contradiction @18.0's own port decision
(whether it writes `atomization` to a carrier instead of `set_graph_attr`).
**Honest caveat that must go to the Director:** both are **honest-partial** packs — each leaves its
edge half (per-edge `tension`; `field_gradients`) behind D35/D65. Territory's own precedent
rejected honest-partial as "silent scope shrink". This is a scope ruling, not an engineering call.
If the Director accepts it, C-1 is the **only** Consequences train schedulable today and it proves
the carrier pattern for the ~8 systems that inherit it. **WealthDistribution @21.5** is the natural
follow-on once the pattern is proved (its Q6 is the same carrier question).

**C-2 · Policy @17.47 — standalone, and it must NOT be fused with any byte-gated train.**
The only Consequences system with a PORTABLE WITH D-RECORDS verdict on its resolver math and four
per-entity registers, and the batch's reference implementation of the singleton/per-entity split.
It carries **ZERO `graph_content_hash` coverage** — all five registers are `g.graph` metadata and
`tick_capital_stock` is dropped by the `tick_*` prefix filter (`world_state.py:256`) — so a BSL
transcription **cannot be validated by re-running `qa:regression`**. Its oracle is
`test_electoral_goldens.py`'s per-tick event assertions. Three computations stay blocked on
Slice 2. Gated on: Slice 2 for completeness, but a partial pack is defensible here in a way it is
not for C-1, because the portable half is the resolver *math*, not a sliver.

**C-3 · The electoral cluster — Allegiance @17.42 + Electoral @17.45. GATED ON SLICE 4.**
Fuse: they share `allegiance`/`hope` as a same-tick channel (`electoral.py:484-485, 644-645,
709-711, 716, 732-738, 981`), share the five electoral goldens as fixture, and share a behavioral
(not byte) lens. Neither ports at all until the open-cardinality `allegiance` map has storage —
`membership-field-of`, Slice 4, Amendment AG, **Director-escalation-gated**. Separable side-asks
that should not be bundled into the escalation: Allegiance's `sqrt` (one-line intrinsic) and
Electoral's `round()` half-even.

**C-4 · The edge-storage cluster — BLOCKED pending D35/D65.** Sovereignty @17.5, Contradiction
@18.0, EdgeTransition @21.0, FascistFaction @17.4, Doctrine @14.7, Consciousness @17.0, Survival
@15.0. **None is clearable by any query-evaluation slice.** Sub-split by lens when it becomes
schedulable: Survival and Consciousness have **live byte-gated oracles** waiting (`debs`/
`bernie_valve` seed `solidarity_strength=0.4`); the rest largely do not. Contradiction @18.0
additionally carries an **architecture question** — a 19-binding ranked registry, a typed morphism
graph, a Lawverian level-lattice classifier — that is "the largest and differently-shaped unblock
in the estate" and is not answered by any slice or by §3.6.

**C-5 · The graph-shape-verb cluster — BLOCKED pending the Task-12 placeholder-id design.**
CollapseTransition @20.5 (zero portable), Struggle @16.0, FactionInfluence @14.5, OODA @14.0.
FactionInfluence additionally carries the **multi-hop BFS**, which has **no analog at any slice or
ruling** — that is a control-flow shape the execution model does not admit, and it is a harder
blocker than any storage gap in the estate.

**C-6 · The intrinsic-host train — a Rust train, not a port train. WORTH PULLING FORWARD.**
Two mechanical bindings with fully-specified designs: (a) the **RNG binding** — a
`DECLARABLE_INTRINSICS` name plus a `KernelIntrinsicHost::call` arm, kernel already landed,
conformance already ruled to be an ensemble envelope; (b) **`exp`/`log` dispatch** — declarable but
not dispatchable today. Together these gate **8 systems** (Doctrine, Struggle, OODA, Survival,
Consciousness, Community, MarketScissors, ImperialRent) with the shortest runway of any lane on
this list. **Adding `sqrt` in the same train** picks up Allegiance's third blocker for one more line.

**Not scheduled:** EpistemicHorizon @22.0 (NOT-A-PACK on S-23; its M_r formula is filed to the
**verb lane** instead, where it is hash-bearing via `resolve_investigate`); Transport @9.5
(NOT-A-PACK).

### 4.5 Is any slice worth pulling forward?

Ranked by systems gated, against runway:

1. **D35/D65 edge-attribute storage — 11 systems, and it is not on the roadmap.** This is the
   highest-leverage item in the estate by a wide margin. Recommend it be chartered **ahead of
   Slice 2**, because Slice 2 mints references over storage that does not exist, and Slice 2 alone
   clears only Solidarity. It needs a Constitution III.7 ADR, so it is a Director item.
2. **ADR188 Row 7 sigmoid-as-measure design — 8 systems.** Already ruled; the design is owed. A
   design lane, not a language lane.
3. **The intrinsic-host train (RNG + `exp`/`log` + `sqrt`) — 8 systems, shortest runway.** The
   best effort-to-unblock ratio on the list and buildable today.
4. **Task-12 placeholder-id design — 4 systems**, one of which (CollapseTransition) is 100% blocked
   by it.
5. **Slice 2 — 4 named, 1 cleared alone.** Its scope must first be corrected to include the
   substrate edge-attribute *read* method (four independent adjudications).
6. **Slice 4 — 2 systems**, Director-gated, first consumer now present.
7. **Slice 3 — 0 systems.** **Do not pull forward.** Nothing in the adjudicated estate waits on it.

---

## 5. Director-gate register

Every row below is a **morning-report question**, not a decision made here.

### 5a. ADR172 ruling-5 / ADR173 — sigmoid and stipulated-curve reformulations

| # | System | Site | Question |
|---|---|---|---|
| 1 | Survival @15.0 | `survival_calculus.py:43` | May P(S\|A) be transcribed as a stipulated logistic at all, or must it re-derive as a measure over within-class wealth dispersion? |
| 2 | Allegiance @17.42 | same function, reached twice per (class,party) via `counterfactual_hope_gain` | ADR173 retires this form for its *original* use. Re-instantiating it to synthesize a **new** quantity H(c) is a fresh ruling, not an inherited one |
| 3 | ImperialRent @9.0 | Phase 4, same function | Same, on the imperial-subsidy path |
| 4 | FascistFaction @17.4 | `formulas/reactionary.py:89-91` | An imposed sigmoid used **directly as the defection mechanic** |
| 5 | Consciousness @17.0 | `sustained_exploitation.py:198` | A Gaussian (`exp(−d²/2σ²)`) shaping chauvinist pressure — plus its peak/falloff coefficients |
| 6 | Contradiction @18.0 | `formulas/market.py:97-107` | `tanh` = `2·sigmoid(2x)−1`, imposed on the **CANONICAL `price_value` opposition** (ADR078). The `exp`/`log` rewrite would express the prohibited shape out of two permitted intrinsics — routing around a gate that is deliberately mechanical |
| 7 | Contradiction @18.0 | `contradiction.py:453-455` | `exp` shaping `financialization_index` — declarable, but shaping rather than computing a physical quantity |
| 8 | ReserveArmy @5.0 / TickDynamics @4.0 | `reserve_army/calculator.py:32-65` | **Not an open escalation** — ADR188 Row 7 is a **closed ruling with undone design work**. The question is who does the redesign and when |

### 5b. Reserved-line surfaces

| # | System | Surface | Disposition asked |
|---|---|---|---|
| 9 | FascistFaction @17.4 | **`FAC_DECOLONIAL` mis-selection** — `_find_fascist_faction` token-matches `"settler"` inside `"anti-settler abolitionism"`, and `min(candidates)` returns `FAC_DECOLONIAL` over `FAC_RESTORATIONIST`. **LIVE on all five balkanization-seeded goldens**, and its effect (`aligned_faction_id`) **is** byte-gated the moment capture fires | Constitution I.1/I.4. The adjudicator rules this a **Director escalation, not a port-as-is transcription row** — the port must not silently reproduce it |
| 10 | Doctrine @14.7 | The entire 14-node tree, three trunks, two traps, the five-stance reformist fork with zero-`tag_delta` acquisition, the liquidationism absorbing state and its three thresholds | Is transcribing the tree into `.bscn` `defconst`s a Director-gated act? (The adjudicator says yes; **the inventory files no reserved-line section at all**) |
| 11 | Sovereignty @17.5 | `_STANCE_TO_POLICY` (`formulas/balkanization.py:24-28`) + `metabolic_impact_intensify` −0.02 / `_continue` −0.005 / `_cease` **+0.01** | The int-ordinal `extraction_policy` workaround fixes a **hash-bearing declaration order on a reserved axis** (ADR195/196). Director disposition, not a port-time D-record. **Inventory files no check** |
| 12 | ControlRatio @12.0 | The revolution-vs-genocide branch | ADR070/Program 19 rules it explicitly LAST in the emergent-class-partition cutover. Does the joint Class-D train respect that, or does it split? |
| 13 | Community @6.0 | `SUBSTRATE_FLOOR_DEFAULTS` (`consciousness.py:356-455`) — consciousness floors by oppressed nationality | Transcribe verbatim under ADR171? (#536 charter input) |
| 14 | Allegiance @17.42 / FascistFaction @17.4 / Electoral @17.45 | `("fascist","reaction","revanch","settler")` hardcoded **twice**, `allegiance.py:77` and `reactionary.py:71`, with no `defines.yaml` entry | One ideological classification with two engine homes. **One Director ruling, not two** |
| 15 | Contradiction @18.0 | The `national` opposition + `_STANCE_CHAUVINISM_SCORE` | Described, not proposed on. Confirm the restraint is the right level |
| 16 | FactionInfluence @14.5 | `colonial_stance` as I.1's principal-contradiction axis; the four seed factions' stance/`class_reduction` values | Confirm port-as-is |
| 17 | Struggle @16.0 | George Jackson bifurcation — comprador insolvency → LA/periphery fork, `national_identity`, the `SocialRole` taxonomy | Confirm port-as-is |
| 18 | OODA @14.0 | `CLASS_ANALYSIS` theory bonus; `StateFaction`→`PolicyAxis` LEGISLATE proxy | Confirm port-as-is |
| 19 | Consciousness @17.0 | `"national_identity": 0.5` absent-data default, **three copies** (`ideology.py:74,82,89`) — a class with no ideology record is half-nationalist | **Unflagged by the reader.** Must be re-declared explicitly as a `deffield` default and routed past the Director, not inherited silently |
| 20 | ReserveArmy @5.0 | The border-valve throttle — "the settler-wing wage bargain" (`reserve_army.py:86-88`) | **Unflagged by the reader.** Which wing the bribe protects, and whether the valve exists at all, is not a port-time call (ADR171) |
| 21 | TickDynamics @4.0 | Bifurcation-risk directional score + its three defines; the five-share `ClassDistribution` + Feature-016 transition engine; `crisis.dispossession_cascade_milestones` | **Unflagged by the reader** on the ideologically densest system in the batch |

### 5c. Slice-4 escalation and shadow-system dispositions

| # | Item | Question |
|---|---|---|
| 22 | **Slice 4 — first consumer has arrived** | ADR197 rules slice 4 "DEFERRED TO ITS FIRST CONSUMER" under III.7 + Amendment AG. Allegiance @17.42 and Electoral @17.45 are that consumer, and **neither ports at all without it**. Charter it now, or re-sequence both systems behind everything else? |
| 23 | **Honest-partial packs** | Territory's precedent rejected the sliver-only port as "silent scope shrink". C-1 (ContradictionField + FieldDerivative), Policy @17.47 and MarketScissors @17.8 are all honest-partial by construction — portable core, blocked edge half. Does the precedent bind them, or is the carrier pilot worth a partial? |
| 24 | **Shadow systems** | `field_registry` is never wired in production (ContradictionField Computation 1, FieldDerivative Phase 0) → NOT-A-PACK; `transport_demand_signal`, `effective_controller_by_territory`, `sigma_capital_labor`/`derived_class_cell`, `wealth_share` and MarketScissors' `_swell_reserve_army` are all **verified dead outputs with zero readers**. Port them verbatim under port-as-is law, or retire them on the WS4 ledger? |

---

## 6. Coverage honesty

**Out of scope by design.** The five systems the goal lane already accounts for — Vitality @1.0,
**Territory @2.0 (inventoried 2026-08-11, port not started)**, Lifecycle @7.0, Dispossession @10.0,
Metabolism @13.0 — are excluded from the 29-inventory adjudication. Territory appears in this
survey only as the sequencing precedent and as train MB-1.

**INADEQUATE-COVERAGE — a re-read is owed before the system is scheduled (19, plus TickDynamics
below = 20):** Allegiance @17.42, CollapseTransition @20.5, ContradictionField @19.0,
Contradiction @18.0, ControlRatio @12.0 *(narrow)*, Decomposition @11.0 *(narrow but
load-bearing)*, Doctrine @14.7, EdgeTransition @21.0 *(scoped)*, FactionInfluence @14.5,
FascistFaction @17.4, FieldDerivative @20.0, ImperialRent @9.0, MarketScissors @17.8, OODA @14.0,
Solidarity @8.0 *(narrow)*, Sovereignty @17.5, Struggle @16.0, Survival @15.0 *(narrow)*,
Transport @9.5.

Two of these are **already-recommended trains** and their re-reads are cheap and narrow —
Decomposition @11.0 (add `babylon-bsl` source: `structural_verbs.rs`, `evaluator.rs`,
`substrate.rs`) and ControlRatio @12.0 (add `typecheck.rs:246-280` + `rule_pipeline.rs:297` and
`evaluator.rs:1274-1292,1315-1320,1594-1632`). Both must land before MB-3 opens. Production @3.0,
the other recommended train, needs none.

**TickDynamics @4.0 — the one flagged "a re-read is REQUIRED".** Its dormancy analysis, its
scenario count, its D102/string-identity sweep and its reserved-line section are all owed. It is
also the largest unported Material-Base surface. **It must not be scheduled off the current
inventory.**

**COVERAGE NOTE, non-blocking (3):** Community @6.0, ReserveArmy @5.0, Substrate @2.5 — each owes
a small, named set of additions (Community: two row relabels + a `log`-is-a-measure note;
ReserveArmy: a spot-run dormancy re-derivation, a reserved-line section, one D-record row;
Substrate: a reserved-line section recording NONE and the reasoning).

**No coverage label — no re-read owed (6):** Consciousness @17.0 (four corrections + a
reserved-line addendum, no note), Electoral @17.45 (explicitly "needs no re-read"),
EpistemicHorizon @22.0, Policy @17.47, Production @3.0, WealthDistribution @21.5.

20 + 3 + 6 = 29 *(own check, `rg -o 'INADEQUATE[- ]COVERAGE…'` over each file)*.

**Two cross-inventory inconsistencies this survey flags rather than resolves.**
(i) **ControlRatio @12.0** is graded BLOCKED-on-Q6 with the `:ceiling 1` carrier escape "CONFIRMED
as the only one available", while ContradictionField, FieldDerivative and MarketScissors are graded
**PORTABLE WITH D-RECORDS** on that identical route. The four adjudications cannot all be right
about the same mechanism; the joint Class-D train owes a reconciliation.
(ii) **Slice 2's scope.** Four adjudications independently find Slice 2 must mint the
`GraphSubstrate` edge-attribute *read* method, which ADR197's slice definition does not name. The
next slice-2 design gate owes that correction.

**Standing:** every claim above traces to a named inventory's adjudication section, to the Territory
precedent, to ADR197, to the scout dossier, or to an own-check marked as such. Where the evidence
did not support a train, none is scheduled.

---

## 7. Errata (post-publication corrections)

Per this repo's immutable-history documentation philosophy, the sections above are left as
written — they capture the reasoning that made sense at survey time. This section records where
later work found that reasoning wrong or stale, with a citation to the correction. Entries are
appended in the order they were found, not renumbered.

**2026-08-17 (#576 intrinsic-host train, Tasks 0 and 6, ADR213).**

1. **The `sqrt` row (row 137, and its restatements at lines 241, 265, 281) is stale as to
   authority.** This survey (published 2026-08-12) characterizes `sqrt` as "a one-line
   `DECLARABLE_INTRINSICS` amendment" and groups it with the intrinsic-host train's other two
   items as if all three were equally open. That was already wrong at publication: ADR188 Row 6
   (`ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml`, ratified 2026-08-10 — two days
   *before* this survey) had already ELIMINATED `sqrt` outright, by re-derivation as a measure
   ("platform fit ... the share of a class's interest dimensions a platform satisfies"), not
   merely deferred it. The elimination is mechanically pinned in-tree:
   `rust/crates/babylon-bsl/tests/r9_chapters.rs:2604`, inside
   `exp_log_floor_and_rng_draw_are_declarable` (`:2597-2607`), asserts
   `check_intrinsic_cap("sqrt")` fails, alongside `tanh`/`entropy`/`renormalize`/`abs`/`trunc`.
   The row's claimed consumer
   (Allegiance @17.42) is not where the elimination bites — the actual call sites are
   `src/babylon/formulas/politics.py:145` (`platform_vector`'s L2 norm) and `:227-228`
   (`interest_fit`'s two cosine-similarity norms), reached from `allegiance.py:399,440,504` via
   `interest_fit` — exactly the "platform fit" measure ADR188 Row 6 names. The blocker is
   doctrinal, not numerical: IEEE-754 specifies `sqrt` exactly, so it needed no libm-crossing
   decision the way `exp`/`log` did. `sqrt` was removed from the intrinsic-host train's scope
   entirely (not scheduled, not deferred) — see the plan's own §0.1
   (`docs/superpowers/plans/2026-08-17-576-intrinsic-host.md`) and ADR213.

2. **The `exp`/`log` intrinsic DISPATCH row (row 127) overstates the ready-consumer count.**
   This survey lists "5: Survival, Consciousness, Community, MarketScissors, ImperialRent" as
   gated on dispatch landing. Re-verified against ADR202 (ratified after this survey): Survival's
   `P(S|A)` sigmoid RETIRES under ADR188 Row 7 / ADR173 (re-derived as a measure, never
   transcribed as an `exp` call); Consciousness's Gaussian RETIRES separately, under **ADR202 R7
   alone**, which explicitly REVERSES ADR188 Row 7 for that one site — Row 7's own text says the
   Gaussian "port[s] as-is under exp" (`ADR188_intrinsic_rider_slate_dispositions.yaml:59-60`),
   while ADR202 R7's own text says "ADR188 Row 7 is REVERSED FOR THIS SITE ONLY: the frozen
   Gaussian is NOT transcribed" (`ADR202_t4_curves_ruling_session.yaml:137-138`); citing Row 7 as
   joint authority for both retirements (as an earlier draft of this errata did) has the authority
   chain backwards for the Gaussian's half, even though the conclusion — no live `exp` need at
   that site — is correct. MarketScissors's `tanh` site retires under ADR202 R8. The corrected
   count, independently verified at each source line:
   `log` has **two** ready, doctrinally-clean consumers — Community @6.0's Shannon-entropy
   calculation (`src/babylon/formulas/consciousness_routing.py:45,470`, `_LOG3`/`p*log(p)`) and
   MarketScissors @17.8's monetary anchor (`src/babylon/domain/economics/monetary/anchor.py:89`,
   `math.log(ratio)`); `exp` has **one** — Contradiction @18.0's financialization index
   (`src/babylon/engine/systems/contradiction.py:455`, `math.exp(clamped)`, ADR202 R9 upheld
   verbatim) — itself still blocked on the D35/D65 edge-attribute-storage decision this survey's
   own §3 names. Neither Survival's nor Consciousness's site has a live `exp`/`log` need any
   longer. See ADR213's decision §3 for the full re-derivation table.

3. **The RNG intrinsic binding row (row 130) undersells the actual gap.** This survey describes
   the missing piece as "a `DECLARABLE_INTRINSICS` name + a `KernelIntrinsicHost` arm" — necessary
   but not sufficient. `bsl-language.rst` §3.10's own D69 record fixes the carrier key's `domain`
   component as a closed-vocabulary **enum operand** and `stable_key` as deriving from "the call's
   reference operands" — neither is declarable as written: `<intrinsic-decl>`'s `:params`
   vocabulary is parsed by `parse_intrinsic_type_name`
   (`rust/crates/babylon-bsl/src/declarations.rs:801-813`), which delegates every non-`real` name
   to `parse_type_name` (`:662-699`); together they admit **seven** of §3.1's eight rows, refuse
   `enum` outright at that grammar position, and have no row at all for
   a node/edge reference. Closing that gap would widen `<intrinsic-decl>`'s grammar and move §5.6
   canonical-AST bytes / `rules_hash` — out of scope for a "declarability" fix. The landed design
   (`#576` Task 5) instead keys `domain` on the firing rule's own id string and derives
   `stable_key` from a new length-prefix `framed()` encoder over content ids, not reference
   operands — see `docs/reference/bsl-language.rst` D176/D177 and ADR213. The three-system unblock
   claim (Doctrine, Struggle, OODA) itself stands verified.

4. **The ImperialRent row's (row 83) `exp`/`log` dependency claim is unverified.** Row 83 lists
   "acquiescence sigmoid (`survival_calculus.py:43`)" in its transcendental-dependency column
   against ImperialRent @9.0. Neither of ImperialRent's own source files contains a transcendental
   call: `rg 'math\.(exp|log|sqrt)'` returns **nothing** against
   `src/babylon/engine/systems/phi_distribution.py` or
   `src/babylon/domain/economics/tick/system/imperial_rent.py` (re-verified 2026-08-17, at
   ADR213's own landing). The cited `survival_calculus.py:43` line is Survival's own sigmoid, not
   ImperialRent's — whatever relationship the row intended between the two systems, it is not a
   direct `exp`/`log` call site inside ImperialRent's own code. Filed as **unverified**, not
   corrected to a specific true value — no replacement claim is made in its place.
