# Babylon Simulation Parameter Tuning Brief

**Generated**: 2025-12-31
**Status**: Material Reality Refactor Complete - Ready for Parameter Calibration
**For**: Project Manager (Gemini)

---

## 1. Executive Summary

The Babylon simulation engine models imperial political economy through deterministic systems. We have completed the **Material Reality Refactor** which adds:
- **VitalitySystem**: Entities die when `wealth < consumption_needs`
- **ProductionSystem**: Wealth generated from `labor × biocapacity`

The simulation now has material grounding but requires **parameter calibration** to produce desired pathological scenarios (starvation, glut, collapse).

### Current Health Status

| Scenario | Expected | Actual | Status |
|----------|----------|--------|--------|
| Baseline (52 ticks) | Survives ≥50 ticks | 52 ticks | ✅ PASS |
| Starvation (low extraction) | Comprador dies <10 ticks | Survived | ❌ FAIL |
| Glut (high extraction) | Overshoot >1.0 | 0.03 | ❌ FAIL |

**Diagnosis**: Default parameters are too stable. Need to calibrate edge cases.

---

## 2. Mathematical Core

### 2.1 Fundamental Theorem (Imperial Rent)

```
Φ = α × W_p × (1 - Ψ_p)
```

Where:
- `Φ` = Imperial rent extracted
- `α` = `extraction_efficiency` [0.0, 1.0] — how efficiently core extracts from periphery
- `W_p` = Periphery worker wealth
- `Ψ_p` = Periphery consciousness [0.0, 1.0] — higher consciousness = more resistance

**Weekly Conversion**: `α_tick = α_annual / 52`

### 2.2 Survival Calculus

**Acquiescence Probability** (survival through compliance):
```
P(S|A) = σ(k × (W - s))
```
Where:
- `σ` = Sigmoid function
- `k` = `steepness_k` — sharpness of survival threshold
- `W` = Current wealth
- `s` = `default_subsistence` — minimum wealth needed

**Revolution Probability** (survival through resistance):
```
P(S|R) = O / (O + R)
```
Where:
- `O` = Organization level
- `R` = Repression level

**Rupture Condition**: When `P(S|R) > P(S|A)`, revolution becomes rational.

### 2.3 Tribute Flow (Imperial Circuit)

4-node model: `P_w → P_c → C_b → C_w`

1. **Extraction Phase**: `P_w` loses `Φ` to `P_c`
2. **Tribute Phase**: `P_c` keeps `comprador_cut × total_wealth`, sends rest to `C_b`
3. **Wages Phase**: `C_b` pays `super_wage_rate × pool` to `C_w`
4. **Subsidy Phase**: If `P(S|R)/P(S|A) ≥ threshold`, `C_b` subsidizes `P_c` repression

### 2.4 Material Reality (New)

**Death Check** (VitalitySystem):
```
if wealth < (s_bio + s_class):
    active = False  # Entity dies
```

**Production** (ProductionSystem):
```
produced = base_labor_power × (biocapacity / max_biocapacity)
```
Only `PERIPHERY_PROLETARIAT` and `LABOR_ARISTOCRACY` produce.

---

## 3. Parameter Space

### 3.1 Primary Tuning Parameters (5 in Optuna search)

| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| `economy.extraction_efficiency` | [0.1, 0.9] | 0.8 | Annual extraction rate |
| `economy.comprador_cut` | [0.5, 1.0] | 0.9 | Fraction comprador keeps |
| `economy.super_wage_rate` | [0.05, 0.35] | 0.2 | Super-wages to core workers |
| `survival.default_subsistence` | [0.1, 0.5] | 0.3 | Minimum survival wealth |
| `survival.steepness_k` | [1.0, 10.0] | 10.0 | Survival sigmoid sharpness |

### 3.2 Secondary Parameters (not in current search)

| Category | Parameter | Default | Description |
|----------|-----------|---------|-------------|
| economy | `base_labor_power` | 1.0 | Production per tick (new) |
| economy | `base_subsistence` | 0.0 | Operational cost % |
| economy | `initial_rent_pool` | 100.0 | Starting imperial pool |
| survival | `default_organization` | 0.1 | Fallback org level |
| survival | `default_repression` | 0.5 | Fallback repression |
| solidarity | `activation_threshold` | 0.3 | Min consciousness for transmission |
| solidarity | `mass_awakening_threshold` | 0.6 | Target for MASS_AWAKENING |
| consciousness | `sensitivity` | 0.5 | Drift response speed |
| consciousness | `decay_lambda` | 0.1 | Consciousness decay |
| tension | `accumulation_rate` | 0.05 | Tension from wealth gaps |
| metabolism | `entropy_factor` | 1.2 | Extraction inefficiency |
| struggle | `jackson_threshold` | 0.4 | Bifurcation threshold |

### 3.3 Entity Consumption Needs

Entities have `s_bio` (biological) and `s_class` (class consumption):
- **Periphery Worker**: `s_bio=0.01`, `s_class=0.01` → needs 0.02/tick
- **Comprador**: `s_bio=0.01`, `s_class=0.05` → needs 0.06/tick
- **Core Bourgeoisie**: `s_bio=0.01`, `s_class=0.1` → needs 0.11/tick

---

## 4. Tooling Infrastructure

### 4.1 Available Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| `audit_simulation.py` | Health check (3 scenarios) | `mise run audit` |
| `tune_parameters.py` | 1D parameter sweep | `uv run python tools/tune_parameters.py` |
| `landscape_analysis.py` | 2D parameter grid | `uv run python tools/landscape_analysis.py` |
| `tune_agent.py` | Bayesian optimization (Optuna) | `uv run python tools/tune_agent.py` |

### 4.2 Optuna Configuration

**Sampler**: TPE (Tree-structured Parzen Estimator)
- Multivariate mode enabled
- Seed: 42 for reproducibility

**Pruner**: Hyperband
- `min_resource`: 1 tick
- `max_resource`: 52 ticks
- `reduction_factor`: 3

**Objective Function**:
```python
score = (ticks_survived × 10) + (rent_pool / 10)
```
Higher = better (maximize survival + economic health)

### 4.3 Output Artifacts

| File | Format | Contents |
|------|--------|----------|
| `results/trace.csv` | CSV | 52-tick time series (29 columns) |
| `results/trace.json` | JSON | Full GameDefines + summary metrics |
| `results/landscape.csv` | Matrix CSV | 2D parameter sweep results |
| `results/optuna_study.db` | SQLite | Optuna study persistence |
| `reports/audit_latest.md` | Markdown | Health report |

---

## 5. Current Findings

### 5.1 Landscape Analysis (extraction × comprador_cut)

```
extraction_efficiency \ comprador_cut | 0.5  0.6  0.7  0.8  0.9  1.0
-------------------------------------|-----------------------------
0.10                                 | 52   52   52   52   52   52
0.30                                 | 52   52   52   52   52   52
0.50                                 | 52   52   52   52   52   52
0.70                                 | 52   52   52   52   52   52
0.90                                 | 52   52   52   52   52   52
```

**Finding**: Complete stability across all 30 parameter combinations. This indicates:
1. Default consumption needs are too low relative to production
2. Or initial wealth values provide too much buffer
3. Or the death check needs a higher threshold

### 5.2 Trace Analysis (Default Scenario)

From `trace.json`:
- **Final P_w Wealth**: 0.076 (started at 0.1, declining slowly)
- **Final P_c Wealth**: 0.005 (started at 0.2, nearly depleted)
- **Final C_b Wealth**: 1.12 (accumulating)
- **Max Tension**: 0.15 (low, no ruptures)
- **Peak P_w Consciousness**: 0.65 (approaching awakening)
- **Crossover Tick**: 1 (P(S|R) > P(S|A) very early, but no uprising)

### 5.3 Why Starvation Scenario Fails

The Starvation test uses `extraction_efficiency=0.05` expecting Comprador collapse.

**Issue**: With low extraction:
- Comprador receives very little tribute
- BUT Comprador's consumption_needs (0.06) > extraction income
- VitalitySystem should kill them... BUT
- The `is_dead()` function in audit checks `wealth <= 0.001`
- VitalitySystem checks `wealth < consumption_needs`

**Fix Needed**: Align death threshold or increase consumption_needs.

---

## 6. Recommended Calibration Actions

### 6.1 Immediate Fixes

1. **Align Death Detection**:
   - Audit tool uses `wealth <= 0.001`
   - VitalitySystem uses `wealth < (s_bio + s_class)`
   - Choose one consistent threshold

2. **Increase Consumption Needs**:
   - Current: 0.02 (worker), 0.06 (comprador)
   - Consider: 0.05 (worker), 0.15 (comprador)
   - This creates pressure for death scenarios

3. **Enable Base Subsistence**:
   - Current: 0.0 (disabled)
   - Set to: 0.01-0.05 to drain wealth over time

### 6.2 Expanded Optuna Search

Add these parameters to the search space:

```python
SEARCH_SPACE = {
    "economy.extraction_efficiency": (0.1, 0.9),
    "economy.comprador_cut": (0.5, 1.0),
    "economy.super_wage_rate": (0.05, 0.35),
    "economy.base_labor_power": (0.5, 2.0),      # NEW
    "economy.base_subsistence": (0.0, 0.05),     # NEW
    "survival.default_subsistence": (0.1, 0.5),
    "survival.steepness_k": (1.0, 10.0),
}
```

### 6.3 New Objective Function

Current objective prioritizes survival. Consider multi-objective:

```python
# Balance survival with interesting dynamics
objectives = {
    "survival": ticks_survived / max_ticks,           # [0, 1]
    "tension": max_tension,                           # [0, 1]
    "consciousness": peak_consciousness,              # [0, 1]
    "economic_health": rent_pool / initial_pool,      # [0, ∞)
}

# Pareto optimization or weighted sum
score = (survival × 0.3) + (tension × 0.3) + (consciousness × 0.2) + (economic_health × 0.2)
```

---

## 7. Test Suite Status

```
2898 passed, 6 skipped (Material Reality Refactor complete)
```

Key test files:
- `tests/unit/engine/systems/test_vitality.py` — 7 tests
- `tests/unit/engine/systems/test_production.py` — 11 tests
- `tests/integration/test_material_reality.py` — 10 tests

---

## 8. Next Steps for Gemini PM

1. **Review consumption_needs values** — Are current values realistic for the economic model?

2. **Run expanded Optuna study**:
   ```bash
   uv run python tools/tune_agent.py --trials 200 --study-name material_v1
   ```

3. **Generate new landscape** with base_labor_power vs extraction_efficiency:
   ```bash
   uv run python tools/landscape_analysis.py \
     --param1 economy.extraction_efficiency --range1 0.1:0.9:0.1 \
     --param2 economy.base_labor_power --range2 0.5:2.0:0.25
   ```

4. **Adjust audit scenarios** to use VitalitySystem-compatible thresholds

5. **Document target behaviors** — What should Starvation/Glut scenarios look like?

---

## Appendix: Full GameDefines Defaults

```yaml
economy:
  extraction_efficiency: 0.8
  comprador_cut: 0.9
  base_labor_power: 1.0
  super_wage_rate: 0.2
  initial_rent_pool: 100.0
  base_subsistence: 0.0

survival:
  steepness_k: 10.0
  default_subsistence: 0.3
  default_organization: 0.1
  default_repression: 0.5

solidarity:
  activation_threshold: 0.3
  mass_awakening_threshold: 0.6
  scaling_factor: 0.5

consciousness:
  sensitivity: 0.5
  decay_lambda: 0.1

tension:
  accumulation_rate: 0.05

metabolism:
  entropy_factor: 1.2
  overshoot_threshold: 1.0

struggle:
  jackson_threshold: 0.4
  revolutionary_agitation_boost: 0.5
  fascist_identity_boost: 0.2

timescale:
  tick_duration_days: 7
  weeks_per_year: 52
```
