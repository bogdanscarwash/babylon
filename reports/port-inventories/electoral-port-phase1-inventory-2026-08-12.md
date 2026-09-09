# ElectoralSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ElectoralSystem` (1269 lines, `src/babylon/engine/systems/electoral.py`,
@17.45) is the largest, most architecturally entangled system inventoried in this series to
date — it is Territory's opposite in almost every dimension that matters for porting. Where
Territory was self-contained and structurally dormant on the canonical estate, Electoral pulls in
six domain/formula modules, writes and reads **seven distinct graph-level dict/tuple registers**
(no BSL storage class exists for these today), depends pervasively on a `dict[str, float]`
`allegiance` field with open, scenario-defined cardinality (no BSL field type holds a map at
all), needs edge-attribute reads/writes on three edge families (CLAIMS control_level, SOLIDARITY
strength, TRANSACTIONAL co_optive_dependence — all Slice-2, and edge WRITES hit a declared
substrate gap even past Slice 2: `GraphSubstrate` stores one bare `f64` per edge, nothing more),
and calls one genuinely novel numerical pattern — a bounded 5-iteration clamp-renormalize
fixed-point solver — that has no precedent anywhere else in the estate and no direct BSL
expression. It is, however, **thoroughly exercised by the canonical estate**: five of the twelve
`qa:regression` scenarios (mitterrand/syriza/weimar/debs/bernie_valve, #7–11) seed the full party
terrain and drive ElectoralSystem live, both under the byte-identical hash gate and under a
separate five-arc behavioral-contract suite (`test_electoral_goldens.py`) — the opposite of
Territory's "hand-build every fixture" problem.

**Verdict: BLOCKED — no portable subset exists today.** Every one of Electoral's ten
computations reads or writes at least one of: the open-cardinality `allegiance` dict, a
graph-level dict/tuple register, or an edge attribute. There is no sliver (not even the L-SUSPEND
latch alone) that avoids all three. The single highest-leverage next step is not the query lane
(Slice 1, already landed) but a design decision on how `allegiance` and the six electoral
registers get represented in BSL's closed field/storage vocabulary — most plausibly some
combination of Amendment AG's attributed-membership construct (ratified, but its accessor
`membership-field-of` is Slice-4/Director-escalation-gated, unlanded) and the R9 carrier-node-type
ruling (landed) for the genuinely singleton fields only.

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/electoral.py` | 1269 | **The target.** `ElectoralSystem`, all ten computations (popular front, derecognition, governance-endgame consequences, betrayal windows, the per-sovereign election loop: L-SUSPEND, the count, government formation, legitimation refresh, institution balance shift, H-collapse routing). |
| `src/babylon/formulas/politics.py` | 278 | Pure politics kernel. ElectoralSystem imports exactly two of its eleven functions: `competitiveness` (262-278), `turnout_share` (234-259). The other nine (`valve_multiplier`, `hope_field`, `counterfactual_hope_gain`, `sw_deliverable`, `delivery_ratio`, `delivery_gap`, `platform_vector`, `allegiance_drift`, `apply_allegiance_drift`, `interest_fit`) belong to AllegianceSystem/PolicySystem, out of scope — noted because `platform_vector`/`interest_fit` are the ONLY `math.sqrt` call sites in this module and ElectoralSystem never reaches them. |
| `src/babylon/domain/politics/conjuncture.py` | 135 | `consolidation_pressure` (53-109, pure arithmetic, no transcendentals), `resolve_popular_front_arm` (112-128), `PopularFrontArm` enum (46-50). |
| `src/babylon/domain/politics/governance_endgame.py` | 140 | ElectoralSystem imports only `GovernanceArm`, `RuptureGeometry`, `betrayal_crossed` (70-78). The module's other three functions (`resolve_governance_arm`, `rupture_geometry`, `phi_share`, `dual_power_live`) are PolicySystem's — the fork itself is stamped by PolicySystem @17.47; Electoral only consumes its resolved consequence one tick later. |
| `src/babylon/domain/institution/balance.py` | 122 | `update_internal_balance` (19-122) — ElectoralSystem is its **only** production caller repo-wide (grep-confirmed; `models/events/institution_payloads.py:11` still carries a stale "DEAD-UNTIL-WIRED" docstring from before Electoral wired it). |
| `src/babylon/ooda/state_ai/faction_dynamics.py` | 525 | `renormalize_faction_balance` (171-242) — a bounded 5-iteration clamp/normalize fixed-point solver, imported inline inside `_perturb_faction_balance` (electoral.py:1029). |
| `src/babylon/models/entities/state_apparatus_ai.py` | 369 | `FactionBalance` Pydantic model (113-177) — the state-apparatus faction-weight vector Electoral perturbs. |
| `src/babylon/models/entities/institution.py` | 487 | `InternalBalanceOfForces` (46-107), `FactionShiftEvent`/`BonapartistModeEvent` (imported inline, electoral.py:1116). |
| `src/babylon/models/entities/social_class.py` | — | `IdeologicalProfile` (61-99) — the `ideology` dict shape Electoral reads off SOCIAL_CLASS; `allegiance: dict[str, float]` field declaration (321-330, doc comment self-names the port hazard: "A declared field so the distribution survives the extra='forbid' graph round-trip"). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._wrap_graph` (98-117, the only base-class helper ElectoralSystem actually calls — line 188); `resolve_rng` (35-55, used at the recount tie-break, line 841). Electoral does **not** use `_write_clamped`/`_read`/`_publish` — it duplicates `_publish` as its own static `_emit` (1254-1261). |
| `src/babylon/kernel/event_bus.py` | 288 | `Event` frozen dataclass (33-48). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE`. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `query_territory_claims` (415-430, a bespoke sorted-CLAIMS-edge accessor — see §5/§6), `get_graph_attr`/`set_graph_attr` (350-372, the untyped `Any` graph-level-metadata seam all seven of Electoral's registers ride). |
| `src/babylon/config/defines/politics.py` | 575 | `PoliticsDefines` — 40 fields; Electoral reads 12 of them directly (§2(e) enumerates each). |
| `src/babylon/config/defines/state_apparatus.py` | — | `InstitutionDefines` (`bonapartist_threshold`, `bonapartist_exclusion_threshold`, `alpha_smoothing_rate`) and `StateApparatusAIDefines` (`max_faction_shift_per_tick`) — both composed into `GameDefines` as `.institution`/`.state_ai`. |
| `src/babylon/config/defines/endgame.py` | — | `fascist_majority_fraction`, read via `services.defines.endgame` (electoral.py:415). |
| `src/babylon/models/enums/*.py` (balkanization.py, topology.py, social.py, organizations.py) | — | `ColonialStance`/`ExtractionPolicy` (RESERVED-LINE, National Question), `EdgeMode`, `OrgType`, `StateFaction`, `RulingClassFraction` (via `InternalBalanceOfForces.hegemonic_fraction`, computed field). |
| `src/babylon/engine/scenarios/electoral_fixture.py` | 278 | `apply_political_terrain` — the shared party/sovereign/institution seeding layer for all five electoral goldens (§5/§7). |
| `src/babylon/engine/scenarios/electoral_goldens.py` | 543 | The five `create_*_scenario` factories (mitterrand/syriza/weimar/debs/bernie_valve). |

**Not exercised by electoral.py:** no `src/babylon/formulas/survival_calculus.py` call (that's `hope_field`/`counterfactual_hope_gain`'s, not Electoral's); no direct SQL/persistence access.

**Reference BSL/evaluator sources read for the blocker adjudication** (all read in full or by targeted section, dev tree):
- `rust/crates/babylon-bsl/src/evaluator.rs` lines 480-527 (`UNSERVED_EXPRESSION_HEADS`/`SERVED_QUERY_HEADS` tables — the authoritative Slice-1-vs-later boundary) and 1183-1192 (`field-of`'s NodeRef-only note).
- `rust/crates/babylon-bsl/src/structural_verbs.rs` lines 1-40 (module doc: the declared substrate gap) and 685-698 (`update-edge`/`update-hyperedge`'s refusal text, verbatim).
- `docs/reference/bsl-language.rst` §3.6 lines 2640-2684 (the R9 graph-scope-state ruling — carrier `NodeType`, `:ceiling 1`) and §2.12 lines 2065-2135 / line 4309 (attributed membership, Amendment AG).
- `ai/decisions/ADR189_amendment_ag_attributed_membership_lattice_instances.yaml` (attributed membership's ratified scope and its CommunitySystem precedent).

## 2. COMPUTATION CATALOG (execution order, `step()`, electoral.py:182-234)

### C0 — Guards (electoral.py:188-199)
- **(a)** No `PoliticalFaction` org anywhere ⟹ return immediately (the parties-exist guard,
  "TRAP 3" per the module's own naming); no active `SOCIAL_CLASS` ⟹ return.
- **(b)** `parties = self._political_factions(wrapped)` (240-248); `classes = self._active_classes(wrapped)` (250-258).
- **(c) Reads:** `ORGANIZATION.org_type` (filtered to `OrgType.POLITICAL_FACTION.value`), `SOCIAL_CLASS.active` (default `True`).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### C1 — Window pruning (`_prune_windows`, electoral.py:1241-1250)
- **(a)** Drop every disillusion-window row whose `opened_tick + window_ticks` has elapsed.
- **(b)** `live = {class_id: row for ... if opened_tick + window_ticks > tick}` (1245-1249); rewritten whole.
- **(c) Reads:** graph attr `electoral_disillusion` (`ELECTORAL_DISILLUSION_ATTR`).
- **(d) Writes:** graph attr `electoral_disillusion` (whole-dict rewrite).
- **(e) Defines:** none (the per-row `window_ticks` was captured at window-open time).
- **(f) Events:** none.

### C2 — Popular-front conjuncture (`_popular_front_conjuncture`, electoral.py:318-383; §3.4, U12/ADR139)
- **(a)** Every tick, unconditionally (not clock-gated): measure fascist-consolidation pressure;
  below `popular_front_trigger` the conjuncture (if active) closes; at/above it, a first crossing
  resolves the COMMIT/AUTONOMY forced choice for every party by doctrine stance and opens the
  register; while active, committed orgs accrue CO_OPTIVE dependence toward the "defended apex"
  sovereign and the register's `suppression` (committed share of loyal allegiance mass) updates.
- **(b)** `pressure = consolidation_pressure(...)` (343, delegates to conjuncture.py:53-109 — pure
  arithmetic: `fascist_fraction / max(1, ideology_bearing)` gated by `fascist_majority_fraction`,
  averaged with three binary violence/stance/extraction gates, `max` of the two routes). Register
  write: `register["suppression"] = self._front_suppression(...)` (381, `held / loyal` clamped
  `[0,1]`, electoral.py:472-492). `_accrue_commit_coupling` (494-534): new TRANSACTIONAL edge at
  `co_optive_dependence=rate` or `min(1.0, existing + rate)` on an existing one — **upper-only
  clamp**, no lower bound (safe in practice since `rate >= 0` and the field never goes negative,
  but structurally a different clamp shape than the double-clamps elsewhere).
- **(c) Reads:** `SOCIAL_CLASS.ideology` (dict, `national_identity`/`class_consciousness`), CLAIMS
  edges + `SOVEREIGN.ruling_faction_id` + faction `colonial_stance` (**RESERVED-LINE**, National
  Question), CLAIMS edges + `SOVEREIGN.extraction_policy`, graph attrs `state_violence_index`/
  `state_violence_index_max` (honest-absent — see §5), `ORGANIZATION.acquired_doctrine_ids`
  (**RESERVED-LINE**, doctrine content), graph attr `popular_front`, `SOCIAL_CLASS.allegiance`
  (for `_front_suppression`), CLAIMS edges (`_defended_apex`), existing TRANSACTIONAL edge
  `co_optive_dependence`.
- **(d) Writes:** graph attr `popular_front` (whole-dict rewrite, `{active, since_tick, arms,
  suppression}`); TRANSACTIONAL edge `edge_mode`/`co_optive_dependence` (add or update).
- **(e) Defines:** `politics.popular_front_trigger` (0.6, `[0,1]`, defines.yaml:1115),
  `politics.popular_front_cooptation_rate` (0.05, `[0,1]`, defines.yaml:1116),
  `endgame.fascist_majority_fraction` (0.9, `[0.5,1.0]`, defines.yaml:312).
- **(f) Events:** `POPULAR_FRONT_CALLED` (once, on first crossing; electoral.py:366-374).

### C3 — Derecognition counter-play (`_evaluate_derecognition`, electoral.py:555-637; §3.2 stance 3, U12)
- **(a)** Every tick: an entryist org (holds the `entryism` doctrine stance) whose intra-host
  influence share crosses `host_threat_threshold` against its pole-matched host is permanently
  expelled (absorbing — no re-recognition path).
- **(b)** `influence = own / (own + host_mass)` (electoral.py:621, `own`/`host_mass` from
  `_allegiance_masses`, 639-648 — raw per-class `allegiance` sum, national grain); crossing test
  `influence <= threshold` skip / else expel. Register write is a **stamped-on-first-crossing
  tuple** (never a neutral empty tuple — the module's own "TRAP 3" naming).
- **(c) Reads:** `ORGANIZATION.acquired_doctrine_ids` (**RESERVED-LINE**), `ORGANIZATION.ideology`
  (**RESERVED-LINE**, free string against `_SPOILER_POLES`), `SOCIAL_CLASS.allegiance`, graph attr
  `electoral_derecognized`.
- **(d) Writes:** graph attr `electoral_derecognized` (sorted tuple, append-only).
- **(e) Defines:** `politics.host_threat_threshold` (0.3, `[0,1]`, defines.yaml:1120).
- **(f) Events:** `HOST_DERECOGNIZED` (per newly-expelled org, electoral.py:625-635).

### C4 — Governance-endgame consequences (`_consume_governance_endgame`, electoral.py:849-901; §3.5, U12 D4)
- **(a)** Exactly one tick after PolicySystem stamps a `governance_endgame` fork row (punctual —
  `opened_tick + 1 == tick`), execute the RUPTURE arm's consequence: ALLENDE geometry falls the
  government and suspends the clock (routes through C6's `_suspend`); SYNTHESIS geometry spikes
  hope for every bridged class. CAPITULATE needs nothing here (the delivery-gap machinery IS the
  consequence, per the docstring).
- **(b)** Punctuality test `int(row.get("opened_tick", -2)) + 1 != tick` (877, note the sentinel
  default `-2`, never `-1`, so a missing key can never accidentally match tick `-1`). ALLENDE:
  conditional government-row deletion (only if the seat's `party_id == org_id`, 883), then
  `_suspend`. SYNTHESIS: `HOPE_SPIKE` per bridged class, gain = `defines.hope_spike_gain` (898).
- **(c) Reads:** graph attr `governance_endgame` (**written by PolicySystem @17.47, one tick
  stale — PolicySystem runs AFTER Electoral within the same tick, so this tick's read is last
  tick's write**), graph attr `electoral_governments`, `TERRITORY.legitimation_index` (via
  `_mean_legitimation`), SOLIDARITY edges (via `_has_bridges`).
- **(d) Writes:** graph attr `electoral_governments` (ALLENDE deletion only); plus everything
  `_suspend` writes (C6).
- **(e) Defines:** `politics.hope_spike_gain` (0.3, `>= 0.0`, defines.yaml:1099).
- **(f) Events:** `HOPE_SPIKE` (per bridged class, SYNTHESIS only, 892-901); `ELECTIONS_SUSPENDED`
  + `DISILLUSION_WINDOW_OPEN` (via `_suspend`, ALLENDE only).

### C5 — Betrayal-window opening (`_open_betrayal_windows`, electoral.py:903-932; §3.3, U12, "SYRIZA-voter curve")
- **(a)** Every tick: a class without a live disillusion window whose accumulated `policy_delivery`
  integral has crossed `betrayal_threshold` gets one opened. **Durable by construction** — after a
  window prunes, a class whose integral still stands past threshold re-opens.
- **(b)** `betrayal_crossed(value, threshold)` (930, `threshold > 0.0 and integral >= threshold` —
  a non-positive threshold is a documented mod-switch that disables the curve entirely).
- **(c) Reads:** graph attr `policy_delivery` (**PolicySystem's, one tick stale — same
  tick-order reasoning as C4**), graph attr `electoral_disillusion` (skip-if-present check).
- **(d) Writes:** graph attr `electoral_disillusion` (via `_open_windows`, C10's shared helper).
- **(e) Defines:** `politics.betrayal_threshold` (1.0, `> 0.0`, defines.yaml:1107 — **note**: a
  DIFFERENT, unrelated `edge_transition.betrayal_threshold` (3.0, `[0,10]`) also exists in
  defines.yaml:496; the two share a leaf name across namespaces but Electoral reads only
  `services.defines.politics.betrayal_threshold`, resolved correctly via `defines =
  services.defines.politics` at line 195 — flagged only so a future reader doesn't conflate them).
- **(f) Events:** `DISILLUSION_WINDOW_OPEN` (per newly-betrayed class, via `_open_windows`).

### C6 — Per-sovereign election clock + L-SUSPEND (electoral.py:224-234, 938-983)
- **(a)** For every `SOVEREIGN` whose `JurisdictionLevel` clock divides the tick, with a nonempty
  electorate: if mean claimed-territory legitimation sits below `legitimacy_backfire_threshold`
  AND any `INSTITUTION` is bonapartist-dominant (its `institutionalist_bonapartist` weight above
  threshold, the other two fractions both excluded below a second threshold), suspend — no vote is
  counted, every loyal class opens a disillusion window; otherwise proceed to C7.
- **(b)** Clock: `interval = int(dict(defines.cycle_ticks).get(level, 0)); tick % interval == 0`
  (226-227, `int()` cast of an already-`dict[str,int]`-typed value — not a demotion hazard, just
  belt-and-suspenders). `_level_of` (263-275): a **bounded** walk up the ADMINISTERS DAG
  (`range(len(_LEVEL_BY_DEPTH) + 2)` = 5 iterations max, statically bounded — no unbounded loop).
  L-SUSPEND predicate: `legitimation >= legitimacy_backfire_threshold` short-circuits False (943);
  else `any(_is_bonapartist(...) for node in ALL Institution nodes)` (947-949) — **not scoped to
  the firing sovereign's own institutions; every Institution node graph-wide is checked** (verbatim
  behavior, see §6 for the same non-scoping pattern in C9).
- **(c) Reads:** `SOVEREIGN` nodes, ADMINISTERS edges (`_administers_parent`, `_level_of`), CLAIMS
  edges (`_claimed_territories` → `graph.query_territory_claims`, a **bespoke sorted-edge
  accessor reading `control_level`/`legal_status` edge attributes** — see §6), TENANCY edges
  (`_electorate`), `TERRITORY.legitimation_index`, `INSTITUTION.internal_balance` (dict, read for
  `institutionalist_bonapartist`/`liberal_technocratic`/`revanchist_fascist`).
- **(d) Writes (suspension branch only, `_suspend`, 961-983):** none directly beyond the events +
  `_open_windows`' `electoral_disillusion` write.
- **(e) Defines:** `politics.cycle_ticks` (`{federal:104, state:104, local:52}`, defines.yaml:1083-1086,
  a `dict[str,int]` — the only non-scalar `PoliticsDefines` field Electoral reads), `politics.legitimacy_backfire_threshold`
  (0.35, `[0,1]`, defines.yaml:1121), `institution.bonapartist_threshold` (0.4, `[0,1]`,
  defines.yaml:849), `institution.bonapartist_exclusion_threshold` (0.35, `[0,1]`, defines.yaml:850).
- **(f) Events:** `ELECTIONS_SUSPENDED` (per suspended sovereign, 970-975); `DISILLUSION_WINDOW_OPEN`
  (per loyal class, via `_open_windows`, 983).

### C7 — The count (electoral.py:672-699, 707-843)
- **(a)** Per class, turnout is a law over loyal-allegiance mass, hope, and repression exposure;
  each class's turned-out population splits among parties by allegiance share (abstention residual
  simply doesn't vote); an independent-ballot-line org's votes tax the ideologically-nearest
  duopoly machine before winner resolution; the plurality winner is FPTP, with a seeded coin only
  inside a recount-grade top-two margin.
- **(b)** `turnout_share(base_turnout, loyal_mass, hope, repression_faced, suppression_weight)`
  (formulas/politics.py:234-259: `net = base·loyal_mass·hope − w_sup·repression`, clamped `[0,1]`
  via `min(1.0, max(0.0, net))` — **double clamp, same shape as Territory's Phase-1 pattern**).
  `_count_votes` (721-741): `cast = population * turnout; votes[p] += cast * (mass/loyal_mass)` —
  a class with `loyal_mass <= 0.0` is skipped entirely (no divide-by-zero). `_apply_spoiler_arithmetic`
  (743-826): for each independent-line org (union of `independent_ballot_line`-stance orgs and the
  `electoral_derecognized` register — **an expelled entryist pays the same spoiler tax**), subtract
  `min(spoiler_votes, target_votes)` from the pole-matched highest-vote target, **floored at
  zero**; when several independents run, each applies sequentially in id order against the
  already-mutated tally (documented order-dependence, not a bug). `_resolve_winner` (828-843):
  `shares[0] - shares[1] < recount_margin` ⟹ one `rng.random() < 0.5` draw via `resolve_rng`
  (III.7-seeded); else plain FPTP by `(-votes, id)` sort (679).
- **(c) Reads:** `SOCIAL_CLASS.allegiance`, `.hope`, `.repression_faced`, `.population`
  (default `1`, not `0` — a nonzero default distinct from Territory's convention),
  `ORGANIZATION.acquired_doctrine_ids` (**RESERVED-LINE**), `ORGANIZATION.ideology`
  (**RESERVED-LINE**), graph attr `electoral_derecognized`.
- **(d) Writes:** none (pure computation feeding C8/C9/C10).
- **(e) Defines:** `politics.base_turnout` (0.55, `[0,1]`, defines.yaml:1087),
  `politics.suppression_cost_weight` (0.2, `>= 0.0`, **no upper bound declared** — the field's
  `Field(ge=0.0)` has no `le`, defines.yaml:1119), `politics.recount_margin` (0.005, `[0,1]`,
  defines.yaml:1095).
- **(f) Events:** `ELECTION_HELD` (once per election, 686-699, payload carries turnout,
  competitiveness, winner, spoiler target/shift).

### C8 — Government formation (`_form_government`, electoral.py:989-1042, 1044-1060; renormalize_faction_balance)
- **(a)** The winner is written to the governments register; a `_FACTION_BY_IDEOLOGY`-mapped
  ideology (**RESERVED-LINE content vocabulary — see §6**) nudges every StateApparatus-shaped
  org's `faction_balance` toward the winner's aligned faction, bounded per tick.
- **(b)** `governments[sovereign_id] = {party_id, formed_tick, share}` (1000-1004). `_toward`
  (1044-1060): a proposed all-mass-on-target `FactionBalance`. `renormalize_faction_balance`
  (faction_dynamics.py:171-242) is a **bounded 5-iteration clamp-then-normalize fixed point**:
  clamp each faction's delta from the previous balance to `[-max_shift, +max_shift]`, renormalize
  to sum 1.0 (which can push a delta back out of bound), re-check, repeat up to 5 times, then
  `round(x, 6)` each output. **No other system or module anywhere in the estate exercises this
  loop** — it is genuinely novel among everything inventoried so far (see §4/§6).
- **(c) Reads:** all `ORGANIZATION` nodes with a `faction_balance` dict attr (not type-filtered
  beyond attribute presence).
- **(d) Writes:** graph attr `electoral_governments`; `ORGANIZATION.faction_balance` (dict, all
  matching orgs graph-wide — **not scoped to the firing sovereign**, same pattern as C6/C9).
- **(e) Defines:** `state_ai.max_faction_shift_per_tick` (0.05, `[0,0.2]`, defines.yaml:756).
- **(f) Events:** `GOVERNMENT_FORMED` (once per election, 1013-1022).

### C9 — Legitimation refresh + institution balance shift (electoral.py:1075-1142; §2.5/§3)
- **(a)** Every claimed territory's `legitimation_index` blends toward
  `turnout × competitiveness` (a walkover manufactures less consent than a contest); every
  `INSTITUTION` node's factional balance shifts per the alpha-smoothed crisis/legitimacy law.
- **(b)** `new_index = min(1.0, max(0.0, index + weight*(refresh - index)))` (1090, EMA-style,
  double clamp). `_crisis_intensity` (1131-1142): `in_crisis / len(claimed)` where a territory
  counts if `tick_crisis_phase in {"onset","early","deep"}`. `update_internal_balance` call
  (1121-1127) **does NOT pass `alpha`, `bonapartist_threshold`, or
  `bonapartist_exclusion_threshold`** — see §6 defect note; it silently falls back to the
  function's hardcoded Python defaults (0.05/0.4/0.35), which happen to equal the current
  `defines.yaml` values (institution.alpha_smoothing_rate=0.05, .bonapartist_threshold=0.4,
  .bonapartist_exclusion_threshold=0.35) — a change to any of the three in `defines.yaml` would
  silently NOT reach this call site while still reaching C6's L-SUSPEND check.
- **(c) Reads:** `TERRITORY.legitimation_index`, `TERRITORY.tick_crisis_phase` (str, default
  `"normal"`), all `INSTITUTION` nodes' `internal_balance` (dict, filtered by
  `k != "hegemonic_fraction"` before Pydantic reconstruction — the computed field cannot be passed
  back into the constructor).
- **(d) Writes:** `TERRITORY.legitimation_index`; `INSTITUTION.internal_balance` (dict, ALL
  Institution nodes graph-wide — same non-scoping pattern as C6/C8).
- **(e) Defines:** `politics.legitimation_refresh_weight` (0.5, `[0,1]`, defines.yaml:1106).
  (`institution.alpha_smoothing_rate`/`.bonapartist_threshold`/`.bonapartist_exclusion_threshold`
  are declared but NOT read at this call site — see (b).)
- **(f) Events:** `LEGITIMATION_REFRESH` (per claimed territory, 1092-1101);
  `INSTITUTION_FACTION_SHIFT` (per institution whose hegemonic fraction changed) /
  `INSTITUTION_BONAPARTIST_MODE` (per institution crossing the mode threshold) — dispatched by
  Python `type(event).__name__` string match (1148, a duck-typed dispatch rather than an
  `isinstance` check — noted as a code-quality oddity, not a blocker).

### C10 — H-collapse / disillusion-window routing (electoral.py:1174-1239)
- **(a)** Every electorate class whose plurality party lost (or abstained/had none) opens a
  disillusion window, stamped with whether it has a live SOLIDARITY bridge (routes next tick's
  boosted conversion toward organization vs. fascist alignment — T-7, AllegianceSystem's
  consumption, out of scope here).
- **(b)** `_plurality` (1188-1197): `max(sorted(allegiance), key=...)` — **sorted-then-max, a
  deterministic tie-break to the lexicographically-last key on exact ties** (matches Territory's
  and other systems' `(value, id)` tie-break convention in spirit, though here the tie-break is
  implicit in Python's stable `max` over a pre-sorted iterable rather than an explicit tuple key).
  `_open_windows` (1199-1228): stamps `{opened_tick, window_ticks, bridges_present}` per class.
- **(c) Reads:** `SOCIAL_CLASS.allegiance`, SOLIDARITY edges + `.solidarity_strength` edge
  attribute (`_has_bridges`, 1230-1239 — **an edge-attribute read**), `politics.disillusion_window_ticks`.
- **(d) Writes:** graph attr `electoral_disillusion`.
- **(e) Defines:** `politics.disillusion_window_ticks` (26, `>= 1`, defines.yaml:1104).
- **(f) Events:** `DISILLUSION_WINDOW_OPEN` (per newly-losing class, 1218-1227).

**Events emitted by the whole system: 10 distinct `EventType` values** (grep-confirmed, one call
site each): `POPULAR_FRONT_CALLED`, `HOST_DERECOGNIZED`, `ELECTION_HELD`, `HOPE_SPIKE`,
`ELECTIONS_SUSPENDED`, `GOVERNMENT_FORMED`, `LEGITIMATION_REFRESH`,
`INSTITUTION_BONAPARTIST_MODE`, `INSTITUTION_FACTION_SHIFT`, `DISILLUSION_WINDOW_OPEN`. Per the
task's standing note, TickReport carries no event log today — every one of these is a WS1 (#502)
ledger row, unpinnable by goldens until that lane lands (the goldens instead assert on the
`_RecordingBus`/`state.events` per-tick capture, a test-harness-only channel).

## 3. TYPE INVENTORY

Runtime-storage caveat, same as every prior inventory in this series:
`BabylonGraph.update_node` is a plain dict merge with no mid-tick quantization (`SnapToGrid`
applies only at Pydantic model instantiation) — all in-tick arithmetic below is raw Python
`float`/`int`/`dict`/`tuple`, not the constrained Pydantic types the entity models declare.

| Attribute | Node/graph scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `org_type` | ORGANIZATION | `Literal[OrgType.POLITICAL_FACTION]` (per-subtype) | 4-member closed set | **Enum discriminant** |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean |
| `allegiance` | SOCIAL_CLASS | `dict[str, float]` | keys = live party org ids (open, scenario-defined cardinality); values `[0,1]`, `Σ <= 1` | **open-cardinality map — no BSL field type; the single most load-bearing read in the whole system** |
| `hope` | SOCIAL_CLASS | `float` — **confirmed zero declared field on `SocialClass`** (grep: no `hope\s*:` match anywhere in `models/entities/social_class.py`) | `[0,1]` by convention, unenforced at this layer (no `Field(ge=,le=)` exists to enforce it) | **graph-attribute-only, deliberately undeclared** — the system's own module docstring names this exactly (electoral.py:5: "hope never survives the WorldState round-trip"), which is why C7's `_turnout` can only ever read AllegianceSystem's SAME-tick write, never a persisted value; a BSL port would need the equivalent of a non-persisted/scratch value, or must accept that `hope` is recomputed every tick rather than carried as durable node state |
| `repression_faced` | SOCIAL_CLASS | `Probability` (`Annotated[float, ge=0.0, le=1.0]`, models/types.py:50-56) | `[0,1]` | unit-interval |
| `population` | SOCIAL_CLASS | `int` | `>= 0`, **default 1** (not 0 — distinct from Territory's convention) | integer |
| `ideology` (SOCIAL_CLASS) | SOCIAL_CLASS | `IdeologicalProfile` (`national_identity`, `class_consciousness` both `Annotated[float, ge=0,le=1]`, `agitation` `[0,∞)` unbounded — agitation unread here) | `[0,1]` per sub-field | nested model, two sub-fields read |
| `ideology` (ORGANIZATION) | ORGANIZATION (PoliticalFaction) | `str` (`Field(min_length=1)`) | free-form content vocabulary keyed against `_FACTION_BY_IDEOLOGY`/`_SPOILER_POLES` dicts (`liberal_imperial`/`social_democratic`/`restorationist`/`fascist`) | **RESERVED-LINE — political/ideological content, not type-enforced** |
| `acquired_doctrine_ids` | ORGANIZATION | `tuple[str, ...]` | open set of doctrine-tree content ids | **RESERVED-LINE — doctrine content, open cardinality** |
| `ruling_faction_id` | SOVEREIGN | `str \| None` | node-id reference | **node-reference-valued attribute — no BSL field type stores a node id** |
| `colonial_stance` | ORGANIZATION (ruling faction) | `ColonialStance` (StrEnum, 3 members) | closed set | **Enum discriminant, RESERVED-LINE (National Question, ADR171)** |
| `extraction_policy` | SOVEREIGN | `ExtractionPolicy` (StrEnum, 3 members) | closed set | **Enum discriminant, RESERVED-LINE** |
| `control_level` | CLAIMS edge | `float` | `[0,1]` by convention (UNVERIFIED against a declared Field bound in this survey) | **edge attribute** |
| `legal_status` | CLAIMS edge | `str` | closed-ish string set (`de_jure`/`de_facto`/...) per `ClaimLegalStatus`-style naming (module docstring, balkanization.py:22-24) — read by `query_territory_claims` but not consumed further inside Electoral | **edge attribute, string-typed** |
| `edge_mode` | TRANSACTIONAL edge | `EdgeMode` (StrEnum, 5 members, `CO_OPTIVE` used) | closed set | **Enum discriminant, edge attribute** |
| `co_optive_dependence` | TRANSACTIONAL edge | `float` | `[0,1]`, monotone-accreting (upper-clamped only) | **edge attribute, accumulator** |
| `solidarity_strength` | SOLIDARITY edge | `float` | `>= 0` presence test only (`> 0.0`) | **edge attribute** |
| `legitimation_index` | TERRITORY | `float` | `[0,1]`, default `0.5` | unit-interval (also written by LifecycleSystem @7.0 — two-writer field, see §5) |
| `tick_crisis_phase` | TERRITORY | `str` | `{"normal","onset","early","deep",...}` (compared against `_CRISIS_PHASES` frozenset), default `"normal"` | **string enum-shaped, not a declared StrEnum at the type layer** |
| `internal_balance` | INSTITUTION | `InternalBalanceOfForces` (dict on the wire) | 3 weights `[0,1]` summing to 1.0 (±0.01 tolerance) + `internal_contestation [0,1]` + computed `hegemonic_fraction: RulingClassFraction` (3 members) | nested model with a **computed enum field that must be filtered before reconstruction** |
| `faction_balance` | ORGANIZATION | `FactionBalance` (dict on the wire) | 3 weights `[0,1]` summing to 1.0 (±0.01) + `stability`/`legitimacy` `Probability` + computed `dominant_faction: StateFaction` (3 members) | nested model, same computed-field pattern (unfiltered here — Pydantic's default `extra="ignore"` silently drops it, an inconsistency with `internal_balance`'s explicit filter, not a bug) |
| `state_violence_index` / `_max` | graph-level | `float` | `[0,1]` by convention | **honest-absent — provably 0.0/1.0 on every production path (no writer tree-wide), charter-governed declared-absence row (`sentinels/reachability/registry.py:103-111`)** |
| `electoral_governments` | graph-level | `dict[sovereign_id, {"party_id": str, "formed_tick": int, "share": float}]` | keyed by live sovereign ids | **graph-level dict register, no BSL storage class** |
| `electoral_disillusion` | graph-level | `dict[class_id, {"opened_tick": int, "window_ticks": int, "bridges_present": bool}]` | keyed by live class ids | **graph-level dict register** |
| `popular_front` | graph-level | `{"active": bool, "since_tick": int, "arms": dict[party_id, str], "suppression": float}` | mixed singleton + per-party-keyed sub-map | **graph-level dict register, mixed cardinality** |
| `electoral_derecognized` | graph-level | `tuple[str, ...]` | sorted, append-only | **graph-level set register (structurally the simplest of the six — matches Territory's `under_eviction` one-way-latch precedent per-entity, just not per-node here)** |
| `governance_endgame` | graph-level (PolicySystem's) | `dict[org_id, {"arm": str, "opened_tick": int, "sovereign_id": str, "geometry": str}]` | keyed by org id, **embeds a node-id reference (`sovereign_id`) as a dict value** | **graph-level dict register with an embedded node reference** |
| `policy_delivery` | graph-level (PolicySystem's) | `dict[class_id, {"integral": float, ...}]` | keyed by class id | **graph-level dict register** |

## 4. FLOAT-OP INVENTORY

**No `exp`/`log`/`pow`/sigmoid anywhere in ElectoralSystem's actual call graph** (grep-confirmed
zero hits in electoral.py, formulas/politics.py's `competitiveness`/`turnout_share`,
conjuncture.py, governance_endgame.py, institution/balance.py, faction_dynamics.py's
`renormalize_faction_balance`). `formulas/politics.py`'s only `math.sqrt` call sites
(`platform_vector` line 145, `interest_fit` lines 227-228) belong to functions ElectoralSystem
never imports. **`libm_hazards = False` for this system's direct dependency chain** — a genuine
point of relief given the system's overall complexity.

Shapes, in execution order (grouped by computation, deduplicated where the same shape repeats):

1. **Threshold comparisons / boolean gates** (throughout C2/C3/C5/C6): plain `<`, `>=`, `<=` — no
   hazard.
2. **Weighted-average / EMA blend:** `index + weight * (refresh - index)` (C9, electoral.py:1090)
   — one multiply, one subtract, one add, standard linear interpolation shape, then a double
   clamp `min(1.0, max(0.0, ...))`.
3. **Ratio with divide-by-zero guard:** `own / denom` guarded by `if denom <= 0.0: continue` (C3,
   621-621); `cast * (mass / loyal_mass)` guarded by `if loyal_mass <= 0.0: continue` (C7, 734-740);
   `held / loyal` guarded `if loyal <= 0.0: return 0.0` (C2, 490-492); `in_crisis / len(claimed)`
   guarded by an early `if not claimed: return 0.0` (C9, 1131-1133). **Every division site in the
   system is guarded** — no bare-divide hazard found.
4. **Double clamp, `min(hi, max(lo, x))` shape:** `turnout_share`'s final clamp (formulas/politics.py:259),
   `competitiveness`'s final clamp (278), `_front_suppression` (electoral.py:492),
   `_refresh_legitimation` (1090) — **consistent shape across the system**, matching Territory's
   Phase-1 pattern and NOT matching Territory's Phase-3 upper-only inconsistency.
5. **Upper-only clamp:** `min(1.0, existing + rate)` (`_accrue_commit_coupling`, electoral.py:526)
   — structurally distinct from #4 (no lower bound expressed, though algebraically safe here since
   the accumulator starts at 0 and `rate >= 0`). Flag for port-as-is transcription fidelity (same
   discipline Territory's inventory applied to its own two-clamp inconsistency).
6. **Vote-subtraction floor:** `shift = min(spoiler_votes, target_votes); votes[target] -= shift`
   (C7, `_apply_spoiler_arithmetic`, 823-824) — floors implicitly (subtracting `min(a,b)` from `b`
   can't go negative), no explicit `max(0.0, ...)` needed but worth noting the invariant is
   structural, not defensive.
7. **`Int()` casts (×4):** `int(dict(defines.cycle_ticks).get(level, 0))` (226),
   `int(row.get("opened_tick", -2))` (877), `int(defines.disillusion_window_ticks)` (1210),
   `int(row.get("opened_tick", 0)) + int(row.get("window_ticks", 0))` (1248). **None of these are
   Real→Int demotions of a computed float** — every one casts an already-integer-valued source
   (a `dict[str,int]` define, a tick number, an integer define) defensively. Distinct from
   Territory's `int(current_pop * displacement_rate)` pattern; no `floor`-intrinsic need here.
8. **`round(x, 6)` (×6 total, two call sites, both in ElectoralSystem's own dependency chain):**
   `update_internal_balance` (institution/balance.py:86-89, called from C9) and
   `renormalize_faction_balance` (faction_dynamics.py:237-239 and 510-512 — the second site
   unreachable from ElectoralSystem, called from C8). **No BSL intrinsic serves `round`** — the
   landed set is `{exp, log, floor}` only (declarations.rs:110). Python's `round()` for floats is
   round-half-to-even (banker's rounding) at the underlying binary representation; a naive port to
   Rust's `f64` (`.round()`, round-half-away-from-zero) or a hand-rolled
   `floor(x * 1e6 + 0.5) / 1e6` reconstruction **diverges from Python's semantics exactly at
   halfway-to-6-decimal-places boundary cases** — a genuine, previously-unseen-in-this-series
   float-op hazard, distinct from the exp/log/sigmoid class the task flags but real enough to name.
9. **Bounded 5-iteration clamp-renormalize fixed point** (`renormalize_faction_balance`,
   faction_dynamics.py:171-242, called from C8): clamp three deltas to `[-max_shift, max_shift]`,
   renormalize to sum 1.0 (can re-violate the per-delta bound), re-check convergence, repeat up to
   5 times (a Python `for _iteration in range(5)` — statically bounded, not unbounded), then
   `round(...,6)`. **This is architecturally unlike anything else catalogued in this port-inventory
   series**: it is not a single expression but an iterative numerical solver with an early-exit
   convergence check. See §6 for the port-shape discussion.
10. **Bounded DAG-walk loop** (`_level_of`, electoral.py:263-275): `for _ in range(len(_LEVEL_BY_DEPTH) + 2)`
    (5 iterations max) walking `_administers_parent` — statically bounded, terminates on cycle
    detection (`seen` set) or `None` parent. Not a float-op but included here because it is the
    system's only other bounded-loop numerical procedure besides #9.
11. **Bare non-integer literals:** pervasive (`0.0`/`0.5`/`1.0` throughout every clamp, default,
    and fallback expression — e.g. electoral.py:1090, 492, 707-719). Same **"no bare non-integer
    literal" BSL parser constraint** Territory's inventory found — every one needs a `c`-suffixed
    const or the Real-zero-promotion idiom; not exhaustively line-listed here given the volume
    (dozens of sites), but the shape is uniform and precedented.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.45** (electoral.py:175), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `... FascistFactionSystem → AllegianceSystem (17.42) →
  ElectoralSystem (17.45) → PolicySystem (17.47) → SovereigntySystem → ...`.

- **Reads from same-tick prior systems (fresh, this-tick writes):**
  - `SOCIAL_CLASS.allegiance` / `.hope` — written by **AllegianceSystem @17.42**, same tick,
    immediately prior (electoral.py module docstring lines 3-9 self-documents this; confirmed
    writer at `allegiance.py:172,179`).
  - `SOCIAL_CLASS.repression_faced` — written by **ImperialRentSystem @9.0**
    (`economic.py:640`, Material Base phase, well before Electoral in the same tick).
  - `TERRITORY.tick_crisis_phase` — written by **TickDynamicsSystem @4.0** (Material Base,
    before Electoral).
  - `TERRITORY.legitimation_index` — also written by **LifecycleSystem @7.0**
    (`lifecycle.py:125`, Material Base) — Electoral's own C9 write (17.45) supersedes it later
    the same tick; **StruggleSystem @16.0** reads `legitimation_index` (`struggle.py:274`)
    BEFORE Electoral runs, so Struggle sees only Lifecycle's write, never Electoral's
    same-tick one (Electoral's write is one tick stale from Struggle's perspective).

- **Reads one-tick-stale from same-tick LATER systems** (the position-17.47-runs-after-17.45
  pattern, self-documented in the module's own docstrings):
  - graph attr `governance_endgame` — written by **PolicySystem @17.47** (`policy.py:116`,
    `GOVERNANCE_ENDGAME_ATTR`); Electoral's C4 read this tick is last tick's PolicySystem write.
  - graph attr `policy_delivery` — written by **PolicySystem @17.47** (`policy.py:107`,
    `POLICY_DELIVERY_ATTR`); same one-tick-stale relationship, feeding C5.

- **Writes consumed downstream (same tick or next tick):**
  - graph attr `electoral_governments` — read **same tick** by **PolicySystem @17.47**
    (`policy.py:707`, the delivery ledger's incumbent — the module docstring's own stated design
    intent, lines 99-102); read **next tick** by **DoctrineSystem @14.7** (`doctrine.py:299`,
    explicitly documented "one tick stale") and **StruggleSystem @16.0** (`struggle.py:254`).
  - graph attr `electoral_disillusion` — read **next tick** by **AllegianceSystem @17.42**
    (`allegiance.py:219`, the T-7 boosted-conversion routing — self-documented one-tick lag).
  - graph attr `popular_front` — read **next tick, one-tick-stale**, by two SAME-tick-earlier
    systems: **ConsciousnessSystem @17.0** (`ideology.py:48`, the fascist-channel `suppression`
    throttle) and **AllegianceSystem @17.42** (`allegiance.py:242`, the `arms` valve-exposure
    entanglement) — both run before Electoral (17.0, 17.42 < 17.45) so both see last tick's write.
  - graph attr `electoral_derecognized` — read same tick (**C3's own internal read**, C7's
    `_apply_spoiler_arithmetic`) and **PolicySystem @17.47** (`policy.py:693`, same tick, after).
  - `ORGANIZATION.faction_balance` — read **next tick** by **OODASystem @14.0**
    (`ooda/npc_stub.py:374-389` via `select_npc_actions`, state-AI decision scoring) — OODA
    runs at 14.0 < 17.45, so this tick's Electoral write is picked up only next tick. **No
    other system reads `faction_balance`** (grep-confirmed across `engine/systems/`).
  - `TERRITORY.legitimation_index` (Electoral's own C9 write) — read **next tick** by
    **StruggleSystem @16.0** (see above) and by Electoral's own C6/C9 (`_mean_legitimation`)
    the tick after.
  - `INSTITUTION.internal_balance` — read **same tick** by Electoral's own C6 (across
    sovereigns firing in the same tick — see the non-scoping note below) and **next tick** by
    **PolicySystem @17.47** (`policy.py:307`, the judicial-tolerance-scale input).

- **Non-scoping oddity (verbatim, port-as-is):** C6's L-SUSPEND check, C8's faction-balance
  perturbation, and C9's institution-balance shift all iterate **every** `INSTITUTION`/
  `ORGANIZATION` node graph-wide (`graph.query_nodes(node_type=NodeType.INSTITUTION)` /
  `...ORGANIZATION...`), never filtered to the firing sovereign's own claimed territory or
  jurisdiction. If two sovereigns' clocks land on the same tick (plausible: federal=104,
  state=104 — a state and the federal apex can co-fire), the second sovereign's election
  re-runs C9's `update_internal_balance` over the SAME institution set the first sovereign's
  election just updated, compounding the alpha-smoothed shift twice in one tick against a
  crisis/legitimacy reading scoped only to the second sovereign's own claimed territories. Not
  observed to be exercised by any of the five canonical goldens (each seeds either zero or one
  sub-sovereign, and none co-fires two clocks on the same tick within the run horizons used) —
  recorded here as a defect/oddity per port-as-is law, not fixed.

- **Context/service usage with no BSL equivalent:** `resolve_rng(services, tick)` (electoral.py:841,
  `kernel/system_base.py:35-55`) — the recount-tie-break RNG, seed-deterministic
  (`random.Random(0xBA1AC1A + tick)` fallback). BSL's own `ξ_t` seeded-draw mechanism (cited
  throughout the module's docstrings as "the congress-purge III.7 precedent") is presumably the
  intended analog but was not verified against the current evaluator surface in this survey —
  flagged as UNVERIFIED rather than asserted portable or blocked.

- **DORMANCY on canonical scenarios.** Checked `tools/regression_scenarios.py`'s `SCENARIOS`
  dict (37-... , 12 total keys) and the factories it imports. **Contrary to the module's own
  docstring** ("Byte-safety... The six qa:regression fixtures carry no parties, so all six are
  byte-identical" — electoral.py:52-55, TRUE only of the original six: `imperial_circuit`,
  `two_node`, `starvation`, `glut`, `fascist_bifurcation`, `single_county`), **five newer
  scenarios (#7-11: `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`) DO seed the full
  party terrain via `apply_political_terrain`** (`electoral_fixture.py:118-263`: 4 `PoliticalFaction`
  orgs, 1 donor `Business`, MEMBERSHIP + TRANSACTIONAL funding edges, the spec-070 balkanization
  seed — 4 `BalkanizationFaction` + 3 `Sovereign` nodes incl. the apex `SOV_USA_FED`, real 2024
  Wayne-County electoral INFLUENCES, `SOV_USA_FED`'s CLAIMS — plus one `INSTITUTION`
  (`INST_FED_JUDICIARY`, RSA_JUDICIAL) and, when `include_michigan=True`, `SOV_MI_STATE` +
  the first-ever-produced ADMINISTERS edge) — **and ARE part of the byte-identical `qa:regression`
  hash gate** (they are ordinary `SCENARIOS` dict entries, `tools/regression_scenarios.py:37ff`,
  keys 7-11 of 12). ElectoralSystem is therefore **thoroughly live**, not dormant, on 5/12
  canonical scenarios — the inverse of Territory's situation. Specific residual dormancy found:
  - `state_violence_index`/`_max` — provably 0.0/1.0 on every path checked (§3), the
    violence-route gate in `consolidation_pressure` never fires on any golden or any other
    scenario in the estate.
  - The RUPTURE arm's both geometries (ALLENDE, SYNTHESIS) in C4 — **neither `mitterrand` nor
    `syriza` (the two goldens that exercise `_consume_governance_endgame` at all) ever resolves
    RUPTURE**; both docstrings state the fork resolves CAPITULATE (electoral_goldens.py:220-221,
    260-261). RUPTURE is covered only by hand-built fixtures in `test_electoral.py:774,795`
    (`test_allende_falls_the_government_and_suspends`, `test_synthesis_spikes_hope_through_the_bridges`)
    — a genuine gap between unit coverage and golden/canonical coverage, matching Territory's
    general finding pattern ("hand-built fixtures for the paths the canonical estate never
    reaches") even though the bulk of the system is NOT in that position here.
  - Two-sovereign-same-tick co-firing (the non-scoping oddity above) — not exercised by any
    golden; UNVERIFIED beyond static code reading whether it can occur given `cycle_ticks`
    defaults (federal=104, state=104 — same interval, same-tick co-fire is arithmetically
    possible whenever both sovereigns exist and both clocks started at the same phase).

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface as given (Query lane Slice 1 landed via
`run_once_into`; Slices 2-4 not built; enum fields landed per ADR195/196 for `deffield`-declared
fields with `field-of` refused on them; `deffield` type vocabulary `{int, bool, currency,
probability, intensity, coefficient, enum}`; Currency-typed field STORAGE refused, bare-scaled-Int
is the ADR183 workaround; intrinsics `{exp, log, floor}`; no imposed functional forms (ADR172
ruling 5); events landed but TickReport carries no event log (WS1/#502); two rules at one position
do not yet share pre-state (D116, open); plus the R9 graph-scope-state carrier-`NodeType` ruling
(landed at evaluator level, `E-EVAL-035`) and Amendment AG attributed membership (ratified at
v3.2.0, but its `membership-field-of` accessor is **Slice 4, Director-escalation-gated,
unserved** — `evaluator.rs:505-508`).

| Computation | Verdict | Detail |
|---|---|---|
| C0 Guards (party-exists, active-classes) | **PORTABLE WITH D-RECORD** | `org_type`/`active` are ordinary scalar/enum node reads, no blocker in isolation — but every downstream computation this guard gates IS blocked (see below), so this row is moot without the rest of the system. |
| C1 Window pruning | **BLOCKED — graph-scope register** | `electoral_disillusion` is a `dict[class_id, {...}]` graph attr; no BSL storage class holds it as-is. Decomposable per the R9 ruling to per-`SOCIAL_CLASS` fields (`class/disillusion-opened-tick` int, `class/disillusion-window-ticks` int, `class/disillusion-bridges-present` bool) with an `:optional`/`:default` sentinel for "no window" — architecturally sound, not language-blocked, but a real content-modeling design task with no landed precedent (name: **graph-scope-register decomposition, R9 §3.6 ruling class**). |
| C2 Popular-front conjuncture | **BLOCKED — allegiance map + graph-scope register + edge writes** | Needs `SOCIAL_CLASS.allegiance` (open-cardinality dict — no BSL field type, see below), CLAIMS-edge `control_level`/`colonial_stance` lookups (edge attributes, Slice 2), the `popular_front` register (singleton sub-fields portable via the landed carrier-`NodeType` mechanism; the per-party `arms` sub-map needs per-`ORGANIZATION` enum fields instead — feasible, enum fields ARE landed), and `update-edge` for `co_optive_dependence` accrual — **`update-edge`/`add-edge` field-writes hit the declared substrate gap**: `GraphSubstrate` stores exactly one bare `f64` per edge (`structural_verbs.rs:16-27`); `edge_mode` + `co_optive_dependence` is two fields, and `edge_mode` isn't even numeric. Real blocker on three independent axes. |
| C3 Derecognition counter-play | **BLOCKED — allegiance map** | `_allegiance_masses` sums `SOCIAL_CLASS.allegiance` — the open-cardinality dict blocker (below) is load-bearing here even before touching `electoral_derecognized` (which, alone, IS portable — see its own row). |
| C4 Governance-endgame consequences | **BLOCKED — graph-scope register with embedded node reference** | `governance_endgame` (PolicySystem's, read one-tick-stale) embeds `sovereign_id` as a dict VALUE — a node-id reference with no BSL field type. Decomposing to a per-`ORGANIZATION` field set (arm enum, opened_tick int, geometry enum) is the R9-ruling path for the scalar sub-fields, but the sovereign-reference needs either an edge (Slice 2) or a structural derivation from existing topology (a design decision, not yet made anywhere in the estate). SOLIDARITY-edge `_has_bridges` read is also Slice 2. |
| C5 Betrayal-window opening | **BLOCKED — graph-scope register** | `policy_delivery` (PolicySystem's) is a `dict[class_id, {"integral": float}]` — decomposable to a per-`SOCIAL_CLASS` `integral` field (an unbounded-above accumulator, same Currency-storage-refused shape Territory's `rent_level` hit — ADR183 bare-scaled-Int workaround applies), architecturally simple IF the register-decomposition design lands; today, no storage. |
| C6 Election clock + L-SUSPEND | **BLOCKED — edge attributes (CLAIMS)** | `_claimed_territories`/`_electorate` depend on `query_territory_claims`, a bespoke sorted-CLAIMS-edge accessor reading `control_level` (float) and needing "top claimant by control" logic — needs EdgeRef reads (Slice 2) at minimum; `_level_of`'s bounded ADMINISTERS-DAG walk is pure topology (no edge attrs) and IS expressible via the landed `neighbors`/`fold`-class query heads once a `select-max`-by-edge-attribute isn't required for THIS particular sub-computation (the DAG walk itself doesn't read edge attributes — only the parent existence). L-SUSPEND's own arithmetic (legitimation mean, bonapartist gate) is portable in isolation; gated by the CLAIMS dependency for `_claimed_territories`. |
| C7 The count (turnout, FPTP, spoiler) | **BLOCKED — allegiance map (central)** | Every one of `_turnout`/`_count_votes`/`_apply_spoiler_arithmetic`/`_plurality` reads `SOCIAL_CLASS.allegiance`. This is THE central blocker of the entire system: no BSL field type stores a `dict[str, float]`, and the natural fix — Amendment AG attributed membership (class as member, party as/via a hyperedge, mass as the attributed payload) — has its accessor (`membership-field-of`) parked at **Slice 4, Director-escalation-gated**, per the task's own CURRENT BSL surface note. Nothing in C7 ports until this is resolved. `electoral_derecognized` read (spoiler-tax targeting) is, by itself, portable (see its own row) but doesn't help without `allegiance`. |
| C8 Government formation | **BLOCKED — graph-scope register + novel iterative solver** | `electoral_governments` needs decomposition (per-`SOVEREIGN` fields: `party_id`-as-reference is the same node-reference problem as C4's `sovereign_id`, though here it could plausibly be inverted — store the winning party ON the sovereign as an edge or a per-party `bool` "governs-sovereign-X" field pattern, a design choice not made in this survey). Independent of storage: `renormalize_faction_balance`'s 5-iteration clamp-renormalize fixed point (§4 item 9) has **no direct BSL expression** — BSL rules are single-pass per-position evaluations over pre-tick state, not iterative solvers; the loop is data-INdependent (fixed at 5 iterations) so a manual 5-step unroll into nested `let`-style expressions is conceivable IF BSL supports intermediate bindings within one rule body (not verified in this survey — flagged UNVERIFIED, name the exact grammar production checked when this is picked up). Either way this is a **genuinely novel port pattern with zero precedent** in any landed pack. |
| C9 Legitimation refresh + institution shift | **BLOCKED — graph-scope + `round()` intrinsic gap** | `TERRITORY.legitimation_index` write is an ordinary scalar update-node, portable in isolation. `INSTITUTION.internal_balance` (a 3-weight-plus-contestation dict, computed-enum-filtered on read) needs per-`INSTITUTION` scalar deffields (mechanically fine, enum fields landed) BUT `update_internal_balance`'s `round(x, 6)` (§4 item 8) has no BSL intrinsic — only `{exp, log, floor}` are declarable. A `floor`-based reconstruction is NOT bit-exact to Python's round-half-to-even at boundary cases — name this precisely as the gap when it's picked up, don't silently substitute. The alpha/threshold-not-passed defect (§2 C9(b)) must be transcribed verbatim (port-as-is), not silently repaired. |
| C10 H-collapse / disillusion routing | **BLOCKED — allegiance map + edge attribute (SOLIDARITY)** | `_plurality` needs `allegiance`; `_has_bridges` needs SOLIDARITY-edge `solidarity_strength` (Slice 2). Register write (`electoral_disillusion`) is the same C1 decomposition. |
| `electoral_derecognized` register, standalone | **PORTABLE WITH D-RECORD** | The one genuinely simple register in the system: a flat, append-only, sorted set of expelled org ids — structurally identical to Territory's `under_eviction` one-way-latch precedent, just per-entity rather than per-node-attribute. A per-`ORGANIZATION` `bool` deffield (`org/derecognized`, absorbing, never reverts) ports cleanly the moment ordinary per-node enum/bool fields are usable for this content — which they already are. This row is the single cleanest sliver in the whole system, but it is not independently useful without `allegiance` (C3/C7 both need it in context). |
| `state_violence_index`/`_max` reads | **PORTABLE WITH D-RECORD** | Provably 0.0/1.0 on every production path (§5) — declare `:const` per the Metabolism-D-2/Territory-D-2 "provably uniform" precedent, same ADR183 class. |
| `ColonialStance`/`ExtractionPolicy`/doctrine-content reads | **RESERVED-LINE, not adjudicated** | National Question parameters and doctrine-tree content are Director-reserved ideological surface (ADR171 and the doctrine-tree charter) — described here, not proposed on. |

**No computation in this system is PORTABLE NOW.** The closest thing to a portable sliver
(`electoral_derecognized` alone, or C0's guard reads alone) is not independently meaningful —
every path through the system that would produce an observable game effect touches the
`allegiance` map, a graph-scope register, or an edge attribute before it's done.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_electoral.py` | 850 | **Primary conformance-oracle candidate.** Direct-System unit tests against a REAL `BabylonGraph` (`create_electoral_fixture_scenario().to_graph()`) with hand-stamped attribute overrides and a minimal `_Services`/`_Context` stub (not the full engine) — covers every computation in §2 including the RUPTURE geometries (774, 795) the canonical goldens never reach, both spoiler-pole directions (600-641), derecognition absorption (696-727), and the two disillusion-routing branches (269-311). |
| `tests/unit/engine/systems/test_electoral_goldens.py` | 311 | **Secondary conformance-oracle candidate — the behavioral-contract suite.** Drives all five golden scenarios through the REAL `simulation_engine.step()` (full-engine, WorldState round-trip every tick), asserting on captured `state.events` per tick. Backed by a `blessed(electoral-goldens)` byte-level baseline pair (per the module's own docstring, line 22) — a declared §6.5 ceremony baseline, same governance class as the qa:regression baselines. |
| `tests/unit/engine/scenarios/test_electoral_goldens_factories.py` | 99 | Schema/factory-construction tests for the five `create_*_scenario` functions — validates the scenarios build without engine execution; narrative/schema tier, not a tick-behavior oracle. |
| `tests/unit/engine/scenarios/test_electoral_fixture.py` | 144 | Tests `apply_political_terrain`'s output shape (party count, edge count, balkanization seed presence) — schema tier. |
| `tests/unit/domain/politics/test_conjuncture.py` | 217 | Property-style pure-function tests for `consolidation_pressure`/`resolve_popular_front_arm` — a genuine **behavioral-contract candidate** for the pure-math half of C2, independent of graph/engine machinery. |
| `tests/unit/domain/politics/test_governance_endgame.py` | 123 | Pure-function tests for `betrayal_crossed` (and the PolicySystem-only `resolve_governance_arm`/`rupture_geometry`/`phi_share`/`dual_power_live`) — behavioral-contract candidate for C4/C5's threshold math. |
| `tests/unit/institution/test_balance.py` | 283 | Pure-function tests for `update_internal_balance` — behavioral-contract candidate for C9's alpha-smoothing law, including (presumably) the `round(x,6)` output precision the port needs to reproduce or explicitly diverge from. |
| `tests/unit/formulas/test_politics.py` | 286 | Property tests for the full `formulas/politics.py` module — only the `turnout_share`/`competitiveness` subset is Electoral's; the rest (valve, hope field, platform vector, allegiance drift) is AllegianceSystem's/PolicySystem's, out of scope but sharing the file. |
| `tests/unit/config/test_politics_defines.py` | 70 | `PoliticsDefines` schema validation (bounds, the `cycle_ticks` field validator) — schema tier. |
| `tests/unit/config/test_politics_defines_readers.py` | 40 | Confirms `GameDefines.politics` composition — schema tier. |
| `tests/unit/models/test_politics_events.py` | 80 | Event-payload model tests — schema tier, not tick-behavior. |

**No property-based law-test file exists for ElectoralSystem** (`tests/unit/engine/laws/` has no
`test_law_electoral*.py`, `test_law_faction_influence.py` is the only politics-adjacent law file
present) — unlike Territory's `test_law_territory_system.py`, there is no dedicated
invariant-contract suite (e.g., "allegiance masses conserved," "derecognition never reverts,"
"legitimation_index stays in [0,1] across every path") independent of the two behavioral suites
above. This is a genuine test-estate gap worth naming for whoever picks up the port train, though
outside this Phase-1 inventory's remit to fill.

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` hashes every
node/edge attribute of the `WorldState→graph` projection on all 12 canonical scenarios, including
the five electoral goldens (#7-11) — so ElectoralSystem's outputs ARE under the byte-identical
hash gate today, unlike Territory's near-total dormancy. Residual gaps (RUPTURE geometries,
two-sovereign co-firing) are named in §5/§6 above.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) with fresh `rg`/Read. Four corrections,
seven confirmations. This is the most careful report in its batch on the facts most often gotten
wrong elsewhere (the 12-scenario count, the Slice boundary, the substrate gap) and it needs no
re-read; the corrections below are local.

### CORRECTIONS

1. **CORRECTION — §7's byte-gate claim over-reaches: all six of this system's registers are OUTSIDE
   `graph_content_hash`.** The hash (`tools/regression_test.py:924-964`) is computed over
   `state.to_graph()`'s **nodes and edges**, and its own docstring states: *"Graph *metadata*
   (``g.graph``: economy, event log, opposition states) is also excluded, because the spec's field set
   is nodes/edges/actions."* `electoral_governments`, `electoral_disillusion`,
   `electoral_derecognized` and `popular_front` are `SUPERSTRUCTURE_REGISTERS` members
   (`src/babylon/models/superstructure.py`) that round-trip through `WorldState.superstructure_registers`
   and are re-stamped onto `G.graph`, never onto a node (`_harvest_superstructure_registers`,
   world_state.py:344-352; the re-stamp at :823-830) — so not one of them is hashed. What IS
   byte-gated on the goldens is exactly Electoral's node/edge writes: `TERRITORY.legitimation_index`
   (deliberately kept OUT of `TERRITORY_EXCLUDED_FIELDS` since ADR140 — world_state.py:100-103,
   "the electoral refresh must survive the step() round-trip"), `INSTITUTION.internal_balance`,
   `ORGANIZATION.faction_balance`, and TRANSACTIONAL edge writes to the extent `Relationship`
   declares those fields. The behavioral cover for the registers is `test_electoral_goldens.py`'s
   per-tick `state.events` assertions plus the gate-coverage `SystemEvidence` rows — a real but
   non-byte-identical estate. §§2-3 already identify the registers as graph-level; §7 simply does not
   follow that through.

2. **CORRECTION — `renormalize_faction_balance` is NOT "data-INdependent (fixed at 5 iterations)".**
   `faction_dynamics.py:201-234` is `for _iteration in range(max_iterations)` with an in-loop
   convergence test and an early exit: `all_ok = (abs(fc_new - fc_cur) <= max_shift + 1e-9 and …)`
   followed by `if all_ok: break` (:225-233). The effective iteration count is therefore
   **data-dependent**. The row's proposed remedy — "a manual 5-step unroll into nested `let`-style
   expressions is conceivable" — is not a faithful transcription as written, because the loop body is
   not idempotent past convergence: the non-converged path re-seeds `fc_target`/`ss_target`/
   `sp_target` from the normalized values (:232-234), which an unconditional sixth-through-fifth
   re-application would not reproduce. Any unroll must carry the guard at every step. This
   *strengthens* the row's "genuinely novel port pattern with zero precedent" verdict while changing
   the shape it proposes.

3. **CORRECTION — the `the`/carrier-node status is stated two different ways and only one is right.**
   §6's preamble calls the R9 graph-scope ruling "landed at evaluator level, `E-EVAL-035`", and C1/C5
   say "no BSL storage class holds it as-is". Precisely: §3.6 (bsl-language.rst:2650-2688) is landed
   as a RULING with its manifest/cost layer tested, and the evaluator has a dedicated
   `E-EVAL-035`/`UnhydratedCarrier` path — but the accessor is `("the", "slice 2")` at
   `evaluator.rs:506`, the ONE occurrence of the string `"the"` in that file, so the singleton route
   is **unevaluable**. Simultaneously the ruling's closing paragraph makes the keyed route landed
   today: *"per-sovereign and per-county registers are ordinary nodes of ordinary types, reached by
   ordinary queries"* (:2686-2688) — i.e. `electoral_disillusion` (class-keyed) and
   `electoral_governments` (sovereign-keyed) decompose onto the entity's own node with landed Slice-1
   heads (`nodes`/`field-of`/`update-node`, `SERVED_QUERY_HEADS` evaluator.rs:527, `eval_form` arms
   :556-559). The correct, uniform split — which the sibling PolicySystem inventory states outright —
   is **singleton → carrier + `the` → Slice 2; per-entity → ordinary node fields → landed**. C1/C4/
   C5/C8/C10 should each be re-labelled on that axis rather than all filed as "no storage class".

4. **CORRECTION — narrow one clause of the dormancy paragraph.** "five newer scenarios … ARE part of
   the byte-identical `qa:regression` hash gate" is true of the SCENARIOS membership and false of the
   implied register coverage; see correction 1. Keep the scenario claim, drop the inference.

### CONFIRMATIONS

5. **CONFIRMATION — the 12-scenario count, and the credit for getting it right.** `SCENARIOS` has
   twelve keys (`imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation`,
   `single_county`, `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`, `org_probe`);
   `tests/baselines/` carries twelve matching JSON baselines and twelve dense CSVs;
   `regression_test.py:1363,1424,1777` iterate the dict wholesale. This report is the only one in its
   batch that states the count correctly and says so explicitly.

6. **CONFIRMATION — tick position 17.45** (electoral.py:175), ordering per `_SYSTEM_CLASSES`
   (simulation_engine.py:328-363).

7. **CONFIRMATION — `libm_hazards = False` for this system's own chain.** `formulas/politics.py`'s
   only `math.sqrt` sites are :145 (`platform_vector`) and :227-228 (`interest_fit`), neither of which
   ElectoralSystem imports (it takes exactly `competitiveness` and `turnout_share`);
   `survival_calculus.py:43`'s `math.exp` is not on this path. Verified by direct grep. This is a real
   and load-bearing point of relief given the sibling AllegianceSystem report's `sqrt` blocker.

8. **CONFIRMATION, STRENGTHENED — the edge-attribute blockers.** Beyond `edges`/`edge-between`/`the`
   sitting in `UNSERVED_EXPRESSION_HEADS` at slice 2 (evaluator.rs:503-512), `GraphSubstrate` has no
   edge-attribute reader **at all**: the entire edge surface is
   `add_edge(edge_type, from, to, strength: f64)` (substrate.rs:111-117), `remove_edge` (:124), and
   `edges(edge_type) -> Vec<(NodeId, NodeId)>` (:166); `node_attribute` exists at :141 with no edge
   counterpart, and even the mandatory `:strength` has no reader (`rg -n "strength" substrate.rs` →
   lines 22, 105, 116, all write-side). So Slice 2 must mint the substrate read method as well as the
   `EdgeRef`. And C2's TRANSACTIONAL write needs TWO named fields (`edge_mode` + `co_optive_dependence`)
   on one edge, which a one-`f64`-per-edge substrate cannot hold even past Slice 2 — exactly as the
   row says, with `structural_verbs.rs:15-27` naming it a "declared substrate gap, escalated rather
   than silently absorbed".

9. **CONFIRMATION — the `round(x, 6)` intrinsic gap, at both cited sites.**
   `domain/institution/balance.py:85-89` (`liberal_technocratic=round(new_liberal, 6)` and three
   siblings) and `ooda/state_ai/faction_dynamics.py:237-239` — against
   `DECLARABLE_INTRINSICS = ["exp", "log", "floor"]` (declarations.rs:110). The round-half-to-even vs
   round-half-away-from-zero divergence is correctly named rather than waved off.

10. **CONFIRMATION — `allegiance`'s centrality and its named destination.**
    `SocialClass.allegiance: dict[str, float]` (social_class.py:323); the Amendment AG accessor is
    `("membership-field-of", "slice 4")` at evaluator.rs:511, whose own table comment reads "the
    CanonicalState-widening storage lane — Director-ruled deferred to first consumer". C7's "nothing
    ports until this is resolved" is correct.

11. **CONFIRMATION — the RESERVED-LINE handling.** `ColonialStance`/`ExtractionPolicy` (ADR171,
    National Question) and `acquired_doctrine_ids` (doctrine-tree content) are described and not
    proposed on. Correct discipline. One addendum for the eventual pack: `ORGANIZATION.ideology` is
    matched by substring against hardcoded token tuples in **two** engine files —
    `allegiance.py:77` and `reactionary.py:71`, the identical
    `("fascist", "reaction", "revanch", "settler")` — so any closed-vocabulary re-modelling touches
    both systems at once and needs one Director ruling, not two.

### FINAL VERDICT

**BLOCKED — sustained. No portable subset exists: every one of the ten computations touches the
open-cardinality `allegiance` dict (Slice 4, Director-escalation-gated), a graph-scope register, or
an edge attribute (Slice 2 *plus* the `GraphSubstrate` edge-attribute reader it presupposes, and
plus edge-attribute storage widening for C2's two-field TRANSACTIONAL write). Two refinements to
the shape of the block: the graph-scope-register half splits cleanly — the class-keyed and
sovereign-keyed registers decompose onto ordinary nodes with landed Slice-1 heads, only the true
singletons need the carrier node and `the` (Slice 2) — and `renormalize_faction_balance` is a
convergence-guarded iteration, not a fixed 5-step loop, so any unroll must carry the guard.
Byte-gate reach is narrower than §7 states: none of the six electoral registers is hashed; the gate
covers `TERRITORY.legitimation_index`, `INSTITUTION.internal_balance` and
`ORGANIZATION.faction_balance` only.**
