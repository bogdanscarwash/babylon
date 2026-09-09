# TVT Data Gaps and Implementation Analysis (v2)

**Status**: Implementation Planning Document
**Purpose**: Identify data gaps for revised single-MELT + γ_basket model
**Revision**: Aligned with tvt_mathematical_formalization_v2.md

---

## 1. Critical Data Requirements

### 1.1 National MELT (τ)

| Component | Source | Resolution | Status |
|-----------|--------|------------|--------|
| GDP | BEA | National, annual | ✅ Have |
| Total employment | QCEW | National, quarterly | ✅ Have |
| Mean hours worked | ACS B23020 | National, annual | ⚠️ Need to verify |

**Calculation:**
```
τ = GDP / (employment × mean_hours)
```

**Fallback:**
```
τ = GDP / (employment × 2080)
```

The 2080 assumption is acceptable for τ because we use τ consistently across calculations. The error is systematic and cancels in class position comparisons.

**Expected range:** τ ≈ $55-75/hour for contemporary US.

### 1.2 Basket Visibility (γ_basket)

| Component | Source | Resolution | Status |
|-----------|--------|------------|--------|
| Import share (α) | BEA Trade in Goods | National, annual | ⚠️ Need loader |
| Import value by origin | Census Trade | National, monthly | ⚠️ Need loader |
| ERDI by country | Penn World Tables 10.0 | National, annual | ❌ Need loader |

**Calculation:**
```
γ_import = Σ_origin (import_share[origin] × (1/ERDI[origin]))
α = consumer_imports / total_consumption
γ_basket = 1 / (α/γ_import + (1 - α))
```

**This is the key new data requirement.** Without ERDI data, we cannot compute γ_basket and therefore cannot determine class position.

### 1.3 Reproduction Cost (V_reproduction)

| Component | Source | Resolution | Status |
|-----------|--------|------------|--------|
| Poverty threshold | Census | National, annual | ✅ Available |
| Regional Price Parity | BEA RPP | State/Metro, annual | ⚠️ Need loader |
| Consumer expenditure | BLS CEX | National, annual | ⚠️ Need loader |

**Empirical anchor:** V_reproduction ≈ $12/hour (2024 dollars)

**Derivation:**
- Federal poverty line (family of 4): ~$31,000/year
- Single adult poverty: ~$15,000/year
- At 2080 hours: $15,000/2080 ≈ $7.20/hour (absolute floor)
- Actual subsistence (with housing, healthcare): ~$25,000/year
- At 2080 hours: $25,000/2080 ≈ $12/hour

**Regional adjustment:**
```
V_reproduction[fips] = V_reproduction_national × RPP[fips] / 100
```

Where RPP (Regional Price Parity) adjusts for local cost of living.

### 1.4 Wage Distribution

| Component | Source | Resolution | Status |
|-----------|--------|------------|--------|
| Wages by county | QCEW | County, quarterly | ✅ Have |
| Wage distribution | BLS OES | Metro/State, annual | ⚠️ Have partial |
| Hours by wage level | ACS | County, annual | ⚠️ Need to verify |

**Gap:** QCEW gives total wages, not distribution. Need OES or ACS for wage percentiles to classify workers into class positions.

---

## 2. Penn World Tables Loader (Priority 1)

### 2.1 Data Description

Penn World Tables (PWT) 10.0 provides:
- `pl_gdpo`: Price level of output-side GDP (US = 1.0)
- `cgdpo`: Output-side GDP at current PPP
- `rgdpo`: Output-side GDP at constant 2017 national prices

**ERDI calculation:**
```
ERDI[country, year] = 1 / pl_gdpo[country, year]
```

When pl_gdpo < 1 (cheaper than US): ERDI > 1 (more labor-hours per dollar)

### 2.2 Required Countries

Top US import sources (by consumer goods value):
1. China
2. Mexico
3. Canada
4. Japan
5. Germany
6. Vietnam
7. South Korea
8. India
9. Taiwan
10. Ireland

Plus: Bangladesh, Indonesia, Thailand, Malaysia (textiles/electronics)

### 2.3 Loader Specification

```python
# src/babylon/data/pwt/loader.py

class PWTLoader:
    """Load Penn World Tables ERDI data."""

    def load(self, session: Session) -> LoadStats:
        """Load PWT data into normalized database."""
        # Download from: https://www.rug.nl/ggdc/productivity/pwt/
        # Format: Excel (.xlsx)
        pass

    def get_erdi(self, country_code: str, year: int) -> float:
        """Get ERDI for a country-year."""
        pass

    def get_gamma_import(self, import_shares: dict[str, float], year: int) -> float:
        """Compute γ_import from import shares and ERDI."""
        gamma = 0.0
        for country, share in import_shares.items():
            erdi = self.get_erdi(country, year)
            gamma += share * (1 / erdi)
        return gamma
```

### 2.4 Schema

```sql
CREATE TABLE pwt_erdi (
    country_code TEXT,      -- ISO 3166-1 alpha-3
    country_name TEXT,
    year INTEGER,
    erdi REAL,              -- 1 / pl_gdpo
    pl_gdpo REAL,           -- Original price level
    PRIMARY KEY (country_code, year)
);

CREATE INDEX idx_pwt_year ON pwt_erdi(year);
```

---

## 3. Import Share Data (Priority 1)

### 3.1 Data Sources

**Census USA Trade Online:**
- Imports by country by HS code
- Monthly, back to 1996
- URL: https://usatrade.census.gov/

**BEA Trade in Goods and Services:**
- Annual trade data by country
- Breaks down by goods vs services

### 3.2 Consumer Goods Filter

Not all imports are consumed by workers. Need to filter to consumer goods:

| HS Chapter | Category | Include? |
|------------|----------|----------|
| 01-24 | Food & agricultural | ✅ Yes |
| 25-27 | Mineral products | ❌ No (industrial) |
| 28-38 | Chemicals | ⚠️ Partial (pharmaceuticals yes) |
| 39-40 | Plastics/rubber | ⚠️ Partial |
| 41-43 | Leather | ✅ Yes |
| 50-63 | Textiles/apparel | ✅ Yes |
| 64-67 | Footwear | ✅ Yes |
| 84-85 | Electronics/machinery | ⚠️ Partial (consumer electronics yes) |
| 87 | Vehicles | ✅ Yes |
| 94-96 | Furniture/misc | ✅ Yes |

### 3.3 Calculation

```python
def compute_alpha(year: int) -> float:
    """Import share of consumption basket."""
    consumer_imports = get_consumer_imports(year)  # Filtered by HS
    total_pce = get_personal_consumption_expenditure(year)  # BEA
    return consumer_imports / total_pce

def compute_import_shares(year: int) -> dict[str, float]:
    """Import share by origin country."""
    imports_by_country = get_consumer_imports_by_country(year)
    total = sum(imports_by_country.values())
    return {c: v/total for c, v in imports_by_country.items()}
```

---

## 4. Revised Calculation Pipeline

### 4.1 Annual National Computation

```python
def compute_national_parameters(year: int) -> dict:
    """Compute τ, γ_basket, τ_effective for a year."""

    # Step 1: National MELT
    gdp = get_gdp_national(year)
    employment = get_employment_national(year)
    hours = employment * 2080  # or use ACS mean hours
    tau = gdp / hours

    # Step 2: Import visibility
    import_shares = compute_import_shares(year)
    gamma_import = pwt_loader.get_gamma_import(import_shares, year)

    # Step 3: Basket visibility
    alpha = compute_alpha(year)
    gamma_basket = 1 / (alpha/gamma_import + (1 - alpha))

    # Step 4: Effective threshold
    tau_effective = tau * gamma_basket

    # Step 5: Reproduction floor (inflation-adjusted)
    v_reproduction = 12.00 * cpi_adjustment(year, base_year=2024)

    return {
        'year': year,
        'tau': tau,
        'alpha': alpha,
        'gamma_import': gamma_import,
        'gamma_basket': gamma_basket,
        'tau_effective': tau_effective,
        'v_reproduction': v_reproduction,
    }
```

### 4.2 County-Level Classification

```python
def classify_county_workforce(fips: str, year: int, params: dict) -> dict:
    """Classify workers in a county by class position."""

    tau_eff = params['tau_effective']
    v_rep = params['v_reproduction']

    # Get wage distribution (from OES or ACS)
    wage_dist = get_wage_distribution(fips, year)

    # Classify
    results = {
        'labor_aristocracy': 0,
        'proletariat': 0,
        'subproletariat': 0,
    }

    for wage_bin, count in wage_dist.items():
        hourly = wage_bin  # Assume already hourly
        if hourly > tau_eff:
            results['labor_aristocracy'] += count
        elif hourly > v_rep:
            results['proletariat'] += count
        else:
            results['subproletariat'] += count

    total = sum(results.values())
    return {k: v/total for k, v in results.items()}
```

---

## 5. Acceptable Approximations

### 5.1 Single National γ_basket

**Approximation:** Use one γ_basket for the whole country.

**Justification:** Within a currency zone, everyone buys from the same global market at the same prices. Regional variation in consumption patterns is secondary.

**Error bound:** ±5% on γ_basket (rural vs urban consumption patterns differ slightly).

### 5.2 Fixed Hours Assumption

**Approximation:** L = employment × 2080

**Justification:** Systematic error that cancels when comparing wages to τ.

**Error bound:** ±10-15% on τ level, but ratios (W/τ) are accurate.

### 5.3 V_reproduction as Constant

**Approximation:** V_reproduction = $12/hour nationally, RPP-adjusted by region.

**Justification:** Subsistence floor doesn't vary much across locations after RPP adjustment. The poverty line methodology is well-established.

**Error bound:** ±15% regionally.

---

## 6. Implementation Priority

| Task | Priority | Blocking? | Effort |
|------|----------|-----------|--------|
| Penn World Tables loader | P0 | Yes - no γ_basket without it | Medium |
| Census trade data loader | P0 | Yes - no import shares without it | Medium |
| National MELT calculation | P0 | Yes - foundation for everything | Low |
| OES wage distribution loader | P1 | Yes for county classification | Medium |
| BEA RPP loader | P2 | Refinement only | Low |
| BLS CEX loader | P3 | Refinement only | Medium |

### 6.1 MVP Pathway

For minimum viable class position calculation:

1. **Hardcode γ_basket ≈ 0.68** (estimate from literature)
2. **Compute τ from BEA GDP + QCEW employment**
3. **τ_effective = τ × 0.68**
4. **V_reproduction = $12/hour**
5. **Use QCEW average wages** (not distribution) for county-level approximation

This gets you running immediately. Refine with real ERDI/import data later.

### 6.2 Estimated γ_basket

From Hickel et al. and trade data:

| Year | α (approx) | γ_import (approx) | γ_basket (approx) |
|------|------------|-------------------|-------------------|
| 2010 | 0.20 | 0.40 | 0.71 |
| 2015 | 0.22 | 0.38 | 0.69 |
| 2020 | 0.18 | 0.42 | 0.73 |
| 2024 | 0.20 | 0.35 | 0.68 |

Note: γ_import has been declining (more extraction) but α fluctuates with trade policy.

---

## 7. Validation Tests

### 7.1 Internal Consistency

```python
def test_tau_sanity():
    tau = compute_tau(2023)
    assert 50 < tau < 80, f"τ = {tau} outside expected range"

def test_gamma_basket_sanity():
    params = compute_national_parameters(2023)
    assert 0.5 < params['gamma_basket'] < 0.9

def test_class_shares_sum():
    shares = classify_county_workforce('26163', 2023, params)
    total = sum(shares.values())
    assert abs(total - 1.0) < 0.01
```

### 7.2 Empirical Predictions

| Prediction | Test |
|------------|------|
| τ_effective < τ | Direct computation |
| Labor aristocracy 30-50% nationally | Sum county classifications |
| Oakland labor_arist > Wayne labor_arist | Compare counties |
| Subproletariat correlates with undocumented pop | Regress against ACS citizenship data |

### 7.3 Sensitivity Analysis

```python
def sensitivity_gamma_basket():
    """Test how class shares change with γ_basket."""
    for gamma in [0.55, 0.60, 0.65, 0.70, 0.75, 0.80]:
        params = {'tau': 65, 'gamma_basket': gamma,
                  'tau_effective': 65 * gamma, 'v_reproduction': 12}
        shares = classify_county_workforce('26163', 2023, params)
        print(f"γ={gamma}: LA={shares['labor_aristocracy']:.2%}")
```

If labor aristocracy share swings wildly (e.g., 20% to 70%) across reasonable γ_basket range, the classification is fragile. If it stays within ~10 percentage points, the model is robust.

---

## 8. Open Questions

### 8.1 Service Imports

Current formulation focuses on goods imports. But services are increasingly traded:
- Call centers (India, Philippines)
- Software development (India, Eastern Europe)
- Cloud computing (global)

These don't show up in HS-code trade data but represent real labor arbitrage.

**Potential resolution:** Add service imports from BEA trade data, assign country-level ERDI.

### 8.2 Domestic Production by Immigrant Labor

Goods produced domestically by undocumented workers carry implicit γ < 1:
- Their wages are suppressed below V_reproduction
- The products are consumed at "normal" prices
- Value transfer happens without crossing a border

**Potential resolution:** Model domestic subproletariat production as having γ_domestic < 1, separate from γ_basket.

### 8.3 Financialization

Workers may receive imperial rent through:
- Pension funds invested in peripheral extraction
- 401k holding multinational stocks
- Home equity gains from Fed policy

This isn't captured by wage vs τ_effective comparison.

**Potential resolution:** Out of scope for MVP. The wage-based classification captures the primary mechanism.
