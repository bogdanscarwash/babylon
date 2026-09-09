# Program Brief — The Morphism Compact

**For:** Claude Code, in the `babylon` repo, branching from `dev`.
**Verified against:** `dev` @ `744f865` (2026-07-17). Every file:line below was read on that commit — but **re-verify before relying on any of it**; if a claim here contradicts the code, the code wins and you say so loudly.
**Working title.** Program / spec / ADR numbers are allocated at plan time per `ai/decisions/` convention. Latest on `dev`: ADR078, `specs/115-market-scissors`.

---

## 0. What this program is

Two rulings from an owner design session, one program, two rounds.

**Ruling 1 — the Morphism Compact.** A player verb is not a kind of dialectic; it is a **morphism on the dialectical field**. Objects don't compose — arrows do. Every canonical player verb therefore declares its action on exactly two channels, both of which already exist in the codebase:

1. **The material channel (indirect, unsigned).** The resolver mutates graph state — wealth, infrastructure, solidarity edges, consciousness, presence. At `ContradictionSystem` the `OppositionRegistry` re-measures each opposition's **gap** from `GraphInputs`. The verb's effect on the gap is *mediated entirely by material relations*. This is the Aleksandrov Test in executable form. **Verbs never write gaps.**

2. **The stance channel (direct, signed, audited).** The resolver publishes a `StanceIntervention(target_key, delta_balance, source)` onto the `opposition_interventions` graph attr. At `ContradictionSystem` it is drained consumed-once and shoves the **balance** — which pole leads — never the gap, never `is_principal`.

**Ruling 2 — flow vs stock.** A strike is a *flow interruption*; an attack is a *stock destruction*. A strike withholds labour-power while constant capital sits intact: per-tick, actor-coupled (arrest the strikers and it ends), sustained only as long as the strikers out-last capital's ability to wait. An attack writes to the Ledger: destroy constant capital and the org can be liquidated next tick while the plant stays burned. **Actor-decoupling is the formal content of "the damage is still done."** Both verbs leave permanent residue — they just write it to different layers of the Trinity: the strike's permanence is *relational* (solidarity edges forged in struggle, consciousness, edge-mode transitions — Luxemburg's mass strike as school of war), the attack's is *material*.

---

## 1. Verified state of the code (read these first; do not trust this summary)

**The arrow-side scaffold already exists.** This program is mostly *wiring*, not building.

| Fact | Location |
|---|---|
| `VERB_RESOLVERS: dict[ActionType, VerbResolver]` — 9 resolvers, uniform signature `(action, org_attrs, graph, services) -> ActionResult` | `src/babylon/engine/actions/__init__.py:58` |
| One module per verb: `aid, attack, campaign, educate, investigate, mobilize, move, negotiate, reproduce` | `src/babylon/engine/actions/` |
| `VERB_TO_ACTION_TYPE` — the sole verb-string → ActionType translation; pinned equal to resolver keys | `web/game/engine_bridge.py:148` |
| `StanceIntervention` (frozen: `target_key`, `delta_balance`, `source`) + `apply_interventions` | `src/babylon/domain/dialectics/core/coupling.py:144` |
| `OPPOSITION_INTERVENTIONS_ATTR = "opposition_interventions"`; `_apply_interventions` drains **consumed-once** | `src/babylon/engine/systems/contradiction.py:82`, `:208` |
| The **only** current publisher — the reference pattern to follow | `src/babylon/engine/systems/reactionary.py:175` (`_write_stance_intervention`) |
| Player dispatch call site | `src/babylon/engine/systems/ooda.py:250` |
| **`OODASystem()` @ 14 · `ContradictionSystem()` @ 18** | `src/babylon/engine/simulation_engine.py:397`, `:411` |
| 6 registry keys: `capital_labor`, `wage`, `tenancy`, `atomization`, `imperial`, `price_value` (+ pole names, `level_name`, `antagonistic`) | `src/babylon/domain/dialectics/instances/catalog.py:291-357` |
| Stance defines pattern (`gain` × `cap`) | `src/babylon/data/defines.yaml:884-885` |

**The load-bearing ordering fact:** OODA @14 runs *before* Contradiction @18, so a stance shove published by a verb resolver is consumed **in the same tick**. The materialist-causality order (Material Base → Action → Consequences) already places this seam correctly. Verify this yourself before designing around it.

**`apply_interventions` contract** (read the docstring): deltas sum per target, add to balance, clamp to `[-1,1]`, leading pole recomputed under zero-holds-previous inertia. Gap, rate, and `is_principal` are untouched — *"interventions never re-rank the principal contradiction; that is the registry's job."* This is why the two-channel split is honest rather than decorative.

**Baseline neutrality is free.** `tests/contract/verbs/test_baseline_neutrality.py` pins that no `src/babylon` module writes `player_actions` (`_ALLOWED_READER = "engine/systems/ooda.py"` only reads them). Canonical `qa:regression` scenarios carry no organization nodes — the same reason `DoctrineSystem` @14.7 is byte-safe (`simulation_engine.py:399-402`). **No re-baseline ceremony is expected for either round.** If your design would need one, stop and say so before writing code.

**The attack stub** (`src/babylon/engine/actions/attack.py`, 74 lines):
- `_ATTACK_SELF_HEAT_GAIN = 0.1` — a **hardcoded coefficient**, a live Constitution III.1 violation. Fix it into defines as part of Round 2.
- Infra decrement delegated to `src/babylon/ooda/layer3.py:140` `_propagate_infrastructure` at flat `defines.ooda.attack_infrastructure_delta = 0.1` (`defines.yaml:593`).
- Emits generic `EventType.ORGANIZATIONAL_ACTION`. `events.py:122` has only `INFRASTRUCTURE_CHANGE` — shared by BUILD *and* ATTACK. No circuit-moment granularity.
- No stance shove, no repair flow, no conservation sink, no propagation.

**The flow side already half-exists.** `mobilize` resolves `ActionType.PROTEST`; its docstring reads "Public demonstration / strike," and `defines.yaml:62` carries `mobilize.strike_value_disruption_factor: 0.1`. `ActionType.STRIKE` exists but is reachable by no player verb. Determine what's actually live before designing the flow/stock contrast — do not assume.

**Interlock — The Viable Game** (`docs/superpowers/specs/2026-07-17-viable-game-design.md`, today's HEAD). That program's spine wants `preview_action` costs/warnings rendered before submit (§4d.3), per-target expected deltas (§4d.4), and the event whitelist widened past 44/79 with POGROM/LOCKOUT/VIGILANTISM getting their own EventTypes (§4d.7). **This program supplies the payloads; the spine renders them. Build no UI here.** Coordinate the seam, don't cross it.

---

## 2. Round 1 — The stance channel (8 verbs)

Wire every canonical verb except `attack` into the stance channel.

**Deliverables**

- A shared publisher — one function, one place. `reactionary.py:175` currently owns a private copy of the append-to-graph-attr logic; factor the shared helper so there is exactly one writer implementation. Whether `reactionary.py` adopts it in this program or later is a scope call: propose, don't assume.
- A **`VerbStancePolicy`** declaration per resolver key — frozen Pydantic, exhaustive over `VERB_RESOLVERS`. It names: the opposition key(s) shoved, the defines path for the magnitude, and — for abstainers — an explicit `abstention_reason`. **Abstention is declared data, not absence**, so the contract can assert exhaustiveness and no verb can silently drift out of the compact.
- A `verb_stance:` defines section — one gain coefficient per shoving verb, `cap` where needed. **No magic numbers** (III.1). Follow the `stance_intervention_gain` / `stance_intervention_cap` shape at `defines.yaml:884`.
- Each resolver, on success, computes its shove and publishes.

**The mapping is a proposal, not a ruling.** Take it to `/speckit.clarify`. The owner decides.

| Verb | ActionType | Proposed shove | Reasoning |
|---|---|---|---|
| educate | EDUCATE | `capital_labor` → labour pole | consciousness is the classic shove on who leads |
| reproduce | RECRUIT | `atomization` → unified pole | recruitment is literally de-atomization |
| negotiate | PROPOSE_ALLIANCE | `atomization` → unified pole | inter-org edges are class-level unification |
| aid | PROVIDE_SERVICE | `tenancy` → tenant pole | mutual aid cuts rent's command over reproduction |
| mobilize | PROTEST | `capital_labor` → labour pole | the flow interruption; magnitude scaled by turnout |
| campaign | PROPAGANDIZE | `capital_labor` → labour pole (weaker than educate) | broadcast without organisation is thin |
| move | MOVE | **abstains** | repositioning is material only — presence edges, no dialectical shove |
| investigate | MAP_NETWORK | **abstains** | epistemic: it mutates the *player's information state*, not the dialectical state |

**On investigate — do not force it ontic.** It is a morphism on the fog layer, not the field. Its eventual home is ADR051's deferred `observation_relativity` (frame-dependent `observe()`; "no frame ships today — a single frame computes nothing"). Ship the declared abstention with that reason recorded; do not build a frame.

**Signs may be computed, not constant.** Some verbs' sign is conditional on graph state (see Round 2). The policy declares *which opposition and where the magnitude lives*; the resolver computes the sign; a contract test pins the sign law. Say plainly in the ADR that the sign law lives in code, not defines, and why.

**What Round 1 earns** (III.10 earn-its-keep — no construct ships as vocabulary):
- a **law**: every non-abstaining verb publishes ≥1 intervention on success; abstainers publish zero; the set of policies is exhaustive over the resolver registry.
- a **computation**: `preview_action` (`engine_bridge.py:5644`) derives the predicted shove *from the same declaration the resolver executes*, so preview cannot drift from effect. This is the payoff the Viable Game spine consumes.
- a **prediction**: sign-predictability per verb, pinned as an Amendment-Q behavioral contract.

---

## 3. Round 2 — Attack as stock destruction

Round 1 must be merged before Round 2 starts. One track at a time.

**3a. Circuit-moment taxonomy.** Type the attack by which moment of the circuit of value it strikes — production/extraction, circulation, realisation — each with different visibility (heat) and different propagation. **First, settle whether the epoch-3 vision docs (`ai/epochs/epoch3/kinetic-warfare.yaml`, `state-attention-economy.yaml`, if present) are still canon or archaeology superseded by `ai/spec-prompts/enemy-ai/036-state-verb-system.md`.** This is an owner question — raise it at `/speckit.clarify`, do not guess. It decides whether the visibility grading is ratified design or a dead draft.

**3b. Propagation, honestly scoped.** A production attack decrements industry capacity at a hex; the Leontief pipeline (`src/babylon/domain/economics/tensor_hierarchy/leontief_rent/`, ADR035/046) can propagate removed capacity deterministically through interindustry linkages — a chokepoint sector cascades instead of "50 damage to a node." **Circulation is the trap:** spec-101 D5 gates `TRADE_EDGE` pending 098-LODES, so a circulation attack's propagation may be inert today. Verify what is live. If a moment's propagation cannot execute, **disclose it as a gap rather than shipping dead functionality** — the ADR056 precedent (`trade_multiplier` "disclosed as FR-102-8 rather than shipped as dead functionality") is the governing pattern.

**3c. Damage register + repair flow.** Damage is **persistent-until-repaired at someone's expense**, not eternal. It decays only through allocated investment (state DEVELOP, or an accumulation flow); `repair_rate` from investment/damage. **Without a repair flow you have rebuilt the add-only tension ratchet ADR051 just killed, in a new costume** (VIII.11). The register must be written *only* by the attack resolver, and any system that reads it must early-return on an empty register — that is what keeps canonical byte-identical.

**3d. Devalorisation is double-edged — model it, don't sand it off.** Destroying constant capital is one of capitalism's own crisis-resolution mechanisms (Grossman; the war-destruction dynamic): destroy `c`, and once capital rebuilds at lower value, `r = s/(c+v)` can *recover*. The state verb spec already contains the NEGLECT → blight → INVEST gentrification circuit — destruction as accumulation opportunity. **Attack must not be a monotonic hurt-capital button.** Its economic damage is real but recoverable; its durable payload is political — diverted budget, forced repression spending legitimacy, contradiction intensification.

**3e. Attack's stance shove is computed, and its sign is the Jackson bifurcation.** Attack-generated agitation routes by SOLIDARITY edge presence (Constitution I.4). Attack without a prior base — no Educate, no Aid, no solidarity edges — shoves *toward the capital/reactionary pole*, the same direction `FascistFactionSystem` pushes (`reactionary.py:180`, `delta_balance` positive → capital pole). **This is the mechanic, not a bug: the engine already knows why adventurism fails.** Pin it as a behavioral contract in both directions.

**3f. Conservation.** `qa:regression` pins `total_v` Δ=0.000%. Destroyed value must be a **sanctioned explicit sink** (a `destroyed_value` ledger term), or every attack breaks the audit (`ConservationAuditRow`, `src/babylon/persistence/audit_models.py:36`). The sink term is 0.0 when no attack has occurred — which is every canonical run. Verify against a real Postgres test DB; **do not infer the audit's behaviour from reading code** (ADR056's determinism-proof correction is the cautionary precedent: the planned proof method was wrong and only an empirical run caught it).

**3g. Housekeeping inside Round 2.** Fix the `_ATTACK_SELF_HEAT_GAIN` III.1 violation into defines. Give attack its own EventType(s) per circuit moment with severity and geographic anchor — coordinate with the Viable Game spine's §4d.7 whitelist widening rather than duplicating it.

---

## 4. Constraints (non-negotiable)

- **No MVP scoping.** The full spec is the minimum viable plan. Rounds are dependency order, not scope cuts. Never propose a Phase-1 cut of a specified feature.
- **Constitution.** Every construct traces to a material relation. AI narrates, never adjudicates. No new primitives without an amendment. Verbs never write gaps; the registry owns measurement and principal-ranking.
- **Determinism.** Non-determinism is a bug, full stop. Interventions sum per target — if two verbs shove the same key in one tick, resolution order must be canonical or the hash breaks. Address this explicitly.
- **III.1.** Every coefficient in `defines.yaml`. Zero magic numbers, including the one currently in `attack.py`.
- **TDD + Amendment Q.** Red before green per unit. Behavioral contracts pin what the system *does*.
- **Process.** Branch from `dev`, conventional commits, `mise run check` green per unit, `mise run qa:e2e-regression` before merge. Mermaid for diagrams, never ASCII.
- **Loud Failure (III.11).** Never fabricate data the engine didn't produce. If a propagation path is inert, disclose it; if a criterion can't be met, record it as an open item rather than faking it (ADR051's `rebaseline_acceptance.c_spatial_stddev` is the precedent for an honest FAIL-AS-WRITTEN).

---

## 5. Surface at `/speckit.clarify` — do not guess

1. **The per-verb stance mapping** (§2 table) — owner ruling required on every row, especially `campaign` and `aid`.
2. **Does Article V need amending?** The compact adds "and declares its stance policy on the opposition registry" to "every verb maps to a graph operation." `StanceIntervention` is not a new primitive, so this may be an ADR-only change — but Article V's text is constitutional and the call is the owner's, not yours.
3. **Epoch-3 canon status** (§3a) — canon or archaeology?
4. **Does `reactionary.py` adopt the shared publisher in Round 1, or later?** Touching it changes a shipped system.
5. **Circulation-moment scope** (§3b) — ship disclosed-inert, or scope Round 2 to live propagation only?

One question at a time. Do not batch.

---

## 6. Anti-scope

Do **not** build: any UI (the Viable Game spine owns the screens); the observation frame / `observe()` `frame` param (ADR051-deferred); NPC or state-AI *selection* of verbs (Wave-2/3); a `Contradiction` primitive (it exists — it's the registry); the value→price-of-production transformation (ADR051-deferred); the TopologyMonitor wiring (ADR051-deferred, zero production call sites).

Do **not** reuse `ScheduledBlocShock` (ADR056) for attack damage. It is exogenous scenario data, bloc-Φ-scoped, and explicitly non-agentic (R-AMEND: "blocs never decide to shock themselves"). Its *pattern* — a deterministic register applied as a level-set map in the tick loop — is worth studying. Its identity is not yours to borrow.

---

## 7. First move

Read, in order: `src/babylon/engine/actions/__init__.py`, `src/babylon/domain/dialectics/core/coupling.py`, `src/babylon/engine/systems/contradiction.py`, `src/babylon/engine/systems/reactionary.py`, `src/babylon/engine/actions/attack.py`, `tests/contract/verbs/`, `ai/decisions/ADR051_lawverian_dialectics_refactor.yaml`.

Then report, before writing any spec: which claims in §1 hold, which are stale, and what the ordering seam at @14 → @18 actually does with an intervention published from a resolver. If any load-bearing fact here is wrong, that finding is the first deliverable.
