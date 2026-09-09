Circulation System Reference
============================

API reference for the Capital Volume II circulation layer (Feature 023).

The circulation system models capital as a process cycling through Money,
Productive, and Commodity forms (M-C-P-C'-M'). It adds turnover time,
fixed/circulating capital decomposition, reproduction schema checks,
inventory tracking, and integrated crisis detection.

.. contents:: On this page
   :local:
   :depth: 2

Module: ``babylon.domain.economics.circulation``
------------------------------------------------

Enums
~~~~~

CapitalForm
^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Value
     - Description
   * - ``money``
     - M: Liquidity, purchasing power
   * - ``productive``
     - P: Capital engaged in production
   * - ``commodity``
     - C: Finished goods awaiting sale

ReplacementCyclePosition
^^^^^^^^^^^^^^^^^^^^^^^^

Classifies where an economy sits in the fixed capital replacement cycle.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Value
     - Threshold
     - Description
   * - ``investment_boom``
     - replacement / depreciation > 1.5
     - Excess investment drives replacement wave
   * - ``expansion``
     - ratio > 1.0
     - Investing more than depreciation
   * - ``maintenance``
     - ratio > 0.7
     - Covers basics, gradual decline
   * - ``disinvestment``
     - ratio <= 0.7
     - Capital stock deteriorating

InventoryDiagnosis
^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Value
     - Condition
     - Description
   * - ``normal``
     - raw >= 7 days, finished <= 60 days
     - Healthy inventory levels
   * - ``overproduction``
     - finished > 60 days
     - Cannot sell: realization problem
   * - ``supply_crisis``
     - raw < 7 days
     - Cannot produce: input shortage

CrisisSeverity
^^^^^^^^^^^^^^

Realization crisis severity based on realization rate (realized / produced).

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Value
     - Realization Rate
     - Description
   * - ``normal``
     - > 95%
     - Healthy demand
   * - ``mild_slowdown``
     - > 85%
     - Emerging demand deficiency
   * - ``recession``
     - > 70%
     - Significant demand destruction
   * - ``crisis``
     - <= 70%
     - Systemic realization failure

Data Models
~~~~~~~~~~~

CircuitState
^^^^^^^^^^^^

Distribution of an entity's capital across the three forms at a given tick.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``fips_code``
     - ``str[5]``
     - County FIPS code
   * - ``year``
     - ``int >= 2010``
     - Data year
   * - ``money_capital``
     - ``Currency``
     - M: cash, deposits, receivables
   * - ``productive_capital``
     - ``Currency``
     - P: fixed + working capital in production
   * - ``commodity_capital``
     - ``Currency``
     - C: finished goods awaiting sale
   * - ``fixed_capital``
     - ``Currency``
     - Durable means of production (subset of P)
   * - ``circulating_capital``
     - ``Currency``
     - Raw materials + labor power (subset of P)

**Computed properties** (not stored):

- ``total_capital``: ``money + productive + commodity``
- ``liquidity_ratio``: ``money / total`` (0.0 if total = 0)
- ``commodity_overhang``: ``commodity / total`` (0.0 if total = 0)

TurnoverProfile
^^^^^^^^^^^^^^^

Temporal characteristics of capital circulation for an industry.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``naics_code``
     - ``str``
     - 2-6 digit NAICS industry code
   * - ``working_period_days``
     - ``int > 0``
     - Days of actual labor per production cycle
   * - ``non_working_production_days``
     - ``int >= 0``
     - Days capital sits without labor (drying, aging)
   * - ``purchase_time_days``
     - ``int >= 0``
     - Average days to acquire inputs
   * - ``sale_time_days``
     - ``int >= 0``
     - Average days to sell output
   * - ``fixed_capital_ratio``
     - ``float [0, 1]``
     - Fraction of constant capital that is fixed

**Computed properties**:

- ``production_time``: ``working_period + non_working_production``
- ``circulation_time``: ``purchase_time + sale_time``
- ``turnover_time``: ``production_time + circulation_time``
- ``turnovers_per_year``: ``365 / turnover_time`` (0.0 if turnover_time = 0)
- ``production_ratio``: ``production_time / turnover_time``

AnnualSurplusValue
^^^^^^^^^^^^^^^^^^

Annual surplus value accounting for turnover speed.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``fips_code``
     - ``str[5]``
     - County FIPS code
   * - ``year``
     - ``int``
     - Data year
   * - ``variable_capital_advanced``
     - ``Currency > 0``
     - v: wages for one production cycle
   * - ``surplus_value_per_cycle``
     - ``Currency``
     - s: surplus extracted per cycle
   * - ``turnover_time_days``
     - ``int > 0``
     - Days per complete circuit

**Computed properties**:

- ``rate_of_surplus_value``: ``s / v`` (per cycle)
- ``turnovers_per_year``: ``365 / turnover_time_days``
- ``annual_surplus_value``: ``s * turnovers_per_year``
- ``annual_rate_of_surplus_value``: ``(s/v) * turnovers_per_year``

FixedCapitalItem
^^^^^^^^^^^^^^^^

A durable means of production with per-item depreciation tracking.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``item_id``
     - ``str``
     - Unique identifier
   * - ``category``
     - ``str``
     - machinery, buildings, vehicles, etc.
   * - ``initial_value``
     - ``Currency > 0``
     - Original cost
   * - ``service_life_years``
     - ``float > 0``
     - Expected productive lifetime
   * - ``current_age_years``
     - ``float >= 0``
     - Time since acquisition

**Computed properties**:

- ``annual_depreciation``: ``initial_value / service_life_years`` (straight-line)
- ``remaining_value``: ``max(0, initial_value - annual_depreciation * current_age_years)``
- ``depreciation_fund_required``: ``initial_value - remaining_value``

DepreciationFundState
^^^^^^^^^^^^^^^^^^^^^

Economy-level tracking of fixed capital depreciation and replacement.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``fips_code``
     - ``str[5]``
     - County FIPS code
   * - ``year``
     - ``int``
     - Calendar year
   * - ``total_fixed_capital``
     - ``Currency``
     - Gross value of fixed capital stock
   * - ``accumulated_depreciation``
     - ``Currency``
     - Total depreciation fund accumulated
   * - ``annual_depreciation_flow``
     - ``Currency > 0``
     - Current year's depreciation charges
   * - ``replacement_expenditure``
     - ``Currency``
     - Actual fixed capital purchases this year

**Computed properties**:

- ``fund_adequacy``: ``accumulated_depreciation / annual_depreciation_flow``
- ``replacement_cycle_position``: ``ReplacementCyclePosition`` based on
  ``replacement_expenditure / annual_depreciation_flow`` ratio

CirculationCrisisAssessment
^^^^^^^^^^^^^^^^^^^^^^^^^^^

Integrated crisis assessment combining all Volume II signals.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``realization_crisis``
     - ``bool``
     - C' -> M' failing (commodity overhang > 0.3)
   * - ``turnover_crisis``
     - ``bool``
     - Circuit interrupted (liquidity < 0.1 and circulation > production time)
   * - ``reproduction_crisis``
     - ``bool``
     - Departments out of balance
   * - ``vulnerabilities``
     - ``list[str]``
     - Active vulnerability labels

**Vulnerability derivation rules**:

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Vulnerability
     - Derived From
     - Condition
   * - ``REALIZATION_CRISIS``
     - ``CircuitState.commodity_overhang``
     - > 0.3
   * - ``SUPPLY_CHAIN_CRISIS``
     - ``InventoryState.inventory_problem``
     - == ``SUPPLY_CRISIS``
   * - ``LABOR_SHORTAGE``
     - ``ReproductionAnalysis.sustainability``
     - == ``False``
   * - ``MONETARY_CRISIS``
     - ``CircuitState.liquidity_ratio``
     - < 0.1

Functions
~~~~~~~~~

Circuit State (``circuit.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``advance_circuit(state, turnover, surplus_value, elapsed_days) -> CircuitState``
   Advances capital through the M-C-P-C'-M' circuit. Capital transfers
   fractionally between phases based on elapsed days relative to phase
   duration. Surplus value is created proportionally during the production
   phase. Raises ``ValueError`` if ``elapsed_days`` is negative.

``initialize_circuit_state(fips_code, year, total_capital, turnover) -> CircuitState``
   Distributes initial capital across M/P/C forms proportional to phase
   durations. Money fraction = (purchase + sale) / turnover_time. Splits
   productive capital into fixed/circulating per the profile's ratio.

Turnover (``turnover.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^

``compute_annual_surplus_value(variable_capital, surplus_per_cycle, turnover_time_days, ...) -> AnnualSurplusValue``
   Constructs an ``AnnualSurplusValue`` with turnover-amplified surplus.

``compare_turnover_advantage(fast, slow) -> float``
   Returns ``fast.annual_surplus_value / slow.annual_surplus_value``.

``get_weighted_turnover_profile(industry_weights, source) -> TurnoverProfile | None``
   Computes county-level turnover from employment-weighted industry profiles.
   Returns ``None`` if no profiles resolve.

Fixed/Circulating Capital (``fixed_circulating.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``decompose_constant_capital(total_c, fixed_capital_ratio) -> tuple[Currency, Currency]``
   Splits constant capital into ``(fixed, circulating)`` portions.
   Raises ``ValueError`` if ratio is outside [0, 1].

``update_depreciation_fund(previous, annual_depreciation, replacement_expenditure) -> DepreciationFundState``
   Advances depreciation fund state by one period.

``compute_moral_depreciation(naics_code, physical_remaining_life, economic_remaining_life) -> MoralDepreciation``
   Creates a ``MoralDepreciation`` instance.

Reproduction Schema (``reproduction.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``combine_departments_ii(dept_iia, dept_iib) -> DepartmentRow``
   Sums IIa + IIb into a single Department II row.

``check_simple_reproduction(dept_i, dept_ii, tolerance=0.01) -> ReproductionBalance``
   Checks ``I(v + s) = IIc``. Reports gap and direction of imbalance.

``check_extended_reproduction(dept_i, dept_ii, dept_iii) -> ReproductionAnalysis``
   Checks if Dept III can reproduce all departments' labor power.

``compute_disproportionality(dept_i_output, dept_ii_output, dept_i_share_required) -> DisproportionalityCrisis``
   Computes departmental output imbalance metrics.

Inventory (``inventory.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``compute_realization_metrics(value_produced, value_realized, fips_code, year) -> RealizationCrisis``
   Constructs a ``RealizationCrisis`` with computed gap, rate, and severity.

``detect_realization_crisis(inventory_trend, production_trend) -> bool``
   Returns ``True`` if finished goods are rising while production is
   flat or falling. Returns ``False`` if either list has fewer than 2 elements.

Crisis Assessment (``crisis.py``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``assess_circulation_crisis(circuit_state, turnover, inventory, reproduction_balance, reproduction_analysis, commodity_overhang_threshold=0.3, liquidity_crisis_ratio=0.1) -> CirculationCrisisAssessment``
   Detects realization crisis, turnover disruption, and reproduction
   failure independently. Generates vulnerability strings per the
   derivation rules above. ``commodity_overhang_threshold`` and
   ``liquidity_crisis_ratio`` default to the original hardcoded values;
   production callers pass ``defines.capital_vol2.commodity_overhang_threshold``
   / ``defines.capital_vol2.liquidity_crisis_ratio`` (U7 defines sweep,
   2026-07-21 vol2-circulation-engine program).

Threshold Constants
~~~~~~~~~~~~~~~~~~~

U7 (defines sweep) migrated most of these off module-level ``Final``
constants in ``types/_legacy.py`` — five to ``GameDefines``-backed accessor
functions of the same name (lowercase; consumed inside frozen-model
``computed_field`` properties, which cannot take a call-time parameter),
two to ``assess_circulation_crisis`` keyword parameters (above). Only
``REALIZATION_RATE_NORMAL/SLOWDOWN/RECESSION`` remain bare ``Final``
constants — ``RealizationCrisis.crisis_severity``'s sole production
constructor (``compute_realization_metrics``) has zero production callers,
so migrating them now would repeat the "define with no live consumer"
anti-pattern the sweep exists to avoid.

.. list-table::
   :header-rows: 1
   :widths: 35 10 20 35

   * - Name
     - Value
     - Source
     - GameDefines key
   * - ``overproduction_days_threshold()``
     - 60.0
     - Census M3: avg inventory-to-shipments ~1.3 months; 60 = 1.5x normal
     - ``capital_vol2.overproduction_days_threshold``
   * - ``supply_crisis_days_threshold()``
     - 7.0
     - Standard JIT minimum buffer (BLS lead time 5-10 days)
     - ``capital_vol2.supply_crisis_days_threshold``
   * - ``commodity_overhang_threshold`` (param)
     - 0.3
     - Marx *Capital* II Ch. 16-17: >30% in C form = realization dominates
     - ``capital_vol2.commodity_overhang_threshold``
   * - ``liquidity_crisis_ratio`` (param)
     - 0.1
     - Marx *Capital* II Ch. 15: <10% liquid = cannot purchase inputs
     - ``capital_vol2.liquidity_crisis_ratio``
   * - ``REALIZATION_RATE_NORMAL``
     - 0.95
     - >95% realization = normal friction losses
     - (not yet migrated — dormant, see above)
   * - ``REALIZATION_RATE_SLOWDOWN``
     - 0.85
     - NBER recession classification thresholds
     - (not yet migrated — dormant, see above)
   * - ``REALIZATION_RATE_RECESSION``
     - 0.70
     - <70% = systemic realization failure
     - (not yet migrated — dormant, see above)
   * - ``replacement_boom_ratio()``
     - 1.5
     - BEA Fixed Asset Tables historical correlation
     - ``capital_vol2.replacement_boom_ratio``
   * - ``replacement_expansion_ratio()``
     - 1.0
     - Investment = depreciation = simple reproduction
     - ``capital_vol2.replacement_expansion_ratio``
   * - ``replacement_maintenance_ratio()``
     - 0.7
     - <70% of depreciation = active capital destruction
     - ``capital_vol2.replacement_maintenance_ratio``

Graph Bridge Integration
~~~~~~~~~~~~~~~~~~~~~~~~

The circulation state is serialized to territory nodes as ``tick_``-prefixed
attributes by ``write_tick_state_to_graph()`` in ``tick/graph_bridge.py``.

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Node Attribute
     - Source
   * - ``tick_liquidity_ratio``
     - ``circulation_state.circuit_state.liquidity_ratio``
   * - ``tick_commodity_overhang``
     - ``circulation_state.circuit_state.commodity_overhang``
   * - ``tick_replacement_cycle``
     - ``circulation_state.depreciation_fund.replacement_cycle_position.value``
   * - ``tick_inventory_diagnosis``
     - ``circulation_state.inventory_state.inventory_problem.value``
   * - ``tick_realization_crisis``
     - ``circulation_state.latest_assessment.realization_crisis``
   * - ``tick_turnover_crisis``
     - ``circulation_state.latest_assessment.turnover_crisis``
   * - ``tick_reproduction_crisis``
     - ``circulation_state.latest_assessment.reproduction_crisis``

See Also
~~~~~~~~

- :doc:`/concepts/capital-circulation` — Theoretical rationale for Volume II
- :doc:`/reference/volume-i-production` — Volume I production dynamics reference
- :doc:`/reference/economics-pipeline` — Economics pipeline overview
- :doc:`/concepts/volume-i-theory` — Volume I theoretical explanation
