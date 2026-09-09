# TVT Axiom Extension: Supply Chain Throughput Theory

**Status**: Theoretical Extension
**Purpose**: Formalize the relationship between supply chain position, value throughput, and wage determination
**Integrates with**: tvt_mathematical_formalization_v2.md

---

## 1. The Core Insight

Wages are not proportional to value created. Wages are proportional to **value throughput**—the accumulated extracted value flowing through a supply chain position.

The labor aristocracy occupies positions at the apex of the throughput funnel. They don't create more value per hour; they *coordinate* more already-extracted value, and their wages are their cut of the profit realized on that accumulation.

---

## 2. New Primitive: Supply Chain Depth

**Definition:** Supply chain depth (d) measures how many extraction/processing layers lie below a position.

```
d ∈ {0, 1, 2, 3, 4, 5}

d = 0: Extraction (mines, plantations, raw commodity production)
d = 1: Primary processing (refining, initial manufacturing)
d = 2: Secondary manufacturing (components, assembly)
d = 3: Distribution/logistics (shipping, warehousing)
d = 4: Retail/services (final sale, consumer-facing)
d = 5: Finance/coordination (capital allocation, management)
```

Depth is typed by industry. A single FIPS code contains positions at multiple depths.

---

## 3. Axiom Group T: Throughput Theory

### T1. (Value Creation at Extraction)

Value is created by labor. The foundational value creation happens at depth 0:

```
V_created[d=0] = L[d=0] (in labor-hours)
```

This is the only place where V_created = L directly. Higher depths create less value per hour worked but coordinate more.

### T2. (Throughput Accumulation)

Value throughput at depth d equals own creation plus inflows from lower depths:

```
V_through[d] = V_created[d] + Σ_{i<d} γ_chain[i→d] × V_through[i]
```

Where γ_chain[i→d] is the cumulative visibility of value transferred from depth i to depth d.

For the simplest case (linear chain):
```
V_through[d] = V_created[d] + γ[d-1,d] × V_through[d-1]
```

### T3. (Wage-Throughput Proportionality)

Wages at a position are proportional to throughput intensity, not value created:

```
W[d] = λ[d] × (V_through[d] / L[d])
```

Where:
- V_through[d] / L[d] = throughput intensity (value handled per hour)
- λ[d] = wage share (institutionally determined, varies by depth)

### T4. (Throughput Intensity Definition)

```
τ_through[d] = V_through[d] / L[d]
```

This is what GDP/L measures at a location—NOT a local MELT, but **throughput intensity**.

### T5. (Wage Share Gradient)

λ increases with depth (higher positions capture larger shares):

```
λ[0] < λ[1] < λ[2] < λ[3] < λ[4] < λ[5]
```

Extraction workers get a tiny share of the value they create. Coordination workers get a larger share of the value they handle.

---

## 4. Connecting Throughput to γ

### 4.1 Why γ < 1 Creates the Funnel

When value transfers from depth i to depth j with γ < 1:

```
V_received[j] = γ × V_sent[i]
```

But the *labor embodied* doesn't disappear—it becomes invisible in price-space. The "missing" value (1-γ) becomes profit captured at higher depths.

### 4.2 The Profit Pool

Total profit available at depth d:

```
Π[d] = Σ_{i<d} (1 - γ[i,i+1]) × V_through[i]
```

This is the accumulated "invisible" value from all lower depths. Wages at depth d are paid from this pool.

### 4.3 The LA Wage Source

Labor aristocracy wages come from:

```
W_LA = λ_LA × Π[d_LA] / L[d_LA]
     = λ_LA × [Σ (1-γ) × V] / L
```

Their wage is literally the monetized invisible labor of lower depths.

---

## 5. Revised Geographic Interpretation

### 5.1 τ_local as Throughput Intensity

```
τ_local[fips] = GDP[fips] / L[fips] = τ_through[fips]
```

This measures: "How much accumulated value flows through this location per hour of local labor?"

**Oakland (port):** High τ_local because the port is a chokepoint. Massive extracted value flows through, handled by relatively few workers.

**Wayne (Detroit):** Lower τ_local because value is created here but immediately exits to be captured elsewhere.

### 5.2 The Core/Periphery Gradient

Within the US:
```
Core locations: High τ_through (coordination chokepoints)
Periphery locations: Lower τ_through (value creation, outflow)
```

This explains why τ_local[Oakland] > τ_local[Wayne] without invoking "productivity" differences.

### 5.3 Domestic Value Flow

Value flows from domestic periphery → domestic core:

```
Wayne creates V → flows to Oakland for coordination → Oakland captures share as wages
```

The domestic equivalent of international unequal exchange, but within a single currency zone.

---

## 6. The Unified Class Position Formula

### 6.1 Production Side: Throughput Position

```
π[position] = τ_through[position] / τ_national
```

If π > 1: You're at a coordination node (above-average throughput)
If π < 1: You're at an extraction/creation node (below-average throughput)

### 6.2 Consumption Side: Imperial Subsidy

```
γ_basket = 1 / [α/γ_import + (1-α)]
```

If γ_basket < 1: Your consumption is subsidized by compressed peripheral labor.

### 6.3 Combined Class Position

**Labor Aristocracy** requires BOTH:
1. High throughput position (π > threshold) — you capture value on the production side
2. Imperial subsidy on consumption (γ_basket < 1) — your wages buy compressed labor

```
LA iff W > τ × γ_basket AND π > 1
```

Or equivalently:
```
LA iff τ_through[position] × λ > τ × γ_basket
```

### 6.4 Numerical Example

**Oakland longshoreman:**
- τ_through = $120/hr (massive port throughput)
- λ = 0.45 (strong union captures good share)
- W = $54/hr
- γ_basket = 0.68
- τ = $65/hr
- τ_effective = $44/hr

Position: W ($54) > τ_effective ($44) → **Labor Aristocracy**
Throughput: π = 120/65 = 1.85 → **High coordination position**

**Wayne autoworker:**
- τ_through = $55/hr (manufacturing, value exits)
- λ = 0.35 (weakened union)
- W = $28/hr (after concessions)
- τ_effective = $44/hr

Position: W ($28) < τ_effective ($44) → **Proletariat**
Throughput: π = 55/65 = 0.85 → **Below-average position (value creator)**

---

## 7. The Supply Chain Depth Formula

### 7.1 Depth Index by Industry

Map NAICS to supply chain depth:

| NAICS | Industry | Depth |
|-------|----------|-------|
| 11 | Agriculture | 0 |
| 21 | Mining | 0 |
| 31-33 | Manufacturing | 1-2 |
| 42 | Wholesale | 3 |
| 44-45 | Retail | 4 |
| 48-49 | Transportation | 3 |
| 52 | Finance | 5 |
| 54 | Professional services | 4-5 |
| 55 | Management | 5 |

### 7.2 County Depth Profile

```
D[fips] = Σ_naics (employment[fips,naics] × depth[naics]) / Σ employment
```

Average supply chain depth of a county's employment.

Oakland: D ≈ 3.5 (logistics, finance, services)
Wayne: D ≈ 1.8 (manufacturing-heavy)

### 7.3 Depth-Wage Correlation

```
E[W | D=d] ∝ V_through[d] / L[d]
```

Higher average depth → higher average wages, because higher throughput intensity.

---

## 8. Empirical Predictions

### P1. τ_local correlates with supply chain depth

```
Corr(τ_local[fips], D[fips]) > 0.5
```

### P2. Ports and financial centers have highest τ_local

```
τ_local[port cities] >> τ_local[manufacturing cities]
```

### P3. Wage share (λ) increases with depth

```
λ[finance] > λ[retail] > λ[logistics] > λ[manufacturing] > λ[extraction]
```

### P4. LA concentration tracks D

```
LA_share[fips] ∝ D[fips]
```

Counties with higher average depth have more labor aristocracy.

### P5. Value flow direction

Net value flows from low-D to high-D counties:
```
Σ V_out[low-D counties] > Σ V_in[low-D counties]
```

---

## 9. Implementation

### 9.1 NAICS-Depth Mapping Table

```sql
CREATE TABLE naics_depth (
    naics_2 TEXT PRIMARY KEY,
    industry_name TEXT,
    supply_chain_depth INTEGER,  -- 0-5
    notes TEXT
);
```

### 9.2 County Depth Calculation

```python
def compute_county_depth(fips: str, year: int) -> float:
    """Average supply chain depth of county employment."""
    employment = get_employment_by_naics(fips, year)
    depth_map = get_naics_depth_map()

    weighted_depth = sum(
        emp * depth_map[naics]
        for naics, emp in employment.items()
    )
    return weighted_depth / sum(employment.values())
```

### 9.3 Throughput Intensity

```python
def compute_throughput_intensity(fips: str, year: int) -> float:
    """τ_through = GDP / L for a county."""
    gdp = get_county_gdp(fips, year)
    hours = get_county_hours(fips, year)
    return gdp / hours

def compute_pi(fips: str, year: int, tau_national: float) -> float:
    """Throughput position relative to national."""
    return compute_throughput_intensity(fips, year) / tau_national
```

---

## 10. Summary: The Two Dimensions of Class Position

| Dimension | Measures | Formula | LA Condition |
|-----------|----------|---------|--------------|
| Production | Throughput position | π = τ_through / τ | π > 1 |
| Consumption | Imperial subsidy | γ_basket | γ_basket < 1 |

**Labor Aristocracy = apex of throughput funnel + subsidized consumption**

The wage formula that unifies both:

```
W = λ × τ_through = λ × (V_through / L)

LA iff W > τ × γ_basket
   iff λ × τ_through > τ × γ_basket
   iff λ × π > γ_basket
```

**The geometric meaning:**

Plot π (throughput position) on x-axis, λ (wage share) on y-axis.

The curve λ × π = γ_basket divides the space:
- Above: Labor Aristocracy
- Below: Proletariat

High throughput position (π >> 1) can make you LA even with low wage share.
High wage share (strong union) can make you LA even with moderate throughput.
Low both? Proletariat regardless.

---

## Appendix: The Banana Supply Chain

From the essay, traced through throughput theory:

| Position | Depth | V_through | L | τ_through | W | λ |
|----------|-------|-----------|---|-----------|---|---|
| Plantation worker | 0 | $0.54M | 542K hrs | $1/hr | $1/hr | 1.0 |
| Processing | 1 | $2.8M | 50K hrs | $56/hr | $14/hr | 0.25 |
| Shipping | 2 | $5M | 30K hrs | $167/hr | $35/hr | 0.21 |
| Distribution | 3 | $8M | 40K hrs | $200/hr | $45/hr | 0.23 |
| Retail | 4 | $10.9M | 60K hrs | $182/hr | $18/hr | 0.10 |

Note: Retail has high throughput but low λ (weak labor position). This is why retail workers can be proletariat despite high throughput—the wage share is suppressed.

The longshoreman (shipping, d=2-3) has both high throughput AND strong union (high λ). That's the LA sweet spot.
