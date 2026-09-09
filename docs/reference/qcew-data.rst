QCEW Employment Data
====================

The QCEW (Quarterly Census of Employment and Wages) module provides access to
employment and wage data from the Bureau of Labor Statistics for Babylon's
simulation systems.

Overview
--------

QCEW is integrated into Babylon to provide:

- **County-level employment**: Detailed establishment and worker counts by NAICS industry
- **Location quotients**: Industry concentration metrics for comparative analysis
- **Class composition**: NAICS industry mapping to Marxian class categories

(State and metro aggregate tables were retired 2026-07-17 — ADR075 ruling 1;
county-level ``fact_qcew_annual`` is the canonical QCEW surface.)

All data is stored in ``data/sqlite/marxist-data-3NF.sqlite`` using a normalized
star schema.

Setup
-----

**No setup required for API loading.** QCEW uses a hybrid loading strategy:

- **API (2021-2025)**: Fetches directly from BLS QCEW Open Data API
- **Files (2013-2020)**: Reads from local CSV files (optional, for historical data)

For file-based loading of historical years, download CSV files from:
https://www.bls.gov/cew/downloadable-data-files.htm

Loading Data
------------

Command Line
^^^^^^^^^^^^

.. code-block:: bash

    # Default hybrid loading (API for 2021+, files for historical)
    mise run data:qcew

    # Specific year range
    mise run data:qcew -- --years 2020-2023

    # Force API for all years (may fail for old years not in API)
    mise run data:qcew -- --force-api

    # Force file-based loading (requires downloaded CSVs)
    mise run data:qcew -- --force-files --data-path data/qcew

    # Reset tables before loading
    mise run data:qcew -- --reset

Python API
^^^^^^^^^^

.. code-block:: python

    from babylon.data.qcew import QcewLoader, QcewAPIClient
    from babylon.data.loader_base import LoaderConfig
    from babylon.data.reference.database import get_normalized_session

    # Hybrid loading (default)
    config = LoaderConfig(qcew_years=list(range(2013, 2026)))
    loader = QcewLoader(config)

    with get_normalized_session() as session:
        stats = loader.load(session, reset=True)
        print(f"County: {stats.facts_loaded.get('qcew_county', 0):,}")
        print(f"State: {stats.facts_loaded.get('qcew_state', 0):,}")
        print(f"Metro: {stats.facts_loaded.get('qcew_metro', 0):,}")

    # Direct API access
    with QcewAPIClient() as client:
        # Fetch California state-level data for 2023
        for record in client.get_area_annual_data(2023, "06000"):
            print(f"{record.industry_code}: {record.annual_avg_emplvl:,} workers")

Geographic Levels
-----------------

QCEW data is loaded at the county level based on aggregation level codes:

.. list-table::
   :header-rows: 1
   :widths: 20 20 30 30

   * - Level
     - agglvl_code
     - Fact Table
     - Description
   * - County
     - 70-78
     - ``fact_qcew_annual``
     - 3,200+ US counties

State/metro rollups (``fact_qcew_state_annual``, ``fact_qcew_metro_annual``)
and ``dim_sector`` were retired 2026-07-17 (ADR075 ruling 1) — county-level
``fact_qcew_annual`` is the canonical QCEW table.

Database Schema
---------------

Fact Tables
^^^^^^^^^^^

**fact_qcew_annual** (County Level)

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Column
     - Type
     - Description
   * - county_id
     - FK
     - Reference to dim_county
   * - industry_id
     - FK
     - Reference to dim_industry
   * - ownership_id
     - FK
     - Reference to dim_ownership
   * - time_id
     - FK
     - Reference to dim_time
   * - establishments
     - INT
     - Number of business establishments
   * - employment
     - INT
     - Annual average employment
   * - total_wages_usd
     - DECIMAL
     - Total annual wages paid
   * - avg_weekly_wage_usd
     - INT
     - Average weekly wage per worker
   * - avg_annual_pay_usd
     - INT
     - Average annual pay per worker
   * - lq_employment
     - DECIMAL
     - Location quotient for employment
   * - lq_annual_pay
     - DECIMAL
     - Location quotient for pay
   * - disclosure_code
     - VARCHAR
     - Data suppression indicator

NAICS Hierarchy Levels
^^^^^^^^^^^^^^^^^^^^^^

The ``dim_industry`` table includes a ``naics_level`` column indicating
each code's position in the NAICS hierarchy:

.. list-table::
   :header-rows: 1
   :widths: 15 40 45

   * - naics_level
     - Meaning
     - Example
   * - 0
     - Grand total (all industries)
     - ``10``
   * - 2
     - Sector (includes compound codes)
     - ``31-33``, ``44-45``, ``48-49``
   * - 3
     - Subsector
     - ``336``
   * - 4
     - Industry group
     - ``3361``
   * - 5
     - NAICS industry
     - ``33611``
   * - 6
     - National industry (leaf)
     - ``336111``
   * - 98
     - BLS ownership variants
     - (special codes)
   * - 99
     - BLS supersectors
     - ``1011``--``1029``

.. warning::

   QCEW reports wages at **every** hierarchy level. Parent-level wages
   **include all children**. Summing across levels without filtering
   produces catastrophic double-counting (empirically 10.4x in Wayne
   County). Always filter to ``naics_level = 6`` when aggregating
   wages. See :doc:`/concepts/naics-hierarchy` for full explanation.

National Wages Cache Table
^^^^^^^^^^^^^^^^^^^^^^^^^^

``InterpolatingBEASource`` maintains a materialized cache table
``_cache_national_wages_bea`` to avoid repeated aggregation of the
43M-row ``fact_qcew_annual`` table. The cache stores pre-aggregated
national wages per BEA industry per year.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Column
     - Type
     - Description
   * - bea_industry_id
     - INT
     - FK to ``dim_bea_industry``
   * - year
     - INT
     - Data year
   * - national_wages_usd
     - REAL
     - Sum of ``total_wages_usd`` for all leaf NAICS in this BEA industry
   * - cache_version
     - INT
     - Schema version (current: 2). Stale caches are auto-rebuilt.

The cache is built on first use and persists across sessions. When the
``cache_version`` column is missing or does not match the current version,
the table is dropped and rebuilt with leaf-only NAICS filtering.

API Client
----------

The ``QcewAPIClient`` class provides direct access to the BLS QCEW Open Data API:

.. code-block:: python

    from babylon.data.qcew import QcewAPIClient, QcewAPIError

    with QcewAPIClient() as client:
        try:
            # Fetch by area (state/county FIPS code)
            records = list(client.get_area_annual_data(2023, "01001"))  # Autauga County, AL

            # Fetch by industry (NAICS code)
            records = list(client.get_industry_annual_data(2023, "31-33"))  # Manufacturing

        except QcewAPIError as e:
            print(f"API error {e.status_code}: {e.message}")

API Features:

- **Rate limiting**: 0.5s delay between requests (polite to BLS servers)
- **Retry logic**: Exponential backoff on 429/5xx errors
- **CSV parsing**: Automatic parsing of BLS CSV responses
- **Error handling**: Distinguishes 404 (missing data) from server errors

Marxian Analysis
----------------

QCEW data supports Marxian class analysis through:

**Class Composition by Sector**

Industries are mapped to class composition categories directly on
``dim_industry`` (the ``dim_sector`` rollup was retired 2026-07-17, ADR075):

- ``goods_producing`` - Manufacturing, construction (productive labor)
- ``service_producing`` - Services, retail (mixed productive/unproductive)
- ``circulation`` - Finance, real estate (unproductive, surplus redistribution)
- ``government`` - Public administration (social reproduction)
- ``extraction`` - Mining, resources (ground rent extraction)

**Location Quotients for Labor Market Analysis**

Location quotients (LQ) reveal industry concentration:

- LQ > 1.25: Industry concentrated (potential labor aristocracy formation)
- LQ < 0.75: Industry underrepresented (labor mobility constraints)

**Wage Differentials**

Cross-geographic wage comparisons reveal unequal exchange dynamics:

- Core metro areas: Higher wages, higher productivity
- Periphery counties: Lower wages, extraction patterns

See Also
--------

- :doc:`/concepts/naics-hierarchy` - Why NAICS hierarchy causes double-counting
  and the leaf-only filtering solution
- :doc:`fred-data` - FRED macroeconomic data
- :doc:`census-analysis` - Census ACS demographics
- :doc:`bea-department-mapping` - BEA industry to Marxian department mapping
- :mod:`babylon.data.qcew` - API reference
- `BLS QCEW Open Data <https://www.bls.gov/cew/additional-resources/open-data/>`_
