.. _faf-freight-data:

=====================================
BTS FAF5 Freight Analysis Data
=====================================

.. contents:: Table of Contents
   :local:
   :depth: 2

Reference for the Bureau of Transportation Statistics (BTS) Freight Analysis
Framework (FAF5) data files, including CFS area geography, file inventory,
CSV column schema, and the ``FAFLoader`` API.

For the theoretical background, see :ref:`imperial-rent-field`.
For the schema tables these populate, see :ref:`tensor-hierarchy-schema`.

----

What CFS Areas Are
==================

The Census Bureau's Commodity Flow Survey (CFS) aggregates US counties into
**Commodity Flow Survey (CFS) Areas**—approximately 130 geographic zones
designed for freight flow analysis. CFS areas represent:

- Single large counties (e.g., Los Angeles County alone is one CFS area)
- Multi-county metropolitan regions (e.g., Boston MA-NH CFS Area)
- Remainder-of-state areas (e.g., "Rest of California")

FAF5 uses approximately 132 CFS areas (the exact count varies by version).
These areas are intermediate between individual counties (~3,100) and states
(51), providing sufficient geographic resolution to identify regional value
flow patterns while maintaining manageable matrix dimensions (~130×130).

CFS areas are **not** the same as Census metropolitan statistical areas (MSAs)
or BEA economic areas. They are specific to the CFS/FAF geography.

**CFS area codes** in FAF5 are integer strings (e.g., ``"11"``, ``"119"``,
``"509"``). The region-level CSV uses zero-padded 3-digit codes (``"011"``,
``"119"``, ``"509"``); the loader normalizes these by stripping leading zeros
to produce bare integers.

----

File Inventory
==============

FAF5 data is **not included** in the repository due to file size. Files must
be downloaded manually from the BTS website (see :ref:`faf-obtain-data`).

.. list-table:: FAF5 Data Files in ``data/freight/faf/``
   :header-rows: 1
   :widths: 45 55

   * - File path
     - Contents
   * - ``region/FAF5.7.1_2018-2024.csv``
     - **Primary source** for FAFLoader. Zone-level (CFS Area) O-D flows,
       years 2018–2024. ~2.49M records. 600+ MB.
   * - ``FAF5.7.1_State_2018-2024.csv``
     - State-level aggregated flows (fallback if region/ absent). Lower
       resolution; not used when zone-level file is present.
   * - ``FAF5_metadata.xlsx``
     - Zone names, zone-to-state mapping, SCTG code descriptions, mode codes.
   * - ``region/FAF5_metadata.xlsx``
     - Metadata for the zone-level dataset.
   * - ``truck_origin_factors.csv``
     - County-level disaggregation factors for truck freight (origin).
   * - ``truck_destination_factors.csv``
     - County-level disaggregation factors for truck freight (destination).
   * - ``rail_origin_factors.csv``
     - County-level disaggregation factors for rail freight (origin).
   * - ``rail_destination_factors.csv``
     - County-level disaggregation factors for rail freight (destination).
   * - ``water_origin_factors.csv``
     - County-level disaggregation factors for water freight (origin).
   * - ``water_destination_factors.csv``
     - County-level disaggregation factors for water freight (destination).
   * - ``pipeline_origin_factors.csv``
     - County-level disaggregation factors for pipeline freight (origin).
   * - ``pipeline_destination_factors.csv``
     - County-level disaggregation factors for pipeline freight (destination).
   * - ``county/01_Alabama.zip``
     - County-level estimate downloads (partial; see ``download_county.sh``).
   * - ``download_county.sh``
     - Script to bulk-download all 51 state county ZIP files.

**File search priority in FAFLoader:** The loader searches for FAF5 CSV in this
order:

1. ``data/freight/faf/FAF5.csv`` (exact name)
2. ``data/freight/faf/region/FAF5*.csv`` (zone-level, preferred)
3. ``data/freight/faf/FAF5*.csv`` (any FAF5 CSV at top level, fallback)

The zone-level file is preferred because it has higher geographic resolution
(~130 CFS areas vs. 51 states).

----

CSV Column Schema
=================

FAF5 CSV files use a long-wide format: each row is one (origin, destination,
commodity, mode) combination, with separate columns for each year's flow values.

.. list-table:: FAF5 CSV Column Definitions
   :header-rows: 1
   :widths: 20 15 65

   * - Column
     - Type
     - Description
   * - ``dms_orig``
     - string
     - Origin CFS Area code (integer string, may be zero-padded in region CSV).
       Normalized by FAFLoader to bare integer string.
   * - ``dms_dest``
     - string
     - Destination CFS Area code. Same normalization as origin.
   * - ``sctg2``
     - integer
     - SCTG 2-digit commodity code (1–43; code 40 not used by FAF5).
   * - ``dms_mode``
     - integer
     - Transport mode code (see mode codes below).
   * - ``tons_YYYY``
     - float
     - Estimated shipment weight for year YYYY, in thousands of short tons.
       Multiple year columns present (e.g., ``tons_2018``, ``tons_2019``, ...,
       ``tons_2024``).
   * - ``value_YYYY``
     - float
     - Estimated shipment value for year YYYY, in millions of USD.
   * - ``tmiles_YYYY``
     - float
     - Estimated ton-miles for year YYYY, in millions.

The loader extracts year-specific columns by pattern matching
(``value_{year}``, ``tons_{year}``, ``tmiles_{year}``). If the target year
has no corresponding column, loading returns 0 records with a warning.

Transport Mode Codes
---------------------

.. list-table:: FAF5 Transport Mode Codes
   :header-rows: 1
   :widths: 10 90

   * - Code
     - Description
   * - ``1``
     - Truck (all truck types: single-unit, combination)
   * - ``2``
     - Rail (class I railroads and short lines)
   * - ``3``
     - Water (domestic waterways, Great Lakes, coastal)
   * - ``4``
     - Air (domestic air cargo)
   * - ``5``
     - Pipeline (crude oil, natural gas, refined products)

Mode codes are stored as strings in ``fact_faf_commodity_flow.mode_code``
(``'1'``, ``'2'``, etc.).

SCTG Commodity Code Ranges
---------------------------

The Standard Classification of Transported Goods (SCTG) groups commodities
into 2-digit codes 01–43 (code 40 unused by FAF5). The broad categories used
in ``dim_sctg_commodity.category`` are:

- **agriculture**: Farm products, fish, forest products (early codes)
- **mining**: Minerals, coal, petroleum (middle codes)
- **manufacturing**: Manufactured goods (later codes)

Specific SCTG-to-name mappings are available in the FAF5 metadata XLSX
(``FAF5_metadata.xlsx``, ``SCTG2`` worksheet).

----

FAFLoader API
=============

.. class:: FAFLoader

   Loader for BTS FAF5 commodity flow data into the 3NF SQLite schema.

   Reads FAF5 CSV data and populates:

   - ``dim_cfs_area``: CFS Area dimension records (on-demand)
   - ``dim_sctg_commodity``: SCTG commodity records (on-demand)
   - ``fact_faf_commodity_flow``: O-D flow records by year

   .. method:: __init__(config=None, data_dir=None)

      :param config: ``LoaderConfig`` for batch size and operational settings.
         Default: ``LoaderConfig()`` with ``batch_size=1000``.
      :param data_dir: Base data directory. Default: ``Path("data")`` relative
         to project working directory.

   .. method:: load(session, reset=True, verbose=True, **kwargs) -> LoadStats

      Load FAF5 commodity flow data for a single year.

      :param session: SQLAlchemy ``Session`` connected to the normalized DB.
      :param reset: If ``True``, delete existing records in all three tables
         before loading. Default: ``True``.
      :param verbose: If ``True``, log progress. Default: ``True``.
      :param kwargs: Accepts ``year`` (``int`` or ``str``). Target year for
         extraction from the multi-year CSV. Default: ``2022``.
      :returns: ``LoadStats`` with fields:

         - ``facts_loaded["fact_faf_commodity_flow"]``: rows inserted
         - ``files_processed``: 1 if successful, 0 if CSV not found
         - ``errors``: error strings (non-empty if CSV absent)

      :raises: Does not raise; missing CSV produces an error in ``LoadStats``.

   **Table population details:**

   - ``dim_cfs_area``: Records created on-demand as CFS codes appear in CSV.
     If no metadata file is available, names default to ``"CFS Area {code}"``.
   - ``dim_sctg_commodity``: Records created on-demand as SCTG codes appear.
     Names default to ``"SCTG {code}"`` without metadata.
   - ``fact_faf_commodity_flow``: Batch-inserted in chunks of ``batch_size``
     rows. Zero values stored as ``NULL`` (sparse).

   **In-memory caches:** The loader maintains ``area_cache`` and ``sctg_cache``
   dicts to avoid repeated database lookups during CSV processing. These are
   local to each ``load()`` call.

   **Data source registration:** Creates or retrieves a ``dim_data_source``
   record with ``source_code="BTS_FAF5"`` and
   ``source_agency="Bureau of Transportation Statistics"``.

   **Example:**

   .. code-block:: python

      from babylon.data.bts.faf_loader import FAFLoader
      from babylon.data.reference.database import get_session_factory

      loader = FAFLoader()
      session_factory = get_session_factory()
      with session_factory() as session:
          stats = loader.load(session, year=2022)
      if stats.errors:
          print("FAF CSV not found:", stats.errors[0])
      else:
          print(f"Loaded {stats.facts_loaded['fact_faf_commodity_flow']} flows")

FAFCSVParser API
-----------------

.. class:: FAFCSVParser

   Internal CSV parser. Used by ``FAFLoader``; can be used independently.

   .. method:: extract_year_columns(headers, year) -> dict[str, str]

      Find year-specific column names in the CSV header.

      :param headers: List of CSV header strings.
      :param year: Target year integer.
      :returns: Dict with keys ``'value'``, ``'tons'``, ``'tmiles'`` mapping
         to actual column names. Keys absent if column not found in headers.

   .. method:: parse_row(row, year_cols) -> tuple | None

      Parse a single CSV row into a flow record.

      :param row: Dict of column name → value string (from ``csv.DictReader``).
      :param year_cols: Year column mapping from ``extract_year_columns``.
      :returns: Tuple ``(origin, dest, sctg2, mode, tons, value, tmiles)`` or
         ``None`` if required fields are missing or invalid.

      **CFS code normalization:** Origin and destination codes are parsed as
      integers and converted back to strings, stripping leading zeros:
      ``"011"`` → ``"11"``. Invalid codes (non-numeric) return ``None``.

.. _faf-obtain-data:

How to Obtain the Data
=======================

FAF5 data must be downloaded manually from BTS:

1. Go to `<https://www.bts.gov/faf>`_
2. Under "Data Download", select "FAF5 Data" → "Origin-Destination Data"
3. Download the zone-level CSV (``FAF5.7.1_2018-2024.csv`` or equivalent)
4. Place in ``data/freight/faf/region/``
5. Run ``FAFLoader.load(session, year=2022)``

For county-level disaggregation factors (mode-specific):

.. code-block:: bash

   cd data/freight/faf
   bash download_county.sh   # downloads all 51 state ZIP files to county/

The metadata XLSX (already present at ``data/freight/faf/FAF5_metadata.xlsx``)
contains zone names and zone-to-state mapping.

The factor files populate ``bridge_cfs_county`` via
``python -m babylon_data.faf_bridge`` (babylon-data repo; dry-run default,
``--execute`` to commit). The bridge's single ``allocation_weight`` per
(zone, county) is a Derived 2022-tonnage-weighted mean of the mode × SCTG-G5
factors, origin+destination pooled; per-zone weight sums report the
covered-tonnage share and are not renormalized.

----

Related Documentation
=====================

- :ref:`imperial-rent-field` — Theory behind spatial value extraction
- :ref:`tensor-hierarchy-schema` — ``dim_cfs_area``, ``dim_sctg_commodity``,
  ``fact_faf_commodity_flow`` table definitions
- :ref:`tensor-hierarchy-reference` — ``GeographicFlow`` and
  ``ImperialRentField`` tensor types
- :mod:`babylon.data.bts.faf_loader` — FAFLoader implementation
- :mod:`babylon.domain.economics.tensor_hierarchy.geographic_flow` — Geographic flow
  computation (``DefaultImperialRentComputer``)
