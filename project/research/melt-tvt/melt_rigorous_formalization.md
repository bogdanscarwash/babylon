# MELT Formalization: An MLM-TW Rigorous Treatment

**Status**: Theoretical Decision Document
**Purpose**: Resolve ambiguities in MELT calculation for Babylon simulation
**Sources**: Emmanuel (1969), Amin (2010/2018), Hickel et al. (2021), Percy's labor aristocracy analysis

---

## 1. The Fundamental Claim of MLM-TW Value Theory

From Samir Amin's "Law of Worldwide Value":

> "Labor-power has but a single value, that which is associated with the level of development of the productive forces taken globally... Though there exists but one sole value of labor-power on the scale of globalized capitalism, that labor-power is nonetheless recompensed at very different rates."

This is the starting axiom. Abstract labor creates value at a uniform rate regardless of location. What differs is **remuneration**, not **value creation**.

**Implication for MELT**: There is ONE true MELT globally. Location-specific "apparent" MELTs arise from price distortions, not different rates of value creation.

---

## 2. Two Mechanisms of Value Invisibility

### 2.1 Unequal Exchange (Emmanuel/Amin/Hickel)

**Mechanism**: Wage differentials cause price distortions. When commodities trade at world prices, peripheral labor-time is compressed into fewer core labor-time equivalents.

**Empirical measure**: ERDI (Exchange Rate Deviation Index)

```
ERDI[country] = PPP_GDP / MER_GDP
             = (GDP at domestic purchasing power) / (GDP at market exchange rate)
```

For India (1995): ERDI = 4.12 means US prices were 4.12× higher than Indian prices. When India exports $1 worth of goods (MER), those goods embody the equivalent of $4.12 of labor-time (PPP).

**Value transfer formula** (Hickel et al.):

```
T = X × (d - 1)

Where:
  T = value transferred (drain)
  X = exports from periphery to core (at MER prices)
  d = ERDI_periphery / ERDI_core (price distortion factor)
```

### 2.2 Naturalization (Fortunati)

**Mechanism**: Reproductive labor is ideologically positioned as "not real work." The labor exists but registers as zero in GDP.

**Empirical measure**: γ_III

```
γ_III = paid_care_hours / (paid_care_hours + unpaid_care_hours)
```

Both mechanisms produce the same formal effect: **labor-time that exists but doesn't fully register in prices**.

---

## 3. The MELT Hierarchy

### 3.1 Global True MELT (τ*)

The MELT that would exist under equal exchange—the ratio of global monetary output to global labor time.

```
τ* = Σ_countries GDP[c] / Σ_countries L[c]
```

**This is a single number for the world-system.**

In Amin's terms: this is the monetary expression of the **single global value of labor-power**.

### 3.2 Local Apparent MELT (τ[fips])

What MELT looks like when measured locally:

```
τ[fips] = GDP[fips] / L[fips]
```

This varies by location because:
1. **Composition effects**: Different industrial mix produces different output per hour
2. **Where surplus lands**: GDP measures where value *surfaces*, not where it's *created*
3. **Price distortions**: For international comparison, ERDI compresses peripheral GDP

### 3.3 The Relationship

For international value transfer (cross-currency zones):

```
γ[periphery→core] = τ[periphery] / τ[core]
                  ≈ ERDI[core] / ERDI[periphery]
                  = 1 / d (inverse of Hickel's distortion factor)
```

For domestic value transfer (same currency zone):

γ is NOT directly computable from MELT ratios because there are no ERDI differentials. Instead, we use **indirect indicators** (commuter flows, ownership patterns, hours distribution).

---

## 4. The Problem with GDP/L

Your essay's banana example demonstrates the problem:

> "The ratio of unequal exchange can therefore be calculated as price ratio / labor value ratio, which is 3.89/0.57=68.25. In other words, 1 hour of labor in the imperialist core can purchase 68.25 hours worth of peripheral labor."

**This is NOT what GDP/L measures.**

GDP/L measures output per worker hour at local prices. It does NOT measure:
- The embedded labor-time in traded commodities
- The value transferred through exchange
- The difference between value created and value captured

**What GDP/L actually captures**:
- Productivity (partially)
- Where surplus VALUE surfaces (geographically)
- Capital intensity effects
- Industrial composition

**What GDP/L misses**:
- Labor performed but not priced (reproductive labor)
- Value created in periphery but priced in core
- The gap between wages paid and value of labor-power

---

## 5. Revised MELT Calculation for Babylon

### 5.1 For International γ (Unequal Exchange)

Use Hickel's ERDI method directly:

```python
def compute_gamma_international(erdi_origin: float, erdi_destination: float) -> float:
    """Visibility coefficient for international value transfer.

    γ < 1 means value is compressed (periphery → core transfer).
    γ > 1 means value is expanded (core → periphery, rare).
    γ = 1 means equal exchange.

    Uses ERDI from Penn World Tables.
    """
    return erdi_destination / erdi_origin


def compute_value_transfer(exports_mer: float, gamma: float) -> float:
    """Value transferred through unequal exchange.

    Positive = drain FROM the exporter.
    """
    # At MER prices, the exporter receives exports_mer
    # At fair exchange, they should receive exports_mer / gamma
    # Transfer = (fair value) - (actual value) = exports_mer * (1/gamma - 1)
    return exports_mer * (1/gamma - 1)
```

### 5.2 For Domestic Core/Periphery (Same Currency Zone)

**Key insight from your essay**: Within the US, the imperial rent is embedded in the *currency itself*. Everyone's dollar commands the same global labor.

```
"A homeless person's dollar commands the same global labor as a hedge fund manager's dollar."
```

Therefore, domestic core/periphery does NOT operate through ERDI. It operates through:

1. **Where labor lives vs where value surfaces** (commuter flows)
2. **Where capital income lands** (ownership patterns)
3. **Access to hours** (labor aristocracy hours-hoarding)
4. **Reserve army pressure** (labor discipline mechanism)

**Domestic τ ratio is NOT γ**. It's a proxy for productivity + surplus-location, not value transfer through exchange.

### 5.3 Practical Calculation

For Babylon's Detroit test case:

```python
# INTERNATIONAL: Use ERDI (Penn World Tables)
# US ERDI ≈ 1.0 (reference country)
# China ERDI ≈ 1.7 (2020)
# Congo ERDI ≈ 3.5+ (varies)

gamma_china_to_us = 1.0 / 1.7  # ≈ 0.59
gamma_congo_to_us = 1.0 / 3.5  # ≈ 0.29

# DOMESTIC: Use indirect indicators
def compute_domestic_core_index(
    tau_ratio: float,        # GDP per worker ratio (Oakland/Wayne)
    net_commuter_flow: int,  # Positive = imports labor
    ownership_ratio: float,  # Capital income / labor income
    mean_hours: float,       # Average hours per worker
    reserve_army: float,     # U6 + PTER (inverted)
) -> float:
    """Composite domestic core/periphery indicator.

    NOT a value transfer coefficient (γ), but a position indicator.
    """
    # Normalize each to [0, 1] range
    # Weight and combine
    # Higher = more core-like
    pass
```

---

## 6. The L (Labor Hours) Problem

### 6.1 Why 2080 Is Wrong

The 2080 assumption (employment × 2080 hours/year) is problematic because:

1. **It erases class signal**: Hours access is stratified by class position
2. **It assumes uniform employment**: Ignores part-time, underemployment
3. **It ignores unpaid labor**: Reproductive labor doesn't appear in employment counts

### 6.2 What L Should Include

For rigorous MELT:

```
L_total = L_waged + L_unwaged_reproductive + L_informal

Where:
  L_waged = QCEW employment × ACS mean_hours_worked
  L_unwaged_reproductive = ATUS unpaid household/care hours
  L_informal = (harder to measure—gig work, off-books, etc.)
```

**For τ calculation (GDP/L)**:

Use L_waged only, because GDP doesn't count unwaged labor. This gives τ_apparent.

**For γ_III calculation**:

Use L_waged + L_unwaged in denominator, because γ_III measures what fraction of reproductive labor is priced.

### 6.3 When 2080 Is Acceptable

For **ratios** of τ between locations with similar hours distributions:

```
τ_ratio = (GDP_a / (emp_a × 2080)) / (GDP_b / (emp_b × 2080))
        = (GDP_a / emp_a) / (GDP_b / emp_b)
```

The 2080 cancels. This is fine for **relative** comparisons.

For **absolute** τ values (needed for converting wages to labor-hours), use ACS hours data.

---

## 7. Summary: Three Distinct Quantities

| Quantity | Symbol | Units | Formula | Use |
|----------|--------|-------|---------|-----|
| Global True MELT | τ* | $/hour | ΣG GDP / Σ L | Theoretical reference |
| Local Apparent MELT | τ[fips] | $/hour | GDP[fips] / L[fips] | Productivity proxy |
| International Visibility | γ_int | dimensionless | ERDI_dest / ERDI_origin | Unequal exchange |
| Reproductive Visibility | γ_III | dimensionless | L_paid / (L_paid + L_unpaid) | Shadow labor |
| Domestic Core Index | CI | dimensionless | f(τ, flow, ownership, hours) | Position indicator |

**Critical distinction**:

- γ_int and γ_III are **value transfer coefficients** (how much labor-time survives transformation to price-space)
- CI is a **position indicator** (where you sit in the domestic value geography)

These are NOT interchangeable.

---

## 8. Implementation Implications

### 8.1 Data Requirements

| Quantity | Data Source | Resolution | Status |
|----------|-------------|------------|--------|
| ERDI | Penn World Tables (PWT 10.0) | National, annual | Need loader |
| GDP | BEA | County, annual | Have |
| Employment | QCEW | County, quarterly | Have |
| Mean Hours | ACS B23020 | County, annual | Need to verify |
| Unpaid Hours | ATUS | National, annual | Have loader |
| Commuter Flows | LODES | County-pair, annual | Have crosswalk |
| Ownership Income | ACS | County, annual | Have |

### 8.2 Calculation Sequence

```
Phase 1: Compute τ[fips] for all counties
  - GDP from BEA
  - L from QCEW × ACS hours (or × 2080 for MVP)

Phase 2: Compute γ_III[fips]
  - ATUS national × ACS demographic proxy

Phase 3: Compute domestic indicators
  - Commuter flows from LODES
  - Ownership ratio from ACS
  - Reserve army from BLS LAUS

Phase 4: Compose into ValueTensor
  - T_price = γ × T_labor
  - Shadow subsidy = (1 - γ) × T
```

### 8.3 Falsification Tests

1. **τ[Oakland] > τ[Wayne]** for most years (core has higher GDP/worker)
2. **Net commuter flow: Oakland positive, Wayne negative** (core imports labor)
3. **Ownership ratio: Oakland > Wayne** (surplus surfaces in core)
4. **γ_III varies with female labor force participation** (more commodification = higher γ)
5. **National γ_III ≈ 0.25-0.35** (sanity check against literature)

---

## 9. Theoretical Consequences

### 9.1 What This Resolves

The confusion between:
- **Productivity** (output per hour) and **value** (labor-time content)
- **Where surplus surfaces** (GDP location) and **where labor lives** (residence)
- **Price distortion** (ERDI) and **naturalization** (γ_III)

### 9.2 What This Preserves

From Emmanuel: Wages are exogenous (institutionally determined), not set by productivity.

From Amin: Single global value of labor-power, differential remuneration.

From Hickel: ERDI as the empirical measure of price distortion.

From your essay: Labor aristocracy is about the distribution of labor-power, not commodities.

### 9.3 The Key Formula

Imperial rent for the labor aristocracy:

```
Φ = W_core - V_core

Where:
  W_core = actual wages paid to core workers
  V_core = value of labor-power (reproduction cost)
         = Dept IIa tensor contraction at core prices

If Φ > 0: Worker receives more than they need to reproduce their labor-power.
          The difference comes from imperial extraction (through γ < 1 on imports).
```

This is computable from your existing tensor infrastructure once γ is properly defined.
