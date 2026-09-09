# TVT Physics Engine: Spec-Kit Prompts (v2)

**Purpose**: Prompts for Claude Code + spec-kit to generate specifications for each implementation phase
**Usage**: Feed each prompt to spec-kit after completing the prior phase
**Context**: These prompts assume spec `012-capital-stock-dynamics` is implemented
**Revision**: Incorporates single-MELT + throughput + γ_basket framework

---

## Theoretical Foundation Summary

The revised TVT framework rests on three key insights:

**1. Single National MELT (τ)**
Within a currency zone, there is ONE monetary expression of labor time:
```
τ = GDP_national / L_national ≈ $65/hour (2024)
```

**2. Wages Track Throughput, Not Value Creation**
Workers at supply chain coordination nodes earn proportional to value *flowing through*, not value *created*:
```
W = λ × τ_through
Where:
  τ_through = V_throughput / L_coordination
  λ = wage share (institutional, union power, skill scarcity)
```

**3. The Unified Labor Aristocracy Formula**
Class position depends on production (throughput) AND consumption (imperial subsidy):
```
Labor Aristocracy iff W > τ × γ_basket
                  iff λ × π > γ_basket
Where:
  π = τ_through / τ  (throughput position)
  γ_basket = consumption visibility (imperial subsidy)
```

---

## Phase 2: MELT & Basket Visibility

### Spec ID: `013-melt-basket-visibility`

### Prompt:

```
Create a specification for MELT and Basket Visibility Computation.

CONTEXT:
- MELT (τ) bridges labor-time and money-price: Price = τ × Value
- There is ONE national MELT per currency zone (not location-varying)
- γ_basket measures the imperial subsidy on the consumption basket
- Together, τ and γ_basket determine the Labor Aristocracy threshold

THEORETICAL FOUNDATION (from tvt_mathematical_formalization_v2.md):

**Single National MELT**:
- τ = GDP_national / L_national
- L approximated as: national_employment × 2080 hours/year
- τ is a SCALAR per time period, NOT indexed by location
- Expected range: $55-75/hour for contemporary US

**Why Single MELT?**:
Within a single currency zone:
- No ERDI (Exchange Rate Deviation Index) differential
- No exchange rate compression
- Every dollar commands the same global labor
- Therefore: no price distortion mechanism to create location-varying τ

**Basket Visibility (γ_basket)**:
International γ operates at the border when value enters the US. The basket visibility measures how much the consumption basket benefits from this compression:

```
γ_import = Σ_origin (import_share[origin] × (1/ERDI[origin]))
α = import_share_of_consumption  (from BEA, ~0.25-0.35 for consumer goods)
γ_basket = 1 / [α/γ_import + (1 - α)]
```

When γ_basket < 1, consumption is subsidized by compressed peripheral labor.

**Effective MELT (LA Threshold)**:
```
τ_effective = τ × γ_basket
```

A worker earning W > τ_effective is Labor Aristocracy—they command more global labor-hours through consumption than they expend in production.

REQUIRED OUTPUTS:

**National Parameters (computed annually)**:
- τ[year]: National MELT ($/labor-hour)
- α[year]: Import share of consumption basket
- γ_import[year]: Weighted average import visibility
- γ_basket[year]: Combined basket visibility
- τ_effective[year]: Labor Aristocracy threshold ($/hour)
- V_reproduction: Reproduction floor (~$12/hour, inflation-adjusted)

**Class Position Thresholds**:
| Class | Condition |
|-------|-----------|
| Labor Aristocracy | W > τ_effective |
| Proletariat | τ_effective ≥ W > V_reproduction |
| Subproletariat | W ≤ V_reproduction |

DATA REQUIREMENTS:

**For τ**:
- BEA GDP (national): Have
- QCEW employment (national): Have
- Computation: τ = GDP / (employment × 2080)

**For γ_basket**:
- Penn World Tables 10.0: ERDI by country (NEW LOADER NEEDED)
- Census Trade Data: Import value by origin country (NEW LOADER NEEDED)
- BEA: Consumer goods import share (need to extract)

**For V_reproduction**:
- Census poverty thresholds: ~$12/hour empirical anchor
- BEA Regional Price Parity: For regional adjustment (optional)

MVP APPROACH:
For minimum viable implementation, hardcode:
- γ_basket ≈ 0.68 (estimated from Hickel et al. methodology)
- V_reproduction = $12/hour (2024 dollars)

Compute τ fresh from BEA + QCEW each year.

VALIDATION CRITERIA:
- τ in range $55-75/hour (sanity check)
- γ_basket in range 0.60-0.80
- τ_effective in range $35-55/hour
- Labor aristocracy share ~30-50% nationally

DEPENDENCIES:
- Requires: 012-capital-stock-dynamics
- Data: BEA GDP (national), QCEW (national), Penn World Tables (new)

Reference tvt_mathematical_formalization_v2.md and tvt_data_gaps_v2.md for full theory and data sources.
```

---

## Phase 3: Throughput Position & Domestic Geography

### Spec ID: `014-throughput-position-domestic-geography`

### Prompt:

```
Create a specification for Throughput Position and Domestic Value Geography.

CONTEXT:
- Within the US (single currency zone), there is NO price distortion (everyone spends dollars)
- Domestic core/periphery operates through DIFFERENT mechanisms than international γ
- The key insight: WAGES TRACK THROUGHPUT, NOT VALUE CREATION
- A worker's position in the supply chain funnel determines their wage potential

THEORETICAL FOUNDATION (from tvt_throughput_extension.md):

**The Supply Chain Funnel**:
Value is created at extraction points (depth 0) and flows upward through coordination nodes. At each layer, wages are proportional to ACCUMULATED THROUGHPUT:

```
Depth 0 (extraction):     W ≈ $1-5/hr    (creates value, captures little)
Depth 1 (processing):     W ≈ $10-20/hr
Depth 2 (manufacturing):  W ≈ $20-35/hr
Depth 3 (logistics):      W ≈ $30-50/hr  (port chokepoints)
Depth 4 (retail):         W ≈ $15-25/hr  (high throughput, LOW λ)
Depth 5 (finance):        W ≈ $50-150/hr (highest throughput capture)
```

**Throughput Intensity**:
```
τ_through[fips] = GDP[fips] / L[fips]
```

This is what τ_local measures—NOT a local MELT, but throughput intensity. It tells you how much accumulated value flows through this location per hour of local labor.

**Throughput Position**:
```
π[fips] = τ_through[fips] / τ_national
```

If π > 1: Above-average throughput (coordination chokepoint)
If π < 1: Below-average throughput (value creation/export node)

**The Wage Formula**:
```
W = λ × τ_through

Where λ = wage share (institutional variable):
  - Union density
  - Skill scarcity
  - Bargaining power
  - Historical path-dependence
```

**Why Retail Workers Are Proletariat Despite High Throughput**:
Walmart moves enormous value (high τ_through) but workers capture almost none (λ ≈ 0.05-0.10). The longshoreman has both high τ_through AND high λ (strong union) → Labor Aristocracy.

**The Unified LA Formula**:
```
LA iff W > τ × γ_basket
   iff λ × τ_through > τ × γ_basket
   iff λ × π > γ_basket
```

REQUIRED COMPUTATIONS:

**Throughput Metrics**:
- τ_through[fips, year] = GDP[fips] / (employment[fips] × 2080)
- π[fips, year] = τ_through[fips, year] / τ[year]

**Supply Chain Depth**:
- D[fips, year] = Σ_naics (employment[fips,naics] × depth[naics]) / Σ employment
- Requires NAICS → depth mapping table

**NAICS to Supply Chain Depth Mapping**:
| NAICS | Industry | Depth |
|-------|----------|-------|
| 11 | Agriculture | 0 |
| 21 | Mining | 0 |
| 22 | Utilities | 2 |
| 23 | Construction | 2 |
| 31-33 | Manufacturing | 1-2 |
| 42 | Wholesale | 3 |
| 44-45 | Retail | 4 |
| 48-49 | Transportation | 3 |
| 51 | Information | 4 |
| 52 | Finance | 5 |
| 53 | Real Estate | 5 |
| 54 | Professional Services | 4 |
| 55 | Management | 5 |
| 56 | Admin/Support | 3 |
| 61 | Education | 4 |
| 62 | Healthcare | 4 |
| 71 | Entertainment | 4 |
| 72 | Accommodation/Food | 4 |
| 81 | Other Services | 3 |
| 92 | Government | 4 |

**Wage Share Proxy (λ)**:
Since λ is hard to measure directly, use proxy:
```
λ_proxy[fips, naics] = avg_wage[fips, naics] / τ_through[fips]
```

This captures revealed wage share from actual data.

**Additional Domestic Indicators** (from previous spec):
- NetFlow[fips]: Commuter flow (imports/exports labor)
- OwnershipRatio[fips]: Capital vs labor income
- MeanHours[fips]: Hours access (LA hoards, proletariat rationed)
- ReserveArmyPressure[fips]: Labor discipline (U6, PTER)

COMPOSITE CORE INDEX:
```
CoreIndex[fips] = w1×norm(π) + w2×norm(NetFlow) + w3×norm(OwnershipRatio)
                + w4×norm(MeanHours) - w5×norm(ReserveArmyPressure)
```

DATA REQUIREMENTS:
- BEA GDP by county: Have
- QCEW employment by county by NAICS: Have
- QCEW wages by county by NAICS: Have
- NAICS depth mapping: Create as reference table
- LODES OD files: Need for NetFlow
- ACS income by source: Need for OwnershipRatio
- ACS hours worked: Need for MeanHours
- BLS LAUS: Need for ReserveArmyPressure

VALIDATION CRITERIA (Wayne = 26163, Oakland = 26125):
- π[Oakland] > π[Wayne] (Oakland is coordination node)
- D[Oakland] > D[Wayne] (Oakland has higher average depth)
- NetFlow[Oakland] > 0, NetFlow[Wayne] < 0
- LA_share[Oakland] > LA_share[Wayne]

PREDICTIONS:
- P1: τ_through correlates with supply chain depth D
- P2: High-π counties have more labor aristocracy
- P3: Ports and financial centers have highest τ_through
- P4: λ × π predicts class consciousness better than W alone

DEPENDENCIES:
- Requires: 013-melt-basket-visibility
- Data: BEA (county GDP), QCEW (county employment/wages by NAICS), LODES, ACS, LAUS

Reference tvt_throughput_extension.md for complete theory.
```

---

## Phase 4: Gamma Tensor (International + Reproductive Only)

### Spec ID: `015-gamma-visibility-tensor`

### Prompt:

```
Create a specification for the Gamma (Visibility) Tensor.

CONTEXT:
- γ measures fraction of labor-time that survives transformation to price-space
- CRITICAL REVISION: γ applies ONLY to two mechanisms:
  1. International (ERDI-based compression at borders)
  2. Reproductive (naturalization of domestic labor)
- γ does NOT apply to domestic core/periphery (use π instead—see Phase 3)

THEORETICAL FOUNDATION (from gamma_tensor_theory_v2.md):

**What γ Is**:
- γ = 1: All labor registers in prices (fully visible)
- γ < 1: Some labor is invisible (compressed or naturalized)
- γ is INTENSIVE (weighted-averages under aggregation)

**What γ Is NOT**:
- γ is NOT domestic productivity differences
- γ is NOT "local MELT variation" within a currency zone
- Domestic value geography is captured by π (throughput position), not γ

**Two Distinct Mechanisms**:

**Mechanism A: International (γ_import)**
When goods cross currency zones, ERDI differentials compress peripheral labor:
```
γ[origin→US] = ERDI_US / ERDI_origin = 1 / ERDI_origin

γ_import = Σ_origin (import_share[origin] × γ[origin→US])
```

**Mechanism B: Reproductive (γ_III)**
Reproductive labor is ideologically naturalized as "not real work":
```
γ_III = L_paid_care / (L_paid_care + L_unpaid_care)
```

National estimate: γ_III ≈ 0.25-0.35

TENSOR STRUCTURE:

The gamma tensor has FOUR components, but only γ_III varies domestically:

| Department | γ Value | Source |
|------------|---------|--------|
| I (means of production) | ~1.0 domestic, γ_import for imports | Trade data |
| IIa (wage goods) | ~1.0 domestic, γ_import for imports | Trade data |
| IIb (luxury goods) | ~1.0 domestic, γ_import for imports | Trade data |
| III (reproduction) | γ_III ≈ 0.25-0.35 | ATUS + QCEW |

For domestic production in Depts I/II, γ = 1.0 (no price distortion within currency zone).

**γ_basket (for class position)**:
Combines import visibility with domestic share:
```
γ_basket = 1 / [α/γ_import + (1 - α)]
```

This is computed in Phase 2 and used for class position determination.

DERIVED QUANTITIES:

**Price-Visible Value**:
```
T_price^μ_ν = γ_μ × T^μ_ν × τ
```

**Shadow Value**:
```
S^μ_ν = (1 - γ_μ) × T^μ_ν
Φ_shadow = Σ_μ S^μ_total  (total shadow subsidy)
```

**Two Shadow Subsidies**:
| Subsidy | Formula | Magnitude |
|---------|---------|-----------|
| Imperial | (1 - γ_basket) × consumption | ~$2T/year |
| Reproductive | (1 - γ_III) × T^III_v | ~$2.3T/year |

IMPLEMENTATION:

**γ_III Calculation**:
```python
def compute_gamma_III(
    atus_unpaid_hours: float,  # National from ATUS
    qcew_care_hours: float,    # NAICS 61, 62, 624, 814
) -> float:
    total = atus_unpaid_hours + qcew_care_hours
    return qcew_care_hours / total if total > 0 else 0.0
```

**γ_import Calculation** (requires PWT data):
```python
def compute_gamma_import(
    import_shares: dict[str, float],  # country -> share
    erdi: dict[str, float],           # country -> ERDI from PWT
) -> float:
    return sum(share * (1 / erdi[country])
               for country, share in import_shares.items())
```

**Aggregation Rules**:
γ is intensive—weighted average, not sum:
```
γ[state] = Σ_county (T[county] × γ[county]) / Σ_county T[county]
```

DATA REQUIREMENTS:
- ATUS time use data: Have loader from spec 005
- QCEW care sector employment: Have
- Penn World Tables ERDI: NEW (from Phase 2)
- Census trade data: NEW (from Phase 2)

VALIDATION CRITERIA:
- γ_III in range [0.20, 0.40]
- γ_III correlates with female labor force participation
- γ_import in range [0.40, 0.70]
- Shadow subsidy Φ correlates with profitability

DEPENDENCIES:
- Requires: 013-melt-basket-visibility (for γ_basket components)
- Requires: 005-atus-department-iii (integrate)
- Data: ATUS, QCEW care sectors, PWT, Census Trade

Reference gamma_tensor_theory_v2.md and department_iii_formalization_v2.md.
```

---

## Phase 5: Class Position Engine

### Spec ID: `016-class-position-engine`

### Prompt:

```
Create a specification for the Class Position Classification Engine.

CONTEXT:
- This is the core political output of the TVT physics engine
- Classifies workers/counties into Labor Aristocracy / Proletariat / Subproletariat
- Uses the unified formula from our theoretical framework
- Connects economic quantities to political predictions

THEORETICAL FOUNDATION (from tvt_mathematical_formalization_v2.md):

**The Unified Formula**:
```
Labor Aristocracy iff W > τ × γ_basket
                  iff λ × π > γ_basket

Where:
  W = hourly wage
  τ = national MELT
  γ_basket = consumption visibility (imperial subsidy)
  λ = wage share of throughput
  π = throughput position (τ_through / τ)
```

**Class Thresholds** (2024 estimates):
| Threshold | Symbol | Value | Derivation |
|-----------|--------|-------|------------|
| National MELT | τ | ~$65/hr | GDP / L |
| Basket visibility | γ_basket | ~0.68 | ERDI + trade |
| LA threshold | τ_effective | ~$44/hr | τ × γ_basket |
| Reproduction floor | V_rep | ~$12/hr | Subsistence cost |

**Class Position**:
| Class | Condition | Estimated Share |
|-------|-----------|-----------------|
| Labor Aristocracy | W > τ_effective | ~35% |
| Proletariat | τ_effective ≥ W > V_rep | ~55% |
| Subproletariat | W ≤ V_rep | ~10% |

**Imperial Rent per Hour**:
```
Φ_hour = (W/τ) × (1/γ_basket) - 1
```

If Φ_hour > 0: Worker extracts value from periphery through consumption
If Φ_hour < 0: Worker is net exploited (rare for US workers with γ < 1)
If Φ_hour = 0: Break-even with world-system

**Labor Commanded**:
```
L_commanded = (W/τ) × (1/γ_basket)
```

Hours of global labor commanded per hour worked.

REQUIRED FUNCTIONALITY:

**Individual Classification**:
```python
def classify_worker(
    wage_hourly: float,
    tau_effective: float,
    v_reproduction: float,
) -> ClassPosition:
    if wage_hourly > tau_effective:
        return ClassPosition.LABOR_ARISTOCRACY
    elif wage_hourly > v_reproduction:
        return ClassPosition.PROLETARIAT
    else:
        return ClassPosition.SUBPROLETARIAT
```

**County-Level Classification**:
```python
def classify_county(
    fips: str,
    year: int,
    wage_distribution: WageDistribution,
    params: NationalParameters,
) -> ClassComposition:
    """Returns shares of LA, Proletariat, Subproletariat."""
    la_share = wage_distribution.above(params.tau_effective).sum() / total
    sub_share = wage_distribution.below(params.v_reproduction).sum() / total
    prol_share = 1 - la_share - sub_share
    return ClassComposition(la_share, prol_share, sub_share)
```

**Imperial Rent Calculation**:
```python
def compute_imperial_rent(
    wage: float,
    tau: float,
    gamma_basket: float,
) -> ImperialRent:
    labor_commanded = (wage / tau) * (1 / gamma_basket)
    phi_hour = labor_commanded - 1
    return ImperialRent(
        labor_commanded=labor_commanded,
        phi_hour=phi_hour,
        is_net_extractor=(phi_hour > 0),
    )
```

DATA REQUIREMENTS:
- National parameters: τ, γ_basket, τ_effective, V_rep (from Phase 2)
- Wage distribution by county: OES or ACS (new loader needed)
- QCEW average wages: Have (but need distribution, not just average)

MVP APPROACH:
Use QCEW average wage as point estimate for county classification:
```
If avg_wage[fips] > τ_effective: County is "LA-leaning"
```

This is imprecise but allows progress without full wage distribution data.

VALIDATION CRITERIA:
- LA share nationally ~30-50%
- LA_share[Oakland] > LA_share[Wayne]
- Subproletariat share correlates with undocumented immigrant population
- Class composition correlates with political behavior (voting patterns)

PREDICTIONS:
- P1: Counties with higher π have higher LA share
- P2: Counties with higher D (supply chain depth) have higher LA share
- P3: τ_effective should correlate with historical class consciousness data
- P4: Subproletariat share predicts social instability indicators

DEPENDENCIES:
- Requires: 013-melt-basket-visibility
- Requires: 014-throughput-position-domestic-geography
- Data: Wage distributions (OES/ACS)

This phase produces the core political output. After this, the simulation can classify workers and predict class-based political behavior.
```

---

## Phase 6: Simulation Tick Dynamics

### Spec ID: `017-simulation-tick-dynamics`

### Prompt:

```
Create a specification for Simulation Tick Dynamics.

CONTEXT:
- The simulation advances in discrete time steps (ticks = years)
- State at t+1 depends on state at t plus new inputs
- This is the core "physics engine" that makes the simulation run
- Must implement TSSI temporal determination

THEORETICAL FOUNDATION (from TSSI and TVT docs):

**TSSI Temporal Principle**:
- V[t+1] = P_inputs[t] + L[t→t+1]
- Output value depends on input PRICES (historical cost) plus living labor
- Input prices at t become embedded in output values at t+1

**State Vector per Tick**:

*National Parameters* (computed once per year):
- τ[year]: National MELT
- γ_basket[year]: Basket visibility
- τ_effective[year]: LA threshold
- V_reproduction[year]: Subproletariat threshold

*County-Level State*:
- T^μ_ν[fips, year]: Value tensor (from primitive)
- K[fips, year]: Capital stock
- π[fips, year]: Throughput position
- D[fips, year]: Average supply chain depth
- ClassComposition[fips, year]: LA/Prol/Sub shares
- Domestic indicators: NetFlow, OwnershipRatio, MeanHours, ReserveArmy

*Derived Fields* (recomputed each tick):
- r[fips, year]: Profit rate
- OCC[fips, year]: Organic composition
- e[fips, year]: Exploitation rate

**Update Rules**:

1. Load new data:
   ```
   T^μ_ν[fips, t+1] = hydrate(QCEW[fips, t+1], BEA_ratios[t+1])
   ```

2. Update capital stock:
   ```
   K[fips, t+1] = K[fips, t] × (1 - δ) + Σ_μ c^μ[fips, t+1]
   ```

3. Compute national parameters:
   ```
   τ[t+1] = GDP_national[t+1] / L_national[t+1]
   τ_effective[t+1] = τ[t+1] × γ_basket[t+1]
   ```

4. Update throughput position:
   ```
   π[fips, t+1] = τ_through[fips, t+1] / τ[t+1]
   ```

5. Classify counties:
   ```
   ClassComposition[fips, t+1] = classify(wages[fips], τ_effective[t+1])
   ```

6. Compute derived rates:
   ```
   r[fips, t+1] = s / (K + v)
   OCC[fips, t+1] = c / v
   e[fips, t+1] = s / v
   ```

**Coefficient vs Quantity Distinction**:
- Quantities (T, K, flows) change each tick
- Coefficients (γ_basket, δ) change slowly via α-smoothing:
  ```
  γ[t+1] = α × γ_new + (1-α) × γ[t]
  ```
- α derived from empirical autocorrelation

REQUIRED FUNCTIONALITY:
- SimulationState: Container for all state at time t
- tick(state_t) → state_t+1: Advance simulation one period
- History tracking: Store state snapshots for analysis
- Lazy evaluation: Don't recompute unchanged values

CONSTRAINTS:
- All data flows through tensor layer (no direct DB queries during tick)
- Deterministic: same inputs → same outputs
- Must handle missing data years gracefully

VALIDATION CRITERIA:
- Running 2010-2024 should reproduce historical trajectories
- Profit rate r[t] series should show TRPF tendency
- Class composition should shift during crisis years
- π distribution should be stable (structural)

DEPENDENCIES:
- Requires: All prior phases (012, 013, 014, 015, 016)
- Data: Full 2010-2024 time series loaded

Reference existing simulation architecture in src/babylon/simulation/.
```

---

## Phase 7: Crisis and Devaluation Mechanics

### Spec ID: `018-crisis-devaluation`

### Prompt:

```
Create a specification for Crisis and Devaluation Mechanics.

CONTEXT:
- TRPF creates pressure toward crisis as profit rates decline
- Crisis = discontinuous reset where capital is devalued
- Counter-tendencies (γ < 1 on imports, γ_III < 1 on reproduction) delay crisis
- The simulation needs to model both gradual decline and sudden crisis

THEORETICAL FOUNDATION:

**TRPF with Counter-Tendencies**:
```
r = s / (c + v)
  = e / (OCC + 1)

Where:
  e = s/v (exploitation rate)
  OCC = c/v (organic composition)
```

As OCC rises, r falls—UNLESS counter-tendencies offset:

1. **Increasing e**: Intensify exploitation (limited by resistance)
2. **Cheapening c**: γ_import < 1 reduces constant capital cost
3. **Cheapening v**: γ_III < 1 reduces variable capital cost
4. **Foreign investment**: Export capital to higher-r regions
5. **Devaluation**: Destroy K to restore r

**Crisis Trigger**:
When r falls below threshold despite counter-tendencies:
```
Crisis iff r[t] < r_threshold for N consecutive periods
```

Where:
- r_threshold ≈ 2-3% (minimum viable profit rate)
- N ≈ 2-3 years (persistence requirement)

**Devaluation Mechanism**:
During crisis, capital is destroyed:
```
δ_effective = δ_crisis >> δ_normal

K[t+1] = K[t] × (1 - δ_crisis) + c[t+1]

Where:
  δ_normal ≈ 0.07
  δ_crisis ≈ 0.15-0.25
```

**Class Effects of Crisis**:
- LA share may INCREASE initially (weaker proletariat displaced first)
- Then LA share falls as crisis deepens (LA positions eliminated)
- Subproletariat share increases throughout
- This is the George Jackson bifurcation setup

**Historical Validation**:
- 2008-2010: Should see r decline, crisis trigger, K devaluation
- 2020: COVID shock, should see similar pattern
- 2014-2016 (Detroit specific): Should see local crisis dynamics

REQUIRED FUNCTIONALITY:
- CrisisDetector: Monitor r[t] series for threshold breach
- CrisisEvent: Trigger with parameters (start_t, severity, duration)
- DevaluationMode: Switch δ during crisis period
- CounterTendencyTracker: Monitor γ_basket, γ_III, foreign flows
- RecoveryTransition: Gradual return to normal δ

PARAMETERS TO CALIBRATE:
- r_threshold: Minimum viable profit rate
- N: Consecutive periods below threshold to trigger
- δ_crisis: Crisis-period depreciation rate
- recovery_periods: How long elevated δ persists

VALIDATION CRITERIA:
- Backtest: Model should flag crisis around 2008-2010
- Backtest: Model should flag crisis around 2020
- Post-crisis r should exceed pre-crisis r (profitability restored)
- Class composition should show crisis signature

DEPENDENCIES:
- Requires: 017-simulation-tick-dynamics
- Requires: 012-capital-stock-dynamics
- Data: Full historical series for backtesting
```

---

## Phase 8: Primitive Accumulation & Dispossession

### Spec ID: `019-primitive-accumulation-dispossession`

### Prompt:

```
Create a specification for Primitive Accumulation and Dispossession.

CONTEXT:
- Primitive accumulation is not just historical—it's ONGOING
- Gentrification is internal settler colonialism post-frontier
- Dispossession transfers wealth from proletariat to bourgeoisie/LA
- This mechanism explains how crisis produces proletarianization

THEORETICAL FOUNDATION (from Capital Vol I and TVT docs):

**Ongoing Primitive Accumulation**:
Marx described primitive accumulation as the historical creation of the proletariat through dispossession. But this process continues:
- Foreclosure → homelessness → subproletarianization
- Gentrification → displacement → loss of home equity
- Medical debt → bankruptcy → wealth destruction

**Gentrification as Internal Colonization**:
The same four-node pattern that describes international imperialism applies within the US:
```
{Core, Periphery} × {Bourgeoisie, Proletariat}

Detroit:
- Oakland County = Core (value surfaces here)
- Wayne County = Periphery (value created, extracted)
- Gentrification = value transfer from Wayne → Oakland
```

**Connection to Class Position**:
Dispossession affects V_reproduction (subsistence floor):
- Homeowner: V_rep = operating costs only (~$8/hr)
- Renter: V_rep = rent + operating costs (~$15/hr)
- Foreclosure: V_rep jumps as housing costs increase
- This can push workers from Proletariat → Subproletariat

**Dispossession Events**:
| Event | Mechanism | Class Effect |
|-------|-----------|--------------|
| Foreclosure | Lose home equity | Prol → Sub |
| Eviction | Lose housing stability | Prol → Sub |
| Medical bankruptcy | Lose savings | LA → Prol or Prol → Sub |
| Wage theft | Lose earned income | Intensifies exploitation |

REQUIRED MODELS:

**DispossessionEvent**:
- Type: foreclosure, eviction, bankruptcy, wage_theft
- fips, year
- households_affected
- wealth_destroyed
- class_transitions_triggered

**GentrificationIndex[fips, year]**:
Composite of:
- Housing price acceleration
- Demographic change (race, income)
- Business turnover
- Commute pattern changes

**DispossessionRate[fips, year]**:
- Foreclosure rate (from Eviction Lab, ATTOM)
- Eviction rate
- Bankruptcy rate
- Aggregate measure of wealth destruction

VALIDATION CRITERIA:
- DispossessionRate should spike during 2008-2012 crisis
- GentrificationIndex[inner Detroit] should rise post-2014
- DispossessionRate correlates with Subproletariat share increase
- Wealth destroyed should correlate with subsequent π changes

DEPENDENCIES:
- Requires: 016-class-position-engine
- Requires: 018-crisis-devaluation
- Data: Eviction Lab, ATTOM/CoreLogic, county records

Reference capital_volume_i_integration.md section on Primitive Accumulation.
```

---

## Phase 9: D-P-D' Lifecycle Circuit

### Spec ID: `020-dpd-prime-lifecycle-circuit`

*(Content remains largely the same as original—lifecycle dynamics are orthogonal to MELT/throughput revision)*

---

## Usage Notes

1. **Run in order**: Each phase depends on prior phases
2. **Validate between phases**: Run falsification tests before proceeding
3. **Reference v2 docs**: Use the revised TVT docs (tvt_mathematical_formalization_v2.md, etc.)
4. **Check existing code**: Each prompt references relevant existing modules
5. **Iterate**: If spec-kit output is incomplete, refine prompt with specifics

## Validation Gates

| After Phase | Validation |
|-------------|------------|
| Phase 2 | τ in $55-75 range, γ_basket in 0.6-0.8 range |
| Phase 3 | π[Oakland] > π[Wayne], D correlates with wages |
| Phase 5 | LA share ~30-50% nationally, Oakland > Wayne |
| Phase 6 | Historical trajectory reproduced 2010-2024 |
| Phase 7 | Crisis flagged at 2008-2010 and 2020 |

## Key Theoretical Formulas

```
τ = GDP_national / L_national              (National MELT)
γ_basket = 1 / [α/γ_import + (1-α)]        (Basket visibility)
τ_effective = τ × γ_basket                 (LA threshold)
τ_through = GDP_local / L_local            (Throughput intensity)
π = τ_through / τ                          (Throughput position)
W = λ × τ_through                          (Wage determination)
Φ_hour = (W/τ)(1/γ_basket) - 1             (Imperial rent/hour)

Class position:
  W > τ_effective        → Labor Aristocracy
  τ_effective ≥ W > V_rep → Proletariat
  W ≤ V_rep              → Subproletariat

Equivalent throughput form:
  λ × π > γ_basket       → Labor Aristocracy
```

## Key Project Files (v2)

```
docs/
├── tvt_mathematical_formalization_v2.md   # Revised axioms
├── tvt_throughput_extension.md            # Supply chain theory
├── tvt_data_gaps_v2.md                    # Data requirements
├── tvt_quick_reference.md                 # One-page formula summary
├── gamma_tensor_theory_v2.md              # Revised γ (intl + repro only)
├── department_iii_formalization_v2.md     # Reproductive labor
├── capital_volume_integration_memo.md     # Bridge to Capital concepts
└── [existing Capital volume docs]

src/babylon/economics/
├── tensor.py          # ValueTensor4x3
├── hydrator.py        # MarxianHydrator
├── reproduction.py    # Imperial rent
└── shadow_labor.py    # Shadow labor visibility
```

## Implementation Roadmap Summary (Revised)

| Phase | Spec ID | Core Deliverable | Unlocks |
|-------|---------|------------------|---------|
| 1 | 012-capital-stock-dynamics | K, r, OCC, e | TRPF testing |
| 2 | 013-melt-basket-visibility | τ, γ_basket, τ_effective | LA threshold |
| 3 | 014-throughput-position | π, D, λ_proxy, CoreIndex | Domestic geography |
| 4 | 015-gamma-visibility-tensor | γ_III, γ_import, Φ_shadow | Shadow subsidy |
| 5 | 016-class-position-engine | ClassComposition, Φ_hour | Political output |
| 6 | 017-simulation-tick-dynamics | SimulationState, tick() | Simulation runs |
| 7 | 018-crisis-devaluation | CrisisDetector, DevaluationMode | Full cycle |
| 8 | 019-primitive-accumulation | DispossessionEvent, GentrificationIndex | Gentrification |
| 9 | 020-dpd-prime-lifecycle | DPDState, LegitimationIndex | Intergenerational |

After Phase 9, the physics engine is complete. Next:
- Network topology (solidarity edges)
- George Jackson bifurcation
- Hexagonal map visualization
- Game/simulation UI
