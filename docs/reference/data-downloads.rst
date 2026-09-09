Data Download Instructions
==========================

This guide documents how to download and set up external data files required by
the Babylon data ingestion system. Most loaders fetch data from APIs, but some
require manual file downloads.

Directory Structure
-------------------

.. code-block:: text

    data/
    ├── census/              # Census ACS data (auto-downloaded via API)
    ├── duckdb/              # DuckDB database files (PRIMARY - normalized 3NF)
    │   └── marxist-data-3NF.duckdb  # Main analytical database
    ├── dot/                 # DOT HPMS road segment data (MANUAL DOWNLOAD)
    ├── employment_industry/ # BLS employment data (auto-downloaded)
    ├── energy/              # EIA energy data (auto-downloaded via API)
    ├── fcc/                 # FCC broadband data (downloaded via mise task)
    ├── imperial_rent/       # Trade data files
    ├── lodes/               # Census LODES crosswalk (MANUAL DOWNLOAD)
    ├── mass_labor_hours/    # Labor hours data
    ├── productivity/        # BLS productivity data
    ├── qcew/                # BLS QCEW historical files (MANUAL DOWNLOAD for pre-2021)
    ├── raw_mats/            # USGS materials data
    └── sqlite/              # LEGACY SQLite files (migration source)

Manual Download Instructions
----------------------------

QCEW Historical Files (2013-2020)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The QCEW loader uses the BLS API for years 2021+, but requires local CSV files for
historical years (2013-2020).

**Download Source:** https://www.bls.gov/cew/downloadable-data-files.htm

**Steps:**

1. Navigate to "Annual Averages by State" or "Annual Averages by Area"
2. Download CSV files for each year needed (2013-2020)
3. Place files in ``data/qcew/``

**Expected Format:** CSV files with columns matching BLS QCEW format:

- ``area_fips``, ``own_code``, ``industry_code``, ``year``, ``qtr``
- ``annual_avg_estabs``, ``annual_avg_emplvl``, ``total_annual_wages``, etc.

**Example Files:**

.. code-block:: text

    data/qcew/
    ├── 2013.annual.singlefile.csv
    ├── 2014.annual.singlefile.csv
    ├── ...
    └── 2020.annual.singlefile.csv

DOT HPMS Road Segment Data
~~~~~~~~~~~~~~~~~~~~~~~~~~

The HPMS loader requires spatial road segment data from FHWA.

**Download Source:** https://www.fhwa.dot.gov/policyinformation/hpms.cfm

**Steps:**

1. Navigate to HPMS Public Release data
2. Download the "All Sections" CSV file for the desired year
3. Place in ``data/dot/``

**Expected Filename Pattern:** ``HPMS_Spatial*Sections*.csv``

**Example:**

.. code-block:: text

    data/dot/
    └── HPMS_Spatial_All_Sections_-_2024.csv

**Key Columns Used (2024 export):**

- ``StateID``, ``COUNTY_ID`` - Geographic identifiers
- ``ROUTE_ID``, ``BEGIN_POINT``, ``END_POINT`` - Route segment location
- ``AADT``, ``AADT_SINGLE_UNIT``, ``AADT_COMBINATION`` - Annual average daily
  traffic, with truck splits
- ``F_SYSTEM`` - Functional system classification
- ``NHS``, ``NHFN`` - National Highway System / Highway Freight Network flags
- ``THROUGH_LANES``, ``SPEED_LIMIT``, ``LANE_WIDTH``, ``SectionLength`` -
  Physical characteristics
- ``YEAR_RECORD``, ``line`` - Observation year and WKT LineString geometry

Load with ``python -m babylon_data.dot --execute`` (babylon-data package;
Michigan-scoped by default via ``--state-fips``).

Census LODES Crosswalk
~~~~~~~~~~~~~~~~~~~~~~

The LODES loader provides Census block to county mapping for sub-county analysis.

**Download Source:** https://lehd.ces.census.gov/data/lodes/

**Steps:**

1. Navigate to LODES 8 (or latest version)
2. Download the national crosswalk file: ``us_xwalk.csv.gz``
3. Place in ``data/lodes/`` (can remain gzipped)

**Expected Files:**

.. code-block:: text

    data/lodes/
    ├── us_xwalk.csv      # Uncompressed
    └── us_xwalk.csv.gz   # Or compressed (both work)

**Key Columns Used:**

- ``tabblk2020`` - Census block GEOID (15 digits)
- ``st``, ``cty`` - State and county FIPS codes
- ``trct`` - Census tract code
- ``cbsa``, ``cbsaname`` - Core-Based Statistical Area info

Automated Downloads
-------------------

The following data sources are fetched via APIs and don't require manual downloads:

.. list-table::
   :header-rows: 1
   :widths: 25 35 40

   * - Loader
     - Source
     - API
   * - Census
     - Census Bureau
     - American Community Survey API
   * - QCEW (2021+)
     - BLS
     - QCEW Open Data API
   * - Energy
     - EIA
     - Energy Information Admin API
   * - Employment Industry
     - BLS
     - BLS Public Data API
   * - HIFLD Infrastructure
     - FEMA/Esri
     - ArcGIS FeatureServer

FCC Broadband Data
~~~~~~~~~~~~~~~~~~

FCC data requires a download task before loading:

.. code-block:: bash

    mise run data:fcc-download

This populates ``data/fcc/downloads/`` with the required CSV files.

Verifying Data Files
--------------------

You can verify data files are present by checking the data directories or
running individual loaders.

Or check specific loaders:

.. code-block:: python

    from babylon.data.preflight import run_preflight_checks
    from pathlib import Path

    result = run_preflight_checks(Path('data'), loaders=['qcew', 'dot', 'lodes'])
    for check in result.checks:
        print(f'{check.status.upper():6} {check.check_id}: {check.message}')
        if check.hint:
            print(f'       Hint: {check.hint}')

Environment Variables
---------------------

Some loaders require API keys:

.. list-table::
   :header-rows: 1
   :widths: 25 35 40

   * - Variable
     - Required For
     - Get From
   * - ``ENERGY_API_KEY``
     - EIA Energy API
     - https://www.eia.gov/opendata/register.php
   * - ``CENSUS_API_KEY``
     - Census API (optional)
     - https://api.census.gov/data/key_signup.html
   * - ``FCC_API_KEY``
     - FCC Broadband API
     - https://broadbandmap.fcc.gov/about

Set in your environment or ``.env`` file:

.. code-block:: bash

    export ENERGY_API_KEY="your-eia-api-key"

Data Freshness
--------------

Data files should be updated periodically:

.. list-table::
   :header-rows: 1
   :widths: 25 35 40

   * - Source
     - Update Frequency
     - Recommended Refresh
   * - QCEW
     - Quarterly
     - Annually
   * - HPMS
     - Annually
     - Annually
   * - LODES
     - Annually
     - Every 2-3 years
   * - Census ACS
     - Annually
     - Annually

Troubleshooting
---------------

"No files found matching pattern"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The loader couldn't find expected files. Check:

1. Files are in the correct directory
2. Filenames match the expected pattern
3. Files are not empty or corrupted

"FIPS code not in database"
~~~~~~~~~~~~~~~~~~~~~~~~~~~

The geographic identifier couldn't be matched. Ensure:

1. Census loader has run first (``mise run data:census``)
2. Geographic data covers the required years
3. FIPS codes in source files are valid

API Rate Limits
~~~~~~~~~~~~~~~

For API-based loaders, if you hit rate limits:

1. Wait and retry (most loaders have automatic retry)
2. Use ``--batch-size`` flag to reduce request frequency
3. Check API documentation for limits
