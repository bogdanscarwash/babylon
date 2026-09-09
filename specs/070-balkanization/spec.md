# Feature Specification: Sovereign Topology + Faction Influence + Balkanization

**Feature Branch**: `070-balkanization`
**Created**: 2026-05-17
**Status**: Draft
**Input**: User description: "specification 070 from `reports/aidocs-vs-code-audit-2026-05-16.md` — Sovereign Topology + Faction Influence + Balkanization (Wave 1)"

## Clarifications

### Session 2026-05-18

- Q: How does the player interact with the political topology — as one of the Factions, as a god-mode observer, or both? → A: BOTH modes are supported: a "campaign mode" where the player picks one Faction at game start and all player verbs route through that Faction's vanguard economy (spec-072 integration point), and an "observer mode" (god-mode) where the player can boost any Faction's INFLUENCES and install Sovereigns directly. Mode is selectable at run start.
- Q: What predicate triggers fracture / secession? → A: Composite — secession fires automatically on `SOVEREIGN_COLLAPSE` (parent dissolves, territories partition by INFLUENCES winner) OR when a non-incumbent Faction reaches majority influence in a contiguous geographic sub-region of a still-standing Sovereign. Contiguity is mandatory; scattered influence does not trigger active secession.
- Q: What's the per-tick wallclock budget for the three new Systems combined, relative to spec-069's canonical-run budget? → A: ≤5% combined for FactionInfluence + Sovereignty + CollapseTransition systems, with no single system exceeding 3%. Leaves headroom for downstream specs 071–096 each needing their own slice.
- Q: How do the four Balkanization endgame pathways realize in `GameOutcome`? → A: Conceptual mapping (NOT four parallel new enum values):
  - `REVOLUTIONARY_VICTORY` IS `TRUE_LIBERATION` — same outcome; the existing predicate is strengthened to require ABOLISH-Sovereign-majority + extraction stopped + habitability stabilizing. A revolution that doesn't abolish settler-colonialism is NOT a revolutionary victory; it routes to `RED_OGV`.
  - `RED_OGV` (Occupied Garrison of the Volksgemeinschaft) — **new enum value**; the social-democracy "victory" trap: IGNORE-Sovereign majority + class tension reduced + habitability still declining. Settler-socialism keeps the pipelines flowing under red flags.
  - `ECOLOGICAL_COLLAPSE` — unchanged; catastrophic overshoot path, independent of political alignment (existing predicate retained as-is).
  - `FRAGMENTED_COLLAPSE` — **new enum value**; the police-uprising-as-warlords path: no Faction holds majority, multiple Sovereigns persist in unresolved competition, prefigures spec-081 Warlord Trajectory branching.
  - `FASCIST_VICTORY` IS `FASCIST_CONSOLIDATION` — same outcome; the existing false-consciousness predicate is augmented with a second-route political-violence predicate (UPHOLD-Sovereign majority + max state violence + INTENSIFY extraction). Either route fires the same `FASCIST_CONSOLIDATION` outcome.

  Net `GameOutcome` enum change: ADD `RED_OGV` and `FRAGMENTED_COLLAPSE`; KEEP `IN_PROGRESS` / `REVOLUTIONARY_VICTORY` / `ECOLOGICAL_COLLAPSE` / `FASCIST_CONSOLIDATION`; AUGMENT the predicates for `REVOLUTIONARY_VICTORY` (stricter — colonial-stance gate) and `FASCIST_CONSOLIDATION` (broader — second political-route predicate fires the same value).
- Q: What's the initial `ruling_faction_id` of `SOV_USA_FED` at simulation start? → A: `FAC_RESTORATIONIST` (UPHOLD stance). The MLM-TW theory holds that the US settler-colonial state IS the Restorationist project — settler colonialism is an ongoing structure, not a historical event. Game starts with INTENSIFY extraction from t=0, rapid drift toward FASCIST_CONSOLIDATION unless the player intervenes. This is a hard start with urgency from tick 0; no neutral "liberal" buffer state.

### Session 2026-05-18 (post-/speckit.analyze remediation)

- Post-analyze fix: Extended FR-008 to four canonical Factions
  (added FAC_LIBERAL_IMPERIAL) to align spec.md with research.md
  R-002 / Constitution IV.1 (Detroit-Windsor Boundary). Cleared
  finding C4.
- Post-analyze fix: Reworded FR-021 tiebreaker phrasing to make
  "incumbent ruling_faction" precise. Cleared finding I2.
- Post-analyze fix: Added FR-031a to honor Constitution I.4
  (George Jackson Bifurcation) — REVOLUTIONARY_VICTORY now
  requires cross-divide SOLIDARITY edges; absent that, the run
  routes to RED_OGV instead. Cleared finding C5; added SC-016.
- Post-analyze fix: Expanded FR-038 / FR-039 / FR-040 and added
  FR-040a (SOV_CAN_FED) + FR-040b (SOV_EXTERIOR_NULL). The
  exterior-fallback Sovereign is now concretely named, typed
  (PROVISIONAL with NULL ruling_faction), and seeded; the
  "rest_of_usa" Territory is treated as that Sovereign's initial
  CLAIM target rather than as a Sovereign itself. Cleared
  findings C1 (FR-039 mechanism concretized) and C2 (exterior
  Sovereign defined). Edge cases "Unclaimed Territory" and
  "All-zero influence" updated accordingly. Added SC-017
  (initial-state coverage invariant).
- Post-analyze fix: Added FR-051 (observability projections —
  `observe_sovereign`, `observe_territory`) as engine-side API
  contracts satisfying FR-036 + FR-037; UI rendering deferred
  to spec-042 / spec-085. Cleared finding C3.
- Post-analyze fix: Added FR-052 (principal-contradiction
  cross-reference) clarifying how ColonialStance axis relates
  to but does not displace Constitution I.13's principal-
  contradiction selection mechanism. Cleared finding C6.
- Post-analyze fix: Corrected FR-041 numbering parenthetical
  from "19.5" to "20.5" reflecting current pipeline state.
  Cleared finding I1.

## Background

The 2026-05-16 ai-docs-vs-code audit identifies Balkanization as the
single highest-leverage piece of unimplemented Epoch 3 work: it is
theoretically central ("you cannot build socialism on stolen land"),
independent in dependency terms, and unblocks Reactionary Subject
(spec-071), Demographic Crisis (spec-074), and Warlord Trajectory
(spec-081). The audit's second-pass correction (Part 3-FULL #2) further
merges the previously-proposed "Sovereign Topology + Faction Influence"
residual from `ai-docs/epochs/epoch3/epoch2-persistence.yaml` (v1.1.0
Dynamic Sovereignty + v1.2.0 Balkanization amendments) into the same
spec, yielding the merged scope captured here.

Source-of-truth documents and verification:

- **Primary guidance**: `ai-docs/epochs/epoch3/balkanization-spec.yaml`
  (586 lines, 2025-12-26) and `ai-docs/epochs/epoch3/epoch2-persistence.yaml`
  v1.1.0 + v1.2.0 (lines 78-340 + 576-752, 2025-12-26). These articulate
  the Three Stances, three canonical factions, four endgames, the Red
  Settler Trap, and the Sovereign / Faction / CLAIMS / INFLUENCES
  schema vocabulary. **Treat as guidance, not catechism** — they
  pre-date several stack rotations (KuzuDB → PostgreSQL per ADR030,
  ChromaDB → pgvector, etc., per the audit's drift matrix). Where the
  YAMLs and current code disagree, the codebase wins.
- **Audit source**: `reports/aidocs-vs-code-audit-2026-05-16.md`
  Part 3-FULL Wave 1 — merged scope statement, real-proxy-data
  bootstrap requirement, ~140-180h complexity estimate, downstream-
  unblock relationships.
- **Code reality verified at 2026-05-17**: `src/babylon/models/entities/`
  (no `sovereign.py`, no `faction.py` — both greenfield);
  `src/babylon/models/enums/topology.py` (`EdgeType` has 18+ values,
  none of CLAIMS / INFLUENCES / ADMINISTERS);
  `src/babylon/models/enums/events.py` (`GameOutcome` has 4 values
  today — see §"Relationship to Existing GameOutcome" below);
  `src/babylon/engine/simulation_engine.py` (`_DEFAULT_SYSTEMS` is
  exactly 21 systems, partitioned into `MATERIAL_BASE_SYSTEMS` /
  `ACTION_PHASE_SYSTEMS` / `CONSEQUENCE_SYSTEMS` per spec-056 with
  an import-time assertion). The `FactionBalance` entity in
  `src/babylon/models/entities/state_apparatus_ai.py` IS state-
  internal-factionalism (Finance-Capital / Security-State /
  Settler-Populist via `StateFaction` enum) — a **different
  concept** from the political-coalition Faction in this spec; the
  spec disambiguates by naming the new entity `PoliticalFaction`
  where collision matters (see §"Naming Disambiguation" below).

The simulation has rich materialist substrate (21-system tick
pipeline through spec-066, Territory + heat dynamics, MetabolismSystem,
EXCESSIVE_FORCE → UPRISING cascade, existing endgame detectors,
PostgreSQL runtime via spec-037, Detroit tri-county hex scope via
spec-062), but lacks any notion of contested political authority
over land. Sovereignty today is implicit and unitary. This spec
makes sovereignty plural, contestable, and consequential.

### Naming Disambiguation

The current codebase uses "Faction" in TWO existing senses:
1. `StateFaction` enum + `FactionBalance` entity (spec-039 State
   Apparatus AI) — ruling-class factional balance INSIDE the state
   apparatus: Finance-Capital / Security-State / Settler-Populist.
2. `factions.schema.json` + `targetFactionId` (Movement/Organization
   schemas) — referenced by JSON data but with limited engine wiring.

The Balkanization spec introduces a **third, distinct** Faction
concept: political coalitions that contest sovereignty over
territory (Restorationist Front, Workers' Congress, Decolonial
Front). To avoid silent collisions, this spec uses
`PoliticalFaction` (or `BalkanizationFaction`) as the concrete
entity name in the codebase. The audit's "Faction" terminology is
preserved in spec discussion for theoretical fidelity, but the
implementation MUST use a name that distinguishes from
`StateFaction` / `FactionBalance`.

### Relationship to Existing GameOutcome

Code today defines `GameOutcome` (in
`src/babylon/models/enums/events.py`) with four values:
`IN_PROGRESS`, `REVOLUTIONARY_VICTORY`, `ECOLOGICAL_COLLAPSE`,
`FASCIST_CONSOLIDATION`. Per the 2026-05-18 clarification, the
Balkanization endgames map onto the existing enum as follows
(NOT four parallel new values):

| Balkanization endgame | Maps to GameOutcome | Predicate change |
|---|---|---|
| TRUE_LIBERATION | `REVOLUTIONARY_VICTORY` (same outcome) | Existing predicate STRENGTHENED to require ABOLISH-aligned Sovereign majority + extraction stopped + habitability stabilizing. A revolution that doesn't abolish settler-colonialism routes to `RED_OGV` instead. |
| RED_OGV (Occupied Garrison of the Volksgemeinschaft) | `RED_OGV` (**new enum value**) | The social-democracy "victory" trap: IGNORE-aligned Sovereign majority + class tension reduced + habitability still declining. Settler-socialism keeps the pipelines flowing under red flags. |
| ECOLOGICAL_COLLAPSE | `ECOLOGICAL_COLLAPSE` (same outcome) | Existing predicate UNCHANGED — catastrophic overshoot path, independent of political alignment. |
| FRAGMENTED_COLLAPSE | `FRAGMENTED_COLLAPSE` (**new enum value**) | Police-uprising-as-warlords path: no Faction holds majority, multiple Sovereigns persist in unresolved competition. Prefigures spec-081 Warlord Trajectory branching. |
| FASCIST_VICTORY | `FASCIST_CONSOLIDATION` (same outcome) | Existing false-consciousness predicate (national_identity > class_consciousness) AUGMENTED with a second-route political-violence predicate (UPHOLD Sovereign majority + max state violence + INTENSIFY extraction). Either route fires the same `FASCIST_CONSOLIDATION` outcome. |

**Net enum change**: ADD `RED_OGV` and `FRAGMENTED_COLLAPSE`. KEEP
`IN_PROGRESS`, `REVOLUTIONARY_VICTORY`, `ECOLOGICAL_COLLAPSE`,
`FASCIST_CONSOLIDATION`. AUGMENT predicates for
`REVOLUTIONARY_VICTORY` (stricter — colonial-stance gate) and
`FASCIST_CONSOLIDATION` (broader — second route).

**Naming note**: The audit / theory uses TRUE_LIBERATION and
FASCIST_VICTORY as Balkanization-specific framings. These names
remain as *user-facing labels* in narrative / UI / event payloads
(e.g., the player-visible message on a `REVOLUTIONARY_VICTORY` may
read "TRUE LIBERATION achieved"), but the underlying
`GameOutcome` enum value is the existing one.

## Theoretical Mandate

> The core struggle is SETTLERS vs. INDIGENOUS. The primary failure
> mode for the player is not just "Fascism" but also
> "Settler-Socialism" — movements that claim to be revolutionary but
> refuse to dismantle the colonial relationship to the land.
>
> You cannot build socialism on stolen land.

This spec implements the mechanical consequence of that thesis: a
faction's `ColonialStance` (UPHOLD / IGNORE / ABOLISH) determines the
`ExtractionPolicy` (INTENSIFY / CONTINUE / CEASE) of any Sovereign it
rules, and only `CEASE` allows habitability recovery. Settler-socialist
factions can win the revolution and still lose the planet.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Extraction Policy Decides a Territory's Material Fate (Priority: P1)

A Sovereign authority controls a Territory. The Sovereign's
`ExtractionPolicy` (INTENSIFY, CONTINUE, or CEASE) applies a
deterministic per-tick effect to the Territory's habitability,
producing visibly different long-run trajectories under each policy.
INTENSIFY accelerates the metabolic rift; CONTINUE maintains slow
degradation; only CEASE permits restoration.

**Why this priority**: This is the smallest playable demonstration of
the spec's theoretical thesis. Without this loop, sovereignty is
cosmetic. With it, every other mechanic in the spec gains material
stakes — the player can watch habitability move in response to a
single policy choice.

**Independent Test**: Seed one Sovereign with `ExtractionPolicy=INTENSIFY`
claiming one Territory at habitability=0.8. Tick the simulation
ten times. Habitability falls by ~0.2 (rate −0.02/tick). Switch to
`CONTINUE`; over the next ten ticks the slope flattens to ~−0.005/tick.
Switch to `CEASE`; over the next ten ticks habitability recovers at
~+0.01/tick.

**Acceptance Scenarios**:

1. **Given** a Sovereign with `ExtractionPolicy=INTENSIFY` CLAIMS one Territory at habitability=0.8, **When** ten ticks elapse, **Then** habitability has fallen by 0.2 (±tolerance) consistent with the −0.02/tick metabolic_impact, with no other change.
2. **Given** the same Sovereign switches to `ExtractionPolicy=CEASE` mid-run, **When** five subsequent ticks elapse, **Then** the habitability trajectory reverses (slope changes sign to positive) within those five ticks.
3. **Given** a Sovereign CLAIMS multiple Territories simultaneously, **When** one tick elapses, **Then** every claimed Territory receives the per-tick `metabolic_impact`; no claimed Territory is skipped.
4. **Given** a Territory is unclaimed (no CLAIMS edge from any Sovereign), **When** the tick runs, **Then** no policy-driven habitability change applies to it (only background MetabolismSystem dynamics).

---

### User Story 2 — Factions Contest a Territory and Install Sovereigns (Priority: P2)

Multiple Factions accumulate `INFLUENCES` edges into the same
Territory. At decision moments (e.g., Sovereign collapse, election,
uprising), the Faction with the highest aggregate influence installs
its preferred Sovereign, whose `ColonialStance` then determines the
Sovereign's `ExtractionPolicy` and so its per-tick effect on the
Territory.

**Why this priority**: Without inter-factional competition,
sovereignty is uncontested and the policy choice is pre-decided.
Factional competition is what makes the political-topology layer a
*layer* rather than a fixture — what gives the player something to
push against, build with, or be defeated by.

**Independent Test**: Seed the three canonical Factions
(Restorationist Front [UPHOLD], Workers' Congress [IGNORE], Decolonial
Front [ABOLISH]) with overlapping INFLUENCES on one Territory under a
collapsing Sovereign. Tick to collapse-transition. Verify the
highest-influence Faction wins and the new Sovereign inherits the
corresponding ExtractionPolicy (UPHOLD→INTENSIFY, IGNORE→CONTINUE,
ABOLISH→CEASE).

**Acceptance Scenarios**:

1. **Given** three Factions with influence levels {Restorationist=0.4, Workers'-Congress=0.5, Decolonial=0.1} on one Territory, **When** a collapse-transition runs over that Territory, **Then** Workers'-Congress is identified as the winning Faction and a Sovereign with `extraction_policy=CONTINUE` is installed (or retained) over that Territory.
2. **Given** influences shift to {Restorationist=0.2, Workers'-Congress=0.3, Decolonial=0.5}, **When** the next collapse-transition runs, **Then** the installed Sovereign transitions to a Decolonial-aligned one with `extraction_policy=CEASE` and a `TERRITORY_TRANSITION` event is emitted.
3. **Given** two Factions are tied for winning Faction on a Territory, **When** resolution runs, **Then** a documented tiebreaker (priority: existing-incumbent first, then deterministic-RNG-by-seed) selects the winner and the outcome is reproducible given the seed.
4. **Given** a new Sovereign is created from a Faction victory, **When** the Sovereign is installed, **Then** its initial CLAIMS edge has `control_level=0.8` (not 1.0) to model that consolidation takes time, and `legal_status=DE_FACTO`.
5. **Given** the sum of all INFLUENCES edges into a Territory exceeds 1.0, **When** winning-faction resolution runs, **Then** it still works correctly — INFLUENCES sums are not constrained to ≤1.0 (unlike CLAIMS control_levels).

---

### User Story 3 — Sovereign Collapse and Branching Endgames (Priority: P3)

When a Sovereign's legitimacy or claim-control falls below thresholds,
`SOVEREIGN_COLLAPSE` fires. Contested territories transition to new
Sovereigns based on Faction influence. The cumulative pattern of
collapses across the map drives the game toward one of four named
endgames: `TRUE_LIBERATION`, `RED_OGV`, `FASCIST_VICTORY`, or
`FRAGMENTED_COLLAPSE`. The RED_OGV ending is the "False Summit"
trap — the workers won but the pipelines still flow.

**Why this priority**: Without endgame branching, political topology
has no terminal stakes. The four-endgame branching — and the RED_OGV
trap in particular — is what makes control of sovereignty *matter*
across a full run, and what carries the spec's theoretical critique
into emergent gameplay rather than narration.

**Independent Test**: Construct four synthetic terminal states:
(a) UPHOLD-faction majority + active state violence → FASCIST_VICTORY;
(b) IGNORE-faction majority + class tension reduced + habitability
still crashing → RED_OGV (the False Summit);
(c) ABOLISH-faction majority + extraction stopped + habitability
stabilizing → TRUE_LIBERATION;
(d) no faction majority + multiple competing sovereigns →
FRAGMENTED_COLLAPSE.
Verify the EndgameDetector emits the correct endgame from each
synthetic state and only one endgame event is emitted per run.

**Acceptance Scenarios**:

1. **Given** a Sovereign's `legitimacy <= 0.0`, **When** the CollapseTransition system runs, **Then** a `SOVEREIGN_COLLAPSE` event is emitted exactly once for that Sovereign and the collapse-transition pipeline begins for each claimed Territory.
2. **Given** an `ECOLOGICAL_OVERSHOOT` event has fired for the global state, **When** the CollapseTransition system runs, **Then** every Sovereign with claimed territories in the overshoot zone is marked for collapse and `SOVEREIGN_COLLAPSE` events fire for them in deterministic order.
3. **Given** all active Sovereigns are ABOLISH-aligned with `extraction_policy=CEASE` and global habitability is recovering AND the existing percolation + class_consciousness conditions also hold, **When** the EndgameDetector runs, **Then** the existing `REVOLUTIONARY_VICTORY` endgame fires with the "TRUE LIBERATION" user-facing framing, exactly once, and the simulation terminates (or enters post-endgame observation mode).
4. **Given** an IGNORE-aligned Sovereign holds majority claims, class tension has dropped below the configured floor, but aggregate habitability is below the configured floor and declining, **When** the EndgameDetector runs over a sufficient delay, **Then** the new `RED_OGV` endgame fires with a player-visible message naming the contradiction (workers won, planet lost).
5. **Given** an UPHOLD-aligned Sovereign holds majority claims and exercises maximum state violence with INTENSIFY extraction policy, **When** the EndgameDetector runs, **Then** the existing `FASCIST_CONSOLIDATION` endgame fires (via the new political-violence route, not the false-consciousness route) with the "FASCIST VICTORY" user-facing framing.
6. **Given** no Faction holds majority influence, ≥3 distinct Sovereigns persist with insurgent/occupation/emergency `sovereignty_type`, and the configuration has persisted ≥10 ticks, **When** the EndgameDetector runs, **Then** the new `FRAGMENTED_COLLAPSE` endgame fires with the count of surviving Sovereigns in the payload.
7. **Given** the EndgameDetector has emitted any endgame event, **When** subsequent ticks run, **Then** no second endgame event is emitted in the same run.

---

### User Story 4 — Secession and Civil War as O(1) Edge Rewiring (Priority: P4)

A Faction with strong but non-majority influence in a Sovereign's
territories can secede, fracturing one Sovereign into two with
CLAIMS/INFLUENCES partitioned between the resulting Sovereigns.
Likewise, a hostile Sovereign can conquer (re-claim) a Territory by
overpowering the incumbent's `control_level` on its CLAIMS edge. Both
operations are graph rewirings — they delete or reduce one CLAIMS edge
and create another — and their cost does not scale with the unaffected
territory count.

**Why this priority**: Fracture (secession + conquest + civil war) is
what lets the political topology evolve mid-game rather than only at
endgame. The O(1) cost requirement (per ADR029 + epoch2-persistence
v1.1.0) is what makes fracture cheap enough to use as a recurring
mechanic.

**Independent Test**: Construct a Sovereign claiming N=1000 Territories
with a secessionist Faction holding influence above threshold in K=300
of them. Trigger fracture. Verify (a) the resulting two Sovereigns
together cover the original Sovereign's full territory set, (b) the
seceded Faction's preferred territories migrate to the new Sovereign,
(c) the operation's runtime cost is consistent with edges-on-boundary,
not with N. Benchmark against N ∈ {10, 100, 1000} with the same
boundary size; runtimes are flat in N (per spec-069 wallclock budget
context).

**Acceptance Scenarios**:

1. **Given** a Sovereign CLAIMS N Territories and a secessionist Faction has `influence_level > 0.5` in K of them, **When** fracture executes, **Then** two Sovereigns exist where one CLAIMS K Territories with `legal_status=DE_FACTO` and `control_level=0.9`, the other CLAIMS N−K with reduced `control_level=0.1` and `legal_status=DISPUTED`, and the original CLAIMS edges are removed or modified accordingly.
2. **Given** the fracture operation is benchmarked at N=10, 100, 1000 with K/N=0.3, **When** runtime is measured, **Then** cost growth is bounded by the partition-boundary size and does not scale with the unchanged territory count (consistent with the v1.1.0 Dynamic Sovereignty O(1) edge-rewiring claim).
3. **Given** a Sovereign holds Territories where two distinct seceding Factions both meet the threshold, **When** fracture is attempted, **Then** a `CIVIL_WAR_DECLARED` event is emitted and contested-boundary Territories receive multiple CLAIMS edges with `legal_status=DISPUTED` rather than being silently assigned.
4. **Given** a hostile Sovereign attempts to conquer a Territory by raising its CLAIMS `control_level` while reducing the incumbent's, **When** the incumbent's `control_level` falls below 0.5, **Then** the Territory's effective controller (winning CLAIMS edge) flips and a `TERRITORY_TRANSITION` event is emitted.
5. **Given** a fracture leaves a Sovereign with zero claimed Territories, **When** the cleanup pass runs, **Then** the orphaned Sovereign is deleted (along with its outbound CLAIMS and ADMINISTERS edges) in the same tick — no zombie nodes remain.

---

### Edge Cases

- **Unclaimed Territory**: A Territory with no CLAIMS edges from any
  Sovereign — system MUST add a CLAIMS edge from `SOV_EXTERIOR_NULL`
  (per FR-040b) to that Territory with `legal_status=DE_FACTO`,
  `control_level=1.0`, and emit a `TERRITORY_TRANSITION` event with
  `reason=collapse_partition`. `SOV_EXTERIOR_NULL` itself CLAIMS the
  existing `rest_of_usa` boundary node (per spec-062 R4) at game
  start; orphaned in-scope Territories are reassigned to
  `SOV_EXTERIOR_NULL` mid-game as a sink.
- **All-zero influence**: A Territory where every Faction has zero
  influence — winning-Faction resolution MUST fall back to incumbent
  (the `ruling_faction_id` of the Territory's installed Sovereign);
  if no incumbent or the incumbent Sovereign is `SOV_EXTERIOR_NULL`
  (whose `ruling_faction_id` is NULL), the Territory remains under
  `SOV_EXTERIOR_NULL`'s authority and no `TERRITORY_TRANSITION`
  event fires.
- **Sovereign with zero claimed Territories after partition**:
  Sovereign MUST be deleted in the same tick as the partition; its
  outbound CLAIMS / ADMINISTERS edges are removed; no zombie entity
  remains.
- **Faction with zero INFLUENCES across all Territories**: Faction
  remains in the simulation (it can still be targeted for
  consciousness-drift, recruitment, etc.) but is excluded from
  winning-Faction computations.
- **Concurrent SOVEREIGN_COLLAPSE in one tick**: Only one
  `SOVEREIGN_COLLAPSE` event per Sovereign per tick; collapse-trigger
  predicates MUST be idempotent within a tick.
- **Endgame already triggered**: After any endgame event fires, no
  subsequent endgame event MAY fire in the same run.
- **Secessionist Faction holds majority in 100% of the Sovereign's
  territories**: Fracture is a no-op for territory count; instead the
  ruling_faction_id of the existing Sovereign is replaced by the
  secessionist's preferred ruling Faction (and the extraction_policy
  is recomputed).
- **Sum of CLAIMS control_levels exceeds 1.0**: This is a transient
  "dual power" state allowed during transitions; the system MUST
  surface a `DUAL_POWER_ACTIVE` diagnostic but MUST NOT fail the tick.
- **Red Settler Trap**: A Faction whose consciousness profile is
  socialist-leaning (high `class_reduction`) but whose
  `colonial_stance` is UPHOLD or IGNORE — system MUST surface a
  `RED_SETTLER_TRAP_DETECTED` diagnostic event so the player can
  recognize the contradiction.
- **Sovereign with `dissolved_tick` set**: Treated as historical
  reference only; no per-tick effects, but its Chronicle entries
  remain queryable.
- **Faction founded after game start**: New Factions can be
  introduced mid-game (e.g., a split from an existing Faction);
  their INFLUENCES start at 0 and accumulate via gameplay actions.
- **Scattered influence majority**: A Faction holds
  `influence_level > 0.5` in many Territories but those Territories
  are not contiguous via ADJACENCY — active secession (FR-029a (2))
  MUST NOT fire. The Faction can still take territory through the
  collapse-driven path (FR-029a (1)) when the parent Sovereign
  falls.
- **Secessionist contiguous sub-region equals 100% of parent's
  territories**: Effectively a coup rather than secession. The
  parent Sovereign's `ruling_faction_id` is replaced (matches the
  existing "100% secessionist majority" edge case); no new
  Sovereign node is created.

## Requirements *(mandatory)*

### Functional Requirements

**Entities — Sovereign**:

- **FR-001**: System MUST define a `Sovereign` entity with unique
  identifier (format `SOV_{CODE}`), human-readable name,
  `sovereignty_type`, `legitimacy ∈ [0, 1]`, `color_hex`,
  `capital_territory_id` (nullable), `founded_tick`, `dissolved_tick`
  (nullable), `ruling_faction_id` (nullable), and `extraction_policy`.
- **FR-002**: `sovereignty_type` MUST be enumerable as exactly:
  `RECOGNIZED_STATE`, `PROVISIONAL`, `INSURGENT`, `OCCUPATION`,
  `SECESSIONIST`, `EMERGENCY`.
- **FR-003**: `extraction_policy` MUST be enumerable as exactly:
  `INTENSIFY`, `CONTINUE`, `CEASE` — and MUST be derivable from
  `ruling_faction.colonial_stance` (UPHOLD → INTENSIFY, IGNORE →
  CONTINUE, ABOLISH → CEASE).
- **FR-004**: Each Sovereign MUST expose a per-tick `metabolic_impact`
  computed from `extraction_policy`: −0.02 for INTENSIFY, −0.005 for
  CONTINUE, +0.01 for CEASE.

**Entities — Faction**:

- **FR-005**: System MUST define a `Faction` entity with unique
  identifier (format `FAC_{CODE}`), name, `ideology` string,
  `colonial_stance`, `is_settler_formation`, mechanical multipliers
  (`extraction_modifier`, `violence_modifier`, `class_reduction`,
  `metabolic_reduction`), `color_hex`, and `founded_tick`.
- **FR-006**: `colonial_stance` MUST be enumerable as exactly:
  `UPHOLD`, `IGNORE`, `ABOLISH`.
- **FR-007**: A Faction's mechanical multipliers MUST be derivable
  from its `colonial_stance` per the v1.2.0 mapping (UPHOLD → 1.5 /
  2.0 / 0.0 / −0.5; IGNORE → 0.8 / 0.5 / 0.7 / 0.0; ABOLISH → 0.0 /
  0.3 / 0.5 / +0.8), with the option for per-Faction overrides via
  initial-state JSON.
- **FR-008**: System MUST seed exactly four canonical Factions at
  simulation start: `FAC_RESTORATIONIST` (UPHOLD), `FAC_WORKERS_CONGRESS`
  (IGNORE), `FAC_DECOLONIAL` (ABOLISH), and `FAC_LIBERAL_IMPERIAL`
  (IGNORE — soft-imperial pole, distinct from FAC_WORKERS_CONGRESS in
  ideology and `is_settler_formation=true` pairing; rules SOV_CAN_FED
  per FR-040a). The fourth Faction is added per Constitution IV.1 to
  ground Canada's settler-colonial formation as a first-class entity
  (see research.md R-002 for derivation).

**Edges — CLAIMS (Sovereign → Territory)**:

- **FR-009**: System MUST support `CLAIMS` edges from Sovereign to
  Territory with `control_level ∈ [0, 1]`, `fiscal_status`,
  `legal_status`, `claimed_since_tick`, and `recognition_level ∈ [0, 1]`.
- **FR-010**: `fiscal_status` MUST be enumerable as exactly: `TAXED`,
  `REVOLT`, `BLOCKADE`, `LIBERATED`, `OCCUPIED`.
- **FR-011**: `legal_status` MUST be enumerable as exactly: `DE_JURE`,
  `DE_FACTO`, `DISPUTED`, `OCCUPIED`, `CEDED`.
- **FR-012**: A Territory MAY have multiple CLAIMS edges (dual power);
  the sum of `control_level` values across a Territory's incoming
  CLAIMS edges SHOULD NOT exceed 1.0, but temporary violations during
  transitions MUST be tolerated with a `DUAL_POWER_ACTIVE` diagnostic
  rather than a tick failure.
- **FR-013**: A Sovereign MUST NOT CLAIMS itself (i.e., CLAIMS edges
  are exclusively Sovereign → Territory, not Sovereign → Sovereign).

**Edges — INFLUENCES (Faction → Territory)**:

- **FR-014**: System MUST support `INFLUENCES` edges from Faction to
  Territory with `influence_level ∈ [0, 1]`, `support_type`,
  `cadre_count`, `sympathizer_count`, and `established_tick`.
- **FR-015**: `support_type` MUST be enumerable as exactly:
  `MATERIAL`, `IDEOLOGICAL`, `MILITARY`, `ELECTORAL`, `LABOR`.
- **FR-016**: A Territory MAY have multiple INFLUENCES edges from
  different Factions; the sum of `influence_level` values across a
  Territory's incoming INFLUENCES edges IS NOT capped at 1.0 (this is
  a key distinction from CLAIMS).
- **FR-017**: A Faction MUST NOT INFLUENCES itself (Factions are not
  Territories).

**Edges — ADMINISTERS (Sovereign → Sovereign)**:

- **FR-018**: System MUST support `ADMINISTERS` edges between
  Sovereigns to express hierarchical delegation (e.g., federal → state
  → tribal-recognized). This is the residual modeling layer that lets
  one Sovereign claim a Territory through a delegated lower-tier
  Sovereign.

**Per-tick Dynamics**:

- **FR-019**: For each tick, the system MUST apply each Sovereign's
  per-tick `metabolic_impact` to every Territory it CLAIMS, modifying
  the Territory's `habitability` by that rate.
- **FR-020**: When multiple Sovereigns CLAIMS the same Territory (dual
  power), the system MUST apply the metabolic_impact of the Sovereign
  with the highest `control_level` — not all simultaneously — to
  avoid double-counting habitability change.
- **FR-021**: The system MUST identify, per Territory at decision
  moments (collapse-transition; explicit player verbs in later specs),
  the winning Faction as
  `argmax_f Σ INFLUENCES[f → t].influence_level`. Tiebreaker priority
  on ties: (1) the `ruling_faction_id` of the Territory's currently-
  installed Sovereign (the *incumbent*) — if that Faction is one of
  the tied candidates, it wins; (2) if no installed Sovereign exists
  or the incumbent ruling Faction is not among the tied candidates,
  seed-deterministic RNG with the tied Faction IDs sorted
  lexicographically before sampling.
- **FR-022**: When the winning Faction for a Territory changes across
  ticks, the system MUST emit a `TERRITORY_TRANSITION` event for that
  Territory and update the installed Sovereign accordingly.

**Collapse Transition**:

- **FR-023**: The system MUST emit `SOVEREIGN_COLLAPSE` when any
  collapse predicate is satisfied: `legitimacy <= 0.0`,
  `ECOLOGICAL_OVERSHOOT` event for the Sovereign's claimed footprint,
  `NUCLEAR_EXCHANGE` event, or player-triggered `GENERAL_UPRISING`.
- **FR-024**: On `SOVEREIGN_COLLAPSE`, the collapse-transition
  pipeline MUST execute the five steps from balkanization-spec.yaml
  §collapse_transition: (1) identify the collapsing Sovereign's
  claimed Territories, (2) compute per-Territory faction influence
  sums, (3) determine the winning Faction per Territory, (4) create
  or update Sovereigns for each winning Faction with initial CLAIMS
  edges at `control_level=0.8` and `legal_status=DE_FACTO`, and (5)
  apply the new Sovereign's `metabolic_impact` going forward.
- **FR-025**: On `SOVEREIGN_COLLAPSE`, a `TERRITORY_TRANSITION` event
  MUST be emitted for every Territory the collapsed Sovereign claimed
  before the Sovereign itself is removed.
- **FR-026**: The system MUST emit `FACTION_VICTORY` when a Faction
  achieves a configured supermajority of aggregate influence across
  the active Territory set.

**Fracture / Secession / Civil War**:

- **FR-027**: System MUST support a fracture operation that
  partitions one Sovereign into two by rewiring CLAIMS edges in
  O(1) per unchanged edge — i.e., the operation only touches the
  partition boundary's edges, not the full claim set. This realises
  the v1.1.0 Dynamic Sovereignty performance invariant.
- **FR-028**: On fracture, the system MUST emit `CIVIL_WAR_DECLARED`
  and partition CLAIMS + INFLUENCES between the two resulting
  Sovereigns according to the secessionist Faction's influence
  distribution.
- **FR-029**: Contested partition boundaries (Territories where
  neither resulting Sovereign clearly wins) MUST be left with multiple
  CLAIMS edges of `legal_status=DISPUTED` rather than silently
  assigned; the system MUST NOT drop territory claims during fracture.
- **FR-029a**: Fracture MUST be triggered by exactly two predicates,
  no more (additional triggers are deferred to downstream specs):
  1. **Collapse-driven fracture (passive)**: When a Sovereign emits
     `SOVEREIGN_COLLAPSE` (FR-023), the collapse-transition pipeline
     partitions territories by per-Territory winning-Faction
     (FR-024) — this is fracture as a consequence of dissolution
     and is the path described in balkanization-spec.yaml
     §collapse_transition. The parent Sovereign is removed; new
     Sovereigns are created per winning Faction.
  2. **Active secession (contiguity-gated)**: When a non-incumbent
     Faction holds `influence_level > 0.5` across a contiguous
     geographic sub-region of a still-standing Sovereign's claimed
     Territories — i.e., the sub-region is a connected component
     under the existing `ADJACENCY` edge set — the system emits
     `SECESSION_DECLARED`, then `CIVIL_WAR_DECLARED`, then executes
     the fracture operation. The parent Sovereign survives but
     loses the seceded contiguous bloc.
- **FR-029b**: Contiguity for FR-029a (2) MUST be computed against
  the existing `ADJACENCY` edges on Territory nodes (spec-001
  Sprint 3.5.1, retained in `EdgeType.ADJACENCY`). The contiguous
  sub-region is the connected component of all Territories the
  secessionist Faction holds `influence_level > 0.5` in, traversed
  via ADJACENCY edges. A Faction with high influence in
  geographically scattered Territories (no ADJACENCY connectivity
  between them) MUST NOT trigger active secession — though
  collapse-driven fracture (FR-029a (1)) still partitions those
  territories to the Faction when the parent Sovereign falls.
- **FR-029c**: The contiguity-threshold gate (the `> 0.5` influence
  level + the minimum-sub-region-size) MUST be tunable via
  GameDefines and SHOULD apply hysteresis — the predicate must
  hold for at least N consecutive ticks (default N=3) before
  `SECESSION_DECLARED` fires, to prevent flicker.
- **FR-029d**: In OBSERVER mode (FR-047), the player MAY force
  active secession on any Sovereign by directly emitting
  `SECESSION_DECLARED` with an explicit territory partition,
  bypassing FR-029a (2)'s contiguity gate. Such observer-driven
  fractures MUST be flagged in the audit / chronicle log
  (per FR-049) so they are distinguishable from
  in-simulation-triggered fractures.

**Endgames**:

- **FR-030**: System MUST extend `GameOutcome` by ADDING exactly
  two new enum values: `RED_OGV` (Occupied Garrison of the
  Volksgemeinschaft) and `FRAGMENTED_COLLAPSE`. Existing values
  (`IN_PROGRESS`, `REVOLUTIONARY_VICTORY`, `ECOLOGICAL_COLLAPSE`,
  `FASCIST_CONSOLIDATION`) MUST be preserved. The audit's
  `TRUE_LIBERATION` and `FASCIST_VICTORY` framings are user-facing
  labels (UI / narrative / event payload strings) that map to the
  existing `REVOLUTIONARY_VICTORY` and `FASCIST_CONSOLIDATION`
  enum values respectively — they MUST NOT be added as separate
  enum values (see §"Relationship to Existing GameOutcome").
- **FR-031**: The existing `EndgameDetector` observer
  (`src/babylon/engine/observers/endgame_detector.py`) MUST be
  modified as follows:
  - Augment the `REVOLUTIONARY_VICTORY` predicate to ADDITIONALLY
    require ABOLISH-aligned Sovereign majority (across active
    Territories) + aggregate extraction_policy=CEASE + habitability
    slope ≥ 0 over a configured rolling window. The existing
    percolation_ratio ≥ 0.7 + class_consciousness > 0.8 conditions
    remain; the colonial-stance gate is added on top.
  - Augment the `FASCIST_CONSOLIDATION` predicate to ALSO fire on
    a second route: UPHOLD-aligned Sovereign majority + state
    violence at maximum + aggregate extraction_policy=INTENSIFY.
    The existing false-consciousness route
    (national_identity > class_consciousness for 3+ nodes) remains;
    either route fires the same `FASCIST_CONSOLIDATION` outcome.
  - Add a new `RED_OGV` predicate (see FR-032).
  - Add a new `FRAGMENTED_COLLAPSE` predicate (see FR-032a).
  - The existing `ECOLOGICAL_COLLAPSE` predicate (overshoot_ratio
    > 2.0 for 5 consecutive ticks) MUST remain UNCHANGED.
- **FR-031a**: Per Constitution I.4 (George Jackson Bifurcation),
  the augmented `REVOLUTIONARY_VICTORY` predicate (FR-031) MUST
  additionally require *solidarity across the colonial divide*:
  at least `revolutionary_victory_min_cross_divide_solidarity_edges`
  (tunable via `BalkanizationDefines`, default 5) active
  `EdgeType.SOLIDARITY` edges MUST connect (a) entities with
  `is_settler_formation=true` (or settler-coded Community membership
  per `models/enums/community.py`) to (b) entities with
  `is_settler_formation=false` (Decolonial, New Afrikan, Indigenous,
  etc.). A run with ABOLISH-Sovereign majority + extraction stopped +
  habitability stabilizing but WITHOUT cross-divide solidarity routes
  to `RED_OGV` rather than `REVOLUTIONARY_VICTORY` — abolition
  without coalition is itself a trap. This implements I.4's core
  insight: revolution requires solidarity *across* the colonial
  divide, not just within an oppressed-nation pole.
- **FR-032**: The `RED_OGV` predicate MUST require ALL of: (a)
  IGNORE-aligned Sovereign holds majority of CLAIMS across active
  Territories; (b) aggregate class_tension reduced below a
  configured floor (workers' material conditions improved); (c)
  aggregate habitability below a configured floor AND slope still
  negative (planet still dying). The "False Summit" is specifically
  the contradiction between political victory and ecological
  defeat — all three sub-conditions MUST hold simultaneously.
- **FR-032a**: The `FRAGMENTED_COLLAPSE` predicate MUST require ALL
  of: (a) no single Faction holds majority influence across active
  Territories; (b) ≥3 distinct Sovereigns persist with non-zero
  CLAIMS; (c) at least one Sovereign has `sovereignty_type ∈
  {INSURGENT, OCCUPATION, EMERGENCY}` (signalling the warlord /
  police-uprising character); (d) the configuration has persisted
  for a configurable minimum duration (default ≥ 10 ticks) without
  consolidation. The predicate prefigures spec-081's Warlord
  Trajectory branching.
- **FR-033**: Once any endgame event has been emitted, no further
  endgame events MAY fire in the same run; the existing
  EndgameDetector's "first-fire-wins" priority semantics MUST be
  preserved. Priority order when multiple predicates hold in the
  same tick (most-specific first): `RED_OGV` → `FRAGMENTED_COLLAPSE`
  → `ECOLOGICAL_COLLAPSE` → `FASCIST_CONSOLIDATION` →
  `REVOLUTIONARY_VICTORY` (rationale: `RED_OGV` is a strict subset
  of conditions that would otherwise route to
  `ECOLOGICAL_COLLAPSE`, so it must check first to fire the more
  informative outcome).

**Diagnostics & Player Observability**:

- **FR-034**: The system MUST detect the "Red Settler Trap" pattern
  (a Faction with high `class_reduction` but `colonial_stance ∈
  {UPHOLD, IGNORE}`) and emit a `RED_SETTLER_TRAP_DETECTED`
  diagnostic event.
- **FR-035**: A `DUAL_POWER_ACTIVE` diagnostic MUST be surfaced
  whenever a Territory has more than one CLAIMS edge with
  `control_level > 0.0`.
- **FR-036**: Players MUST be able to observe, per Sovereign:
  current `extraction_policy`, per-tick `metabolic_impact`,
  legitimacy, full claimed-Territory set, and a 20-tick projected
  habitability trajectory for the claimed footprint.
- **FR-037**: Players MUST be able to observe, per Territory: the
  current winning Faction, the second-place Faction, the influence
  margin between them, the currently-installed Sovereign, and the
  effective CLAIMS control_level.

**Initial State & Real-Proxy-Data Bootstrap**:

- **FR-038**: Initial state MUST seed the four canonical Factions
  (Restorationist, Workers' Congress, Decolonial, Liberal-Imperial
  per FR-008) plus three canonical Sovereigns: `SOV_USA_FED`
  (FR-040), `SOV_CAN_FED` (FR-040a), and `SOV_EXTERIOR_NULL`
  (FR-040b — the exterior fallback Sovereign for orphaned-Territory
  resolution).
- **FR-039**: Initial INFLUENCES distributions MUST be derived from
  real proxy data sources loaded via the existing reference data
  pipeline and persisted as one INFLUENCES edge per
  `(faction_id, territory_id)` pair with non-zero proxy value:
  - **FAC_WORKERS_CONGRESS**: union-employment-share per
    county-year computed from QCEW (`own_code='3'` + historically-
    unionized NAICS filter), prorated to res-7 hexes via LODES
    residential density; `support_type=LABOR`.
  - **FAC_DECOLONIAL**: per-hex intersection area with Natural
    Earth AIANNH (American Indian / Alaska Native / Native
    Hawaiian) polygons, normalized to [0, 1]; `support_type=IDEOLOGICAL`.
  - **FAC_RESTORATIONIST**: Republican-vote-share per county
    (most-recent presidential election from MIT Election Lab or
    Census Bureau fixture if catalog amendment pending), prorated
    to res-7 hexes via LODES residential density;
    `support_type=ELECTORAL`.
  - **FAC_LIBERAL_IMPERIAL**: the complement of
    `(restorationist_share + workers_congress_share)` per hex,
    clamped to `[0, 0.4]` so the IGNORE-faction cannot dominate
    without player intervention; `support_type=IDEOLOGICAL`.
  Arbitrary or synthetic-only initial distributions are NOT
  acceptable for the canonical starting state. The seed-data
  format and schema are defined in
  `contracts/seed_influences.schema.json`; the computation pipeline
  is implemented as part of the foundational seeding tasks
  (tasks.md Phase 2).
- **FR-040**: At least one historical Sovereign (`SOV_USA_FED` —
  United States Federal Government) MUST be present at simulation
  start, claiming all in-scope Territories with `legal_status=DE_JURE`
  and `control_level=1.0`. Its `ruling_faction_id` MUST be
  `FAC_RESTORATIONIST` (UPHOLD stance), making `extraction_policy =
  INTENSIFY` and `metabolic_impact = −0.02` the initial per-tick
  effect on all claimed Territories. This grounds the simulation
  start in the MLM-TW thesis that the US settler-colonial state IS
  the Restorationist project; players begin with active urgency
  rather than a neutral buffer state. The `default: "CONTINUE"` in
  the v1.2.0 schema is the SCHEMA default for unruled Sovereigns;
  the INITIAL-STATE seed override is INTENSIFY.
- **FR-040a**: A second canonical Sovereign `SOV_CAN_FED` (Government
  of Canada) MUST be present at simulation start per Constitution
  IV.1 (Detroit-Windsor Boundary Condition). Attributes:
  `sovereignty_type=RECOGNIZED_STATE`, `legitimacy=0.85`,
  `ruling_faction_id=FAC_LIBERAL_IMPERIAL` (IGNORE stance →
  `extraction_policy=CONTINUE`, `metabolic_impact=−0.005`).
  SOV_CAN_FED's initial CLAIMS include the existing Canadian
  boundary node (`canada` per spec-062 R4) plus DE_JURE claims on
  every in-scope US Territory that has a LODES OD-matrix
  `workplace_dest=canada` edge (representing Canadian-firm
  operations in Detroit). This grounds Canada as a first-class
  territorial substrate, not a closed-loop boundary node.
- **FR-040b**: A third canonical Sovereign `SOV_EXTERIOR_NULL`
  (Exterior Fallback) MUST be present at simulation start.
  Attributes: `sovereignty_type=PROVISIONAL`, `legitimacy=0.0`,
  `ruling_faction_id=NULL`, `extraction_policy=CONTINUE` (the
  v1.2.0 schema default for unruled Sovereigns), no
  `capital_territory_id`. SOV_EXTERIOR_NULL initially CLAIMS only
  the existing `rest_of_usa` Territory boundary node (per
  spec-062 R4) at `legal_status=DE_JURE` and `control_level=1.0`.
  Mid-game, SOV_EXTERIOR_NULL is the fallback Sovereign that
  receives any in-scope Territory orphaned by collapse-transition
  with all-zero INFLUENCES — see the "Unclaimed Territory" and
  "All-zero influence" edge cases. The `Sovereign` Pydantic
  validator (per FR-003) MUST permit `ruling_faction_id=NULL`
  alongside `extraction_policy=CONTINUE` as a special-cased
  combination (no stance-policy derivation required when the
  ruling Faction is NULL).

**Pipeline Integration**:

- **FR-041**: The new political-topology dynamics MUST integrate with
  the existing 21-system tick pipeline (per spec-066 ADR044) by
  adding three new Systems at named positions:
  `FactionInfluenceSystem` (after OODA at position 14, before
  Survival at 15), `SovereigntySystem` (after Consciousness at 17,
  before Contradiction at 18), and `CollapseTransitionSystem`
  (after FieldDerivative at 20, before EdgeTransition at 21). The
  audit's suggested half-positions (14.5 / 17.5 / 20.5 — the audit
  originally said "19.5" against an older pipeline snapshot; per the
  current 21-system pipeline the CollapseTransitionSystem position
  is 20.5) are
  guidance; concrete insertion points are an implementation detail
  the plan phase will finalize.
- **FR-042**: Per spec-056, each new System MUST be added to exactly
  one of `MATERIAL_BASE_SYSTEMS`, `ACTION_PHASE_SYSTEMS`, or
  `CONSEQUENCE_SYSTEMS` in `engine/simulation_engine.py`; the
  import-time partition-integrity assertion MUST continue to pass.
  Default classification proposal:
  - `FactionInfluenceSystem` → ACTION_PHASE or CONSEQUENCE (resolved
    by plan phase)
  - `SovereigntySystem` → CONSEQUENCE
  - `CollapseTransitionSystem` → CONSEQUENCE
- **FR-043**: The existing `MetabolismSystem` MUST be extended to
  consume each Sovereign's per-tick `metabolic_impact` and apply it
  to `territory.habitability`. No other System MAY write to
  `metabolic_impact` outside the sovereignty pipeline.
- **FR-044**: All new dynamics (winning-faction argmax, fracture
  partitioning, endgame detection, collapse transition) MUST be
  deterministic given the seed, preserving byte-identical replay per
  spec-069's determinism gate.
- **FR-045**: Entity names in the codebase MUST disambiguate from the
  existing `FactionBalance` / `StateFaction` (state-internal
  ruling-class factionalism per spec-039). The Balkanization Faction
  entity MUST use a distinct name (e.g., `PoliticalFaction` or
  `BalkanizationFaction`); the audit's "Faction" terminology is
  retained in user-facing labels and spec discussion but the
  concrete entity name MUST avoid silent collision with existing
  Faction semantics.

**Chronicle / History**:

- **FR-046**: When CLAIMS or INFLUENCES edges are mutated, the
  system MUST be capable of producing an audit / snapshot row
  (per epoch2-persistence §chronicle, realised on the PostgreSQL
  runtime rather than the original KuzuDB design) so that "who
  controlled X at tick T?" queries are answerable post-hoc.
  The concrete schema (audit table vs JSON-history column vs
  event-stream) is a plan-phase decision; the requirement is the
  post-hoc queryability.

**Player Modes**:

- **FR-047**: System MUST support two player modes selectable at
  run start:
  - `CAMPAIGN`: Player picks one Faction at game start (from the
    three canonical Factions or any seeded alternative). All
    player verbs route through that Faction's vanguard economy
    (spec-072 integration point). Other Factions are NPC-controlled
    via downstream OODA systems.
  - `OBSERVER`: Player has god-mode authority — can boost any
    Faction's INFLUENCES, install Sovereigns directly, force
    CollapseTransitions on any Sovereign, and inspect all
    Sovereign / Faction state including normally-hidden internals.
    Used for sandbox / observation / regression-test scenarios.
- **FR-048**: The selected mode MUST be persisted with the run's
  initial state and be queryable by downstream systems (so spec-072
  Vanguard Economy knows whether to apply CAMPAIGN-mode action
  resolution or skip it for an OBSERVER-mode run).
- **FR-049**: CAMPAIGN-mode player actions on Faction influence
  MUST be subject to the same Vanguard-economy resource costs that
  apply to NPC-controlled Factions (no "free moves"). OBSERVER-mode
  actions MUST bypass Vanguard-economy resource costs and MUST be
  flagged in the audit / chronicle log as observer-mode-mutations
  so they are distinguishable from in-simulation Faction actions.
- **FR-050**: Determinism (FR-044, SC-011) MUST hold within each
  mode separately: re-running a CAMPAIGN-mode run with the same
  seed + same player-Faction-choice + same player-action-sequence
  produces byte-identical replay; OBSERVER-mode runs are
  deterministic given the same observer-mutation sequence.

**Observability Projections (Engine-Side API)**:

- **FR-051**: The engine MUST expose two structured-projection
  functions in the `observers/` layer that satisfy FR-036 + FR-037
  at the API contract level (NOT a UI rendering — UI is deferred
  to spec-042 Game UI Overhaul and spec-085 Tutorial):
  - `observe_sovereign(graph, sovereign_id, horizon_ticks=20) →
    SovereignProjection`: returns a frozen Pydantic
    `SovereignProjection` model containing
    `extraction_policy`, `metabolic_impact`, `legitimacy`,
    `claimed_territory_ids: tuple[str, ...]`, and
    `projected_habitability: tuple[float, ...]` (length =
    horizon_ticks; deterministic extrapolation from current
    metabolic_impact assuming no policy change).
  - `observe_territory(graph, territory_id) → TerritoryProjection`:
    returns a frozen `TerritoryProjection` with
    `winning_faction_id`, `runner_up_faction_id`,
    `influence_margin`, `installed_sovereign_id`,
    `effective_control_level`, and `is_dual_power` (bool).
  Both projections MUST be pure (read-only on the graph, no
  mutations, no I/O) and MUST be exposed through the existing
  `SimulationObserver` protocol. SC-007 + SC-013 are satisfied
  when the projection contracts are testable in `tests/integration/`;
  rendering them to a UI is downstream.

**Principal-Contradiction Cross-Reference**:

- **FR-052**: The settler-vs-anti-settler axis introduced by
  spec-070 (Faction.colonial_stance + Sovereign.extraction_policy
  + ColonialStance enum) operates AS A MODEL of the principal
  contradiction at the political-topology layer, NOT as a
  replacement for the existing dialectical-field-topology engine's
  contradiction-selection mechanism (Constitution I.13 + spec-002).
  Specifically:
  - The `ColonialStance` axis is a *first-class attribute* of
    PoliticalFaction nodes; it is NOT a Dialectic primitive
    (per I.19) and does NOT participate in
    `ContradictionFieldSystem` / `FieldDerivativeSystem`
    (positions 19, 20) dialectical-field computation directly.
  - When a `SOVEREIGN_COLLAPSE` event fires, the colonial-stance
    distribution of the surviving Sovereigns SHOULD be surfaced
    as an event-payload field
    (`competing_colonial_stances: dict[ColonialStance, int]`) so
    downstream principal-contradiction-tracking systems (future
    spec, possibly spec-088 TRPF-activation or a successor) can
    treat the colonial axis as a candidate principal contradiction
    during collapse phases.
  - This spec does NOT modify the principal-contradiction
    selection algorithm. Wiring the colonial-stance axis into
    I.13's selection algorithm is deferred to a follow-up spec
    (target: an engagement spec between Balkanization and
    dialectical-field-topology, post-070).

### Key Entities

- **Faction** (`FAC_{CODE}`): A political-organizational coalition.
  Carries `colonial_stance ∈ {UPHOLD, IGNORE, ABOLISH}`, mechanical
  multipliers (extraction, violence, class, metabolic), and a
  `is_settler_formation` flag. Four canonical starters per FR-008:
  Restorationist Front (UPHOLD, settler), Workers' Congress (IGNORE,
  settler — the RED_OGV trap), Decolonial Front (ABOLISH,
  non-settler), Liberal-Imperial Bloc (IGNORE, settler — soft-imperial
  pole; rules SOV_CAN_FED).
- **Sovereign** (`SOV_{CODE}`): An authority that CLAIMS Territories
  and applies a per-tick `metabolic_impact` to them per its
  `extraction_policy`. Installed by the winning Faction (or NULL for
  the exterior fallback). `sovereignty_type` distinguishes
  recognized-state vs insurgent vs occupation vs secessionist vs
  emergency vs provisional. Three canonical starters per FR-040 /
  FR-040a / FR-040b: SOV_USA_FED (Restorationist-ruled, INTENSIFY,
  hard-start), SOV_CAN_FED (Liberal-Imperial-ruled, CONTINUE,
  Detroit-Windsor boundary), SOV_EXTERIOR_NULL (NULL ruling Faction,
  PROVISIONAL type, CONTINUE policy, exterior-fallback sink for
  orphaned Territories — claims rest_of_usa at start).
- **Territory** (existing entity, extended): Gains an installed-
  Sovereign attribute via incoming CLAIMS edges and surfaces
  winning-Faction via incoming INFLUENCES edges. Habitability dynamics
  gain a sovereign-driven additive term.
- **INFLUENCES edge** (Faction → Territory): Quantifies faction pull.
  `influence_level ∈ [0, 1]`, `support_type` (MATERIAL / IDEOLOGICAL /
  MILITARY / ELECTORAL / LABOR), cadre/sympathizer counts. Sum across
  factions CAN exceed 1.0.
- **CLAIMS edge** (Sovereign → Territory): Asserts authority.
  `control_level ∈ [0, 1]`, `fiscal_status` (TAXED / REVOLT /
  BLOCKADE / LIBERATED / OCCUPIED), `legal_status` (DE_JURE / DE_FACTO
  / DISPUTED / OCCUPIED / CEDED), recognition_level. Sum across
  sovereigns SHOULD NOT exceed 1.0 except transiently.
- **ADMINISTERS edge** (Sovereign → Sovereign): Hierarchical
  delegation; upper-tier sovereign delegates administration to a
  lower-tier sovereign.
- **ColonialStance enum**: `UPHOLD`, `IGNORE`, `ABOLISH` — the
  fundamental political axis.
- **ExtractionPolicy enum**: `INTENSIFY`, `CONTINUE`, `CEASE` —
  derived from ColonialStance.
- **SovereigntyType enum**: `RECOGNIZED_STATE`, `PROVISIONAL`,
  `INSURGENT`, `OCCUPATION`, `SECESSIONIST`, `EMERGENCY`.
- **FiscalStatus enum**: `TAXED`, `REVOLT`, `BLOCKADE`, `LIBERATED`,
  `OCCUPIED`.
- **LegalStatus enum**: `DE_JURE`, `DE_FACTO`, `DISPUTED`, `OCCUPIED`,
  `CEDED`.
- **SupportType enum**: `MATERIAL`, `IDEOLOGICAL`, `MILITARY`,
  `ELECTORAL`, `LABOR`.
- **Event types introduced**: `SOVEREIGN_COLLAPSE`,
  `TERRITORY_TRANSITION`, `FACTION_VICTORY`, `SECESSION_DECLARED`,
  `CIVIL_WAR_DECLARED`, `RED_SETTLER_TRAP_DETECTED`,
  `DUAL_POWER_ACTIVE`, `RED_OGV_ENDGAME`,
  `FRAGMENTED_COLLAPSE_ENDGAME`. (No separate
  `TRUE_LIBERATION_ENDGAME` or `FASCIST_VICTORY_ENDGAME` events —
  those user-facing framings are surfaced via payload metadata on
  the existing `REVOLUTIONARY_VICTORY` and `FASCIST_CONSOLIDATION`
  endgame events respectively.)
- **GameOutcome values added**: `RED_OGV`, `FRAGMENTED_COLLAPSE`
  (exactly two new values; see §"Relationship to Existing
  GameOutcome" for the full mapping).
- **Observability projections** (per FR-051): `SovereignProjection`
  (frozen Pydantic; fields: `extraction_policy`,
  `metabolic_impact`, `legitimacy`, `claimed_territory_ids`,
  `projected_habitability` over a configurable horizon).
  `TerritoryProjection` (frozen Pydantic; fields:
  `winning_faction_id`, `runner_up_faction_id`, `influence_margin`,
  `installed_sovereign_id`, `effective_control_level`,
  `is_dual_power`). These are engine-side projection contracts;
  UI rendering is deferred to spec-042 / spec-085.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Starting from the canonical Detroit tri-county seed,
  the simulation can reach each of the five distinct endgame
  outcomes — `REVOLUTIONARY_VICTORY` (TRUE_LIBERATION framing),
  `RED_OGV`, `ECOLOGICAL_COLLAPSE`, `FRAGMENTED_COLLAPSE`,
  `FASCIST_CONSOLIDATION` (FASCIST_VICTORY framing) — at least
  once across a stochastic ensemble of runs differing only in RNG
  seed and player-equivalent inputs, demonstrating branching is
  genuine rather than degenerate.
- **SC-002**: Across 100 stochastic runs of equal length, each of
  the five outcomes is observed at least once. Given the
  Restorationist-ruled SOV_USA_FED initial state (FR-040), the
  baseline expectation is that `FASCIST_CONSOLIDATION` is the
  modal outcome for zero-player-intervention runs; reaching
  `REVOLUTIONARY_VICTORY` requires sustained player intervention
  in CAMPAIGN mode (or OBSERVER-mode setup) to flip enough
  INFLUENCES toward an ABOLISH-aligned Faction.
- **SC-003**: An `ExtractionPolicy` change on a Sovereign produces
  a detectable change in habitability slope over its claimed
  territories within 5 ticks, with no other intervening event.
- **SC-004**: A sovereign fracture operation completes in time
  consistent with the partition-boundary size only, not with total
  claimed-Territory count — verified empirically by benchmarking
  against N ∈ {10, 100, 1000} with same boundary size.
- **SC-005**: Initial-state seeding loads three Factions with
  proxy-data-derived influence distributions (union density for
  Workers' Congress, AIANNH boundaries for Decolonial, recent
  presidential election results for Restorationist) covering the
  Detroit tri-county footprint such that every in-scope Territory
  has either ≥1 non-zero INFLUENCES edge or is held by the exterior
  fallback Sovereign.
- **SC-006**: A Red-Settler-Trap test scenario (a Faction with
  `class_reduction ≥ 0.6` and `colonial_stance ∈ {UPHOLD, IGNORE}`)
  triggers `RED_SETTLER_TRAP_DETECTED` within 10 ticks of the
  trap-condition becoming true.
- **SC-007**: A player viewing the running simulation can identify
  each Sovereign's current `extraction_policy` and a 20-tick
  projected habitability trend from observable state alone, without
  inspecting source code or internal logs.
- **SC-008**: A change in INFLUENCES large enough to flip the
  winning Faction in a Territory produces a `TERRITORY_TRANSITION`
  event in the same tick that the flip is computed.
- **SC-009**: On `SOVEREIGN_COLLAPSE`, every Territory the collapsed
  Sovereign claimed receives a `TERRITORY_TRANSITION` event in the
  same tick; zero orphaned claims remain after collapse processing.
- **SC-010**: Across the full test ensemble, no run emits more than
  one endgame event; once an endgame fires, subsequent ticks emit
  no further endgame events. The priority order from FR-033 is
  honored in cases where multiple predicates hold simultaneously.
- **SC-011**: Re-running the same seed + same player-equivalent
  inputs produces byte-identical sequences of Faction / Sovereign /
  Territory state mutations and event emissions — determinism is
  preserved consistent with spec-069's byte-identical-replay gate.
- **SC-012**: The `RED_OGV` endgame predicate, when activated in
  a scripted test, produces a player-visible message that explicitly
  names the contradiction between political victory and ecological
  defeat (the spec's theoretical critique surfaces through emergent
  gameplay rather than narration alone). The `REVOLUTIONARY_VICTORY`
  outcome's player-visible message, when the colonial-stance gate
  is satisfied, MUST surface the "TRUE LIBERATION" framing.
- **SC-013**: A sovereign-Territory CLAIMS edge with `control_level
  ∈ {1.0, 0.7, 0.4, 0.1, 0.0}` produces the documented semantic
  state (full control / strong / contested / nominal / symbolic)
  observable in player-facing UI.
- **SC-014**: The three new Systems (`FactionInfluenceSystem`,
  `SovereigntySystem`, `CollapseTransitionSystem`) combined consume
  ≤5% of spec-069's canonical-run per-tick wallclock budget, with
  no single system exceeding 3% — measured across the Detroit
  tri-county footprint at steady state (no collapse-transition
  pending) and verified by the existing benchmark harness.
- **SC-015**: At fracture-event-firing peaks (active secession or
  collapse-transition involving ≥100 territories partitioned in a
  single tick), the combined three-system tick cost may exceed
  the steady-state 5% budget but MUST NOT exceed 15% — fracture
  is a transient cost spike, not a sustained budget.
- **SC-016**: A scripted test scenario satisfying every condition
  of the augmented `REVOLUTIONARY_VICTORY` predicate EXCEPT the
  cross-divide-solidarity requirement (FR-031a) MUST emit `RED_OGV`,
  not `REVOLUTIONARY_VICTORY`, when the EndgameDetector evaluates
  the predicate. A symmetric scenario satisfying ALL conditions
  (including ≥ `revolutionary_victory_min_cross_divide_solidarity_edges`
  cross-divide SOLIDARITY edges) MUST emit `REVOLUTIONARY_VICTORY`
  with the "TRUE LIBERATION" user-facing framing.
- **SC-017**: After initial-state seeding completes
  (postgres_initialization run, FR-038–FR-040b applied), every
  in-scope Territory (Detroit tri-county hex set) MUST satisfy at
  least ONE of the following: (a) has ≥1 INFLUENCES edge with
  `influence_level > 0.0`, or (b) is claimed by `SOV_EXTERIOR_NULL`
  (post-collapse-transition fallback). No in-scope Territory may
  be both un-influenced AND un-claimed at the start of tick 1.

## Assumptions

- **Pipeline integration via spec-066 ADR044**: The 21-system tick
  pipeline established by spec-066 is the integration point. This
  spec adds three new Systems (`FactionInfluenceSystem`,
  `SovereigntySystem`, `CollapseTransitionSystem`) to that pipeline
  at the audit-recommended positions (14.5, 17.5, 19.5).
- **GraphProtocol supports new node/edge types**: The existing
  GraphProtocol (NetworkX adapter with `_node_type` polymorphism)
  supports adding `Sovereign` and `Faction` as new node types and
  `INFLUENCES` / `CLAIMS` / `ADMINISTERS` as new edge types without
  requiring a new graph backend.
- **Scope is Detroit tri-county at MVP with statewide-expansion
  schema readiness**: Initial seed data targets the spec-062 MVP
  footprint. The data shape must accommodate statewide and
  continental expansion without schema change, but the rollout
  itself is out of scope.
- **Real proxy data is available in the existing reference DB**:
  Union density (LODES / QCEW derivations), AIANNH areas (TIGER),
  and recent presidential election results are already loaded into
  `marxist-data-3NF.sqlite` or can be loaded via existing
  ingestion patterns; this spec does NOT introduce new external-data
  ingestion pipelines.
- **Persistence via existing PostgreSQL graph bridge**: Per ADR037
  + spec-061, Sovereign / Faction nodes plus their edges are
  persisted via PostgreSQL runtime using the established node /
  edge JSON-attribute schema. The Chronicle / EdgeSnapshot
  infrastructure from epoch2-persistence.yaml v1.0 (designed for
  KuzuDB) is realised here as PostgreSQL audit / history tables;
  no new database engine is introduced.
- **Endgame detection via existing EndgameDetector observer**: The
  four new endgame pathways are detected by extending the existing
  `EndgameDetector` observer
  (`src/babylon/engine/observers/endgame_detector.py`), not by
  creating a parallel detector. The existing
  `REVOLUTIONARY_VICTORY` / `ECOLOGICAL_COLLAPSE` /
  `FASCIST_CONSOLIDATION` predicates continue to fire; the new
  Balkanization predicates run alongside.
- **Greenfield for Sovereign / Faction / CLAIMS / INFLUENCES / ADMINISTERS**:
  Code at 2026-05-17 contains NO `sovereign.py` or
  political-coalition `faction.py` in `models/entities/`, and the
  `EdgeType` enum has no `CLAIMS` / `INFLUENCES` / `ADMINISTERS`
  values. This spec is fully greenfield for the political-topology
  layer; nothing is "amend an existing class" — it is "add new
  classes that integrate with existing systems".
- **Existing `FactionBalance` is unrelated**: The current
  `FactionBalance` entity + `StateFaction` enum
  (Finance-Capital / Security-State / Settler-Populist) model
  state-internal ruling-class factionalism (spec-039). This spec's
  Faction is a separate, third concept (political coalitions
  contending for territorial sovereignty) and the implementation
  MUST name its entity to avoid collision (see FR-045).
- **Determinism preserved**: All new dynamics (winning-Faction
  argmax with tiebreaker, fracture partitioning, endgame detection)
  are deterministic given the seed, matching the byte-identical-
  replay invariant established by spec-069.
- **Faction-mechanical-multipliers are derivable but overridable**:
  The colonial_stance → multiplier mappings (1.5/2.0/0.0/−0.5 etc.)
  are the canonical defaults; per-Faction seed JSON may override
  individual values to support non-canonical Faction definitions.
- **Tribal sovereignty recognition is in scope at the schema
  level**: The `ADMINISTERS` edge and `SovereigntyType=RECOGNIZED_STATE`
  vs `SECESSIONIST` distinction is sufficient to model federal /
  state / tribal-recognized sovereignty hierarchy. Detailed
  tribal-treaty-rights modeling is deferred (potential future spec).

### Out of Scope

- **Per-Territory ethnonational composition modeling**: Detailed
  demographic-composition mapping beyond proxy-data seeding for
  INFLUENCES is deferred (potential future demographic-cartography
  spec).
- **Kinetic civil-war combat resolution**: Civil war here is the
  topology operation (fracture + edge rewiring + contested-boundary
  resolution + DUAL_POWER_ACTIVE diagnostics). Detailed kinetic
  warfare resolution — force-power, blowback, QRF — is spec-075's
  domain.
- **Faction-internal cohesion dynamics**: Faction is treated as an
  aggregated influence-carrying entity. Internal cohesion mechanics
  (Iron Law of Oligarchy, factionalism within a faction) are
  spec-073's domain.
- **Faction action verbs**: REPRODUCE / EDUCATE / MOBILIZE / ATTACK
  resolvers and the per-action Vanguard Economy resolution are
  spec-072's and spec-075's domain. This spec produces the political
  topology those verbs operate on, not the verbs themselves.
- **Religious-institution field contribution to influence**: The
  ISA-religious-as-field-source mechanic is spec-095's domain;
  initial INFLUENCES seeding here uses only union-density / AIANNH /
  election-result proxy data.
- **Higher-dimensional ideology manifolds beyond colonial_stance**:
  The 3-valued ColonialStance is the v1.2.0 axis; additional
  ideology axes (e.g., authoritarian vs libertarian) are deferred.

## Dependencies

- **spec-037 (Postgres Runtime Database)** — REQUIRED, complete.
  New Sovereign / Faction node schemas and CLAIMS / INFLUENCES /
  ADMINISTERS edge schemas live in existing runtime tables.
- **spec-061 (Real-Backend Wireup)** — REQUIRED, complete. PostgreSQL
  bridge consumed by hex / county hydration.
- **spec-062 (Cross-Scale Integration)** — REQUIRED, complete. The
  `rest_of_usa` exterior boundary node established by R4 is reused
  as the fallback Sovereign for orphaned Territories.
- **spec-066 (Marx Coherence Fixes)** — REQUIRED, complete. The
  21-system tick pipeline established by ADR044 is the integration
  point; new Systems are inserted at named positions within it.
- **spec-069 (SQLite Cache Optimization)** — REQUIRED, complete.
  Byte-identical-replay determinism gate is the invariant this
  spec must preserve.
- **ADR029 (Hybrid Graph Architecture)** — REFERENCE. Establishes
  the v1.1.0 Dynamic Sovereignty principle (sovereignty as edges,
  not properties; O(1) edge rewiring). This spec realises that
  principle on the PostgreSQL graph bridge (per ADR030 succession)
  rather than the original KuzuDB design.

## Downstream Unblocks

Per the audit dependency graph, completion of this spec unblocks:

- **spec-071** (Reactionary Subject) — needs Faction model to
  assign drifted nodes to fascist Faction; provides material for
  Doctrine NATIONAL_CHAUVINISM tag in spec-080.
- **spec-074** (Demographic Crisis & Resolution Pathway selector)
  — needs Faction / Sovereign model for the IMPERIAL pathway's
  biocap-transfer along EXPLOITATION edges.
- **spec-081** (Warlord Trajectory Branching) — needs the
  political-topology + four-endgame substrate for branching
  attribution and Trajectory A / B / C selection.
