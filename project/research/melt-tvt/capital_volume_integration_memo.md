# TVT Framework Integration: Capital Volume Concepts

**Status**: Bridge Document
**Purpose**: Connect Capital Volume I/II/III integration docs to revised TVT framework
**Related**: `tvt_mathematical_formalization_v2.md`, `tvt_throughput_extension.md`

---

## 1. Overview

The Capital Volume integration documents (I, II, III) identify key Marxian concepts requiring implementation. This memo clarifies how those concepts integrate with the revised TVT framework:

- **Single national MELT** (τ)
- **Basket visibility** (γ_basket) for consumption subsidy
- **Throughput position** (π) for supply chain geography
- **Labor aristocracy threshold** (τ_effective = τ × γ_basket)

---

## 2. Volume I Concepts in New Framework

### 2.1 Reserve Army of Labor

**Original concept:** The reserve army disciplines wages by creating labor competition.

**Integration with TVT:**

The reserve army determines λ (wage share of throughput):

```
W = λ × τ_through

λ = f(reserve_ratio, union_density, skill_scarcity)
```

When reserve_ratio ↑:
- λ ↓ (workers accept smaller share)
- W ↓ even if τ_through unchanged
- More workers fall below τ_effective → proletarianization

**Data source:** BLS U-6, LAUS by county

### 2.2 The Working Day

**Original concept:** Absolute surplus value extraction through lengthening the working day.

**Integration with TVT:**

Hours worked directly affects L (labor input):

```
τ = GDP / L
L = employment × mean_hours
```

When mean_hours ↑:
- L ↑
- τ ↓ (same GDP spread over more hours)
- τ_effective ↓
- Threshold for LA falls → more workers appear as LA

But this is misleading—their *condition* hasn't improved, just the threshold calculation. The correct interpretation: hours variation should be captured in W (hourly wage), not in τ.

**Implementation:** Use consistent 2080 hours for τ, but track actual hours in class position calculation.

### 2.3 Relative Surplus Value

**Original concept:** Productivity gains reduce necessary labor time, increasing surplus.

**Integration with TVT:**

Productivity improvements increase τ_through at downstream nodes:

```
τ_through[downstream] ↑ when productivity[upstream] ↑
```

The same value flows through fewer hours of coordination labor, increasing throughput intensity at chokepoints.

**Implication:** Automation in manufacturing (upstream) benefits logistics/retail workers (downstream) through throughput concentration—even as manufacturing workers are displaced.

### 2.4 Primitive Accumulation

**Original concept:** Historical (and ongoing) dispossession that creates proletariat.

**Integration with TVT:**

Primitive accumulation determines:
1. Initial distribution of π (who's at extraction vs coordination nodes)
2. Historical γ_basket advantage (settler consumption subsidized from origin)
3. V_reproduction baseline (land ownership vs rent-dependence)

**For Detroit test case:** Gentrification as ongoing primitive accumulation—transferring residents from stable V_reproduction to precarious (rent burdened → V_reproduction ↑ → subproletarianization).

---

## 3. Volume II Concepts in New Framework

### 3.1 Circuits of Capital

**Original concept:** M-C-M' describes capital's circuit through money, commodities, money-plus.

**Integration with TVT:**

The throughput model tracks value through supply chain nodes:

```
V_through[node] = V_in - V_consumed + V_created
```

Each node captures share as wages (λ × τ_through) and profit.

The circuit completes when final commodity sale realizes accumulated value in money form.

### 3.2 Turnover Time

**Original concept:** Faster turnover increases annual profit rate.

**Integration with TVT:**

Turnover affects when value transfers between nodes:

```
Annual_profit = (s per cycle) × (cycles per year)
```

High-turnover sectors (retail) vs low-turnover (construction) have different temporal dynamics even at same τ_through.

### 3.3 Departments of Production

**Original concept:** Department I (means of production) and II (consumption goods) must balance for reproduction.

**Integration with TVT:**

Extended to four departments: I, IIa, IIb, III

Each department has:
- Value tensor (T^μ_ν)
- Visibility (γ_μ—but only γ_III differs significantly domestically)
- Throughput position (average π for industries in that department)

**Balance conditions:**
```
Dept I output = Σ_μ T^μ_c  (constant capital demand)
Dept IIa output = Σ_μ T^μ_v  (variable capital demand)
Dept III output = L (labor power supply)
```

---

## 4. Volume III Concepts in New Framework

### 4.1 Transformation Problem

**Original concept:** How do labor-values transform into prices of production?

**Integration with TVT:**

The single national MELT IS the transformation:

```
P = τ × V
```

For domestic production, this is straightforward. International trade introduces γ_import compression.

**TSSI interpretation preserved:** Inputs valued at historical cost, outputs at current prices, temporal sequence respected.

### 4.2 Tendency of Rate of Profit to Fall (TRPF)

**Original concept:** Rising organic composition (c/v) compresses profit rate.

**Integration with TVT:**

```
r = s / (c + v) = (s/v) / (c/v + 1) = e / (OCC + 1)
```

As OCC ↑, r ↓ even with constant exploitation rate e.

**Counter-tendencies via γ and π:**

1. **γ_import < 1:** Cheap peripheral inputs reduce c
2. **γ_III < 1:** Unpaid reproductive labor reduces v
3. **High π positions:** Capture more s through throughput

The labor aristocracy benefits from all three counter-tendencies.

### 4.3 Equalization of Profit Rates

**Original concept:** Capital flows toward high-profit sectors until rates equalize.

**Integration with TVT:**

Within a currency zone, profit rate equalization drives π toward equilibrium:

```
If r[sector A] > r[sector B]:
  Capital flows A → B
  τ_through[A] ↓, τ_through[B] ↑
  Eventually r[A] ≈ r[B]
```

**But:** Imperial rents (γ < 1) and reproductive subsidies (γ_III < 1) create persistent differentials that capital mobility cannot arbitrage away—they're baked into the price structure.

### 4.4 Commercial and Financial Capital

**Original concept:** Non-productive capital shares in surplus through circulation activities.

**Integration with TVT:**

Commercial/financial capital = highest π positions:

```
Supply chain depth 4-5:
  - Retail: high τ_through, low λ (proletarian despite throughput)
  - Finance: high τ_through, high λ (labor aristocracy)
```

Financial capital's profit comes from controlling where value surfaces (π maximization), not from producing value.

---

## 5. Revised Definitions

### 5.1 Value Components

| Component | Definition | Computation |
|-----------|------------|-------------|
| c (constant) | Value transferred from past labor | BEA intermediate consumption / τ |
| v (variable) | Labor power cost | Wages / τ |
| s (surplus) | Unpaid labor | (Output - c - v) |
| V_total | Total value | c + v + s = L (labor hours) |

### 5.2 Profit Rate

```
r = s / (c + v)
r_apparent = r_true + Δr_imperial + Δr_reproductive

Where:
  Δr_imperial = f(1 - γ_basket)      # Import subsidy
  Δr_reproductive = f(1 - γ_III)     # Shadow labor subsidy
```

### 5.3 Class Position

```
W > τ × γ_basket          → Labor Aristocracy
τ × γ_basket ≥ W > V_rep  → Proletariat
W ≤ V_rep                 → Subproletariat

Equivalently:
λ × π > γ_basket          → Labor Aristocracy
```

---

## 6. Data Mapping

| Marx Concept | TVT Quantity | Data Source |
|--------------|--------------|-------------|
| Value of commodity | T^μ_ν | QCEW wages / τ |
| Constant capital | c | BEA intermediate inputs |
| Variable capital | v | QCEW wages |
| Surplus value | s | GDP - c - v |
| Organic composition | c/v | Computed |
| Rate of exploitation | s/v | Computed |
| Profit rate | s/(c+v) | Computed |
| Reserve army | RA | BLS U-6, LAUS |
| MELT | τ | GDP / L |
| Imperial rent | Φ_imp | Σ imports × (1 - γ) |
| Shadow subsidy | Φ_III | T^III × (1 - γ_III) |
| Throughput | τ_through | County GDP / County L |
| LA threshold | τ_effective | τ × γ_basket |

---

## 7. Implementation Notes

### 7.1 What Changes from Previous Docs

| Previous | Revised | Reason |
|----------|---------|--------|
| Location-varying τ[fips] | Single national τ | No ERDI within currency zone |
| γ for domestic core/periphery | π for throughput position | Different mechanism |
| MELT as productivity proxy | MELT as value-price bridge | Theoretical clarity |
| Wages track productivity | Wages track throughput × λ | Supply chain insight |

### 7.2 What Stays the Same

- Four-department structure (I, IIa, IIb, III)
- Value tensor (T^μ_ν) in labor-hours
- γ_III for reproductive labor
- TRPF with counter-tendencies
- Reserve army disciplinary function
- Primitive accumulation as ongoing process

### 7.3 New Additions

- π (throughput position) as core domestic geography metric
- λ (wage share) as institutional/power variable
- Supply chain depth mapping (NAICS → depth)
- γ_basket from ERDI + import share
- Unified LA formula: λ × π > γ_basket

---

## 8. Next Steps

1. **Update Capital Volume I doc:** Add reserve army → λ connection, hours → τ clarification
2. **Update Capital Volume II doc:** Add throughput framing for circuits
3. **Update Capital Volume III doc:** Integrate γ_basket with counter-tendencies
4. **Deprecate:** Old location-varying MELT calculations
5. **Implement:** Penn World Tables loader for ERDI data (P0)
6. **Implement:** NAICS → supply chain depth mapping
7. **Validate:** Detroit test case with new framework
