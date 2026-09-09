# TVT Quick Reference: The Labor Aristocracy Formula

**One-page summary of the class position calculation**

---

## The Two Dimensions

| Dimension | Question | Measure |
|-----------|----------|---------|
| **Production** | Where do you sit in the supply chain? | π = τ_through / τ |
| **Consumption** | How subsidized is your basket? | γ_basket |

**LA = apex of throughput funnel + subsidized consumption**

---

## The Core Insight

A US worker's class position depends not just on their wage relative to national average, but on what their consumption **commands** globally.

When γ_basket < 1, every dollar of consumption commands more than one dollar's worth of global labor. Even a "low-wage" worker can be labor aristocracy if their consumption is subsidized by imperial extraction.

---

## The Formulas

### National MELT
```
τ = GDP_US / L_US ≈ $65/hour (2024)
```

### Basket Visibility
```
γ_basket = 1 / [α/γ_import + (1 - α)]

Where:
  α ≈ 0.20 (import share of consumption)
  γ_import ≈ 0.35 (weighted average ERDI of imports)

γ_basket ≈ 0.68
```

### Effective MELT (Labor Aristocracy Threshold)
```
τ_effective = τ × γ_basket ≈ $44/hour
```

### Reproduction Floor (Subproletariat Threshold)
```
V_reproduction ≈ $12/hour
```

---

## Class Position

| Condition | Class | Share (est.) |
|-----------|-------|--------------|
| W > $44/hr | Labor Aristocracy | ~35% |
| $44 ≥ W > $12/hr | Proletariat | ~55% |
| W ≤ $12/hr | Subproletariat | ~10% |

---

## Imperial Rent per Hour

For a worker earning W:
```
Φ_hour = (W/τ) × (1/γ_basket) - 1
```

**Examples:**

| Wage | Calculation | Φ_hour | Interpretation |
|------|-------------|--------|----------------|
| $85/hr | (85/65) × (1/0.68) - 1 | +0.92 | Extracts 0.92 hours per hour worked |
| $44/hr | (44/65) × (1/0.68) - 1 | 0.00 | Break-even |
| $20/hr | (20/65) × (1/0.68) - 1 | -0.55 | "Exploited" but still benefits from γ < 1 |
| $12/hr | (12/65) × (1/0.68) - 1 | -0.73 | Below reproduction, net exploited |

Note: Even the $20/hr worker commands (20/65) × (1/0.68) = 0.45 hours of global labor per dollar spent, vs 0.31 hours at γ = 1. They benefit from imperial extraction even while being exploited domestically.

---

## Political Interpretation

**Why doesn't the US working class radicalize despite wage stagnation?**

Because τ_effective falls as:
- α increases (more imports)
- γ_import decreases (more peripheral extraction)

Even as nominal wages stagnate, *real position relative to world-system* improves through cheaper imports.

**Why are retail workers proletariat despite high throughput?**

Because λ (wage share) is crushed. Walmart moves enormous V_through but workers capture almost none of it. The formula:

```
W = λ × τ_through
```

Low λ keeps you proletariat even at a high-throughput chokepoint.

**Who is the revolutionary subject?**

Workers where W < τ_effective AND who are excluded from the imperial consumption subsidy:
- Undocumented workers (wages suppressed, consumption not subsidized)
- Incarcerated/formerly incarcerated (excluded from formal economy)
- Those on reservations, in colonized territories
- The nationally oppressed whose consumption patterns differ

---

## Data Requirements (MVP)

| Parameter | Source | Can Hardcode? |
|-----------|--------|---------------|
| τ | BEA GDP + QCEW | No - compute |
| γ_basket | PWT + Census Trade | Yes → 0.68 |
| V_reproduction | Census poverty | Yes → $12/hr |

For MVP: hardcode γ_basket = 0.68, compute τ fresh, classify by wage.

---

---

## The Throughput Insight

**Wages track throughput, not value creation.**

```
τ_through[position] = V_throughput / L_coordination
```

| Depth | Position | V_through | Typical W |
|-------|----------|-----------|-----------|
| 0 | Extraction (plantation) | V_created only | $1/hr |
| 1 | Processing | V_0 + V_1 | $5/hr |
| 2 | Manufacturing | Σ V_i | $15/hr |
| 3 | Logistics (port) | Σ V_i (chokepoint) | $35/hr |
| 4 | Retail/Services | Σ V_i | $18/hr |
| 5 | Finance/Coordination | Σ V_i | $100+/hr |

**The wage formula:**
```
W = λ × τ_through

Where λ = wage share (union strength, position power)
```

High τ_through + high λ = Labor Aristocracy (longshoreman)
High τ_through + low λ = Proletariat (retail worker)
Low τ_through = Proletariat or worse (extraction)

---

## Key Equations Summary

```
τ = GDP / L                              (National MELT)
γ_basket = 1 / [α/γ_import + (1-α)]      (Basket visibility)
τ_effective = τ × γ_basket               (LA threshold)
τ_through = V_throughput / L             (Throughput intensity)
π = τ_through / τ                        (Throughput position)
W = λ × τ_through                        (Wage determination)
Φ_hour = (W/τ)(1/γ_basket) - 1           (Imperial rent/hour)
L_commanded = (W/τ)(1/γ_basket)          (Labor commanded/hour)

Class position:
  W > τ_effective        → Labor Aristocracy
  τ_effective ≥ W > V_rep → Proletariat
  W ≤ V_rep              → Subproletariat

Equivalent form:
  λ × π > γ_basket       → Labor Aristocracy
```
