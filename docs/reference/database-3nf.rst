.. _database-3nf:

=================================
3NF Normalized Database Reference
=================================

This document describes the schema and contents of ``marxist-data-3NF.duckdb``,
the normalized data warehouse supporting the Babylon simulation engine.

Overview
========

The database implements a **Third Normal Form (3NF)** star schema design with:

- **33 dimension tables** (``dim_*``) - Reference data with surrogate keys
- **34 fact tables** (``fact_*``) - Measurements linking to dimensions
- **3 bridge tables** (``bridge_*``) - Many-to-many relationships

Coverage
--------

==================  ==========================================
Aspect              Coverage
==================  ==========================================
Temporal Range      1949-2025
Geographic Scope    3,222 US counties, 52 states, 392 metros
Countries           263 countries with world-system tier
Industries          2,283 NAICS codes
Fact Records        Millions of QCEW records, 100K+ trade records
==================  ==========================================

Data Sources
------------

==================  =============================================  ===========
Source              Description                                    Frequency
==================  =============================================  ===========
Census ACS          American Community Survey 5-Year Estimates     Annual
FRED                Federal Reserve Economic Data                  Various
QCEW                Quarterly Census of Employment and Wages       Annual
Census Trade        International Trade Statistics                 Monthly
EIA                 Energy Information Administration              Annual
USGS                Mineral Commodity Summaries                    Annual
==================  =============================================  ===========

Marxian Classifications
=======================

The schema embeds Marxian analytical categories throughout:

World-System Tiers
------------------

Countries are classified using Wallerstein's world-systems theory:

====================  =======================================================
Tier                  Description
====================  =======================================================
``core``              Imperial metropoles (USA, EU, Japan)
``semi_periphery``    Intermediate states (Mexico, Brazil, China)
``periphery``         Exploited nations subject to unequal exchange
====================  =======================================================

Class Composition
-----------------

Industries are classified by their role in value production:

======================  =====================================================
Composition             Description
======================  =====================================================
``goods_producing``     Manufacturing, construction, agriculture
``service_producing``   Services, retail, healthcare
``circulation``         Finance, real estate, management (non-productive)
``government``          Public administration, state apparatus
``extraction``          Mining, resource extraction
======================  =====================================================

Labor Type
----------

Occupations are classified by relation to surplus value:

================  =========================================================
Type              Description
================  =========================================================
``productive``    Creates surplus value (factory workers, farmers)
``unproductive``  Realizes but doesn't create value (sales, advertising)
``reproductive``  Maintains labor power (healthcare, education)
``managerial``    Coordinates exploitation (managers, supervisors)
================  =========================================================

Wealth Classes (Babylon)
------------------------

FRED wealth percentiles map to Babylon simulation classes:

=============  ========================  ======================================
Percentile     Class                     Role in Simulation
=============  ========================  ======================================
Top 1%         ``core_bourgeoisie``      Ruling class, capital owners
90-99%         ``petty_bourgeoisie``     Small capitalists, professionals
50-90%         ``labor_aristocracy``     Privileged workers (imperial rent)
Bottom 50%     ``internal_proletariat``  Exploited workers
=============  ========================  ======================================

Worker Class (Marxian)
----------------------

Census worker classifications map to Marxian categories:

===================  ======================================================
Category             Description
===================  ======================================================
``proletariat``      Wage workers selling labor power
``petty_bourgeois``  Self-employed, own means of production
``state_worker``     Government employees (state apparatus)
``unpaid_labor``     Unpaid family workers (hidden exploitation)
===================  ======================================================

Dimension Tables
================

Geographic Dimensions
---------------------

dim_state
^^^^^^^^^

US states and territories (52 records).

=============  ============  ================================================
Column         Type          Description
=============  ============  ================================================
state_id       INTEGER       Primary key (surrogate)
state_fips     VARCHAR(2)    FIPS state code (unique)
state_name     VARCHAR(100)  Full state name
state_abbrev   VARCHAR(2)    Two-letter abbreviation
=============  ============  ================================================

dim_county
^^^^^^^^^^

US counties (3,222 records).

=============  ============  ================================================
Column         Type          Description
=============  ============  ================================================
county_id      INTEGER       Primary key (surrogate)
fips           VARCHAR(5)    5-digit FIPS code (unique)
state_id       INTEGER       Foreign key to dim_state
county_fips    VARCHAR(3)    3-digit county FIPS within state
county_name    VARCHAR(200)  Full county name
=============  ============  ================================================

dim_metro_area
^^^^^^^^^^^^^^

Metropolitan Statistical Areas (392 records).

==============  ============  ================================================
Column          Type          Description
==============  ============  ================================================
metro_area_id   INTEGER       Primary key (surrogate)
geo_id          VARCHAR(20)   Census geographic ID (unique)
cbsa_code       VARCHAR(10)   Core Based Statistical Area code
metro_name      VARCHAR(200)  Metropolitan area name
area_type       VARCHAR(10)   Type: ``msa`` or ``csa``
==============  ============  ================================================

dim_country
^^^^^^^^^^^

Countries and trade regions (263 records).

==================  ============  ================================================
Column              Type          Description
==================  ============  ================================================
country_id          INTEGER       Primary key (surrogate)
cty_code            VARCHAR(10)   Census Bureau country code (unique)
country_name        VARCHAR(200)  Country or region name
is_region           BOOLEAN       True if aggregate region, not country
world_system_tier   VARCHAR(20)   ``core``, ``semi_periphery``, or ``periphery``
==================  ============  ================================================

Economic Dimensions
-------------------

dim_industry
^^^^^^^^^^^^

NAICS industry codes (2,283 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
industry_id             INTEGER       Primary key (surrogate)
naics_code              VARCHAR(20)   NAICS code (unique)
industry_title          VARCHAR(300)  Industry description
naics_level             INTEGER       Hierarchy level (2-6 digit)
parent_naics_code       VARCHAR(20)   Parent industry code
sector_code             VARCHAR(2)    2-digit sector code
class_composition       VARCHAR(20)   Marxian classification (see above)
has_productivity_data   BOOLEAN       Has BLS productivity data
has_fred_data           BOOLEAN       Has FRED unemployment data
has_qcew_data           BOOLEAN       Has QCEW employment data
======================  ============  ================================================

dim_ownership
^^^^^^^^^^^^^

QCEW ownership codes (7 records).

==============  ============  ================================================
Column          Type          Description
==============  ============  ================================================
ownership_id    INTEGER       Primary key (surrogate)
own_code        VARCHAR(2)    BLS ownership code (unique)
own_title       VARCHAR(50)   Ownership description
is_government   BOOLEAN       True if government entity
is_private      BOOLEAN       True if private sector
==============  ============  ================================================

Demographic Dimensions
----------------------

dim_income_bracket
^^^^^^^^^^^^^^^^^^

Household income brackets (16 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
bracket_id       INTEGER       Primary key (surrogate)
bracket_code     VARCHAR(20)   Census variable code (unique)
bracket_label    VARCHAR(100)  Human-readable label
bracket_min_usd  INTEGER       Lower bound (NULL for lowest)
bracket_max_usd  INTEGER       Upper bound (NULL for highest)
bracket_order    INTEGER       Sort order
===============  ============  ================================================

dim_employment_status
^^^^^^^^^^^^^^^^^^^^^

Labor force status (7 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
status_id        INTEGER       Primary key (surrogate)
status_code      VARCHAR(20)   Census variable code (unique)
status_label     VARCHAR(100)  Status description
is_labor_force   BOOLEAN       True if in labor force
is_employed      BOOLEAN       True if employed
status_order     INTEGER       Sort order
===============  ============  ================================================

dim_worker_class
^^^^^^^^^^^^^^^^

Class of worker (21 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
class_id         INTEGER       Primary key (surrogate)
class_code       VARCHAR(20)   Census variable code (unique)
class_label      VARCHAR(200)  Worker class description
marxian_class    VARCHAR(20)   Marxian category (see above)
class_order      INTEGER       Sort order
===============  ============  ================================================

dim_occupation
^^^^^^^^^^^^^^

Occupation categories (73 records).

===================  ============  ================================================
Column               Type          Description
===================  ============  ================================================
occupation_id        INTEGER       Primary key (surrogate)
occupation_code      VARCHAR(20)   Census variable code (unique)
occupation_label     VARCHAR(300)  Occupation description
occupation_category  VARCHAR(100)  Broad category
labor_type           VARCHAR(20)   Marxian classification (see above)
occupation_order     INTEGER       Sort order
===================  ============  ================================================

dim_education_level
^^^^^^^^^^^^^^^^^^^

Educational attainment (25 records).

==================  ============  ================================================
Column              Type          Description
==================  ============  ================================================
level_id            INTEGER       Primary key (surrogate)
level_code          VARCHAR(20)   Census variable code (unique)
level_label         VARCHAR(200)  Education level description
years_of_schooling  INTEGER       Approximate years
level_order         INTEGER       Sort order
==================  ============  ================================================

Housing Dimensions
------------------

dim_housing_tenure
^^^^^^^^^^^^^^^^^^

Housing ownership status (2 records).

==============  ============  ================================================
Column          Type          Description
==============  ============  ================================================
tenure_id       INTEGER       Primary key (surrogate)
tenure_type     VARCHAR(20)   ``owner`` or ``renter`` (unique)
tenure_label    VARCHAR(100)  Description
is_owner        BOOLEAN       True if owner-occupied
==============  ============  ================================================

dim_rent_burden
^^^^^^^^^^^^^^^

Rent as percentage of income (10 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
burden_id               INTEGER       Primary key (surrogate)
bracket_code            VARCHAR(20)   Census variable code (unique)
burden_bracket          VARCHAR(50)   Percentage range
burden_min_pct          NUMERIC(5,2)  Lower bound percentage
burden_max_pct          NUMERIC(5,2)  Upper bound percentage
is_cost_burdened        BOOLEAN       True if >=30% of income
is_severely_burdened    BOOLEAN       True if >=50% of income
bracket_order           INTEGER       Sort order
======================  ============  ================================================

Wealth Dimensions (FRED)
------------------------

dim_wealth_class
^^^^^^^^^^^^^^^^

FRED wealth percentile groups (4 records).

================  ============  ================================================
Column            Type          Description
================  ============  ================================================
wealth_class_id   INTEGER       Primary key (surrogate)
percentile_code   VARCHAR(20)   FRED percentile code (unique)
percentile_label  VARCHAR(100)  Human-readable label
babylon_class     VARCHAR(50)   Babylon simulation class mapping
================  ============  ================================================

dim_asset_category
^^^^^^^^^^^^^^^^^^

Asset types for wealth analysis (3 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
category_id             INTEGER       Primary key (surrogate)
category_code           VARCHAR(20)   Category code (unique)
category_label          VARCHAR(100)  Asset type description
marxian_interpretation  TEXT          Theoretical significance
======================  ============  ================================================

dim_fred_series
^^^^^^^^^^^^^^^

FRED time series metadata (28 records).

====================  ============  ================================================
Column                Type          Description
====================  ============  ================================================
series_id             INTEGER       Primary key (surrogate)
series_code           VARCHAR(50)   FRED series code (unique)
title                 VARCHAR(300)  Series description
units                 VARCHAR(50)   Measurement units
frequency             VARCHAR(20)   Monthly, Quarterly, Annual
seasonal_adjustment   VARCHAR(50)   Seasonally Adjusted or Not
source                VARCHAR(100)  Data source agency
====================  ============  ================================================

Energy Dimensions (EIA)
-----------------------

dim_energy_table
^^^^^^^^^^^^^^^^

EIA data table definitions (20 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
table_id                INTEGER       Primary key (surrogate)
table_code              VARCHAR(20)   EIA table number (unique)
title                   VARCHAR(300)  Table description
category                VARCHAR(100)  Category (overview, sector, petroleum, etc.)
marxian_interpretation  TEXT          Theoretical significance
======================  ============  ================================================

dim_energy_series
^^^^^^^^^^^^^^^^^

EIA time series within tables (222 records).

==============  ============  ================================================
Column          Type          Description
==============  ============  ================================================
series_id       INTEGER       Primary key (surrogate)
table_id        INTEGER       Foreign key to dim_energy_table
series_code     VARCHAR(50)   EIA series code (unique)
series_name     VARCHAR(200)  Series description
units           VARCHAR(50)   Measurement units
column_index    INTEGER       Position in source table
==============  ============  ================================================

Materials Dimensions (USGS)
---------------------------

dim_commodity
^^^^^^^^^^^^^

Mineral commodities (85 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
commodity_id            INTEGER       Primary key (surrogate)
code                    VARCHAR(20)   Commodity code (unique)
name                    VARCHAR(200)  Commodity name
is_critical             BOOLEAN       True if critical mineral
primary_applications    TEXT          Main industrial uses
marxian_interpretation  TEXT          Theoretical significance
======================  ============  ================================================

dim_commodity_metric
^^^^^^^^^^^^^^^^^^^^

Commodity measurement types (593 records).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
metric_id               INTEGER       Primary key (surrogate)
code                    VARCHAR(20)   Metric code (unique)
name                    VARCHAR(200)  Metric description
units                   VARCHAR(50)   Measurement units
category                VARCHAR(50)   Metric category
marxian_interpretation  TEXT          Theoretical significance
======================  ============  ================================================

dim_import_source
^^^^^^^^^^^^^^^^^

Import source countries (43 records).

================  ============  ================================================
Column            Type          Description
================  ============  ================================================
import_source_id  INTEGER       Primary key (surrogate)
country           VARCHAR(100)  Country name
commodity_count   INTEGER       Number of commodities imported
map_class         VARCHAR(50)   Classification for mapping
================  ============  ================================================

Time and Reference Dimensions
-----------------------------

dim_time
^^^^^^^^

Time dimension (568 records).

=============  ============  ================================================
Column         Type          Description
=============  ============  ================================================
time_id        INTEGER       Primary key (surrogate)
year           INTEGER       Year (1949-2025)
month          INTEGER       Month (1-12, NULL for annual)
quarter        INTEGER       Quarter (1-4, NULL for monthly/annual)
is_annual      BOOLEAN       True if annual aggregate
=============  ============  ================================================

dim_gender
^^^^^^^^^^

Gender categories (3 records).

=============  ============  ================================================
Column         Type          Description
=============  ============  ================================================
gender_id      INTEGER       Primary key (surrogate)
gender_code    VARCHAR(10)   Code (unique)
gender_label   VARCHAR(20)   Label (Male, Female, Total)
=============  ============  ================================================

dim_data_source
^^^^^^^^^^^^^^^

Data source metadata (1 record currently).

======================  ============  ================================================
Column                  Type          Description
======================  ============  ================================================
source_id               INTEGER       Primary key (surrogate)
source_code             VARCHAR(50)   Source identifier (unique)
source_name             VARCHAR(200)  Full source name
source_year             INTEGER       Reference year
source_agency           VARCHAR(100)  Publishing agency
coverage_start_year     INTEGER       First year of data
coverage_end_year       INTEGER       Last year of data
======================  ============  ================================================

Fact Tables
===========

Census Facts
------------

fact_census_income
^^^^^^^^^^^^^^^^^^

Household income distribution by county (51,552 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
county_id        INTEGER       FK to dim_county
source_id        INTEGER       FK to dim_data_source
bracket_id       INTEGER       FK to dim_income_bracket
household_count  INTEGER       Number of households
===============  ============  ================================================

**Primary Key**: (county_id, source_id, bracket_id)

fact_census_median_income
^^^^^^^^^^^^^^^^^^^^^^^^^

Median household income by county (3,221 records).

=================  =============  ================================================
Column             Type           Description
=================  =============  ================================================
county_id          INTEGER        FK to dim_county
source_id          INTEGER        FK to dim_data_source
median_income_usd  NUMERIC(10,2)  Median household income
=================  =============  ================================================

**Primary Key**: (county_id, source_id)

fact_census_housing
^^^^^^^^^^^^^^^^^^^

Housing tenure by county (6,444 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
county_id        INTEGER       FK to dim_county
source_id        INTEGER       FK to dim_data_source
tenure_id        INTEGER       FK to dim_housing_tenure
household_count  INTEGER       Number of households
===============  ============  ================================================

**Primary Key**: (county_id, source_id, tenure_id)

fact_census_rent
^^^^^^^^^^^^^^^^

Median rent by county (3,212 records).

===============  =============  ================================================
Column           Type           Description
===============  =============  ================================================
county_id        INTEGER        FK to dim_county
source_id        INTEGER        FK to dim_data_source
median_rent_usd  NUMERIC(8,2)   Median gross rent
===============  =============  ================================================

**Primary Key**: (county_id, source_id)

fact_census_rent_burden
^^^^^^^^^^^^^^^^^^^^^^^

Rent burden distribution by county (32,220 records).

===============  ============  ================================================
Column           Type          Description
===============  ============  ================================================
county_id        INTEGER       FK to dim_county
source_id        INTEGER       FK to dim_data_source
burden_id        INTEGER       FK to dim_rent_burden
household_count  INTEGER       Number of renter households
===============  ============  ================================================

**Primary Key**: (county_id, source_id, burden_id)

Employment Facts (QCEW)
-----------------------

fact_qcew_annual
^^^^^^^^^^^^^^^^

Employment and wages by county/industry/ownership (2,867,930 records).

======================  =============  ================================================
Column                  Type           Description
======================  =============  ================================================
county_id               INTEGER        FK to dim_county
industry_id             INTEGER        FK to dim_industry
ownership_id            INTEGER        FK to dim_ownership
time_id                 INTEGER        FK to dim_time
establishments          INTEGER        Number of establishments
employment              INTEGER        Average monthly employment
total_wages_usd         NUMERIC(15,2)  Total annual wages
avg_weekly_wage_usd     INTEGER        Average weekly wage
avg_annual_pay_usd      INTEGER        Average annual pay
lq_employment           NUMERIC(8,4)   Location quotient (employment)
lq_annual_pay           NUMERIC(8,4)   Location quotient (pay)
disclosure_code         VARCHAR(5)     Data suppression code
======================  =============  ================================================

**Primary Key**: (county_id, industry_id, ownership_id, time_id)

This is the largest table with 2.8M+ records covering employment
across all US counties, industries, and ownership types.

Trade Facts
-----------

fact_trade_monthly
^^^^^^^^^^^^^^^^^^

Monthly trade by country (107,676 records).

======================  =============  ================================================
Column                  Type           Description
======================  =============  ================================================
country_id              INTEGER        FK to dim_country
time_id                 INTEGER        FK to dim_time
imports_usd_millions    NUMERIC(12,2)  Monthly imports (millions USD)
exports_usd_millions    NUMERIC(12,2)  Monthly exports (millions USD)
======================  =============  ================================================

**Primary Key**: (country_id, time_id)

FRED Facts
----------

fact_fred_national
^^^^^^^^^^^^^^^^^^

National macroeconomic indicators (53 records currently).

==============  =============  ================================================
Column          Type           Description
==============  =============  ================================================
series_id       INTEGER        FK to dim_fred_series
time_id         INTEGER        FK to dim_time
value           NUMERIC(20,6)  Observation value
==============  =============  ================================================

**Primary Key**: (series_id, time_id)

fact_fred_wealth_levels
^^^^^^^^^^^^^^^^^^^^^^^

Wealth levels by class and asset type (schema ready, no data).

===============  =============  ================================================
Column           Type           Description
===============  =============  ================================================
series_id        INTEGER        FK to dim_fred_series
wealth_class_id  INTEGER        FK to dim_wealth_class
category_id      INTEGER        FK to dim_asset_category
time_id          INTEGER        FK to dim_time
value_millions   NUMERIC(20,2)  Wealth amount (millions USD)
===============  =============  ================================================

**Primary Key**: (series_id, wealth_class_id, category_id, time_id)

fact_fred_wealth_shares
^^^^^^^^^^^^^^^^^^^^^^^

Wealth shares by class and asset type (schema ready, no data).

===============  =============  ================================================
Column           Type           Description
===============  =============  ================================================
series_id        INTEGER        FK to dim_fred_series
wealth_class_id  INTEGER        FK to dim_wealth_class
category_id      INTEGER        FK to dim_asset_category
time_id          INTEGER        FK to dim_time
share_percent    NUMERIC(8,4)   Share of total (percentage)
===============  =============  ================================================

**Primary Key**: (series_id, wealth_class_id, category_id, time_id)

Energy Facts (EIA)
------------------

fact_energy_annual
^^^^^^^^^^^^^^^^^^

Annual energy data (15,990 records).

==============  =============  ================================================
Column          Type           Description
==============  =============  ================================================
series_id       INTEGER        FK to dim_energy_series
time_id         INTEGER        FK to dim_time
value           NUMERIC(20,6)  Observation value
==============  =============  ================================================

**Primary Key**: (series_id, time_id)

Bridge Tables
=============

bridge_county_metro
-------------------

Maps counties to metropolitan areas (schema ready, no data).

===================  ============  ================================================
Column               Type          Description
===================  ============  ================================================
county_id            INTEGER       FK to dim_county
metro_area_id        INTEGER       FK to dim_metro_area
is_principal_city    BOOLEAN       True if county contains principal city
===================  ============  ================================================

**Primary Key**: (county_id, metro_area_id)

Query Examples
==============

Class Composition by County
---------------------------

Analyze employment distribution by Marxian class composition
(``dim_sector`` was retired 2026-07-17, ADR075; ``class_composition``
comes directly from ``dim_industry``)::

    SELECT
        c.county_name,
        s.state_abbrev,
        i.class_composition,
        SUM(q.employment) as total_employment,
        SUM(q.total_wages_usd) as total_wages
    FROM fact_qcew_annual q
    JOIN dim_county c ON q.county_id = c.county_id
    JOIN dim_state s ON c.state_id = s.state_id
    JOIN dim_industry i ON q.industry_id = i.industry_id
    JOIN dim_time t ON q.time_id = t.time_id
    WHERE t.year = 2023 AND i.class_composition IS NOT NULL
    GROUP BY c.county_name, s.state_abbrev, i.class_composition
    ORDER BY total_employment DESC;

Rent Burden by Income
---------------------

Find counties with severe rent burden::

    SELECT
        c.county_name,
        s.state_abbrev,
        m.median_income_usd,
        r.median_rent_usd,
        (r.median_rent_usd * 12) / m.median_income_usd * 100 as rent_pct_income
    FROM fact_census_median_income m
    JOIN fact_census_rent r ON m.county_id = r.county_id
    JOIN dim_county c ON m.county_id = c.county_id
    JOIN dim_state s ON c.state_id = s.state_id
    WHERE m.median_income_usd > 0
    ORDER BY rent_pct_income DESC
    LIMIT 20;

Trade Deficit by World-System Tier
----------------------------------

Calculate trade balance by world-system position::

    SELECT
        c.world_system_tier,
        t.year,
        SUM(f.imports_usd_millions) as total_imports,
        SUM(f.exports_usd_millions) as total_exports,
        SUM(f.exports_usd_millions - f.imports_usd_millions) as trade_balance
    FROM fact_trade_monthly f
    JOIN dim_country c ON f.country_id = c.country_id
    JOIN dim_time t ON f.time_id = t.time_id
    WHERE c.world_system_tier IS NOT NULL
    GROUP BY c.world_system_tier, t.year
    ORDER BY t.year DESC, c.world_system_tier;

See Also
========

- :ref:`census-analysis` - Census data analysis guide
- :ref:`fred-data` - FRED data loader documentation
- :ref:`class-dynamics` - Class dynamics ODE system
- :doc:`/how-to/load-db` - Data loading workflows
