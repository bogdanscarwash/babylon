# Spec Prompt: Magic Constants Audit & Derivation Pipeline

**Feature**: 023-constants-provenance-audit
**Priority**: HIGH
**Dependencies**: Feature 017 (Tick Dynamics), Feature 002 (Dialectical Field Topology), Feature 021 (Capital Volume I)

---

## Problem Statement

`src/babylon/config/defines.py` centralizes ~80+ numerical constants across 18 subsection models (CrisisDefines, EconomyDefines, SolidarityDefines, ConsciousnessDefines, TerritoryDefines, TopologyDefines, etc.). Centralization was the right first step. The problem is provenance: most of these constants have no derivation path. Constitution Article III.1 ("No Magic Constants") requires every number to trace to primitives or real data. The current codebase is in systematic violation of this principle.

This is not a cosmetic issue. When `consciousness.sensitivity = 0.5` governs how quickly 2 million simulated people radicalize, the difference between 0.4 and 0.6 is the difference between "Detroit stays quiet" and "Detroit burns." An ungrounded constant is an ungrounded prediction. The simulation's claim to empirical testability depends on eliminating these.

## Scope

This is a **research-first investigation feature**, not an implementation sprint. The deliverable is a complete audit with classification, derivation paths, and a phased remediation plan — not code changes (yet).

## Investigation Requirements

### Phase 0: Exhaustive Census

**Objective**: Produce a complete inventory of every numerical constant in the simulation.

Search the entire codebase — not just `defines.py`. Constants hide in:

1. `src/babylon/config/defines.py` and `src/babylon/data/defines.yaml` — the canonical location
2. `src/babylon/formulas/constants.py` — re-exported constants
3. Inline literals in system implementations (`src/babylon/engine/systems/`, `src/babylon/engine/*.py`)
4. Test fixtures that encode assumptions (`tests/constants.py`, test files with hardcoded expected values that implicitly define parameters)
5. Threshold values in compound predicates (Feature 002 edge transition conditions)
6. Default arguments in function signatures throughout `src/babylon/`
7. Comments marked `# STUB`, `# TODO`, `# PLACEHOLDER`, `# MAGIC` or similar

For each constant found, record:
- **Location**: file path + line number
- **Current value**: the number
- **Stated purpose**: from docstring/comment/Field description (if any)
- **Consumers**: which systems/functions read this value (trace the dependency graph)

Produce the inventory as a structured data file (YAML or CSV), not prose.

### Phase 1: Classification

Classify every constant from the Phase 0 inventory into exactly one of five tiers:

#### Tier A: Tensor-Derivable (SHOULD NOT BE CONSTANTS)

These are quantities the existing tensor/graph architecture can already compute, making the constant redundant. The tensor pipeline (QCEW → ValueTensor4x3 → derived rates) and the graph topology (NetworkX solidarity/extraction edges) contain the information these constants attempt to approximate.

**Investigation methodology for each candidate**:
1. Identify what quantity the constant is trying to capture
2. Trace whether that quantity exists (or could exist) as a derived field from the ValueTensor4x3, CountyEconomicState, or the NetworkX graph
3. If derivable: specify the exact derivation formula using existing model fields
4. If derivable but infrastructure is missing: specify what's missing (a calculator, a data source, a tensor field)

**Expected candidates** (verify — do not take this list as exhaustive):
- `economy.trpf_coefficient` — the tensor pipeline computes profit rates from c/v/s directly
- `economy.trpf_efficiency_floor` — counter-tendencies should produce floors emergently
- `economy.rent_pool_decay` — imperial rent depletion should follow extraction dynamics
- `solidarity.superwage_impact` — computable as Φ = W_core - V_core from the tensor
- `consciousness.sensitivity` and `consciousness.decay_lambda` — per the Dialectical Consciousness Model design doc, these should be replaced by tensor integration (cumulative extraction, immiseration, invisibility)
- `tension.accumulation_rate` — tension from wealth gaps should emerge from tensor differentials
- `territory.displacement_rate` — Census/ACS migration data exists
- `territory.rent_spike_multiplier` — market data during eviction waves
- `working_day.absolute_visibility` / `relative_visibility` — Department III g₃₃ framework replaces these
- `survival.base_mortality_factor` / `inequality_impact` — CDC WONDER + Census derivable

#### Tier B: Eliminable (Dead or Redundant)

Constants that serve no current purpose, are consumed by dead code, or duplicate information available elsewhere.

**Investigation methodology**:
1. Trace all consumers of each constant
2. If zero consumers: dead constant
3. If consumers are themselves dead/deprecated code: dead constant
4. If the constant duplicates a quantity computed elsewhere: redundant

#### Tier C: Genuine Calibration Parameters (KEEP — require sweep infrastructure)

These are irreducible parameters that the simulation genuinely needs, whose values can only be determined through calibration against historical data or sensitivity analysis. They represent real theoretical unknowns.

**Criteria for Tier C**:
- The quantity cannot be derived from existing primitives or data sources
- The quantity has a clear physical/theoretical interpretation
- The quantity's value materially affects simulation outcomes (i.e., it's not just an engineering tolerance)
- A calibration methodology exists or can be specified (historical data source, sensitivity range, acceptance criterion)

**For each Tier C constant, document**:
- Theoretical meaning (what does this number represent in the real world?)
- Sensitivity: how much do simulation outcomes change across the plausible range?
- Calibration source: what historical data or empirical finding constrains this?
- Recommended sweep range for parameter search
- Whether existing tooling (`mise run analysis:optuna`, `mise run analysis:sobol`, `mise run analysis:sweep`) can calibrate it

**Expected candidates** (verify):
- `crisis.r_threshold` — what profit rate constitutes crisis (Piketty/WID constrains)
- `crisis.n_consecutive` — how many bad quarters before crisis onset (NBER recession dating)
- `reserve_army.sigmoid_k` and `sigmoid_r0` — Phillips curve steepness/midpoint (BLS calibratable)
- `struggle.jackson_threshold` — George Jackson bifurcation threshold (historical network density data)
- `topology.gaseous_threshold` / `condensation_threshold` — percolation theory provides guidance per lattice type
- `carceral.control_capacity` — BJS provides this directly
- The five thresholds identified in the Dialectical Consciousness Model design doc: RUPTURE_THRESHOLD, SHARED_THRESHOLD, COLLECTIVE_THRESHOLD, SOLIDARITY_LABOR_COEFFICIENT, JACKSON_THRESHOLD

#### Tier D: Engineering/Precision Constants (KEEP — not subject to calibration)

Pure implementation choices that don't affect theoretical outcomes.

**Criteria**: changing the value within reasonable range does not alter simulation dynamics, only numerical precision or performance.

**Expected candidates**:
- `precision.decimal_places`, `precision.epsilon`, `precision.comparison_epsilon`
- `precision.rounding_mode`
- `timescale.tick_duration_days`, `timescale.weeks_per_year`
- `behavioral.loss_aversion_lambda` (external empirical finding — Kahneman-Tversky, not ours to calibrate)
- `crisis.crisis_period_ticks = 13` (engineering choice — prime for desync)

#### Tier E: Game Design Knobs (KEEP — acknowledged fabrications)

**This is a simulation game, not a federal statistical model.** Some constants exist because the game needs them to produce interesting dynamics, and deriving them from real data would be excruciatingly painful, infeasible, or simply not worth the effort. These are legitimate.

**Criteria for Tier E**:
- The constant governs gameplay feel, pacing, or balance rather than a falsifiable theoretical claim
- Real-world data either doesn't exist, would require a PhD dissertation to extract, or would produce a number no more trustworthy than informed game design intuition
- The constant is clearly documented as a design choice (not smuggled in as if it were empirically grounded)
- Changing the value produces different gameplay experiences but doesn't invalidate the simulation's theoretical claims

**The key distinction**: A Tier E constant is honest about being made up. A magic number pretends to be derived. The sin isn't fabrication — it's fabrication disguised as science. If a constant is Tier E, label it as such in the code (`# GAME_DESIGN: tuned for pacing, no empirical basis`) so nobody wastes time trying to derive it later.

**Expected candidates** (verify — some of these may turn out to be Tier C instead):
- Bourgeoisie policy deltas (wage/repression adjustments) — unless the Organization-as-Agent architecture replaces them entirely
- `struggle.solidarity_gain_per_uprising` — real-world "solidarity produced per riot" is not a measurable quantity
- `struggle.wealth_destruction_rate` — property damage estimates exist but the mapping to simulation units is arbitrary
- `territory.clarity_profile_coefficient` — unclear what this even means materially; may be pure game feel
- Phase staggering delays in `CarceralDefines` (decomposition_delay, control_ratio_delay) — pacing knobs
- Some of the endgame thresholds — win/loss conditions are game design, not science

### Phase 2: Cross-Reference Against Data Sources

For every Tier A and Tier C constant, cross-reference against the Constitution's approved data source table (Article III.4):

| Source | Available Data |
|--------|---------------|
| QCEW | Labor hours by industry/county |
| Census/ACS | Demographics, income, migration |
| BEA | GDP, input-output tables |
| FRED | Macro indicators, time series |
| HIFLD | Infrastructure, critical facilities |
| BTS | Freight flows |
| FCC | Communications infrastructure |
| ATUS | Time use (reproductive labor) |
| CDC WONDER | Mortality (structural violence) |
| Piketty/WID | Wealth distribution, r calibration |
| PWT | ERDI, PPP (unequal exchange) |
| Eviction Lab | Eviction filing rates |
| US Courts | Bankruptcy filings |
| ATTOM/CoreLogic | Foreclosure rates |
| Fed SCF | Savings rates by income bracket |

For each constant: does a data source exist that could replace or constrain it? If yes, specify the derivation path. If no, document that the constant is genuinely unconstrained and note whether new data sources should be added to the approved list.

### Phase 3: Dependency Graph

Produce a dependency analysis showing which constants are consumed by which systems, and which systems would be affected by removing/replacing each constant. This should identify:

1. **Cascade risks**: constants where replacement would require changes across multiple systems
2. **Isolated constants**: constants consumed by a single system (easy wins for replacement)
3. **Coupled clusters**: groups of constants that are consumed together and should be addressed as a unit

Format: Mermaid dependency graph + summary table.

### Phase 4: The Bourgeoisie Policy Section (Special Attention)

The `EconomyDefines` section contains a cluster of policy-delta constants (`bribery_wage_delta`, `austerity_wage_delta`, `iron_fist_repression_delta`, `crisis_wage_delta`, `crisis_repression_delta`) that govern bourgeoisie NPC decisions. These are particularly suspect because:

1. They're flat deltas applied to state variables, not derived from any theoretical framework
2. The thresholds they respond to (`bribery_tension_threshold`, `iron_fist_tension_threshold`) operate on an undefined "tension" scale
3. The entire decision model may be superseded by the Organization-as-Agent architecture (Constitution I.16-I.17)

Investigate: should this entire subsection be flagged for architectural replacement rather than constant-by-constant remediation?

### Phase 5: The Territory Section (Special Attention)

`TerritoryDefines` contains the densest cluster of ungrounded constants in the codebase (~15 parameters). Many of these encode spatial dynamics that should derive from the graph topology and Census/ACS data:

- Heat dynamics (decay, gain, spillover, thresholds) — should these be field derivative operations on the contradiction field (Feature 002)?
- Displacement routing (priority modes, thresholds for elimination/containment) — should these derive from the carceral system's state rather than be stipulated?
- `concentration_camp_decay_rate = 0.2` — CDC WONDER mortality data exists for exactly this purpose

Investigate whether TerritoryDefines can be substantially collapsed by wiring it to Feature 002's field topology.

## Deliverables

1. **`reports/constants-inventory.yaml`** — Phase 0 exhaustive census (structured data, not prose)
2. **`reports/constants-classification.md`** — Phase 1 five-tier classification with reasoning for each constant
3. **`reports/constants-data-sources.md`** — Phase 2 cross-reference against approved data sources
4. **`reports/constants-dependency-graph.md`** — Phase 3 dependency analysis with Mermaid diagrams
5. **`reports/constants-remediation-plan.md`** — Phased remediation plan ordering constants by:
   - Impact (how much simulation behavior changes)
   - Difficulty (how many systems must change)
   - Data readiness (whether the replacement data source already exists in the pipeline)

## Constraints

- **Read-only investigation**: This feature produces reports, not code changes. Code changes come in follow-up features scoped by the remediation plan.
- **Constitution compliance**: Every recommendation must be checked against Article III methodology constraints.
- **No new magic**: If the investigation reveals that replacing a magic constant requires introducing a *different* magic constant (e.g., a normalization bound), flag this explicitly rather than hiding the problem. If the replacement is genuinely infeasible and the constant should be Tier E, say so — honest fabrication is better than laundered fabrication.
- **Scope boundary**: Do not investigate constants in test fixtures unless they implicitly define simulation parameters. Test-only engineering constants (tolerances, fixture sizes) are out of scope.

## Success Criteria

- SC-001: Every constant in `defines.py` and `defines.yaml` appears in the inventory (zero omissions)
- SC-002: Every constant has exactly one tier classification (A/B/C/D/E) with documented reasoning
- SC-003: Every Tier A constant has a specific derivation formula or a documented infrastructure gap
- SC-004: Every Tier C constant has a calibration source and recommended sweep range
- SC-005: The dependency graph identifies at least the top 5 highest-impact replacement targets
- SC-006: The remediation plan provides a sequenced implementation order that respects system dependencies
- SC-007: Every Tier E constant is explicitly labeled as a game design choice with a comment explaining why real data is infeasible or unnecessary

## Context Documents

The investigator should read these before beginning:

- `specify/memory/constitution.md` and `specify/memory/constitution/article-iii-methodology.md` — the rules
- `src/babylon/config/defines.py` — the target
- `src/babylon/data/defines.yaml` — the YAML source
- `src/babylon/formulas/constants.py` — re-exported constants
- The Dialectical Consciousness Model design doc (pasted in project knowledge) — the theoretical framework for replacing consciousness/solidarity constants
- `specs/017-simulation-tick-dynamics/spec.md` — the tick pipeline that consumes most constants
- `specs/002-dialectical-field-topology/spec.md` — the field topology that should replace several territory constants
- `specs/021-capital-volume-i/spec.md` — reserve army and dispossession constants
- `CLAUDE.md` — project conventions and commands
