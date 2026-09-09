Capital Volume I Production Dynamics Reference
===============================================

Technical reference for the three Capital Volume I production dynamics
mechanisms implemented in Feature 021. These mechanisms model *why*
tensor values change — the causal engines behind value production,
wage discipline, and wealth seizure.

.. contents:: On this page
   :local:
   :depth: 2

Reserve Army of Labor
---------------------

Data Types
~~~~~~~~~~

.. list-table:: ReserveArmyState Fields
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
     - Calendar year (2005–2030)
   * - ``floating_reserve``
     - ``int >= 0``
     - Workers between jobs (approx. U-3 count)
   * - ``latent_reserve``
     - ``int >= 0``
     - Underemployed/discouraged (approx. U-6 − U-3)
   * - ``stagnant_reserve``
     - ``int >= 0``
     - Chronic irregular employment (PTER count)
   * - ``pauperized``
     - ``int >= 0``
     - Unable to work (Census disability + institutionalized)
   * - ``labor_force``
     - ``int > 0``
     - Total civilian labor force

**Computed properties** (not stored):

- ``total_reserve``: ``floating + latent + stagnant`` (excludes pauperized per Marx)
- ``reserve_ratio``: ``total_reserve / labor_force``, clamped to [0, 1]

.. list-table:: ReserveArmyDynamics Fields
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``fips_code``
     - ``str[5]``
     - County FIPS code
   * - ``tick``
     - ``int >= 0``
     - Simulation tick
   * - ``mechanization_displacement``
     - ``int >= 0``
     - Workers displaced by automation this tick
   * - ``firm_failures``
     - ``int >= 0``
     - Workers from bankrupt enterprises
   * - ``expansion_absorption``
     - ``int >= 0``
     - Workers hired during expansion
   * - ``emigration``
     - ``int >= 0``
     - Workers leaving territory

**Computed**:
``net_inflow = (mechanization_displacement + firm_failures)``
``- (expansion_absorption + emigration)``

Wage Pressure Formula
~~~~~~~~~~~~~~~~~~~~~

The wage pressure coefficient maps ``reserve_ratio`` to a multiplicative
wage reduction using a bounded sigmoid:

.. math::

   \text{raw}(r) = \frac{1}{1 + e^{-k(r - r_0)}}

.. math::

   \text{baseline} = \text{raw}(0)

.. math::

   \text{wage\_pressure}(r) = C \cdot \frac{\text{raw}(r) - \text{baseline}}{1 - \text{baseline}}

Where:

- :math:`r` = ``reserve_ratio`` ∈ [0, 1]
- :math:`k` = sigmoid steepness (default: 20.0)
- :math:`r_0` = sigmoid midpoint (default: 0.08)
- :math:`C` = ceiling (default: 0.5, prevents total wage elimination)

The wage is then modified:

.. math::

   w' = w \cdot (1 - \text{wage\_pressure})

.. list-table:: ReserveArmyDefines Parameters
   :header-rows: 1
   :widths: 30 15 15 40

   * - Parameter
     - Default
     - Range
     - Description
   * - ``sigmoid_k``
     - 20.0
     - (0, 100]
     - Sigmoid steepness
   * - ``sigmoid_r0``
     - 0.08
     - (0, 1]
     - Reserve ratio at sigmoid midpoint
   * - ``wage_pressure_ceiling``
     - 0.5
     - (0, 1]
     - Maximum wage pressure (prevents total elimination)
   * - ``min_employed_fraction``
     - 0.01
     - [0, 1]
     - Reserve-ratio saturation floor — NOT a wage-pressure parameter (see
       note below)

**Implementation:**
:py:class:`babylon.domain.economics.reserve_army.calculator.DefaultWagePressureCalculator`
consumes ``sigmoid_k``/``sigmoid_r0``/``wage_pressure_ceiling``.
``min_employed_fraction`` is instead consumed by
:py:meth:`babylon.domain.economics.reserve_army.accumulation.DefaultAccumulationLoopCalculator.compute_reserve_ratio`,
which clamps the derived ``reserve_ratio`` to ``[0, 1 - min_employed_fraction]``
rather than a bare ``[0, 1]`` (U8 defines sweep, vol1-value-production
program — the field had zero consumers before this).

**System:**
:py:class:`babylon.engine.systems.reserve_army.ReserveArmySystem`
— Position 5 in ``_DEFAULT_SYSTEMS``

Dispossession Events
--------------------

Data Types
~~~~~~~~~~

.. list-table:: DispossessionType Enum (8 categories)
   :header-rows: 1
   :widths: 30 70

   * - Value
     - Description
   * - ``FORECLOSURE``
     - Bank seizure of mortgaged property
   * - ``EVICTION``
     - Removal of tenant
   * - ``TAX_SALE``
     - Seizure for unpaid property taxes
   * - ``EMINENT_DOMAIN``
     - State seizure for public use
   * - ``WAGE_THEFT``
     - Unpaid wages, tip theft, misclassification
   * - ``INCARCERATION_SEIZURE``
     - Asset forfeiture from carceral system
   * - ``PENSION_DEFAULT``
     - Corporate bankruptcy eliminating earned pension
   * - ``GENTRIFICATION_DISPLACEMENT``
     - Forced relocation due to rent increases

.. list-table:: TerritoryDispossessionState Fields
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
     - Calendar year (2005–2030)
   * - ``foreclosure_rate``
     - ``float [0, 1]``
     - Foreclosures per mortgaged unit
   * - ``eviction_rate``
     - ``float [0, 1]``
     - Evictions per renter household
   * - ``displacement_rate``
     - ``float [0, 1]``
     - Net out-migration due to housing costs
   * - ``concentrated_ownership``
     - ``float [0, 1]``
     - Fraction owned by institutional investors
   * - ``absentee_landlord_share``
     - ``float [0, 1]``
     - Fraction of rentals owned by non-residents

Intensity Formula
~~~~~~~~~~~~~~~~~

Composite dispossession intensity is a weighted sum of rate components:

.. math::

   I = w_f \cdot r_f + w_e \cdot r_e + w_d \cdot r_d + w_t \cdot c_o + w_m \cdot a_l

Where:

- :math:`r_f, r_e, r_d` = foreclosure, eviction, displacement rates
- :math:`c_o, a_l` = concentrated ownership, absentee landlord share
- :math:`w_*` = configured weights (see table below)

Result is clamped to [0, 1].

.. list-table:: DispossessionDefines Weights
   :header-rows: 1
   :widths: 35 15 50

   * - Parameter
     - Default
     - Description
   * - ``weight_foreclosure``
     - 0.40
     - Foreclosure events
   * - ``weight_eviction``
     - 0.30
     - Eviction events
   * - ``weight_displacement``
     - 0.15
     - Gentrification displacement
   * - ``weight_tax_sale``
     - 0.05
     - Tax sale events (read against the ``concentrated_ownership`` proxy)
   * - ``weight_eminent_domain``
     - 0.02
     - Eminent domain (read against the ``absentee_landlord_share`` proxy)
   * - ``deadweight_loss_fraction``
     - 0.05
     - Fraction of transferred value lost (not received)
   * - ``transfer_scale``
     - 0.01
     - Scale factor for wealth transfer: ``territory_wealth * intensity * scale``

.. note::
   The ``DispossessionType`` enum above lists 8 categories (FR-004's
   per-event-type record taxonomy), but only 5 have a territory-level rate
   or structural-proxy field feeding the composite intensity formula above:
   ``weight_wage_theft``/``weight_incarceration_seizure``/
   ``weight_pension_default`` were removed from ``DispossessionDefines``
   (U8 defines sweep, vol1-value-production program) — they had zero
   consumers and no data source exists to feed them honestly (would require
   new ingestion, out of this program's scope). See
   ``babylon.config.defines.economy_labor``'s module docstring.

Value Transfer Accounting
~~~~~~~~~~~~~~~~~~~~~~~~~

All value transfers maintain balanced accounting:

.. math::

   V_{\text{total}} = V_{\text{received}} + V_{\text{deadweight}}

.. math::

   V_{\text{deadweight}} = V_{\text{total}} \cdot f_d

Where :math:`f_d` is the ``deadweight_loss_fraction`` (default 0.05).

Transfers are clamped to available wealth — a territory's
wealth cannot go negative.

**Implementation:**
:py:class:`babylon.domain.economics.dispossession.intensity.DispossessionIntensityCalculator`

**System:**
:py:class:`babylon.engine.systems.dispossession_events.DispossessionEventSystem`
— Position 8 in ``_DEFAULT_SYSTEMS``

Working Day Classification
--------------------------

Data Types
~~~~~~~~~~

.. list-table:: ExploitationMode Enum
   :header-rows: 1
   :widths: 30 70

   * - Value
     - Description
   * - ``ABSOLUTE_DOMINANT``
     - Long hours, low productivity growth
   * - ``RELATIVE_DOMINANT``
     - Standard hours, high productivity growth
   * - ``MIXED``
     - Blend of both modes

.. list-table:: WorkingDayState Fields
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``fips_code``
     - ``str[5]``
     - County FIPS code
   * - ``naics_sector``
     - ``str[2]``
     - 2-digit NAICS sector code
   * - ``year``
     - ``int``
     - Calendar year (2005–2030)
   * - ``avg_weekly_hours``
     - ``float [0, 168]``
     - Average actual hours worked per week
   * - ``labor_intensity_index``
     - ``float > 0``
     - Output per hour relative to baseline (1.0 = baseline)

Classification Logic
~~~~~~~~~~~~~~~~~~~~

.. math::

   \text{mode} = \begin{cases}
   \text{ABSOLUTE} & \text{if } h > h_a \text{ and } i < i_l \\
   \text{RELATIVE} & \text{if } h \leq h_r \text{ and } i > i_h \\
   \text{MIXED} & \text{otherwise}
   \end{cases}

.. list-table:: WorkingDayDefines Thresholds
   :header-rows: 1
   :widths: 35 15 50

   * - Parameter
     - Default
     - Description
   * - ``absolute_hours_threshold`` (:math:`h_a`)
     - 45.0
     - Hours above which = ABSOLUTE (with low intensity)
   * - ``relative_hours_threshold`` (:math:`h_r`)
     - 40.0
     - Hours at/below which = RELATIVE (with high intensity)
   * - ``intensity_threshold_high`` (:math:`i_h`)
     - 1.2
     - Intensity above which = RELATIVE (with low hours)
   * - ``intensity_threshold_low`` (:math:`i_l`)
     - 1.1
     - Intensity below which = ABSOLUTE (with high hours)
   * - ``absolute_visibility``
     - 1.0
     - Consciousness visibility for ABSOLUTE
   * - ``relative_visibility``
     - 0.3
     - Consciousness visibility for RELATIVE

Visibility Modifier
~~~~~~~~~~~~~~~~~~~

The visibility modifier affects consciousness dynamics — absolute
exploitation is *visible* to workers (long hours are experienced directly)
while relative exploitation is *invisible* (productivity gains are abstract):

.. math::

   v = \begin{cases}
   v_a & \text{if ABSOLUTE} \\
   v_r & \text{if RELATIVE} \\
   v_r + t \cdot (v_a - v_r) & \text{if MIXED}
   \end{cases}

Where :math:`t = \frac{h - h_r}{h_a - h_r}`, clamped to [0, 1].

**Implementation:**
:py:class:`babylon.domain.economics.working_day.classifier.DefaultWorkingDayClassifier`

Data Sources (3NF Schema)
-------------------------

.. list-table:: Fact Tables Added (Feature 021)
   :header-rows: 1
   :widths: 35 25 40

   * - Table
     - Primary Key
     - Description
   * - ``fact_bls_unemployment_decomposition``
     - (county_id, time_id)
     - County-level LAUS: labor_force, employed, U-3, U-6, PTER, discouraged, marginally_attached
   * - ``fact_eviction_lab_filing``
     - (county_id, time_id)
     - Eviction filings, executions, rates, renter households
   * - ``fact_foreclosure_rate``
     - (county_id, time_id)
     - Foreclosure filings, completions, rates, mortgaged units
   * - ``fact_census_institutional_ownership``
     - (county_id, time_id)
     - Housing tenure, institutional/absentee ownership, renter migration

Event Types
-----------

Four new :py:class:`~babylon.models.enums.EventType` members:

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Event Type
     - Description
   * - ``RESERVE_ARMY_PRESSURE``
     - Reserve army wage pressure applied to territory
   * - ``DISPOSSESSION_EVENT``
     - Aggregate dispossession recorded for territory-tick
   * - ``VALUE_TRANSFER``
     - Inter-territory value transfer from dispossession
   * - ``EXPLOITATION_MODE_SHIFT``
     - Exploitation mode reclassified for territory-sector

System Execution Order
----------------------

The simulation engine executes 18 systems in materialist causal order:

.. list-table::
   :header-rows: 1
   :widths: 5 25 70

   * - #
     - System
     - Purpose
   * - 1
     - VitalitySystem
     - Biological cost + death
   * - 2
     - TerritorySystem
     - Land state updates
   * - 3
     - ProductionSystem
     - Value creation
   * - 4
     - TickDynamicsSystem
     - Economic state evolution
   * - **5**
     - **ReserveArmySystem**
     - **Wage pressure from unemployment (Feature 021)**
   * - 6
     - SolidaritySystem
     - Political organization
   * - 7
     - ImperialRentSystem
     - Value extraction
   * - **8**
     - **DispossessionEventSystem**
     - **Value transfer from dispossession (Feature 021)**
   * - 9
     - DecompositionSystem
     - LA decomposition
   * - 10
     - ControlRatioSystem
     - Guard:prisoner ratio
   * - 11
     - MetabolismSystem
     - Environmental degradation
   * - 12
     - SurvivalSystem
     - Risk assessment
   * - 13
     - StruggleSystem
     - Action/revolt
   * - 14
     - ConsciousnessSystem
     - Ideological drift
   * - 15
     - ContradictionSystem
     - Tension aggregation
   * - 16–18
     - Field Topology Systems
     - Contradiction field, derivatives, edge transitions

See Also
--------

- :doc:`/concepts/volume-i-theory` — Theoretical exposition of Capital Volume I mechanisms
- :doc:`/reference/formulas` — Complete formula reference
- :doc:`/reference/systems` — All simulation systems
- :doc:`/reference/configuration` — GameDefines parameter reference
