# The Wiring Completeness Doctrine — full detail

Merged from two architect passes (strike exemplar + ledger/law), re-verified against the
tree on 2026-07-29. Every claim carries `file:line`. Corrections to the architect inputs
are marked ⚠ and are authoritative over the raw architect text reproduced in Part IV.

---

## Part 0 — Verification log (this pass)

| Claim under test | Verdict | Evidence |
|---|---|---|
| No player verb mints SOLIDARITY | **FALSE — 3 producers** | `apply_mass_work_solidarity` (`engine/actions/_mass_work.py:62`) called from `educate.py:72`, `aid.py:67`, `campaign.py:120`. The ledger architect said AID only; the strike architect's count of three is correct. |
| `sentinels/wiring/` exists | **FALSE — absent** | `ls src/babylon/sentinels/` → 31 packages, no `wiring`. ADR109 §7.1 chartered it 2026-07-21. Row **G1**. |
| Bourgeoisie matrix has no concession-under-pressure arm | **TRUE** | `formulas/dynamic_balance.py:82-118`. BRIBERY requires `pool_ratio >= high_threshold AND aggregate_tension < bribery_tension_threshold`. Raising tension can only reach IRON_FIST / AUSTERITY / NO_CHANGE. |
| `state_violence_index` has no production writer | **TRUE** | Readers: `electoral.py:413`, `endgame_detector.py:545`, `conjuncture.py:58-59,106`. Zero writers. `endgame_detector.py:542` comment says "written by spec-039" — it never landed. ⟹ `violence_gate ≡ 0.0`, conjuncture capped at 2/3. |
| `org.budget` has no system writer | **TRUE** | `rg budget src/babylon/engine/systems/*.py` → two docstring mentions only (`transport.py:15`, `doctrine.py:11`). Decremented by `aid.py:101`, `reproduce.py:79`. One-way ratchet. |
| `social_class.organization` has no player-side producer | **TRUE** | Only runtime write is `territory.py:370` `organization=0.0` (PENAL_COLONY suppression, doc'd `:321`). Readers `survival.py:68,149`, `struggle.py:560`, `allegiance.py:470`. The engine only DESTROYS it. |
| `ActionType.STRIKE` has no resolver | **TRUE** | Sole production reference: eligibility row `ooda/action_eligibility.py:56-58`. Absent from `VERB_RESOLVERS` (`engine/actions/__init__.py:58-68`). `base_cost_strike` orphan at `config/defines/ooda.py:302`. |
| Five `EdgeType` members wholly dead | **TRUE** | `EdgeType.RECRUITMENT / EMPLOYMENT / TARGETS / OWNED_BY / JURISDICTION` → **zero** `EdgeType.<M>` references tree-wide beyond their own declarations (`models/enums/topology.py:110-121`). |
| BSL has no `update-edge` verb | **TRUE** | `docs/reference/bsl-language.rst:645-652`: seven structural verbs; `add-edge` payload is `:strength` only. **Forces the strike's state onto a node field.** |
| `render_epilogue` has zero production callers | **TRUE** | `projection/vault/render_epilogue.py:60`; refs are its own `__all__` (`:35`) + a docstring xref (`epilogues.py:16`). |
| `is_goal` has no production consumer | **TRUE** | Sole non-test reader is the shape invariant `domain/doctrine/validation.py:167`. Declared `models/entities/doctrine.py:160`. |
| Vocabulary family is "3 rules" (per CLAUDE.md) | ⚠ **FALSE — 6** | `sentinels/vocabulary/checks.py:3-4` "Six gating rules", `:483` "All six rules gate". **CLAUDE.md doc drift to fix.** |
| FRAGMENTED_COLLAPSE crisis operand unsatisfiable | **TRUE, and sharper** | `endgame_detector.py:610-611` compares against **raw strings** `{"insurgent","occupation","emergency"}`. `SovereigntyType.INSURGENT/OCCUPATION/EMERGENCY` have **zero** references anywhere. ⟹ `crisis_gate ≡ 0.0`. |
| SECESSIONIST never stamped in production | ⚠ **FALSE — stamped as a raw string** | `collapse_transition.py:234` `sovereignty_type="secessionist"`, `:162` `="provisional"` — bare strings, bypassing the enum. This is a *second* defect (raw-string stamping, the `balkanization_faction` shape) layered on D1. |
| Ledger row arithmetic "19 of 26 rows" | ⚠ **25 rows** | A1–A8 (8) + B1 (1) + C1–C5 (5) + D1–D5 (5) + E1 (1) + F1–F4 (4) + G1 (1) = **25**. |

**Net:** the defect is worse than the brief stated in three places (dead edge vocabulary,
`violence_gate ≡ 0`, raw-string sovereignty stamping) and better in one (SOLIDARITY already
has three player-side producers — a strike design must reckon with that wire, not duplicate it).

---

## Part I — The clause (as it should enter the Game Design Standard)

See the compressed section returned to the parent agent. Its normative content:

- **W.1 the construct triple** — writer / consumer / sentinel, with three sub-rules on
  "writer" (a fixture is not a writer; a build-time seed is not a runtime writer; a literal
  constant is not a writer) and one on "consumer" (a validator-only read is not a consumer).
- **W.2 the five dispositions** — WIRE / CHARTER / BLOCKED / RULED-ABSENT /
  RETIRE-WITH-RECORD. Silence is a red gate.
- **W.3 six enforcement rules** — the registry (W.3.1) plus five new checks
  (gate-operand writers, tested-enum emitters, fallback coverage, outcome-operand
  reachability, ledger completeness).
- **W.4/W.5 the ledger** — 25 rows, ~110 constructs, sequenced by P27 phase.

### W.0 — inherited constraints, not re-litigated

- **Article V closed at nine verbs.** Every verb-surface wire is *parameter growth* on a
  fixed generator — `sub_mode` / target-sort / edge-type authorization. Live precedents:
  `mobilize.py:137-154` (`sub_mode='canvass'`), `negotiate.py:37-40` (`mode='coalition'`),
  both gated via `_capability.py:54,64`. A tenth stem is a constitutional event
  (`ai/wiring-doctrine.md:84-89`).
- **The five doctrine levers**, and only these: mode authorization, edge-type
  authorization, cost/efficacy coefficient (**missing today** — §Lever 3), valve coupling,
  trap exposure. Verb-LEVEL gating is unconstitutional.
- **L-SPEND** — verbs spend, never mint; no resolver carries `creates_value`.
- **No imposed functional forms** (ADR172 ruling 5 / ADR173). A gap that seems to need a
  new curve is a W-𝔇 registration (measure over shipped fields, fresh per tick, no
  accumulator, UNPOSITIONED on absent inputs) or it escalates.
- **Fixture-vehicle doctrine** — outcomes are never asserted subjects. This is *why*
  W.3.5 must be a **static** proof: a dynamic test structurally cannot prove outcome
  reachability without asserting the outcome.

---

## Part II — The strike exemplar (mechanism-true, compressed)

### II.1 Surface

`Mobilize`, `params["sub_mode"]="strike"`, dispatched by `resolve_mobilize`
(`mobilize.py:102-183`) exactly as `canvass` is. `VERB_RESOLVERS` and
`VERB_TO_ACTION_TYPE` untouched — nine stems hold. `ActionType.STRIKE` stays resolver-less
and keeps failing loud; reviving it would require a `VERB_RESOLVERS` key, i.e. the tenth-verb
break `build.py:13-24` documents. It, its eligibility row (`action_eligibility.py:56-61`)
and `base_cost_strike` (`ooda.py:302`) are **RETIRE-WITH-RECORD candidates**, flagged not touched.

**Eligibility — material conditions only, no doctrine gate.** Four conjuncts, each with a
distinct loud `failure_reason`:

1. target is a `social_class`;
2. target stands in a live wage/extraction relation — ≥1 incoming `WAGES` (`production.py:227-244`)
   or ≥1 outgoing `EXPLOITATION` (`economic.py:268`). **This is the conspicuous absence closed at its root.**
3. the org has an organized base — `MEMBERSHIP` (`mobilize.py:214`) or `SOLIDARITY`
   (`_mass_work.py:107`) org→class. Makes Canvass/Educate/Aid the *prerequisite play*.
4. affordability against `org.budget`, using the `aid.py:90-98` insufficient-budget shape
   verbatim. `fund == 0.0` is legal — an unfunded strike is short by material consequence, not by penalty.

The basic stoppage is **ungated by doctrine**: withdrawal of labour is a capacity of
organized workers, not a doctrinal unlock. Sub-mode gating of the *basic* act is the
verb-gating error in miniature. Lever 1 is reserved for the general-strike escalation.

**Verb plate (W-P).** `preview_verb` (`preview.py:99-196`) today gives `mobilize` a flat
heuristic (`:168-171`) and knows nothing of sub-modes. The plate must show declared inputs
and the material fork, never a success roll: `withdrawal_declared`; `withdrawal_effective`
via the *same pure helper the resolver uses* (the `preview_consciousness_delta` parity
discipline, `:43-96`); **hold-out weeks** = `fund ÷ (population × subsistence_threshold ×
subsistence_multiplier)` (arithmetic off `economic.py:233-237`, `social_class.py:58,351`);
replaceability = the TENANCY territory's `reserve_ratio` (`reserve_army.py:75`); and the
employer's `pool_ratio` / `capital_labor` gap — **which zone of the decision matrix you are
in** (`dynamic_balance.py:46-51`), never a predicted outcome. `success_probability` is
removed or UNPOSITIONED for this sub-mode; `VerbPreview` is frozen, so that needs a
field-level ruling — flagged. Amendment-S: recognizer inputs only, no control inputs back.

### II.2 Issuance — one node field, four BSL verbs

New real Pydantic field `social_class.labor_withdrawal: Probability = 0.0`
(beside `organization:355`, `subsistence_threshold:351`).

**Why a node field, not the WAGES edge:** BSL cannot write an edge attribute
(`bsl-language.rst:645-652`); the `remove-edge`+`add-edge` idiom (`:688-692`) would destroy
`value_flow` (`economic.py:534-536`) and risk an I.15 `E-EVAL-030` mode violation
(`:703-705`); a **model field round-trips** through `from_graph()` so no
`EXTRA_STAMPABLE_ATTRIBUTES` exemption is needed and `check:vocabulary` passes by
construction. **Stated cost:** the withdrawal is per-class, hitting all that class's
employers at once. Multi-employer targeting needs the edge form (an eighth structural verb
= grammar change). `_find_tenancy_target` returning the *first* TENANCY match
(`production.py:222-225`) is a latent multi-territory bug on the same axis — flagged, untouched.

```
(update-node org.id    organization.budget           (sub <f>))
(update-node target.id social_class.wealth           (add <f>))
(update-node target.id social_class.labor_withdrawal (set <w>))
(emit EventType.WORK_STOPPAGE (org org.id) (class target.id)
      (withdrawal <w>) (fund <f>) (relation <edge-type-scope>))
```

**What issuance deliberately does NOT do:**

- **No SOLIDARITY write.** ADR087 (`_mass_work.py:9`) rules PROTEST a solidarity
  *consumer*, never a producer. Making the strike a producer contradicts a ratified ruling
  **for this verb** ⟹ escalation, not improvisation. Organization-building routes through
  `social_class.organization` instead (§II.5), a genuinely unclaimed channel.
- **No `heat`, no `ideology.agitation`** at issuance. State attention must be a *response*
  to the stoppage, not a stipulation of it — it arrives via the `EventInterceptor` chain
  (`kernel/interceptor.py:106-131`, priority 90-100 "Security/State"), where a blocked
  stoppage is a recorded `BlockedEvent` (`:87-104`), never a silent no-op.
- **No `w_paid` write. Load-bearing.** `w_paid`/`v_produced` are the wage⇄value
  counit-defect pair (`economic.py:522-531`) that `ideology.py:239-255` turns into
  `class_wage_balance` → `chauvinist_pressure`. Strike pay in `w_paid` with `v_produced ≈ 0`
  yields `balance = +1.0` (`formulas/contradiction.py:67-97`) = **maximum chauvinist
  pressure** — the engine would read union mutual aid as imperial bribe and route the
  strike's own energy to fascism. The `aid.py:100-103` idiom (write `wealth`, never
  `w_paid`) avoids this by construction.
- **No `creates_value`** (L-SPEND). Budget→wealth is a transfer; the withdrawal *un-creates*
  value upstream, which is `ProductionSystem`'s own accounting (`creates_value: False`, `production.py:73`).

`EventType.WORK_STOPPAGE` is new (Modulus C/G/P, no amendment), severity **derived** from
kind × terminal proximity, never a hand-tiered dict. Named to avoid the live `CAPITAL_STRIKE`
(`events.py:177`) and `LOCKOUT` (`:164`). **The dual already exists:** `LOCKOUT`
(`ooda/action_effects.py:355-366`) is the employer withdrawing wages, attenuating WAGES
`value_flow` directly. The asymmetry should be *preserved*: the employer commands the wage,
the worker commands only their own labour — so the strike writes a participation
coefficient and lets production recompute.

### II.3 Resolution — emergent, zero new functional forms

**Primary: withdrawal reduces value produced.** `ProductionSystem @3.0`
(`production.py:175`) becomes `produced_value = effective_labor_power × population ×
participation × bio_ratio` with `participation = 1 − w_eff` — the same product with a real
factor, a C-family tensor composition, not a new curve. Downstream all existing: LA
wealth-gain and `la_production` fall (`:182-194,207`) → `productivity_value` and
`total_wages` fall (`economic.py:438,453,507`) → class `wealth`/`w_paid` fall (`:513-531`) →
subsistence burns regardless (`:230-237`) → `SurvivalSystem @15` recomputes
`p_acquiescence` from `wealth_per_capita` (`survival.py:143,154-158`). **P(S|A) falls
because the strike is materially costly.** Periphery classes are direct producers
(`production.py:46,179-181`) so a periphery stoppage impoverishes the striker *directly*.

**The Φ mechanism — the game loop, mechanized.** On EXPLOITATION,
`rent = extraction_efficiency × worker_wealth × (1 − consciousness) × (1 − w_eff)`
(`economic.py:289`) ⟹ less `tribute_inflow` / `current_pool` (`:328-329`), the imperial
bribe's sole source (`:456-458`). Then, entirely through shipped code:

```
w_eff ↑ → current_pool ↓ → pool_ratio ↓ (economic.py:683)
      → calculate_bourgeoisie_decision leaves BRIBERY (dynamic_balance.py:93-98)
      → wage_rate ↓ (economic.py:726-729) → super_wage_bonus ↓ (:457-458)
      → LA w_paid ↓ (:529) → class_wage_balance turns NEGATIVE (ideology.py:243-245)
      → chauvinist_pressure = max(0, balance) × scale = 0 (ideology.py:252-254)
      → effective_solidarity no longer suppressed (consciousness_routing.py:320-329)
      → agitation routes to r instead of f
```

The labour aristocracy becomes available to the revolutionary pole **because the bribe
stopped**, not because a mechanic said so. Negative Φ (ruling 16) is real: `SUPERWAGE_CRISIS`
already fires on pool exhaustion (`economic.py:462-487`).

**Effective withdrawal — I-FRESH, three shipped quantities, recomputed every tick:**

```
w_eff = w_declared × (1 − reserve_ratio) × holdout
```

`reserve_ratio` from the TENANCY territory (`reserve_army.py:75-91`) — **the reserve army
breaks strikes**, Marx's own mechanism, already in the engine as `wage_pressure`
(`:94-107`), now with its missing second face. `holdout` = **the fraction of the class whose
wealth clears subsistence**, computed locally from the same two inputs `SurvivalSystem` uses
(`survival.py:127-143`), avoiding any I-ORD cross-partition read. That is **exactly ADR173's
ruled formulation** (`bsl-architecture-standard.md:311-321`) reused, not a second construct.
Two consequences fall out free:

1. **Strike capacity and acquiescence capacity are the same quantity.** The LA sustains long
   strikes *precisely because it is bribed*. When Φ collapses, strikes get shorter and more
   desperate while P(S|R) becomes competitive — economic strike → political rupture is **one
   crossing of two existing curves**, not a scripted phase change.
2. It inherits ADR173's open obligations verbatim (OQ-1e C/G/P derivation under Axiom A0;
   audit Q3 canonical within-class distribution). The lane must not pretend to close them,
   and must not smuggle a `steepness_k` back in as a "strike resolve" coefficient (S-7).

**Duration is arithmetic, not a timer.** `w_declared` persists until rescinded (weekly
Paradox standing-order grain), but `w_eff` decays materially: each struck tick the class
receives less wage and burns subsistence, so `holdout → 0`. **Strike length = fund ÷ burn
rate, raced against the employer's tolerance for lost production** — both sides shipped
quantities. Mutual aid literally buys weeks.

**The employer's reply — the fork the player must learn to read.** `ContradictionSystem @18`
writes fresh `tension` from wealth asymmetry (`contradiction.py:146-151,192-199`) — a
*measurement*, never an assertion. Then `calculate_bourgeoisie_decision`
(`dynamic_balance.py:82-118`, verified this pass):

| pool_ratio | tension | verdict | effect |
|---|---|---|---|
| `< critical (0.1)` | any | **CRISIS** | wage −0.15, repression +0.20 |
| `≥ high (0.7)` | `< 0.3` | **BRIBERY** | wage +0.05 |
| `< low (0.3)` | `> 0.5` | **IRON_FIST** | repression +0.10 |
| `< low (0.3)` | `≤ 0.5` | **AUSTERITY** | wage −0.05 |
| else | — | **NO_CHANGE** | nothing |

**There is no concession-under-pressure branch.** Raising tension can never *buy* a wage
rise. Core wages rise when **imperial rent is available**, not when workers fight — Emmanuel
and Amin, already compiled. Repression lands on `repression_faced`, the P(S|R) *denominator*
(`survival.py:130,162`) and the spark driver (`struggle.py:336,342`), so IRON_FIST
simultaneously depresses P(S|R) and raises the EXCESSIVE_FORCE spark (`struggle.py:341-364`).

**Positionality is already in the code.** `_STRUGGLING_ROLES = {PERIPHERY_PROLETARIAT,
LUMPENPROLETARIAT}` (`struggle.py:50-55`) **structurally excludes the LA from the uprising
path** (`:332-333`). Identical verb, identically issued, categorically different reachable
consequences: an LA strike can gain organization and tension but `UPRISING` is
**unreachable** — a prohibition realized as an absence
(`bsl-architecture-standard.md:438-451`); a periphery strike can cross
`(spark OR P(S|R) > P(S|A)) AND agitation > threshold` (`struggle.py:367-371`) into
`UPRISING`, whose path severs outgoing EXPLOITATION edges (`:683-720`), draining Φ at source.

### II.4 Doctrine differentiation — the five levers

- **Lever 1 (mode authorization).** Basic stoppage ungated. Reserved: `mobilize:strike:general`
  — one act across every class the org holds MEMBERSHIP/SOLIDARITY with. Gate via
  `grants_verb_mode` (`_capability.py:54-61`), refusing loudly (`campaign.py:100-105` idiom).
  `trade_unionism` (`doctrine_tree_mvp.json:21-42`) has **no `capabilities` block at all** —
  adding one is a data-only change; `_capability.py:14-16` guarantees that is the only step.
- **Lever 2 (edge-type authorization)** via `grants_edge_type` (`_capability.py:64-71`):
  `"wages"` = the economic strike (§II.3 primary only); `"exploitation"` = the
  secondary/solidarity strike, **the only path to the Φ mechanism**. Internationalism thus
  becomes *mechanically* the difference between a wage dispute and an attack on imperial
  rent, routed through the already-ruled NATIONAL_CHAUVINISM⟷internationalism axis.
  ⚠ `class_struggle_elections` / `independent_ballot_line` already declare
  `edge_types: ["solidarity"]` (`doctrine_tree_mvp.json:81,123`) that **nothing reads** —
  `grants_edge_type` is only ever called for MEMBERSHIP (`mobilize.py:145`). Dead data today.
- **Lever 3 (cost/efficacy) — MISSING, and squatting outside the registry.**
  `DoctrineCapability` has exactly three fields, frozen, `extra="forbid"`
  (`models/entities/doctrine.py:48-61`). Proof of the squat: `apply_mass_work_solidarity(…,
  efficiency: float = 1.0)` (`_mass_work.py:68,89-93`) is a real efficacy dial whose only
  non-default caller **hard-codes the stance's value at the call site**
  (`campaign.py:146`, `services.defines.politics.debs_solidarity_efficiency`); same shape at
  `aid.py:38` (`_AID_EFFICIENCY = 1.0`, a module literal). That is a Modulus Θ quantity
  outside the registry — rehome before wiring through it.
  **Proposal:** `DoctrineCapability.coefficients: tuple[str, ...]` — **slugs naming which
  `GameDefines` coefficient a stance selects**, never free floats. Tree says *which*,
  `defines.yaml` owns the *value*. Strike slugs: `strike_organization_gain`,
  `strike_fund_efficiency` (retires `aid.py:38`), `strike_solidarity_efficiency` (retires
  `campaign.py:146`, iff the Director rules the ADR087 exemption).
- **Lever 4 (valve coupling).** Precedent `decouples_cadre_valve` (`_capability.py:74-85`).
  Strike analogue: where the stoppage's political energy routes — electoralist pairs toward
  `AllegianceSystem @17.42` / `ElectoralSystem @17.45`; abstentionist pairs into
  `SOLIDARITY_MASS` / `organization`. A new pathway on the fixed instrument panel, never a new needle.
- **Lever 5 (trap exposure).** No new trap. `liquidationism` is already an absorbing state
  gated on measured practice (`doctrine_tree_mvp.json:144-158`, `militancy: −3`). A strike
  **settled by contract** — via `Negotiate(coalition)`, stamping `edge_mode=CO_OPTIVE` and
  accruing `co_optive_dependence` (`negotiate.py:14-18`) → `CO_OPTIVE_SHARE` ↑ — drifts you
  in. A strike that builds `SOLIDARITY_MASS` keeps you out. **The economism trap is already
  built**; the strike is a new road into it. There is no `economism` node or tag and there
  should not be: it is expressed structurally, as `trade_unionism`'s drift.

### II.5 The organization channel (ADR087-compatible)

Because ADR087 forbids the SOLIDARITY route for this verb, the "school of war" runs through
`social_class.organization` — the P(S|R) numerator, a real field with **no player-side
producer** (verified §0). Each struck tick:

```
(update-node target.id social_class.organization (add <strike_organization_gain × w_eff>))
```

clamped by `Probability`. **Symmetry check:** `territory.py:370` already *destroys*
`organization` on eviction; a stoppage building it is the missing dual. Materially honest —
collective action is where a class's capacity for collective action comes from.

### II.6 Typed motions + sentinel rows (ADR109)

| # | Wire (source → target) | Field / seam | Motion | Sentinel row |
|---|---|---|---|---|
| W1 | resolver → `ProductionSystem @3.0` | `labor_withdrawal` (`production.py:175`) | **W-C** | `seam_algebra` `GATE_REGISTRY` row (`registry.py:434`) — the new read needs a production supplier; `MODEL_FIELDS_BY_NODE_TYPE` coverage free via the real field |
| W2 | resolver → `ImperialRentSystem @9` | same → `rent` (`economic.py:289`) | **W-C** | `GATE_REGISTRY` row + `liveness` row (multiplier exercised by production, not only fixtures) |
| W3 | W2's rent reduction → `tribute_inflow`/`current_pool`/L-RECEIPTS | `economic.py:310-329` | **W-A4** | `conservation/registry.py` row: `Σ EXPLOITATION_FLOW(w) == Σ(0) × (1 − w_eff)`; residual = **ALARM**, never warning |
| W4 | `ProductionSystem` → wages phase | `la_production` (`production.py:207` → `economic.py:438,453`) | **W-C (widened)** | ε-row amendment: range now includes strike-induced zeros; `unconsumed` must still see it read |
| W5 | material effects → `capital_labor`/`wage` oppositions | `catalog.py:396,414-416`; `contradiction.py:192-199` | **W-𝔇 (no new registration)** | catalog property test: fresh-per-tick, **no accumulator**; UNPOSITIONED leg when withdrawal removes all flow. Registering a "strike" opposition is **FORBIDDEN** — AE(ii), algebra closed |
| W6 | `territory.reserve_ratio` → `w_eff` | `reserve_army.py:75` via TENANCY | **W-C + W-G caveat** | gate row **plus** W-G empty-iteration/wrong-rung check; v1 restricts to the single TENANCY target and the sentinel **pins** that restriction (never an unweighted mean of an intensive) |
| W7 | `PolicySystem @17.47` `labor_law` overlay → efficacy + repression legality | `POLICY_OVERLAYS_ATTR`, prior-tick | **W-C — closes an OPEN ledger row** (`wiring-doctrine.md:104`) | `test_policy.py::TestOverlayConsumers` pattern vs dry twins; register-absent ⟹ **byte-identical** math (`reserve_army.py:62-91` precedent) |
| W8 | resolver → `organization` → `SurvivalSystem @15` | `survival.py:129,152,160-163` | **W-C** | `liveness` row: `organization` gains its FIRST player-side producer; `inert`/stub-vs-calculator pair confirms reachability |
| W9 | `emit WORK_STOPPAGE` → interceptor → heat/repression | `kernel/interceptor.py:106-131` | **W-C + event rule** | static check: no `logging` site emits catalog kinds; severity **derived**, not hand-tiered |
| W10 | `preview_verb` → plate | `preview.py:99-196` | **W-P** | DeclaredView citation + Amendment-S grep-gate (recognizer inputs only); resolver-parity helper shared |
| W11 | new gap-ledger row | `ai/wiring-doctrine.md` §4 carries **no** player-side STRIKE row | governance | add per the doctrine's own §7; cite W7 as the row it closes |

**Modulus classification of the whole lane:** no new sort, node kind, edge kind, or
constructor family ⟹ **C/G/P, normal development, NO amendment.** New coefficients ⟹
**Θ**, declared tier + envelope. Two genuine escalations: the ADR087 exemption and the
`DoctrineCapability` schema extension. Neither improvised.

### II.7 Verification — fixture-vehicle doctrine, red phase first

- **W-A issuance is inert about outcomes** (the core leg): assert only budget↓, wealth↑,
  `labor_withdrawal == w`, one event. Then assert **negatively**: no `heat`, no `agitation`,
  no `tension`, no `w_paid`, no wage change, no SOLIDARITY edge. A resolver that stipulates
  an outcome fails here.
- **Dry-twin mechanism legs** (two graphs differing in one declared input):
  **W-B** `la_production(w)/la_production(0) == 1 − w_eff` exactly; **W-C1** `current_pool`
  delta equals the un-extracted rent, L-RECEIPTS matching W3's identity; **W-D** `w_eff`
  responds to `reserve_ratio` alone; **W-E** `w_eff → 0` as wealth/pop crosses subsistence,
  duration = `fund ÷ burn` by arithmetic, and assert **no `steepness_k`-style knob** anywhere.
- **W-F the fork is READ, not written:** two fixtures differing only in `initial_pool`;
  **record** which `BourgeoisieDecision` fired, assert it is the function
  `dynamic_balance.py:82-118` declares. Never "the strike won"/"was crushed".
- **W-G prohibitions as absences:** LA vs periphery; `UPRISING` structurally unreachable for
  the former. **Mutation leg: adding LABOR_ARISTOCRACY to `_STRUGGLING_ROLES` must turn the
  test red** — that is what makes the ruling load-bearing.
- **W-H the `w_paid` inversion trap:** strike pay on `wealth` not `w_paid`; assert
  `chauvinist_pressure` does **not** rise on a struck tick.
- **W-I** `check:vocabulary` green with no new exemption; `to_graph()`→`from_graph()` round-trip.
- **W-J/W-K** `qa:regression` byte-identical with no strike declared; two-process determinism;
  `qa:vault-regression-ci` (separate estate); a new `strike` qa scenario under a declared
  ceremony (`Baselines: blessed(<slug>)`). Mutation validation **local only, never CI**.
- **Tutorial coverage row** — the strike is the natural first lesson in reading `pool_ratio`
  before acting; the tutorial IS the BDD suite.

---

## Part III — The consolidated ledger (25 rows, ~110 constructs)

Motions: **W-C** dataflow · **W-𝔇** opposition · **W-G** scale adjunction · **W-P**
projection · **W-A4** conservation. P27 phases per
`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md:502-556`.

### Group A — Verb surface / Article V (the instant defect)

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| A1 | **EXPLOITATION flow has no modulation surface.** `rent` (`economic.py:296`) uses a single global `extraction_efficiency` (`config/defines/economy_basic.py:155`, α=0.8) | W-C + Θ_theory | per-edge/per-org efficacy coefficient in the Θ registry, tier + envelope; ε-rows both sides | W.3.1 row; gate-satisfaction | 2a | **Y** — this IS lever 3 |
| A2 | **`ActionType.STRIKE` has no resolver.** Sole ref `action_eligibility.py:58` | W-C (parameter growth, not a 10th stem) + W-A4 | `verb_modes` entry `mobilize:strike`; budget row proving it **spends** | `test_capability_gated_verbs.py` pattern | 2a | **Y** — Article V; seed not ruled |
| A3 | **WAGES has an employer-side withdrawal, no labour-side mirror.** LOCKOUT attenuates `value_flow` (`action_effects.py:354-364`) | W-C + W-𝔇 (existing `wage` opposition) | ε-row naming the strike's write; the Φ-coupling read so economism is *legible* | W-𝔇 property suite (fresh, UNPOSITIONED) | 2a | **Y** — the LA/economism reading is the line |
| A4 | **`ActionType.EXPROPRIATE` fully dead** — one ref, `action_eligibility.py:65` | W-C **or** RETIRE-WITH-RECORD | if wired: target-sort + W-A4 row (moves a stock, must not mint) | R2 | 2a | **Y** |
| A5 | **`BUILD_INFRASTRUCTURE` implemented, tested, deliberately unregistered** (`build.py:51-91`; gate doc'd `:13-24`) | W-C + W-P | `ActionSpec` row (blocked on the same registry as `wiring-doctrine.md:113`) + preview entry | verb-rule check, §7.8 (BLOCKED) | 2a + 4 | **Y** |
| A6 | **No verb mints PRESENCE; MOVE desynchronizes it.** Producer only `world_state.py:754,762` (`to_graph()`, i.e. session construction); consumers `epistemic_horizon.py:174`, `territory_effects.py:224-372`; `move.py:66,69` writes `territory_ids` only | W-C + W-P | MOVE's write-set extended to the PRESENCE edge set; invariant `PRESENCE set ≡ territory_ids` as an A4 residual | W.3.1 row + round-trip coherence leg | 3 | **N** — repair inside an existing verb's declared effect |
| A7 | **No budget replenishment — one-way ratchet to zero.** Seeded `_legacy.py:953`; decremented `aid.py:101`, `reproduce.py:79`; **zero system writers** (verified) | **W-A4** (a Snk with no Src) + W-C | budget row: Src set, Snk set, residual evaluator, residual = **ALARM** | conservation-registry presence + residual-severity check | 2a + 3 | **Y** — where org money comes from is a theory question |
| A8 | **Six more `ActionType` members with no adjudication** — ORGANIZE, FUNDRAISE, EMPLOY, COUNTER_INTEL, DENOUNCE (proposed `npc_stub.py:45-66`, scored `layer3.py:137,184,211`, no resolver); `RED_BROWN_COUP` eligibility-disabled for every org type (`action_eligibility.py:118-122`) yet has a builder + coefficient. ⚠ the AP table **is** complete over all 26 members (`ooda.py:431-457`) — `base_cost_strike` is *reachable, never reached* | per-member W-C or RETIRE-WITH-RECORD | one disposition row each | R2 | 0 → 2a | **Partial** — FUNDRAISE/EMPLOY/EXPROPRIATE ideological; COUNTER_INTEL/DENOUNCE mechanical |

### Group B — Dead edge vocabulary

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| B1 | **Five `EdgeType` members, zero refs tree-wide** — RECRUITMENT, EMPLOYMENT, TARGETS, OWNED_BY, JURISDICTION (`topology.py:110-121`). Verified this pass. Raw-string lookalikes are unrelated (a hydrator column, an Institution node attr, stub payload keys). Contrast the live sibling: MEMBERSHIP has exactly one producer, `mobilize(canvass)` | W-C ×5 or RETIRE-WITH-RECORD ×5 | if wired: source/target sorts + ε-rows. RECRUITMENT and EMPLOYMENT are exactly the org↔class and business↔class relations a "one org, ANY line" campaign needs | R2 + `fabricated_edge_sources` | 0 → 2a | **N** retire · **Y** wire |

### Group C — Attributes, gate operands, model fields

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| C1 | **`colonial_stance` — no runtime writer.** Six readers (`faction_influence.py:171`, `contradiction.py:645`, `collapse_transition.py:306`, `electoral.py:434`, `reactionary.py:221`, `endgame_detector.py:806`) + 2 projections; written only in seed JSON and tests. It is the *principal national axis* input | W-C on a live W-𝔇 axis | the material conditions under which a stance moves; four defaults exist (`balkanization.py:62-74`) | W.3.2 (R1) + W.3.1 | 3 | **Y** — a faction changing colonial stance IS the line |
| C2 | **`state_violence_index` read-only, default 0.0** ⟹ `violence_gate ≡ 0.0`, conjuncture capped at **2/3** (`conjuncture.py:19,106`). spec-039's writer never landed (`endgame_detector.py:542`) | W-C | the index's definition (Θ_theory tier + envelope) + its writer's pipeline position | W.3.2 (R1) + W.3.4 | 3 | **N** to wire · **Y** on the *definition* |
| C3 | **`coverage_pct` constant** — `reference/schema.py:185` filled with literal `Decimal("100.00")` (`tools/ingest_tiger_geometry.py:185`). This number IS the hex→county allocation weight | **W-G** (`allocate ⊣ aggregate`) + W-A4 | real area-intersection fractions from TIGER; conservation law that weights sum to 1 per county | W-G empty-iteration + stub-vs-calculator | 0 → 2c | **N** |
| C4 | **`is_goal` has no production consumer** — sole reader the shape invariant `validation.py:167`; one node true (`doctrine_tree_mvp.json:205`) | W-P **or** RETIRE-WITH-RECORD | if projected: DeclaredView row. If retired: the ADR line | W2 half of W.3.1 | 2a | **Y** — ruling 1 (patterns-are-the-verdict) may make a "victory-condition leaf" ideologically wrong |
| C5 | **`DoctrineTag.MILITANCY` has no consumer outside the doctrine packages.** Grep excluding `models/*/doctrine` + `domain/doctrine` → **empty**. Yet it carries real `tag_deltas` (`doctrine_tree_mvp.json:149`, `militancy: −3`) moving a number nothing reads | W-𝔇 or W-C | the opposition/measure it feeds; `shadow=True` first per the ADR077 promotion ladder | W-𝔇 property suite + R2 | 2a | **Y** — what militancy does is the insurrectionist-line question |

### Group D — Outcome & narration reachability ("patterns are the verdict")

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| D1 | **Three crisis `SovereigntyType`s never emitted** ⟹ FRAGMENTED_COLLAPSE has a statically unsatisfiable operand. `endgame_detector.py:610-611` compares **raw strings** `{"insurgent","occupation","emergency"}`; the enum members have zero refs. ⚠ **Second defect:** `collapse_transition.py:162,234` stamps `sovereignty_type="provisional"/"secessionist"` as **bare strings**, bypassing the enum — the `balkanization_faction` shape | W-C + W.3.5 static proof | the material conditions minting each crisis type; **never** an outcome assertion | **W.3.5** + a raw-string-stamping rule | 3 | **Y** |
| D2 | **Sovereignty CLAIMS seeding** — ADR080 narrowed `SOV_EXTERIOR_NULL`; P25 U6/ADR132 landed the tri-county seed; ADR109's row still marks live-campaign endgame reachability **OPEN** (`wiring-doctrine.md:117`) | Θ_data seed + W-C | a live-campaign reachability run, not a fixture | W.3.5 | 0 → 3 | **N** verify · **Y** if the seed's politics change |
| D3 | ⚠ **19 of 98 `EventType`s drop to `None` at the bus→pydantic boundary** (recomputed from `sentinels/seam/checks.py:308-331`: 79 handled, 19 unhandled). The brief's "34/98" and `reports/seam-wiring-punchlist.md:124`'s "45 of 79" are both stale. The 19 include **ENDGAME_REACHED, RED_OGV_ENDGAME, FRAGMENTED_COLLAPSE_ENDGAME, CONSCIOUSNESS_SHIFT, SOLIDARITY_AWAKENING, FASCIST_CONVERGENCE, BIFURCATION_TENDENCY_CHANGE** — precisely the verdict surface | W-P | one builder + payload model per member kept; a RETIRE row per member dropped | promote `check_event_coverage` from ADVISORY (`seam/checks.py:348-352`) to GATING | 3 | **N** wire · **Y** on which are intentionally non-narrative |
| D4 | **`render_epilogue` has zero production callers** (`render_epilogue.py:60`). The verdict's own page is never rendered | W-P | DeclaredView row + the call site in the vault render pass | `producers_without_production_caller` (`inert/checks.py:435`) — **investigate why it did not fire** | 4 | **N** wire · **Y** on epilogue content |
| D5 | **6 crafted-but-unreachable narrator templates** — ecological_collapse, eviction_pipeline, fascist_consolidation, heat_change, revolutionary_victory, solidarity_formed (`seam-wiring-punchlist.md:120-123`) | W-P | outcome-aware narration keys, or deletion | promote `check_narrator_vocabulary` to GATING once ruled | 4 | **Y** — content ruling |

### Group E — Assets

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| E1 | **39 SFX + 13 music tracks unwired to the only client.** Sole tree-wide consumers are their own generators; `rg "audio\|sfx\|rodio\|music" rust/` → **empty**: the Rust client (which IS `babylon play`) has zero audio surface | **W-P** (a cue is a one-way projection of an event kind — recognizer input only; Amendment-S) | declared cue table `(EventType \| pattern \| UI binding) → asset id` + music suite→phase map | **new asset-cue coverage check**, mirroring `check:tutorial-coverage`'s shape | 4 | **N** — aesthetic line already ruled |

### Group F — Forward-design constructs (born under this clause, so born wired)

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| F1 | **4 doctrine trunks (ruling 6) vs 3 live** (`doctrine.py:55-57`; tree 6/3/3). Ordered (Major,Minor) over 4 trunks = **one registered opposition**, `sign(w)` = principal aspect — a W-𝔇 registration, not new mathematics | W-𝔇 + W-C | `OppositionSpec` + measure over shipped fields (fresh, no accumulator), `shadow=True` first; the 4th trunk's node set | W-𝔇 property suite + catalog tests | 2a | **Y** — the 4th trunk's identity is the line |
| F2 | **`labor_law` / `police_budget` / `war_posture` read-sides OPEN** (`wiring-doctrine.md:104`). `labor_law` is the **state-side half of A2** | W-C | overlay→base read-sides, register-gated by pipeline position (I-ORD) | `TestOverlayConsumers` pattern | 2c | **N** generally · **Y** where it sets strike legality |
| F3 | **Metabolic axis has no player-side generator** — ECOLOGICAL_COLLAPSE reachable only by neglect | **RULED-ABSENT** (a disposition, not a wire) | the Director's one-line ruling recorded at the axis's site | ledger row only | 0 | **Y** |
| F4 | **P(S\|R) denominator has no hardening generator** — nothing hardens against Repression (`formulas/survival_calculus.py:46-65` is `cohesion/(repression+eps)`) | **RULED-ABSENT** or a new `sub_mode` | the ruling, or the sub-mode's capability row | ledger row, or capability pattern | 0 → 2a | **Y** — reserved to the Director |

### Group G — The doctrine's own dead construct

| # | Construct | Motion | Declared data | Sentinel | Phase | Dir |
|---|---|---|---|---|---|---|
| G1 | **The wiring registry chartered by ADR109 §7.1 was never built.** `src/babylon/sentinels/wiring/` absent (verified); §4's hand-maintained table still stands where the registry was to supersede it (`wiring-doctrine.md:204`). Every row above is currently enforced by **prose only** | W-C on the *tooling* graph: §4 rows → `sentinels/wiring/registry.py` → `check:sentinels-static` | the frozen row shape (motion, producer anchor, consumer anchor, data feed, gate ref, status), seeded from §4 + Groups A–F | itself, plus §7.1's AST-resolution / charter-citation / blocker-citation checks | **0** — it is the instrument every disposition is recorded in | **N** — ratified plumbing |

### Accounting

- **25 rows**, ~**110 individual constructs** (19 EventTypes + 52 assets + 6 narrator
  templates + 5 edge types + 6 ActionTypes + 3 sovereignty types + 19 singleton rows).
  ⚠ corrects the ledger architect's "41 constructs / 26 rows".
- **Director-gated: 17 of 25 rows** (A1–A5, A7, A8-partial, B1-wire, C1, C4, C5, D1, D2-conditional,
  D3-partial, D4-partial, D5, F1, F3, F4). That ratio is the honest measure of how much of
  this defect is *ideological* rather than technical: the wiring is incomplete largely
  because the questions underneath it were never put to the Director.
- **Sequencing implied by the ledger itself:** **G1 first** (the instrument) → Phase-0
  disposition pass over A8/B1/C3/D2/F3/F4 (rulings and retirements, cheap, unblocks the
  freeze tag) → A1–A7 + C1/C2/C5 + F1 in **2a** as the verb-algebra and intrinsic tables
  land → D1/D3/D4 + A6 in **3** → E1 + D5 in **4** with the client.

---

## Part IV — Consolidated Director options (deduped, 12)

Merges the strike architect's 4 pedagogy options with the ledger architect's 10, collapsing
the overlap (strike O1+O2 ≡ ledger "A2/A3 economism") into one framed decision.

1. **Strike surface framing** — (i) `mobilize:strike` sub-mode, capability-gated, stem count
   stays 9 (mechanically ready today); (ii) an edge-type authorization on MOBILIZE so
   striking is an EXPLOITATION/WAGES *target* rather than a mode; (iii) **RULED-ABSENT** —
   withdrawal of labour is a class act the engine adjudicates from conditions, never a
   player button. Each is a different theory of *who acts*.
2. **Does capital concede to domestic struggle?** (the load-bearing pedagogy fork).
   (i) **The bribe, not the fight, sets the core wage** (RECOMMENDED; zero new math): the
   matrix stands as verified — BRIBERY needs high pool AND low tension, so a core wage
   strike yields organization, tension, real cost, and **zero wage gain while Φ holds**; the
   only route to a wage movement is draining the pool, i.e. lever 2's `exploitation`
   authorization, which needs an internationalist stance. Teaches Emmanuel/Amin; economism
   is not punished by a debuff, it simply *doesn't work*. Cost: reads as a bug without the
   narrator and the decision-zone plate.
   (ii) **Add a CONCESSION arm** (high pool AND high tension → wage↑): then, entirely
   through shipped code, `super_wage_bonus ↑ → w_paid ↑ → class_wage_balance more positive →
   chauvinist_pressure RISES → effective_solidarity suppressed → the strike's own agitation
   routes to the FASCIST pole`, while `wealth ↑ → p_acquiescence ↑`. The player **wins the
   strike and moves away from rupture** — victory as defeat, emergent, no new formula. But
   it is a **theory change dressed as a coefficient** (it asserts core capital concedes to
   domestic pressure, which (i) denies) and must be an explicit ruling + ADR.
   (iii) A strike that doesn't touch Φ actively feeds NATIONAL_CHAUVINISM.
   Also to rule: how this composes with the **capital-side** reform ceiling (ADR135 §2.4 —
   investment strike, bond discipline, judicial strike-down, federal preemption): symmetric
   levers, or asymmetric by design?
3. **Class-fraction positionality** (~90% built; needs legibility only). `_STRUGGLING_ROLES`
   already excludes the LA from the uprising path; `_DIRECT_PRODUCER_ROLES` already routes
   production differently. With the holdout term, **the LA strikes LONGER precisely because
   it is bribed** while the periphery strikes shorter and more explosively — and a periphery
   uprising severs EXPLOITATION edges, draining Φ at source. Teaches late-MIM(P) as playable
   structure. Cost: needs narrator + plate or players read unreachability as an engine bug;
   the σ-gradient has zero code, so finer intra-core stratification is not yet expressible.
4. **The strike as school of war** (RECOMMENDED as a *companion* to 1 or 3, not an
   alternative). A struck tick's sole reliable gain is `social_class.organization` — the
   P(S|R) numerator, a field the engine currently only **destroys** (`territory.py:370`).
   Gives the strike a permanent product that survives losing it, and makes a decade of
   "failed" strikes the material precondition for rupture. Also gives lever 3 its flagship
   coefficient (`strike_organization_gain` per stance). Requires ruling the **ADR087
   tension**: (a) grant the strike a narrow solidarity-producer exemption, or (b) use the
   organization channel only (recommended — needs no exemption, closes a cleaner gap).
5. **A7 budget replenishment — where does an org's money come from?** (i) dues proportional
   to MEMBERSHIP mass (mass work becomes materially self-financing); (ii) a FUNDRAISE
   stem/sub-mode (money becomes a verb; risks a resource minigame); (iii) expropriation only
   (militancy pays for itself); (iv) **RULED-ABSENT** — budget is a depleting endowment and
   running out IS the pattern. Each writes a different politics into the org loop.
6. **C4 `is_goal`** — (i) RETIRE as ideologically wrong under ruling 1; (ii) reinterpret as
   a *projection* label (the line's self-understanding, shown to the player, adjudicating
   nothing); (iii) keep it and accept a tree-goal verdict alongside the pattern verdict.
7. **C5 MILITANCY — what does the tag DO?** (i) a W-𝔇 co-input to the P(S|R) **numerator**
   (capacity for confrontation); (ii) a repression-exposure multiplier on the
   **denominator** (militancy attracts the state); (iii) both — the sharpest trade in the
   game; (iv) retire it and read militancy off practice instead. Note (i) and (ii) have
   **opposite signs on the same wire**.
8. **D1 crisis sovereignty** — which material conditions mint an INSURGENT / OCCUPATION /
   EMERGENCY sovereign? Until ruled, FRAGMENTED_COLLAPSE is statically unreachable.
   (i) the consent-insolvency adjunction-failure trigger already seeded; (ii) CLAIMS/fiscal
   collapse (REVOLT/BLOCKADE already declared); (iii) EMERGENCY from the live L-SUSPEND path.
   Bundled question: fix the raw-string stamping at `collapse_transition.py:162,234`.
9. **F1 the 4th doctrine trunk** — three live are REFORMIST / SCIENTIFIC / INSURRECTIONIST.
   The 4th's identity, and which (Major,Minor) pairing counts as *the* registered opposition
   with `sign(w)` as principal aspect, is reserved.
10. **F3/F4 the two ruled absences** — ecology as pure consequence-field, and "the state
    always sees you eventually" (no clandestinity generator). Both defensible as design;
    both currently **indistinguishable from oversight**. Rule each ABSENT with a recorded
    one-line rationale, or charter one sub-mode each.
11. **B1 the five dead edge types** — retiring is cheap and honest, but RECRUITMENT and
    EMPLOYMENT are exactly the org↔class and business↔class relations a "one org, ANY line"
    30–80h campaign will want. Options: retire TARGETS/OWNED_BY/JURISDICTION now and CHARTER
    RECRUITMENT/EMPLOYMENT to the org loop; retire all five and re-mint under BSL.
12. **Enforcement posture** — three advisory checks today cover the verdict surface
    (`check_event_coverage`, `check_narrator_vocabulary`). Promote both now (verdict surface
    becomes build-breaking); promote after the Phase-0 disposition pass; or leave advisory
    and rely on the ledger.

---

## Part V — Consolidated risks

1. **THEORY-INVERTING REFACTOR (highest).** Strike pay MUST write `wealth`, MUST NOT write
   `w_paid`. `w_paid`/`v_produced` feed `class_wage_balance` → `chauvinist_pressure`; strike
   pay in `w_paid` with `v_produced ≈ 0` gives `balance = +1.0` = maximum chauvinist
   pressure, so the engine reads union mutual aid as imperial bribe and routes the strike's
   energy to fascism. Mitigation: witness W-H + a comment at the write site naming the inversion.
2. **The clause is only as strong as its instrument, and the instrument does not exist.**
   If G1 is not built FIRST, this becomes a second unenforced doctrine layered on the first —
   the exact failure mode it exists to prevent.
3. **ADR087 contradiction — escalation required, do not improvise.** `_mass_work.py:9`
   explicitly rules PROTEST a solidarity consumer, never a producer, for this verb.
4. **Row-count inflation.** R2 applied naively across every `StrEnum` under `models/enums/`
   surfaces far more than 25 rows, many legitimately non-emitted. Without a cited-exemption
   path from day one (the `tutorial_coverage` / `ATTRIBUTE_EXEMPTIONS` precedent) the check
   gets turned off rather than satisfied. Same hazard for W.3.4 (`.get(field, 0.0)` is an
   engine-wide idiom, many instances legitimately defensive) — shipping it GATING before an
   exemption registry exists invites a blanket-exemption commit that hollows it out.
5. **The ledger's own decay rate is evidence for computing counts, never writing them.**
   Three figures in the inputs were stale (34/98, 45 of 79, 41/26) and one was wrong in the
   opposite direction (SOLIDARITY producers). **No number belongs in the Standard's prose** —
   every count must be emitted by a sentinel.
6. **BSL expressibility ceiling.** No `update-edge` verb; `add-edge` carries only
   `:strength`. The natural representation (withdrawal on the WAGES edge) is not expressible,
   forcing the node form ⟹ a strike is per-class against all that class's employers at once.
   Multi-employer targeting needs an eighth structural verb (grammar change). Related latent
   bug: `_find_tenancy_target` returns the FIRST TENANCY match (`production.py:222-225`).
7. **Intensive-aggregation variance error (W6).** `reserve_ratio` is territory-grain;
   folding it onto a multi-territory class by unweighted mean is the known error. v1
   restricts to the single TENANCY target and the sentinel **pins** the restriction.
8. **The verb-surface rows cannot be closed without rulings, but are the rows the game most
   needs.** If the playability re-cut schedules them before the rulings land, the pressure
   will be to improvise a strike verb into a milestone — explicitly forbidden. Hold A1–A5
   **BLOCKED-on-Director**, not OPEN.
9. **A6 and C2 are behavioural changes, not plumbing.** Making PRESENCE the declared product
   of MOVE changes the edge set a running graph carries (baselines, possibly the tick hash).
   Wiring a `state_violence_index` writer moves every conjuncture-derived value. Both need
   declared ceremonies.
10. **A7 wired wrong is worse than left alone.** Any budget source that mints without a
    matched Src is a W-A4 violation and, from a verb, an L-SPEND violation. The residual
    evaluator must ALARM, not warn, or the fix reproduces the bug class the doctrine exists against.
11. **ADR173 obligations inherited, not closed.** The holdout term reuses the ruled
    "measure of members clearing subsistence", still owing its C/G/P derivation (OQ-1e) and
    canonical within-class distribution (audit Q3). Must not smuggle a steepness knob back as
    a "strike resolve" coefficient (fresh S-7 violation).
12. **`DoctrineCapability` schema change** touches a shared P25 contract (frozen,
    `extra="forbid"`, read by every stance). Needs its own unit and sentinel; coefficients
    must be **slugs resolving into `GameDefines`**, never free floats, or the Θ registry is bypassed.
13. **Golden-estate drift.** `qa:regression` byte-identical (register-absent ⟹ identical
    math) AND `qa:vault-regression-ci` — a separate estate that renders `observe()` pages and
    can drift while every checkpoint stays identical.
14. **Dead-construct debt left in place** (`ActionType.STRIKE`, its eligibility row,
    `base_cost_strike`) risks a future agent "connecting" them into a 10th verb — a
    constitutional event. Retire by a separate declared motion.
15. **Rows interact; per-row dispositioning will miss it.** A1 × A3 × F2 is **one**
    mechanism, not three; C5 × F4 is **one** trade. The ledger needs a declared **coupling
    column**, or these rows must be ruled as bundles.
16. **E1's cue table is a Phase-2 content decision even though the audio is Phase 4.**
    Authored late, the cue→EventType mapping gets invented by whoever wires the audio rather
    than derived from the event catalog — and a cue reading a control input instead of a
    recognizer input trips the Amendment-S tripwire.

---

## Part VI — Raw architect outputs (as received, superseded where ⚠ above)

The two architect JSON payloads are preserved verbatim in the workflow transcript. Their
substantive content is merged above; where this document marks ⚠ the correction is
authoritative. Specifically superseded: the ledger architect's SOLIDARITY-producer count
(one → three), its SECESSIONIST claim (never stamped → stamped as a raw string), its row
arithmetic (26 → 25), and its "41 constructs" figure (→ ~110). Confirmed against the tree
and carried forward unchanged: the vocabulary 6-rules doc drift, the missing
`sentinels/wiring/` package, `violence_gate ≡ 0.0`, the five dead edge types, the budget
ratchet, `organization` having no player-side producer, the BSL verb set, and the
bourgeoisie decision matrix having no concession arm.
