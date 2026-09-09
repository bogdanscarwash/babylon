# Topological Value Theory: Mathematical Formalization (v2)

**Status**: Axiomatic Foundation Document
**Purpose**: Rigorous specification of TVT's mathematical structure
**Revision**: Incorporates single-MELT model with γ_basket for labor aristocracy determination

---

## 1. Primitive Notions

The following are undefined terms from which all other concepts are derived:

| Primitive | Symbol | Interpretation |
|-----------|--------|----------------|
| Concrete labor time | L ∈ ℕ | Integer hours of sector-typed human labor |
| Location | fips ∈ F | Geographic unit (FIPS code, 5-digit string) |
| Time period | t ∈ T | Discrete time index (year) |
| Use-value type | u ∈ U | Physical output category |

---

## 2. Axiom System

### Axiom Group A: Value Conservation

**A1. (Labor-Value Conservation)**
Total value created in period t equals total labor performed in period t.

```
∀t: Σ_fips V[fips, t] = Σ_fips L[fips, t]
```

**A2. (Value Redistribution)**
Value is neither created nor destroyed in exchange; it is only redistributed.

```
∀t, ∀(a,b) ∈ edges: V_sent[a→b, t] = V_received[a←b, t] / γ[a,b,t]
```

**A3. (Stock-Flow Consistency)**
Capital stock evolves by accumulation minus depreciation.

```
K[fips, t+1] = K[fips, t] × (1 - δ) + I[fips, t]
```

### Axiom Group B: Single-System Temporalism

**B1. (Input-Output Sequence)**
Output value at t+1 depends on input prices at t plus living labor during [t, t+1].

```
V[fips, t+1] = P_inputs[fips, t] + L[fips, t→t+1]
```

**B2. (Historical Cost)**
The value of constant capital is the price actually paid, not replacement cost.

```
c[fips, t] = P_paid[fips, t]
```

**B3. (Single National MELT)**
Within a currency zone, there is ONE monetary expression of labor time.

```
τ[t] = GDP[national, t] / L[national, t]
```

Units: $/labor-hour. This is a scalar per time period, NOT indexed by location.

**B4. (Labor-Hour Conversion)**
Any monetary quantity M converts to labor-hours via:

```
L_equivalent = M / τ
```

### Axiom Group C: International Value Transfer (Unequal Exchange)

**C1. (ERDI Definition)**
The Exchange Rate Deviation Index measures price distortion between currency zones:

```
ERDI[country] = GDP_PPP[country] / GDP_MER[country]
```

For the US (reference country): ERDI_US ≈ 1.0

**C2. (International Visibility)**
For value flowing across currency zones:

```
γ_international[origin → US] = ERDI_US / ERDI_origin = 1 / ERDI_origin
```

When ERDI_origin > 1 (periphery), γ < 1: value is compressed.

**C3. (Value Transfer at Border)**
When the US imports goods worth X (at MER) from a country with ERDI > 1:

```
Labor_embodied = X / τ_origin = X × ERDI_origin / τ_US
Labor_paid_for = X / τ_US
Value_transferred = Labor_embodied - Labor_paid_for = X × (ERDI_origin - 1) / τ_US
```

**C4. (Chain Composition)**
For value flowing through multiple currency zones:

```
γ[a→c via b] = γ[a→b] × γ[b→c]
```

### Axiom Group D: Consumption Basket Visibility

**D1. (Import Share)**
Each location has an import share of its consumption basket:

```
α[fips, t] ∈ [0, 1]
```

Approximation: Use national import share for consumer goods from BEA.

**D2. (Average Import Visibility)**
The weighted average visibility of imports:

```
γ_import[t] = Σ_origin (import_share[origin, t] × γ_international[origin])
```

Computed from trade data × ERDI by origin country.

**D3. (Basket Visibility)**
The effective visibility of the consumption basket:

```
γ_basket[fips, t] = 1 / [α/γ_import + (1 - α)]
```

When α > 0 and γ_import < 1: γ_basket < 1 (imperial subsidy on consumption).

**D4. (Effective MELT)**
The wage threshold adjusted for imperial subsidy:

```
τ_effective[t] = τ[t] × γ_basket[t]
```

This is what a worker must earn to break even with global labor—to neither extract from nor be extracted by the world-system through their consumption.

### Axiom Group E: Class Position

**E1. (Reproduction Cost)**
V_reproduction is the wage required to reproduce labor-power locally:

```
V_reproduction[fips, t] = Dept_IIa_output[fips, t] / L_reproductive[fips, t]
```

Empirical anchor: V_reproduction ≈ $12/hour (2024 dollars) as subsistence floor.

**E2. (Class Stratification)**
Class position is determined by wage relative to two thresholds:

```
Labor Aristocracy:  W > τ_effective
Proletariat:        τ_effective ≥ W > V_reproduction
Subproletariat:     W ≤ V_reproduction
```

**E3. (Imperial Rent per Hour)**
For a worker earning W with basket visibility γ_basket:

```
Φ_hour = (W/τ) × (1/γ_basket) - 1
```

If Φ_hour > 0: Worker extracts value from periphery through consumption.
If Φ_hour < 0: Worker is net exploited (rare for US workers with γ_basket < 1).
If Φ_hour = 0: Break-even with world-system.

**E4. (Labor Commanded)**
Hours of global labor commanded per hour worked:

```
L_commanded = (W/τ) × (1/γ_basket)
```

Labor aristocracy iff L_commanded > 1.

### Axiom Group F: Reproductive Labor Visibility

**F1. (Department III Visibility)**
Reproductive labor has separate visibility from commodity production:

```
γ_III[fips, t] = L_paid_care[fips, t] / (L_paid_care[fips, t] + L_unpaid_care[fips, t])
```

**F2. (Shadow Subsidy)**
Value produced but not priced in Department III:

```
Φ_shadow[fips, t] = (1 - γ_III[fips, t]) × T^III_total[fips, t]
```

**F3. (Total Extraction)**
A worker may benefit from both imperial rent and domestic shadow subsidy:

```
Φ_total = Φ_imperial + Φ_shadow
```

### Axiom Group G: Departmental Structure

**G1. (Four Departments)**

```
μ ∈ {I, IIa, IIb, III}

I   = Production of means of production
IIa = Production of wage goods (necessary consumption)
IIb = Production of luxury goods
III = Production of labor-power (reproductive labor)
```

**G2. (Three Value Components)**

```
ν ∈ {c, v, s}

c = constant capital (transferred from past dead labor)
v = variable capital (wages paid)
s = surplus value (unpaid labor)
```

**G3. (Value Tensor)**

```
T^μ_ν[fips, t] ∈ ℝ^(4×3)
```

Stored in labor-hours (converted from wages via τ).

### Axiom Group H: Network Structure

**H1. (Graph Representation)**
The economy is a directed graph G = (N, E) where:
- N = nodes (locations × classes)
- E = edges (value flow relationships)

**H2. (Edge Types)**

```
EdgeType ∈ {EXTRACTIVE, TRANSACTIONAL, SOLIDARISTIC, ANTAGONISTIC}
```

**H3. (Topology as Material Base)**
Network structure is part of the relations of production, not superstructure.

---

## 3. Defined Terms

### 3.1 National MELT

```
τ[t] = GDP_US[t] / L_US[t]
```

Where L_US = total labor hours worked nationally (QCEW employment × ACS mean hours).

Empirical range: τ ≈ $55-75/hour for contemporary US.

### 3.2 Basket Visibility

```
γ_basket = 1 / [α/γ_import + (1 - α)]
```

**Worked example:**
- α = 0.25 (25% of consumption is imports)
- γ_import = 0.35 (weighted average ERDI of import origins ≈ 2.86)
- γ_basket = 1 / [0.25/0.35 + 0.75] = 1 / [0.714 + 0.75] = 1/1.464 ≈ 0.68

### 3.3 Effective MELT

```
τ_effective = τ × γ_basket
```

**Worked example (continued):**
- τ = $65/hour
- γ_basket = 0.68
- τ_effective = $65 × 0.68 = $44.20/hour

### 3.4 Class Thresholds

| Threshold | Symbol | Empirical Value (2024) | Derivation |
|-----------|--------|------------------------|------------|
| National MELT | τ | ~$65/hour | GDP / total hours |
| Effective MELT | τ_effective | ~$44/hour | τ × γ_basket |
| Reproduction cost | V_reproduction | ~$12/hour | Subsistence floor |

### 3.5 Class Position Examples

| Occupation | Typical W | vs τ_eff | vs V_rep | Class Position |
|------------|-----------|----------|----------|----------------|
| Software engineer | $85/hr | > | > | Labor Aristocracy |
| Longshoreman | $55/hr | > | > | Labor Aristocracy |
| Retail worker | $18/hr | < | > | Proletariat |
| Fast food | $15/hr | < | > | Proletariat |
| Undocumented farmworker | $8/hr | < | < | Subproletariat |
| Gig worker (net) | $10/hr | < | < | Subproletariat |
| Tipped minimum | $2.13/hr + tips | varies | varies | Varies |

---

## 4. Domestic Value Geography

Within the US (single currency zone), there is no ERDI differential. Domestic core/periphery operates through different mechanisms.

### 4.1 Domestic Indicators

| Indicator | Symbol | Formula | Meaning |
|-----------|--------|---------|---------|
| GDP per worker | τ_local | GDP[fips]/L[fips] | Where value surfaces |
| Net commuter flow | F_net | inflow - outflow | Labor import/export |
| Ownership ratio | R_own | Y_capital / Y_labor | Where surplus lands |
| Mean hours | H_mean | total_hours / employed | Hours access |
| Reserve army | RA | U6 + PTER | Labor discipline |

### 4.2 Domestic Core Index

```
CoreIndex[fips] = w₁×norm(τ_local) + w₂×norm(F_net) + w₃×norm(R_own)
                + w₄×norm(H_mean) - w₅×norm(RA)
```

Initial weights: w_i = 0.20 each. Calibrate empirically.

**Interpretation**: CoreIndex measures where you sit in domestic value geography—not *how much* value transfers (γ does that internationally), but *where* you're positioned relative to domestic extraction patterns.

### 4.3 Why Domestic τ_local ≠ γ

Oakland has higher τ_local than Wayne. This reflects:
- Industrial composition (different sectors)
- Where commuters work vs live
- Where capital is concentrated

It does NOT reflect price distortion (same currency). The imperial rent is already embedded in the dollar. Domestic variation is about who holds how many dollars and why.

---

## 5. Data Sources

### 5.1 For τ (National MELT)

| Component | Source | Frequency |
|-----------|--------|-----------|
| GDP | BEA | Annual |
| Employment | QCEW | Quarterly |
| Hours | ACS B23020 or QCEW×2080 | Annual |

### 5.2 For γ_basket

| Component | Source | Frequency |
|-----------|--------|-----------|
| Import share (α) | BEA Trade | Annual |
| Import origins | Census Trade | Monthly |
| ERDI by country | Penn World Tables | Annual |

### 5.3 For V_reproduction

| Component | Source | Frequency |
|-----------|--------|-----------|
| Consumer expenditure | BLS CEX | Annual |
| Regional price parity | BEA RPP | Annual |
| Poverty thresholds | Census | Annual |

### 5.4 For γ_III

| Component | Source | Frequency |
|-----------|--------|-----------|
| Paid care hours | QCEW (NAICS 62, 624, 814) | Quarterly |
| Unpaid care hours | ATUS | Annual (national) |
| Demographic proxy | ACS | Annual (county) |

---

## 6. Calculation Sequence

### Phase 1: National Parameters

```python
# Compute once per year, nationally
tau = gdp_national / hours_national  # $/hour

# Compute gamma_import from trade data
gamma_import = sum(import_share[origin] * (1/erdi[origin]) for origin in countries)

# Compute basket visibility
alpha = total_imports / total_consumption  # from BEA
gamma_basket = 1 / (alpha/gamma_import + (1 - alpha))

# Compute effective threshold
tau_effective = tau * gamma_basket

# Set reproduction floor (empirically anchored)
v_reproduction = 12.00  # $/hour, adjust for inflation
```

### Phase 2: Class Position by County

```python
def classify_county(fips: str, year: int) -> dict:
    """Classify workers in a county by class position."""

    # Get wage distribution from QCEW
    wages = get_wage_distribution(fips, year)

    # Classify each wage bin
    labor_aristocracy = wages[wages > tau_effective].sum()
    proletariat = wages[(wages > v_reproduction) & (wages <= tau_effective)].sum()
    subproletariat = wages[wages <= v_reproduction].sum()

    return {
        'labor_aristocracy_share': labor_aristocracy / wages.sum(),
        'proletariat_share': proletariat / wages.sum(),
        'subproletariat_share': subproletariat / wages.sum(),
    }
```

### Phase 3: Imperial Rent Calculation

```python
def imperial_rent_per_hour(wage: float, tau: float, gamma_basket: float) -> float:
    """Hours of peripheral labor extracted per hour worked."""
    labor_commanded = (wage / tau) * (1 / gamma_basket)
    return labor_commanded - 1  # Net extraction

def total_imperial_rent(fips: str, year: int) -> float:
    """Total imperial rent extracted by workers in a county."""
    wages = get_wages(fips, year)
    hours = get_hours(fips, year)

    phi_per_hour = imperial_rent_per_hour(wages/hours, tau, gamma_basket)
    return phi_per_hour * hours  # Total hours extracted
```

---

## 7. Validation Criteria

### 7.1 Sanity Checks

| Check | Expected | Falsified if |
|-------|----------|--------------|
| τ range | $55-75/hour | Outside $40-100 |
| γ_basket | 0.60-0.80 | Outside 0.4-0.95 |
| τ_effective | $35-55/hour | Outside $25-70 |
| Labor aristocracy share | 30-50% | Outside 15-70% |

### 7.2 Empirical Predictions

**P1.** τ_effective should correlate with historical class consciousness data (strike frequency, union density).

**P2.** Counties with higher labor_aristocracy_share should show lower support for redistributive policies.

**P3.** γ_basket should decrease over time as trade integration increases (more imports, more imperial subsidy).

**P4.** Subproletariat share should correlate with undocumented immigrant population.

**P5.** Oakland should have higher labor_aristocracy_share than Wayne.

### 7.3 Falsification Conditions

The model is falsified if:

1. τ_effective calculation produces values that imply >80% or <10% labor aristocracy nationally
2. γ_basket shows no correlation with import penetration
3. Class position shows no correlation with political behavior
4. V_reproduction threshold fails to predict material deprivation indicators

---

## 8. Notation Reference

| Symbol | Meaning | Units |
|--------|---------|-------|
| τ | National MELT | $/labor-hour |
| τ_effective | Imperial-adjusted threshold | $/labor-hour |
| τ_local | Local GDP per worker-hour | $/labor-hour |
| γ_basket | Consumption basket visibility | dimensionless |
| γ_import | Average import visibility | dimensionless |
| γ_III | Reproductive labor visibility | dimensionless |
| α | Import share of consumption | dimensionless |
| ERDI | Exchange Rate Deviation Index | dimensionless |
| V_reproduction | Reproduction cost | $/labor-hour |
| W | Wage rate | $/labor-hour |
| Φ_hour | Imperial rent per hour | labor-hours |
| L_commanded | Labor commanded per hour worked | labor-hours |
| T^μ_ν | Value tensor | labor-hours |
| CoreIndex | Domestic position indicator | dimensionless |

---

## 9. Theoretical Summary

**The Core Formula:**

```
Labor Aristocracy iff W > τ × γ_basket
```

This captures the MLM-TW insight: it's not just about your wage relative to national average (W vs τ), but about what your consumption commands globally (adjusted by γ_basket).

A $40/hour worker looks exploited relative to a $85/hour tech worker. But if γ_basket = 0.68, then τ_effective = $44/hour, and the $40/hour worker is... still proletariat. Just barely.

Change γ_basket to 0.55 (more import dependence, lower γ_import), and τ_effective = $36/hour. Now that $40/hour worker is labor aristocracy.

**The political consequence:** As global integration deepens (α increases) and peripheral wages stay suppressed (γ_import stays low), γ_basket falls, τ_effective falls, and more US workers cross into labor aristocracy territory—even as their nominal wages stagnate.

This explains the puzzle: why does the US working class not radicalize despite wage stagnation? Because their real position relative to the world-system is improving through cheaper imports, even as their position relative to US capital deteriorates.
