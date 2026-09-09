# TVT Physics Engine: Spec-Kit Prompts (v3)

**Purpose**: Prompts for Claude Code + spec-kit to generate specifications for each implementation phase
**Usage**: Feed each prompt to spec-kit after completing the prior phase
**Context**: These prompts assume spec `012-capital-stock-dynamics` is implemented
**Revision**: Incorporates wealth-based class position + precarity framework

---

## Theoretical Foundation Summary

The revised TVT framework rests on four key insights:

**1. Single National MELT (τ)**
Within a currency zone, there is ONE monetary expression of labor time:
```
τ = GDP_national / L_national ≈ $65/hour (2024)
```

**2. Class Position = Wealth Percentile (Stock), Not Income (Flow)**
Class is determined by accumulated wealth, not income rate:

| Class | Wealth Percentile | Pop Share | Wealth Share |
|-------|-------------------|-----------|--------------|
| Bourgeoisie | Top 1% | 1% | ~33% |
| Petit Bourgeoisie | 90th-99th | 9% | ~33% |
| Labor Aristocracy | 50th-90th | 40% | ~33% |
| Proletariat | Bottom 50%, stable | ~35% | ~0% |
| Lumpenproletariat | Bottom 50%, precarious | ~15% | ~0% |

**3. Imperial Rent (Φ_hour) is Separate from Class**
A proletarian can have Φ_hour > 0 (benefit from cheap imports) while remaining proletarian. They *consume* the imperial subsidy rather than *accumulating* it as wealth.

**4. Proletariat/Lumpen Distinction = Precarity**
The lumpenproletariat are those excluded from stable participation in the formal labor market. Operationalized via: U-6 rate, PTER, discouraged workers, incarceration, disability without accommodation.

---

## Phase 2: MELT & Imperial Rent

### Spec ID: `013-melt-imperial-rent`

### Prompt:

```
Create a specification for MELT and Imperial Rent Computation.

CONTEXT:
- MELT (τ) bridges labor-time and money-price: Price = τ × Value
- There is ONE national MELT per currency zone (not location-varying)
- Imperial rent (Φ_hour) measures extraction rate through consumption
- CRITICAL: Φ_hour is SEPARATE from class position (see Phase 3)

THEORETICAL FOUNDATION:

**Single National MELT**:
- τ = GDP_national / L_national
- L approximated as: national_employment × 2080 hours/year
- τ is a SCALAR per time period, NOT indexed by location
- Expected range: $55-75/hour for contemporary US

**Basket Visibility (γ_basket)**:
International γ operates at the border when value enters the US:

```
γ_import = Σ_origin (import_share[origin] × (1/ERDI[origin]))
α = import_share_of_consumption  (~0.25-0.35 for consumer goods)
γ_basket = 1 / [α/γ_import + (1 - α)]
```

γ_basket ≈ 0.68 (empirically grounded in ERDI data, not tuned to produce target LA share)

**Imperial Rent Per Hour**:
```
τ_effective = τ × γ_basket
Φ_hour = (W/τ) × (1/γ_basket) - 1
```

Φ_hour measures how much value a worker extracts through consumption per hour worked:
- Φ_hour > 0: Worker extracts value (consumption subsidized)
- Φ_hour < 0: Worker is net exploited
- Φ_hour = 0: Break-even at W = τ_effective

**CRITICAL DISTINCTION**:
Φ_hour is a FLOW measure (extraction rate). Class position is a STOCK measure (accumulated wealth).
A proletarian earning $50/hour has Φ_hour > 0 but remains proletarian if they have no accumulated wealth.
They consume the imperial subsidy rather than accumulating it as property.

REQUIRED OUTPUTS:

**National Parameters (computed annually)**:
- τ[year]: National MELT ($/labor-hour)
- α[year]: Import share of consumption basket
- γ_import[year]: Weighted average import visibility
- γ_basket[year]: Combined basket visibility (default 0.68 for MVP)
- τ_effective[year]: Imperial rent threshold ($/hour)

**Imperial Rent Calculation**:
- Φ_hour(W): Imperial rent per hour for wage W
- Φ_aggregate: Σ(Φ_hour × hours) across population — for Hickel validation

DATA REQUIREMENTS:

**For τ**:
- BEA GDP (national): Have
- QCEW employment (national): Have
- Computation: τ = GDP / (employment × 2080)

**For γ_basket (MVP)**:
- Hardcode γ_basket = 0.68
- Later: Penn World Tables ERDI + Census Trade Data

VALIDATION CRITERIA:
- τ in range $55-75/hour (sanity check)
- γ_basket in range 0.60-0.80
- τ_effective in range $35-55/hour
- Φ_aggregate ≈ $10T+/year (Hickel-scale drain validation)
- Mean wage (~$35/hour) produces Φ_hour > 0 (validates aggregate extraction)
- Median wage (~$28/hour) produces Φ_hour < 0 (validates not everyone extracts)

DEPENDENCIES:
- Requires: 012-capital-stock-dynamics
- Data: BEA GDP (national), QCEW (national)

Reference tvt_mathematical_formalization_v2.md for full theory.
```

---

## Phase 3: Wealth-Based Class Position

### Spec ID: `014-wealth-based-class-position`

### Prompt:

```
Create a specification for Wealth-Based Class Position Classification.

CONTEXT:
- Class position is determined by WEALTH PERCENTILE (stock), not income (flow)
- This is orthodox Marxism: class = relation to property
- The 40% LA share emerges naturally from wealth distribution, not parameter tuning
- Imperial rent (Φ_hour) is calculated separately and does NOT determine class

THEORETICAL FOUNDATION:

**The Pareto Pattern (empirically stable)**:
US wealth distribution follows a consistent pattern:
- Top 1% owns ~33% of wealth
- Next 9% (90th-99th) owns ~33% of wealth
- Next 40% (50th-90th) owns ~33% of wealth
- Bottom 50% owns ~0% (often negative net worth)

**Class Structure**:

| Class | Wealth Percentile | Pop Share | Defining Feature |
|-------|-------------------|-----------|------------------|
| Bourgeoisie | ≥99th | 1% | Owns means of production |
| Petit Bourgeoisie | 90th-99th | 9% | Small capital, professional-managerial |
| Labor Aristocracy | 50th-90th | 40% | Positive net wealth, stake in system |
| Proletariat | <50th, stable employment | ~35% | No net wealth, sells labor regularly |
| Lumpenproletariat | <50th, precarious/excluded | ~15% | Excluded from stable labor market |

**Why Wealth, Not Income**:
- A $60k/year worker with no savings ≠ a $45k/year worker with home equity
- Class is about ACCUMULATED extraction, not flow rate
- Home ownership is the primary wealth vehicle for the 50th-90th bracket
- Dispossession (foreclosure) = wealth destruction → class transition

**The Proletariat/Lumpen Distinction = Precarity**:
The lumpenproletariat are those excluded from stable participation in the formal labor market:
- Not just "unemployed" — that's too narrow
- Precarity = instability of relation to labor market
- Key populations: discouraged workers, incarcerated, chronically underemployed, undocumented with suppressed wages

REQUIRED FUNCTIONALITY:

**ClassPosition Enum (5 values)**:
```python
class ClassPosition(Enum):
    BOURGEOISIE = auto()           # Top 1% wealth
    PETIT_BOURGEOISIE = auto()     # 90th-99th percentile
    LABOR_ARISTOCRACY = auto()     # 50th-90th percentile
    PROLETARIAT = auto()           # Bottom 50%, stable employment
    LUMPENPROLETARIAT = auto()     # Bottom 50%, precarious/excluded
```

**PrecarityStatus Enum**:
```python
class PrecarityStatus(Enum):
    STABLE = auto()              # Regular W-2, predictable hours
    PRECARIOUS = auto()          # Employed but unstable (gig, PTER)
    MARGINALLY_ATTACHED = auto() # Want work, not currently searching
    EXCLUDED = auto()            # Discouraged, incarcerated, disabled w/o accommodation
```

**Classification Methods**:
```python
def classify_by_wealth_percentile(wealth_percentile: float) -> ClassPosition:
    """Classify by national wealth percentile (0-100).

    For bottom 50%, defaults to PROLETARIAT.
    Use classify_by_wealth_and_precarity for full classification.
    """
    if wealth_percentile >= 99:
        return ClassPosition.BOURGEOISIE
    elif wealth_percentile >= 90:
        return ClassPosition.PETIT_BOURGEOISIE
    elif wealth_percentile >= 50:
        return ClassPosition.LABOR_ARISTOCRACY
    else:
        return ClassPosition.PROLETARIAT  # Default; refine with precarity

def classify_by_wealth_and_precarity(
    wealth_percentile: float,
    precarity: PrecarityStatus
) -> ClassPosition:
    """Full classification using wealth + precarity."""
    if wealth_percentile >= 50:
        return classify_by_wealth_percentile(wealth_percentile)
    elif precarity in (PrecarityStatus.STABLE, PrecarityStatus.PRECARIOUS):
        return ClassPosition.PROLETARIAT
    else:
        return ClassPosition.LUMPENPROLETARIAT
```

DATA REQUIREMENTS:

**National (Fed SCF)**:
- Wealth percentile thresholds (triennial)
- Net worth distribution
- Home equity distribution

**County-Level Proxies**:

| Indicator | Source | Operationalizes |
|-----------|--------|-----------------|
| Home ownership rate | ACS B25003 | LA share proxy |
| Median home value | ACS B25077 | Wealth magnitude |
| Investment income share | IRS SOI | Petit bourgeoisie proxy |
| U-6 unemployment | BLS LAUS | Precarity (broad) |
| PTER rate | ACS B23023 | Precarity (underemployment) |
| NILF want work | ACS B23005 | Precarity (discouraged) |
| Incarceration rate | BJS / Vera | Exclusion |
| Disability employment gap | ACS C18120 | Exclusion |

**County-Level Estimation**:
```python
def estimate_la_share(fips: str, year: int) -> float:
    """Estimate Labor Aristocracy share from home ownership proxy."""
    homeownership_rate = get_acs_homeownership(fips, year)
    # ~65% national homeownership, but ~30% underwater or minimal equity
    equity_factor = 0.6  # Calibrate from SCF
    return homeownership_rate * equity_factor

def estimate_lumpen_share(fips: str, year: int) -> float:
    """Estimate lumpenproletariat share from precarity indicators."""
    u6_rate = get_u6_unemployment(fips, year)
    u3_rate = get_u3_unemployment(fips, year)
    nilf_want_work = get_nilf_want_work_rate(fips, year)
    incarceration_rate = get_incarceration_rate(fips, year)

    # Weight toward hard exclusion over soft precarity
    lumpen_share = (
        0.4 * nilf_want_work +
        0.3 * (u6_rate - u3_rate) +
        0.2 * incarceration_rate +
        0.1 * pter_rate * 0.5
    )
    return lumpen_share
```

VALIDATION CRITERIA:
- LA share = 40% nationally (by definition, 50th-90th percentile)
- Proletariat + Lumpen = 50% nationally (bottom half)
- Lumpen share ~10-20% nationally (validate against BLS data)
- Home ownership rate correlates with LA share proxy (r > 0.7)
- Oakland LA_share > Wayne LA_share (Oakland has higher homeownership + values)

FALSIFICATION:
- If wealth percentile thresholds don't produce coherent class structure
- If precarity indicators don't distinguish proletariat/lumpen meaningfully
- If county proxies fail to correlate with national SCF patterns

DEPENDENCIES:
- Requires: 013-melt-imperial-rent
- Data: Fed SCF (national), ACS (county), BLS LAUS (county)

Reference tvt_mathematical_formalization_v2.md Axiom Group E for theory.
```

---

## Phase 4: Throughput Position & Domestic Geography

### Spec ID: `015-throughput-position-domestic-geography`

### Prompt:

```
Create a specification for Throughput Position and Domestic Value Geography.

CONTEXT:
- Within the US (single currency zone), there is NO price distortion via ERDI
- Domestic value geography operates through THROUGHPUT, not visibility (γ)
- Key insight: WAGES TRACK THROUGHPUT, NOT VALUE CREATION
- Throughput position (π) explains why some locations have higher wages

THEORETICAL FOUNDATION:

**The Supply Chain Funnel**:
Value is created at extraction points and flows upward through coordination nodes.
At each layer, wages are proportional to ACCUMULATED THROUGHPUT:

```
Supply Chain Depth:
  d=0: Extraction (mines, farms)      → creates value, captures little
  d=1: Processing (refineries)
  d=2: Manufacturing (factories)
  d=3: Logistics (ports, warehouses)  → coordination chokepoints
  d=4: Retail/Services                → high throughput, variable λ
  d=5: Finance (banks, funds)         → highest throughput capture
```

**Throughput Intensity**:
```
τ_through[fips] = GDP[fips] / L[fips]
```

This is NOT a local MELT—it's throughput intensity. It measures how much accumulated value flows through this location per hour of local labor.

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
```

**Why Retail Workers Are Proletariat Despite High Throughput**:
Walmart moves enormous value (high τ_through) but workers capture almost none (λ ≈ 0.05).
The longshoreman has both high τ_through AND high λ (strong union).

**Connection to Class Position**:
Throughput position (π) and wage share (λ) determine INCOME.
Income determines rate of wealth ACCUMULATION.
Accumulated wealth determines CLASS POSITION.

```
High π × High λ → High W → Fast accumulation → LA
High π × Low λ → Low W → Slow/no accumulation → Proletariat
Low π × Any λ → Low W → Slow/no accumulation → Proletariat
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
```python
λ_proxy[fips, naics] = avg_wage[fips, naics] / τ_through[fips]
```

**Additional Domestic Indicators**:
- NetFlow[fips]: Commuter flow (LODES)
- OwnershipRatio[fips]: Capital vs labor income (ACS)
- MeanHours[fips]: Hours access (ACS)
- ReserveArmyPressure[fips]: Labor discipline (U-6, PTER)

VALIDATION CRITERIA:
- π[Oakland] > π[Wayne] (Oakland is coordination node)
- D[Oakland] > D[Wayne] (Oakland has higher average depth)
- High-π counties have higher average wages
- Ports and financial centers have highest τ_through

PREDICTIONS:
- P1: τ_through correlates with supply chain depth D
- P2: High-π, high-λ counties have higher LA share
- P3: High-π, low-λ counties have higher proletariat share (Walmart effect)
- P4: π predicts income; income predicts wealth accumulation rate

DEPENDENCIES:
- Requires: 014-wealth-based-class-position
- Data: BEA (county GDP), QCEW (county employment/wages by NAICS), LODES, ACS

Reference tvt_throughput_extension.md for complete theory.
```

---

## Phase 5: Gamma Tensor (International + Reproductive Only)

### Spec ID: `016-gamma-visibility-tensor`

### Prompt:

```
Create a specification for the Gamma (Visibility) Tensor.

CONTEXT:
- γ measures fraction of labor-time that survives transformation to price-space
- CRITICAL: γ applies ONLY to two mechanisms:
  1. International (ERDI-based compression at borders)
  2. Reproductive (naturalization of domestic labor)
- γ does NOT apply to domestic core/periphery (use π instead)

THEORETICAL FOUNDATION:

**What γ Is**:
- γ = 1: All labor registers in prices (fully visible)
- γ < 1: Some labor is invisible (compressed or naturalized)
- γ is INTENSIVE (weighted-averages under aggregation)

**What γ Is NOT**:
- γ is NOT domestic productivity differences
- γ is NOT "local MELT variation" within a currency zone
- Domestic value geography is captured by π (throughput position)

**Two Distinct Mechanisms**:

**Mechanism A: International (γ_import)**
When goods cross currency zones, ERDI differentials compress peripheral labor:
```
γ[origin→US] = 1 / ERDI[origin]
γ_import = Σ_origin (import_share[origin] × γ[origin→US])
```

**Mechanism B: Reproductive (γ_III)**
Reproductive labor is ideologically naturalized as "not real work":
```
γ_III = L_paid_care / (L_paid_care + L_unpaid_care)
```
National estimate: γ_III ≈ 0.25-0.35

**Shadow Subsidies**:
| Subsidy | Formula | Magnitude |
|---------|---------|-----------|
| Imperial | (1 - γ_basket) × consumption | ~$2T/year |
| Reproductive | (1 - γ_III) × T^III_v | ~$2.3T/year |

Both subsidize capital accumulation but through different channels.

REQUIRED OUTPUTS:

**γ_III Calculation**:
```python
def compute_gamma_III(
    atus_unpaid_hours: float,  # National from ATUS
    qcew_care_hours: float,    # NAICS 61, 62, 624, 814
) -> float:
    total = atus_unpaid_hours + qcew_care_hours
    return qcew_care_hours / total if total > 0 else 0.0
```

**Shadow Subsidy**:
```python
def compute_shadow_subsidy_III(T_III_v: float, gamma_III: float, tau: float) -> float:
    """Reproductive labor shadow subsidy."""
    return (1 - gamma_III) * T_III_v * tau
```

DATA REQUIREMENTS:
- ATUS time use data: Have loader from spec 005
- QCEW care sector employment: Have
- Penn World Tables ERDI: From Phase 2 (or hardcode)

VALIDATION CRITERIA:
- γ_III in range [0.20, 0.40]
- γ_III correlates with female labor force participation
- Shadow subsidy Φ_III ≈ $2T+/year magnitude check

DEPENDENCIES:
- Requires: 015-throughput-position-domestic-geography
- Requires: 005-atus-department-iii (integrate)
- Data: ATUS, QCEW care sectors

Reference gamma_tensor_theory_v2.md and department_iii_formalization_v2.md.
```

---

## Phase 6: Class Dynamics Engine

### Spec ID: `017-class-dynamics-engine`

### Prompt:

```
Create a specification for the Class Dynamics Engine.

CONTEXT:
- We now have: wealth-based class position, imperial rent (Φ_hour), throughput (π), visibility (γ)
- This phase integrates them into a unified dynamics engine
- Key output: how do class positions change over time?

THEORETICAL FOUNDATION:

**Class Transitions**:

| Transition | Mechanism | Trigger |
|------------|-----------|---------|
| LA → Proletariat | Dispossession | Foreclosure, bankruptcy, medical debt |
| Proletariat → LA | Accumulation | Sustained W > subsistence + positive savings rate |
| Proletariat → Lumpen | Precaritization | Job loss + inability to find stable work |
| Lumpen → Proletariat | Stabilization | Gain stable employment |

**Wealth Accumulation Rate**:
```
dWealth/dt = (W - C) × savings_rate

Where:
  W = wage income = λ × τ_through
  C = consumption (subsistence + discretionary)
  savings_rate = f(income level, γ_basket subsidy)
```

Workers with Φ_hour > 0 accumulate faster (their consumption is subsidized).
But if W ≈ C (living paycheck to paycheck), no accumulation regardless of Φ_hour.

**Dispossession Events**:
- Foreclosure: Lose home equity → exit LA if that was primary wealth
- Medical bankruptcy: Lose savings → may exit LA
- Eviction: Housing instability → may trigger precaritization

**Connection to Crisis**:
During crisis (TRPF trigger):
- Unemployment spikes → precaritization → Proletariat → Lumpen
- Foreclosures spike → LA → Proletariat
- Wage compression → slower accumulation → mobility freeze

REQUIRED FUNCTIONALITY:

**ClassTransitionEngine**:
```python
class ClassTransitionEngine:
    def compute_accumulation_rate(
        self,
        wage: float,
        consumption: float,
        phi_hour: float,
    ) -> float:
        """Annual wealth accumulation rate."""
        ...

    def check_dispossession_risk(
        self,
        fips: str,
        year: int,
    ) -> float:
        """Probability of dispossession event."""
        # Based on: foreclosure rate, bankruptcy rate, eviction rate
        ...

    def simulate_transitions(
        self,
        class_distribution: ClassDistribution,
        economic_conditions: EconomicConditions,
    ) -> ClassDistribution:
        """Simulate one period of class transitions."""
        ...
```

**ClassDistribution**:
```python
class ClassDistribution(BaseModel):
    fips: str
    year: int
    bourgeoisie_share: float      # ~1%
    petit_bourgeoisie_share: float # ~9%
    labor_aristocracy_share: float # ~40%
    proletariat_share: float       # ~35%
    lumpenproletariat_share: float # ~15%

    def validate_sums_to_one(self) -> bool:
        return abs(sum(self.shares()) - 1.0) < 0.001
```

DATA REQUIREMENTS:
- Foreclosure rates: Eviction Lab, ATTOM/CoreLogic
- Bankruptcy rates: US Courts, by district → county
- Eviction rates: Eviction Lab
- Savings rates by income: Fed SCF

VALIDATION CRITERIA:
- Class distribution stable in non-crisis years (small transitions)
- Crisis years (2008-2012, 2020) show accelerated downward mobility
- Foreclosure rate correlates with LA → Proletariat transitions
- Recovery years show gradual upward mobility

DEPENDENCIES:
- Requires: 014-wealth-based-class-position
- Requires: 013-melt-imperial-rent
- Data: Eviction Lab, bankruptcy courts, SCF

```

---

## Phase 7: Simulation Tick Dynamics

### Spec ID: `018-simulation-tick-dynamics`

### Prompt:

```
Create a specification for Simulation Tick Dynamics.

CONTEXT:
- The simulation advances in discrete time steps (ticks = years)
- State at t+1 depends on state at t plus new inputs
- Integrates all prior phases into unified state evolution

THEORETICAL FOUNDATION:

**State Vector per Tick**:

*National Parameters*:
- τ[year]: National MELT
- γ_basket[year]: Basket visibility
- γ_III[year]: Reproductive visibility

*County-Level State*:
- T^μ_ν[fips, year]: Value tensor
- K[fips, year]: Capital stock
- π[fips, year]: Throughput position
- D[fips, year]: Average supply chain depth
- ClassDistribution[fips, year]: 5-class distribution
- Precarity indicators: U-6, PTER, NILF

*Derived Fields*:
- r[fips, year]: Profit rate
- Φ_aggregate[year]: Total imperial rent (Hickel validation)

**Update Rules**:

1. Load new economic data (QCEW, BEA)
2. Compute national parameters (τ, γ)
3. Update throughput positions (π)
4. Compute imperial rent flows (Φ_hour distribution)
5. Check dispossession triggers
6. Simulate class transitions
7. Update class distribution
8. Compute derived rates (r, OCC, e)

**Coefficient vs Quantity Distinction**:
- Quantities (T, K, flows) change each tick
- Coefficients (γ, thresholds) change slowly via α-smoothing
- Class distribution changes based on transition dynamics

VALIDATION CRITERIA:
- Running 2010-2024 reproduces historical trajectories
- Class distribution shifts appropriately during crisis years
- Φ_aggregate maintains Hickel-scale magnitude

DEPENDENCIES:
- Requires: All prior phases (012-017)
- Data: Full 2010-2024 time series

```

---

## Phase 8: Crisis and Devaluation Mechanics

### Spec ID: `019-crisis-devaluation`

### Prompt:

```
Create a specification for Crisis and Devaluation Mechanics.

CONTEXT:
- TRPF creates pressure toward crisis as profit rates decline
- Crisis triggers accelerated class transitions (dispossession, precaritization)
- This is where the George Jackson bifurcation becomes relevant

THEORETICAL FOUNDATION:

**Crisis Trigger**:
```
Crisis iff r[t] < r_threshold for N consecutive periods
```

**Class Effects of Crisis**:

| Phase | Effect |
|-------|--------|
| Early crisis | Lumpen share ↑ (layoffs → precaritization) |
| Deep crisis | LA share ↓ (foreclosures → dispossession) |
| Recovery | Slow reversal, hysteresis effects |

**The George Jackson Bifurcation**:
Crisis produces either fascism or revolution depending on:
- Solidarity network topology (are workers connected across class lines?)
- Legitimation index (do people believe the system can recover?)
- Which class bears the burden (LA losing status → reactionary politics)

**Dispossession Cascade**:
```
Crisis → Foreclosures ↑ → LA → Proletariat
      → Layoffs ↑ → Proletariat → Lumpen
      → Wage compression → Accumulation stops
```

REQUIRED FUNCTIONALITY:
- CrisisDetector: Monitor r[t] for threshold breach
- DispossessionCascade: Model accelerated transitions during crisis
- BifurcationRisk: Assess fascism vs solidarity trajectory

VALIDATION CRITERIA:
- 2008-2012: Model reproduces observed class composition shifts
- Foreclosure spike correlates with LA share decline
- Recovery shows hysteresis (class composition doesn't fully restore)

DEPENDENCIES:
- Requires: 018-simulation-tick-dynamics
- Requires: 017-class-dynamics-engine

```

---

## Phase 9: Primitive Accumulation & Dispossession

### Spec ID: `020-primitive-accumulation-dispossession`

### Prompt:

```
Create a specification for Primitive Accumulation and Dispossession.

CONTEXT:
- Primitive accumulation is ONGOING, not just historical
- Gentrification is internal settler colonialism
- Dispossession is the mechanism that moves people DOWN the class structure

THEORETICAL FOUNDATION:

**Dispossession = Downward Class Mobility via Wealth Destruction**:

| Event | Wealth Effect | Class Transition |
|-------|---------------|------------------|
| Foreclosure | Lose home equity | LA → Proletariat |
| Eviction | Lose housing stability | Proletariat → Lumpen risk |
| Medical bankruptcy | Lose savings | LA → Proletariat |
| Wage theft | Lose earned income | Intensifies exploitation |
| Incarceration | Lose employment + assets | → Lumpen |

**Gentrification as Internal Colonization**:
- Rising property values → rising property taxes → displacement
- Displacement → loss of community wealth → class transition
- New residents = typically higher wealth percentile

**Connection to Wealth-Based Class**:
Dispossession directly attacks the STOCK (wealth) that determines class position.
This is why it's so effective at class restructuring—it doesn't just reduce income,
it destroys accumulated position.

REQUIRED MODELS:

**DispossessionEvent**:
```python
class DispossessionEvent(BaseModel):
    type: Literal["foreclosure", "eviction", "bankruptcy", "incarceration"]
    fips: str
    year: int
    households_affected: int
    wealth_destroyed: float
    class_transitions: dict[str, int]  # e.g., {"LA_to_Proletariat": 500}
```

**GentrificationIndex**:
```python
def compute_gentrification_index(fips: str, year: int) -> float:
    """Composite gentrification pressure indicator."""
    components = {
        'home_price_growth': get_home_price_growth(fips, year),
        'rent_growth': get_rent_growth(fips, year),
        'demographic_change': get_income_demographic_change(fips, year),
        'business_turnover': get_business_turnover(fips, year),
    }
    return weighted_sum(components)
```

VALIDATION CRITERIA:
- Dispossession rate spikes during 2008-2012
- GentrificationIndex correlates with subsequent demographic change
- Dispossession events correlate with LA share decline

DEPENDENCIES:
- Requires: 017-class-dynamics-engine
- Data: Eviction Lab, ATTOM, court records

```

---

## Usage Notes

1. **Run in order**: Each phase depends on prior phases
2. **Validate between phases**: Run falsification tests before proceeding
3. **Key theoretical commitments**:
   - Class = wealth percentile (stock), not income (flow)
   - Imperial rent (Φ_hour) is separate from class position
   - Domestic geography = throughput (π), not visibility (γ)
   - Proletariat/Lumpen distinction = precarity

## Validation Gates

| After Phase | Validation |
|-------------|------------|
| Phase 2 | τ in $55-75 range, Φ_aggregate ≈ Hickel scale |
| Phase 3 | LA = 40%, class distribution sums to 100% |
| Phase 4 | π[Oakland] > π[Wayne], depth correlates with τ_through |
| Phase 6 | Crisis years show accelerated downward mobility |
| Phase 8 | 2008-2012 reproduces observed dispossession cascade |

## Key Theoretical Formulas

```
# MELT and Imperial Rent (FLOW measures)
τ = GDP_national / L_national
γ_basket = 1 / [α/γ_import + (1-α)]
τ_effective = τ × γ_basket
Φ_hour = (W/τ)(1/γ_basket) - 1

# Class Position (STOCK measure)
Bourgeoisie:        wealth_percentile ≥ 99
Petit Bourgeoisie:  90 ≤ wealth_percentile < 99
Labor Aristocracy:  50 ≤ wealth_percentile < 90
Proletariat:        wealth_percentile < 50, stable employment
Lumpenproletariat:  wealth_percentile < 50, precarious/excluded

# Throughput (domestic geography)
τ_through = GDP_local / L_local
π = τ_through / τ
W = λ × τ_through

# Accumulation (connects flow to stock)
dWealth/dt = (W - C) × savings_rate
```

## Implementation Roadmap Summary

| Phase | Spec ID | Core Deliverable | Unlocks |
|-------|---------|------------------|---------|
| 1 | 012-capital-stock-dynamics | K, r, OCC, e | TRPF testing |
| 2 | 013-melt-imperial-rent | τ, γ_basket, Φ_hour | Imperial rent quantification |
| 3 | 014-wealth-based-class-position | 5-class structure, precarity | Class distribution |
| 4 | 015-throughput-position | π, D, λ_proxy | Domestic geography |
| 5 | 016-gamma-visibility-tensor | γ_III, shadow subsidies | Reproductive labor |
| 6 | 017-class-dynamics-engine | Transitions, accumulation | Class mobility |
| 7 | 018-simulation-tick-dynamics | SimulationState, tick() | Simulation runs |
| 8 | 019-crisis-devaluation | CrisisDetector, bifurcation | Full cycle |
| 9 | 020-primitive-accumulation | Dispossession, gentrification | Settler colonialism |

After Phase 9: Network topology, solidarity edges, George Jackson bifurcation, hex map visualization.
