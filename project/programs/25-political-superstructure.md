# Program 25 — The Political Superstructure (Electoralism)

**Chartered:** 2026-07-22 (BD in-session goal: "Expanding the playable interface earlier as we
discussed with electoralism and filling in all of those things that emerged about organizations
that we already discussed"). **Authority:** `ai/_inbox/the-electoral-question.md` (BD-authored
design, all 8 §7 rulings APPROVED — ADR126), Constitution v2.16.0, Modulus C/G/P (no amendment).
**Branch:** `feature/political-superstructure` off dev @ `e0ece454`. **The one engine train**
(interleaving rule); the Archive-side train is `feature/interface-refinement` — lane split
declared in `ai/_inbox/POLITICAL-SUPERSTRUCTURE-LANE-ACTIVE.md`.

**Launch rulings (BD-delegated "i trust your judgment", recorded in ADR127):**
- **TRAP 1 ruled (b):** build the REAL Agitation→Organization conversion pathway (new
  production mechanics, U8). Throttling `route_agitation_to_ternary`'s revolutionary channel
  would conflate class_consciousness with Organization — P(S|R)'s numerator is Organization
  (T-5), and the valve must bind the real quantity or L-VALVE pins a lie. Also discharges the
  org-layer gap that `organization` is a static seeded attribute today.
- **TRAP 2 ruled shadow-first + declared promotion ceremony at U10:** keeps the 8-for-8
  shadow-first catalog discipline and structural byte-safety; §2.6's "electoral question
  becomes the principal contradiction" payoff arrives at a NAMED ADR078-style promotion
  ceremony (baselines regenerated, R-PROOF doc), never silently unreachable.
- **TRAP 3 is discipline, not a ruling:** every class-node write in U3/U8 sits behind a
  parties-exist guard / absence sentinel — never stamp a neutral 0.0 onto a node that did not
  carry the attribute; each of U3/U8/U9/U10/U11/U12 carries `qa:regression` byte-identical as
  its unit gate. **Charter correction (learned at U1):** the gate's defines-provenance
  tooth re-hashes GameDefines, so ANY defines-category unit carries a metadata-only
  ceremony (`Baselines: blessed(...)`, defines_hash + generated_at cells only, verified
  cell-by-cell) — `blessed(p25-politics-defines-hash)` is U1's. VALUE baselines still
  move only at U13's declared golden ceremony.

The unit train below is the recon-corrected plan (5-agent launch recon + Opus synthesis,
2026-07-22; recon corrections are folded in-line — s=p+i+r+t lives in
`domain/economics/distribution/`, catalog is 18→19, qa:regression is 6 scenarios, and
`debt_service`/equalization-wire/InternalBalanceOfForces-activation are BUILDS, not rewires).

---

# PROGRAM 25 CHARTER — The Political Superstructure (Electoralism)

Branch `feature/political-superstructure` @ `e0ece454` (already cut at correct tip). Design = full `ai/_inbox/the-electoral-question.md`, 8 rulings approved (ADR126). Modulus C/G/P, one-engine-train. **ADR block claimed: ADR127–ADR140** (charter = ADR127; per-unit assignments below; `political_form` catalog index bound at U3's implementation commit, not pre-pinned). **RENUMBER 2026-07-23 (BD-approved plan):** mainline took ADR138 for the ledger DOMAIN-contracts unit (pg-domain, PR #295) while this lane was in flight; U12 → ADR139, U13 → ADR140.

Recon corrections folded into every unit: `s=p+i+r+t` lives in `domain/economics/distribution/{types,calculator}.py` (NOT `engine/systems/distribution.py` — dead scaffold); `RuleBasedStateAI` lives in `ooda/state_ai/decision.py` (untouchable — interface/adversary train owns `ooda/`); catalog is **18→19** not the brief's stale "10→11"; qa:regression is **6 scenarios** not 5; `debt_service`, the equalization investment-strike wire, and `InternalBalanceOfForces` activation are **GAPs the brief mislabels "already built"** and must be constructed, not rewired.

---

## THE 3 RISKIEST CORRECTNESS TRAPS (flag at launch, resolve before the units that touch them)

**TRAP 1 — The valve has no attach point (Recon 4 §7, the single most consequential finding).** §2.5's central law `Agitation→Organization conversion ×= (1 − v·H)` has **nothing to multiply**. `organization` (the `P(S|R)` numerator) is a static seeded class attribute; the only dynamic write is a *reset-to-zero* on dispossession (`territory.py:370`). Agitation routes to ternary *consciousness* (`route_agitation_to_ternary`, `consciousness_routing.py:288`), never to Organization. There is no "conversion efficiency" scalar in existence. Building the valve is **new production code + a modeling ruling**, not a wire: either (a) redefine the valve target as `route_agitation_to_ternary`'s revolutionary channel (conflates class_consciousness with Organization — a real theory decision worth a ruling), or (b) build a genuine Agitation→Organization pathway from scratch. A unit that "wires the valve" onto the existing SOLIDARITY-edge indirection would silently *not model the thesis*. **Owned by U8; L-VALVE must bind the real conversion quantity.**

**TRAP 2 — `political_form` shadow-first is structurally barred from ever being principal (Recon 1, Recon 5).** §2.6's headline payoff — "the electoral question becomes the principal contradiction of the era" — requires `political_form` to *lead* the principal scorer. But `_principal_key` (`opposition.py:580-591`) filters `draft.key not in self._shadow_keys`: a shadow opposition **can never be principal**. Every catalog addition since `price_value` shipped shadow-first (8-for-8). Following that discipline by default makes §2.6 **unreachable** — the program ships the opposition and the payoff never fires, undetected, until someone notices the scorer never surfaces it. **Owned by U3; needs an explicit BD ruling at launch:** canonical-from-birth (breaks the 8-for-8 discipline; requires a byte-safety proof that a canonical measure returning `NoDataSentinel` in org-less scenarios doesn't perturb principal selection on the 6 baselines) **or** shadow-first *plus a declared ADR078-style promotion ceremony* scheduled into U10.

**TRAP 3 — Byte-safety is NOT the free "org-less no-op" the brief claims for the class-touching systems (Recon 3, Recon 5 §1).** The DoctrineSystem precedent (iterate empty `ORGANIZATION` query → zero writes, zero RNG draw) holds *only* for systems reading exclusively org-keyed state. But AllegianceSystem writes an **H field over SocialClass nodes** and the valve multiplies a **SocialClass-node conversion quantity** — and SocialClass nodes exist in all 6 baselines. Byte-identity then hinges on a proof chain: no parties ⟹ `H=0` (L-HOPE-MATERIAL) ⟹ valve multiplier *exactly* `1.0` ⟹ **and the code must not write the attribute at all** (stamping `hope=0.0` on a class node that never carried it changes the graph serialization/hash even though the value is neutral). Same hazard for `political_form`'s class-node balance measure: it must return `NoDataSentinel`, never `0.0`. A naive unconditional `node["hope"] = 0.0` drifts all 6 baselines. **Every class-node write in U3/U8 must sit behind an `if parties_exist` / absence-sentinel early return, and each of U3/U8/U9/U10 runs `qa:regression` byte-identical as a unit gate.**

---

## UNIT TRAIN

### U1 — Program keel + `politics:` defines namespace (ADR127)
- **(a)** Charter ADR127 + claim block ADR127–ADR140; `ai/state.yaml` program entry; retire the untracked `ADVERSARY-TRAIN-ACTIVE.md` inbox marker, add `POLITICAL-SUPERSTRUCTURE` program marker. Build the `politics:` defines category (§5.3) with A6 tiers declared at birth: Θ_data (`cycle_ticks{federal,state,local}`, electoral rules, base turnout by class), Θ_theory (`capital_tolerance`, `phi_social_share`, valve sign, funding-identity invariant), Θ_feel (`valve_strength`, `hope_spike_gain`, `disillusion_window_ticks`, `betrayal_threshold`, `office_capture_rate`, `split_asset_retention`, `sect_isolation_rate`, `boycott_conversion`, `popular_front_trigger`, `donor_platform_weight`, `suppression_cost_weight`, `host_threat_threshold`, `legitimacy_backfire_threshold`).
- **(b)** `src/babylon/config/defines/politics.py` (new, frozen BaseModel w/ `@model_validator` enforcing `no θ holds SW > t + Φ_slice` — Θ_theory anti-list row); `config/defines/__init__.py` (+import/+`__all__`); `config/defines/_assembler.py` (docstring bullet + Field decl + `_from_yaml_dict` line); regenerate `src/babylon/data/defines.yaml` via `tools/generate_defines_config.py`; `ai/decisions/ADR127_*.yaml` + `index.yaml`.
- **(c)** `tests/unit/config/test_constants_sync.py` (roundtrip/`--check` sync-guard — extends automatically); new `tests/unit/config/test_politics_defines.py` (validator invariants incl. the mint-value ban).
- **(d)** Byte-safe by construction: a new defines category read by *no system*. `qa:regression` untouched.
- **(e)** None (root).

### U2 — Event vocabulary (ADR128)
- **(a)** 13 new EventTypes + typed `SimulationEvent` subclasses + `EVENT_BUILDERS` entries + severity derivation: `ELECTION_HELD`, `GOVERNMENT_FORMED`, `POLICY_ENACTED/STRUCK/PREEMPTED`, `CAPITAL_STRIKE`, `DELIVERY_GAP_CROSSED`, `HOPE_SPIKE`, `DISILLUSION_WINDOW_OPEN`, `LEGITIMATION_REFRESH`, `ELECTIONS_SUSPENDED`, `POPULAR_FRONT_CALLED`, `LINE_STRUGGLE_SPLIT`.
- **(b)** `models/enums/events.py` (append, spec-tagged comments); `models/events/politics_payloads.py` (new subclasses); `engine/event_builders.py` (`_BUILDERS` entries, no `.get()` fallback — loud). No publisher yet.
- **(c)** Count-pin bumps: `tests/unit/models/test_enums.py:334` (84→97), `tests/unit/topology/test_phase_transition.py:45` (dup 84→97), stale comments in `test_chronicle_adapter.py:162,170,211`; `test_event_builders.py:31` coverage (64→77); `test_event_severity_single_source.py` — **every new type must resolve through `resolve_severity`** (derived, never hand-tiered) or reds.
- **(d)** Byte-safe: no system publishes any new type yet; event enum growth doesn't touch tick math.
- **(e)** None (parallel to U1).

### U3 — `political_form` opposition, shadow-first, inert measure (ADR129 — **index assigned at THIS commit**)
- **(a)** Catalog entry #19 `political_form`: poles `self_organization ⟷ representation`; unity/gap/balance per §2.6 (balance = signed share of class political labor-hours per channel, I-FRESH, no accumulator); couplings `wage feeds political_form`, `political_form constrains atomization`, `imperial transforms political_form`. **Resolve TRAP 2 ruling first.** Measure returns `NoDataSentinel` when no party/organ political-labor flows exist (the org-less case). Alphabetical index between `national` and `price_value`.
- **(b)** `domain/dialectics/instances/catalog.py` (`build_default_registry` binding @ ~:706); measure fn in same module or `domain/dialectics/instances/measures/political_form.py`; `ai/decisions/ADR129_*.yaml`.
- **(c)** `tests/unit/dialectics/test_catalog.py:28` `test_eighteen_oppositions_bound`→`test_nineteen_...` (19-tuple), `:52` `test_shadow_bindings` (if shadow ruling). New test: measure returns `NoDataSentinel` on all 6 qa:regression WorldStates. **Unit gate: `qa:regression` byte-identical** (proves TRAP 3 for the dialectics path — the measure must not perturb the registry aggregate on org-less states).
- **(d)** Shadow (or canonical-with-absence-sentinel): the ContradictionField/registry aggregate is unchanged on the 6 baselines because the measure yields absence, not `0.0`. Byte-identity is a *proved unit gate here*, not assumed.
- **(e)** ← none for the binding; the *live balance measure* stays inert until U5 seeds political-labor flows (that's the byte-safety mechanism, not a defect).

### U4 — OrganizationComponent carried task (org entity fill-in) (ADR130)
- **(a)** Port the deprecated shim's capacity semantics into `Organization` as constrained types (`cohesion`, `cadre_level`/`cadre_count`, `opsec`, `coherence`, `reputation` — currently unconstrained graph-dict floats); verify `to_graph`/`from_graph` round-trip (the known `from_graph`-drops-non-core gotcha); port the 274-line contract suite to the entity; **delete** `models/components/organization.py` + its `warnings.catch_warnings()` suppress-dance test + the lazy `__getattr__` in `components/__init__.py`.
- **(b)** `models/entities/organization.py`; `models/world_state.py` (from_graph reconstruction); `models/components/organization.py` (DELETE); `components/__init__.py:44-63` (DELETE lazy-load); move/rewrite `tests/unit/models/components/test_organization.py` → `tests/unit/models/entities/test_organization_capacity.py`.
- **(c)** Ported 274-line contract suite (now against the entity); `check:vocabulary` (new constrained fields must be real declared fields — the attribute-shape sentinel).
- **(d)** Byte-safe: qa:regression seeds zero orgs; adding fields to `Organization` + reconstructing them in `from_graph` cannot change org-less scenario hashes. Unit gate confirms.
- **(e)** None; but **U5/U11 depend on this** (they read/write the new typed capacity fields).

### U5 — Party seeding + relational producers (MEMBERSHIP + donor funding) (ADR131)
- **(a)** New engine scenario builder seeding NPC `PoliticalFaction` orgs per sovereign: two duopoly machines (LIBERAL_IMPERIAL / RESTORATIONIST-aligned) + latent socdem/fascist currents. **Build the missing producers:** MEMBERSHIP edges party→social_class (weighted — the consumer set at `composition.py`/`community.py`/`reactionary.py` walks an always-empty population today, Recon 2b); donor-dependence funding inflow Business/finance→PoliticalFaction (**no such machinery exists** — Recon 2e — new edge-type-or-attribute for `funding_share(d)`). This is the producer layer that makes political-labor flows exist ⟹ activates U3's balance measure and U7's platform inputs.
- **(b)** `engine/scenarios/electoral_fixture.py` (new; or extend `detroit_tri_county`); funding representation in `models/enums/topology.py` (reuse TRANSACTIONAL/INFLUENCES per §5.2 no-new-edge rule) + producer in the builder; `sentinels/vocabulary/registry.py` if new stamped attrs.
- **(c)** New scenario-builder tests (party/MEMBERSHIP/funding topology asserted); `check:vocabulary` (NodeType.* stamping, real declared fields).
- **(d)** Byte-safe: this is a **new** scenario, not one of the qa:regression 6. Zero edits to the 6 factories.
- **(e)** ← U4 (typed org capacity fields).

### U6 — spec-070 ELECTORAL faction seeding / NodeType.FACTION honest-empty repair (ADR132; ratifies ADR049)
- **(a)** Close the FACTION honest-empty gap (wiring-doctrine `W-C — OPEN`, `:113`): ratify ADR049 / land catalog v2.6.4 `election_lab` entry; replace the Detroit-only Census-fixture computation with the MIT Election Lab-driven pipeline; **re-key output from res-7 H3 to county FIPS** (post-Amendment-U substrate mismatch, Recon 2d); fix the `seed_sovereigns.json` LODES-vs-H3 ID-namespace bug (100%-`SOV_EXTERIOR_NULL`); port the seed from the disabled legacy web bridge into an `engine.scenarios` builder so `NodeType.FACTION` nodes exist headless. Feeds `weimar` + RESTORATIONIST capture + `FactionInfluenceSystem`/`FASCIST_RECRUITMENT` capture arm (inert today, Recon 4 §8). Shares the RED_OGV repair umbrella.
- **(b)** `data/game/balkanization/compute_seed_influences.py`; `data/game/balkanization/seed_sovereigns.json`; `.specify/memory/data-catalog.yaml` (`election_lab`); `specs/070-balkanization/tasks.md` (T112/T118 → checked); `engine/scenarios/` builder integration; `ai/decisions/ADR049_*.yaml` (proposed→accepted), `ADR132_*.yaml`.
- **(c)** `projection/faction.py` fixture-harvest test now non-empty on the electoral scenario; balkanization seed round-trip tests; CI-no-drive-parquet discipline honored (deterministic artifact).
- **(d)** Byte-safe: FACTION nodes land only in electoral/balkanization scenarios, none of the qa:regression 6.
- **(e)** Independent of U4/U5 (disjoint entity family — `BalkanizationFaction` is NOT `PoliticalFaction`, Recon 2d); **U10/U12 depend on it** (weimar routing, RESTORATIONIST capture).

### U7 — Pure politics kernel: `formulas/politics.py` + `domain/politics/` (ADR133)
- **(a)** All pure functions, zero system wiring: `platform(p)` (base-composition + donor-weighted + inertia, II.2 never stored); `fit`, allegiance-drift Θ-projection (align/media/contact/betrayal terms); **H field** `H(c)=Σ allegiance·viability·max(0,ΔP(S|A))` (counterfactual sigmoid eval — reuse `formulas/survival_calculus.py`); **valve law** `(1−v·H)`; funding identity `SW_deliverable/delivery_ratio/delivery_gap`; turnout; L-PRZ closed form.
- **(b)** `formulas/politics.py` (pure, RST docstrings); `domain/politics/{platform,allegiance,hope,funding}.py`; decide registry membership (`engine/formula_registry.py` — if registered, bump `test_formula_registry.py:126` 24→N; **open kickoff item** per Recon 5).
- **(c)** `tests/unit/formulas/test_politics.py` + property laws provable in pure form now: **L-VALVE** (`∂conv/∂H ≤ 0`, monotone in `valve_strength`), **L-CEILING** arithmetic (`SW_delivered ≤ t + Φ_slice`), **L-HOPE-MATERIAL** (`H=0` ⟺ no platform improves counterfactual `P(S|A)`), **L-PRZ** (breadth↑ ⟹ mean base-alignment↓).
- **(d)** Byte-safe: pure module, no caller in any system.
- **(e)** ← U1 (defines).

### U8 — AllegianceSystem @17.42 + **the valve** (Agitation→Organization pathway) (ADR134)
- **(a)** **Resolve TRAP 1 ruling first.** Build the Agitation→Organization conversion pathway (new production code — none exists). AllegianceSystem reads class material + platforms + ISA_COMM influence + fascist drift (@17.4) → writes allegiance attrs + H field (guarded: **no write when no parties**, TRAP 3). Apply valve `×(1−v·H)` to the *real* conversion quantity. `HOPE_SPIKE` event.
- **(b)** `engine/systems/allegiance.py` (new, `position=17.42`, CONSEQUENCE); the conversion pathway (likely `engine/systems/ideology.py` coupling or a new class-node accrual — **NOT `ooda.py`**); `simulation_engine.py:324` (+1 class); wiring-doctrine row (W-C/W-P, ships its sentinel).
- **(c)** `test_system_order.py` (30→31, `expected_order` insert, `_EXPECTED_CONSEQUENCE`, contiguity); `test_gate_coverage.py:24` + `test_all_thirty_systems_present` (→31); **CoverageGap row** in `regression_scenarios.py` (copy OODA/Doctrine idiom verbatim, remediation = electoral goldens). **Unit gate: `qa:regression` byte-identical** — proves H=0 no-op AND valve `×1.0` genuine no-op. L-VALVE now bound to the live quantity.
- **(d)** Org-less ⟹ H=0 (L-HOPE-MATERIAL) ⟹ valve exactly 1.0 ⟹ no class-node write (guarded). Proved, not asserted.
- **(e)** ← U7 (kernel), U5 (parties/platform inputs), U4 (org capacity).

### U9 — PolicySystem @17.47 + LEGISLATE resolver + the reform ceiling (ADR135)
- **(a)** LEGISLATE's missing resolver (`resolve_legislate` does not exist today — Recon 2f): agenda queue (graph register); fiscal check against live `s=p+i+r+t` (`domain/economics/distribution/`) + Φ inflow + **`debt_service` (BUILD — no such quantity exists, `DebtAccumulation` never constructed, Recon 4 §3)**; **capital veto gauntlet** — investment-strike (**WIRE the equalization operator `Δc=α(r−r̄)c`, currently zero production callers, Recon 4 §4** — the brief's "already built" is false), bond discipline (endogenous interest — live), judicial strike-down (**activate `InternalBalanceOfForces` — dead-until-wired, Recon 4 §10** — needs Institution nodes), federal preemption (ADMINISTERS DAG); overlay writes (wage_floor/social_wage/labor_law/police_budget/border_regime/war_posture) effective **next tick**; delivery_gap. `POLICY_ENACTED/STRUCK/PREEMPTED`, `CAPITAL_STRIKE`. `creates_value=False`.
- **(b)** `engine/systems/policy.py` (new, `position=17.47`); `ooda/state_ai/legislate_effects.py` is the *security-capacity* consumer — do **not** overload it; new resolver in `engine/actions/` or `domain/politics/policy.py`; equalization wire into engine; `debt_service` in `domain/economics/`; `simulation_engine.py`; **I-ORD sentinel** (the base-never-reads-superstructure-this-tick direction check — a Program 25 deliverable, Recon 3, not a precedent) as this system's W-C dataflow row.
- **(c)** `test_system_order.py` (31→32); CoverageGap row; **L-CEILING A4 cross-check** (`creates_value=False`, `SW_delivered ≤ t+Φ_slice`); **Unit gate: `qa:regression` byte-identical** (no state apparatus + empty agenda ⟹ no-op).
- **(d)** Org-less/agenda-less ⟹ empty-query no-op (DoctrineSystem idiom, reads only agenda/state-apparatus-keyed state — NOT class fields).
- **(e)** ← U7, U8 (allegiance/delivery-gap feed), U4.

### U10 — ElectoralSystem @17.45 (clocked) — completes the ambient machine (ADR136)
- **(a)** Election clock per `JurisdictionLevel` (`cycle_ticks`, congress-clock idiom); turnout (base × allegiance concentration × H − suppression); seats/executive (FPTP terrain, claims-overlay districts, I.20 — substrate untouched); **FactionBalance perturbation** (W-C write into `state_apparatus_ai.py` field, bounded by `max_faction_shift_per_tick=0.05`; ooda reads it — no `ooda/` edit); `InternalBalanceOfForces` shift; legitimation refresh (`turnout×competitiveness`); H collapse + **T-7 routing** (bridges→radicalize / no-bridges→`fascist_pull`); repression-backfire coupling (`legitimacy_backfire_threshold` modulates `repression_backfire`, `struggle.py:315`); `ELECTIONS_SUSPENDED` (bonapartist). **If TRAP 2 ruled shadow-first, the declared `political_form` promotion ceremony lands here.** ξ_t only for recount-grade ties (congress precedent).
- **(b)** `engine/systems/electoral.py` (new, `position=17.45`); reads `domain/institution/balance.py`, `domain/bifurcation/legitimation.py`; `simulation_engine.py`; wiring rows (W-C ×3).
- **(c)** `test_system_order.py` (32→33, final); CoverageGap row; **L-SUSPEND** (legitimation floor + bonapartist ⟹ suspension); **Unit gate: `qa:regression` byte-identical** (no parties ⟹ clock never fires, ξ_t never drawn).
- **(d)** Org-less ⟹ zero party orgs ⟹ no election event ⟹ no FactionBalance write, no RNG draw. Pure empty-query no-op.
- **(e)** ← U8 (allegiance/H), U9 (delivery gaps → legitimation decay), U6 (FACTION nodes for weimar/routing), U5.

### U11 — The doctrine fork §3: derived verdict + five stances (ADR137)
- **(a)** Extend the `trap_condition` DSL vocabulary from the 3 `DoctrineTag` members to **measured practice** (`_resolve_tag` + `Mapping[DoctrineTag,float]` signature generalized to a typed measured-practice namespace — Recon 1: edge-portfolio composition, `co_optive_share`, office tenure, delivery dependence; do NOT fake pseudo-tags); re-found reformist trunk as **capability rewires** (verb target-sorts, edge types built, cadre-H coupling) not punitive static `tag_deltas`; CLASS_ANALYSIS decay-when-vetoed (Unit-6b pattern extended); five stances forked under `trade_unionism` (Abstention/Boycott, Debs class-struggle, Entryism w/ host-discipline projection, Independent Ballot Line, Governance Road); `liquidationism` → **detected absorbing state** (`SOLIDARITY_mass→0 ∧ co_optive_share>threshold ∧ class_character→PETTY_BOURGEOIS`); officeholder capture (org-level `institutional_pull`, ADR084 KeyFigure stays retired); line-changes-as-splits (existing DT-5 purge, `split_asset_retention`); **absorb `trap_detection.py`** liberal-trap factors → `political_form` measures, retire hardcoded thresholds (ruling 8, III.1). Verb parameter growth (Campaign(Election,mode), Mobilize(CANVASS), Negotiate(Coalition)) via `engine/actions/` resolvers.
- **(b)** `data/game/doctrine_tree_mvp.json` (content re-found); `models/{enums,entities}/doctrine.py`; `domain/doctrine/{mechanics,congress}.py` (DSL generalization); `engine/systems/doctrine.py`; `engine/actions/campaign.py` (new resolver, **not `ooda.py`/tui/cli/play**); `engine/trap_detection.py` (factors migrated, thresholds retired); `ai/decisions/ADR137_*.yaml`. *ActionSpec registry entries (`game/actions/registry.py`) are the interface-train seam — cite as blocking-dependency in the Verb sentinel row rather than authoring interface files.*
- **(c)** Doctrine golden-chain hash tests; DSL-vocabulary tests; `check:vocabulary`; **Unit gate: `qa:regression` byte-identical** (DoctrineSystem still no-op on org-less 6).
- **(d)** Org-less ⟹ empty ORGANIZATION query ⟹ DoctrineSystem no-op preserved (its own module-docstring contract).
- **(e)** ← U3 (`political_form` measures), U5 (party orgs), U8–U10 (ambient machine the stances respond to).

### U12 — Conjunctures + governance endgame + the Third-Worldist ledger §3.4/§3.5/§4 (ADR139)
- **(a)** Popular-front conjuncture generation (`fascist_consolidation` axis crossing `popular_front_trigger` ⟹ forced choice for *every* line incl. abstentionists, `POPULAR_FRONT_CALLED`); governance endgame SYRIZA fork (capitulate/rupture arms; betrayal integral `b(c)=Σgap` crossing `betrayal_threshold`, `DISILLUSION_WINDOW_OPEN`; reads `DUAL_POWER_ACTIVE` @17.5 — Allende geometry vs synthesis window; `LINE_STRUGGLE_SPLIT`); periphery mirror (low-Φ sovereigns, immediate ceiling); L-RECEIPTS provenance (BoundaryFlowRegister path from every delivered social-wage unit to source EXPLOITATION flow — *no flow without a row*).
- **(b)** `domain/politics/{conjuncture,governance_endgame}.py`; `engine/systems/{electoral,policy}.py` (hook points); feeds `engine/observers/endgame_detector.py` axes (no new win condition, I.11). Narration via `intelligence` only (election-night captions, touches no register).
- **(c)** **L-RECEIPTS** test (registry-path completeness); DUAL_POWER read-path test; **Unit gate: `qa:regression` byte-identical**.
- **(d)** Org-less ⟹ no governing party, no axis crossing from electoral inputs ⟹ no-op.
- **(e)** ← U10, U11, U6 (FACTION routing).

### U13 — The five goldens + property-law estate + **golden ceremony** (ADR140)
- **(a)** Register `mitterrand`/`weimar`/`debs`/`syriza`/`bernie_valve` as scenarios (§5.5); the 3 new systems now earn **real `SystemEvidence`** here, upgrading their U8–U10 CoverageGap rows; ship the full property-law estate as behavioral contracts (L-VALVE, L-CEILING, L-PRZ, L-HOPE-MATERIAL, L-RECEIPTS, L-SUSPEND). **Mint baselines via the declared ceremony** — `test(baselines): electoral goldens` subject, drift table via `tools/generate_ceremony_message.py`, `Baselines: blessed(electoral-goldens)` trailer.
- **(b)** `tools/regression_scenarios.py` (`SCENARIOS` + `SCENARIO_COVERAGE_DATA` literals + `PENDING_CEREMONY` until minted); scenario factories in `engine/scenarios/`; `tests/baselines/{mitterrand,weimar,debs,syriza,bernie_valve}.json` + `dense/*.csv`; `tests/unit/engine/systems/test_electoral_goldens.py`.
- **(c)** `qa:regression` now green on **11** scenarios; `check:gate-coverage-truth`; dense-golden gate; L-* property suite. **The original 6 baselines are byte-identical throughout** (the 5 goldens are *separate* scenarios — the declared ceremony mints only the new rows, never touches the 6).
- **(d)** This is the **declared golden ceremony** — the single sanctioned point where new baselines enter. No pre-ceremony drift anywhere in U1–U12 (each carried its own byte-identity unit gate).
- **(e)** ← all prior (integration capstone).

---

## Dependency graph (compressed)

```
U1 ─┬─ U4 ── U5 ─┬───────────── U8 ─── U9 ─── U10 ─┬─ U11 ─ U12 ─ U13
U2 ─┤            U6 ────────────────────────────────┘
U3 ─┴─ U7 ───────────────────────┘
```

- **Parallelizable (read-only/pure, no shared write):** U1‖U2‖U3 (foundations), U6 alongside U4/U5, U7 after U1.
- **Strict serial spine (byte-identity gate each):** U8→U9→U10 (system-order literal + count pins mutate on each; single-flight to avoid baseline collisions), then U11→U12→U13.
- **Ambient machine (§2) fully lands at U10, before the doctrine fork (§3) opens at U11** — as required.
- **Five goldens + ceremony are U13** (late unit) — as required.

## Wiring-doctrine (ADR109) rows shipped per unit
Each system unit ships its motion-class sentinel row, not a bare import: U8 = W-C+W-P (allegiance/H producer) + the valve W-C into Agitation→Org; U9 = W-C+W-A4 (LEGISLATE overlays, `creates_value=False` conservation closure, L-CEILING) + the **I-ORD** direction sentinel (base-never-reads-superstructure-this-tick — new deliverable); U10 = W-C ×3 (FactionBalance, InternalBalanceOfForces, legitimation); U11's Verb-parameter growth cites the **interface-train ActionSpec registry as its blocking dependency** rather than authoring `ooda.py`/tui/cli/play files. `political_form` catalog index (19) is bound at U3's implementation commit, never pre-pinned (three lanes — `metabolic`, `national`-extension, `political_form` — race the append slot; ADR126 ruling 2).
