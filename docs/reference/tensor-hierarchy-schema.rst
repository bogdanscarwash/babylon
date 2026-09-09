.. _tensor-hierarchy-schema:

=========================================
Tensor Hierarchy Database Schema
=========================================

.. contents:: Table of Contents
   :local:
   :depth: 2

Data dictionary for the six SQLite tables added in Feature 025 (tensor
hierarchy). These tables extend the existing 3NF schema in
``src/babylon/data/reference/schema.py``.

For architectural context, see :ref:`tensor-hierarchy-concept`.
For the loader APIs that populate these tables, see :ref:`bea-io-tables`
and :ref:`faf-freight-data`.

----

BEA I-O Tables
==============

dim_bea_io_table_type
----------------------

Classifies which BEA Input-Output table a coefficient was derived from.
One row per table type (USE, MAKE, SUPPLY, TOTAL_REQ).

.. list-table::
   :header-rows: 1
   :widths: 20 12 8 60

   * - Column
     - Type
     - Null
     - Description
   * - ``id``
     - INTEGER
     - NOT NULL
     - Auto-increment primary key.
   * - ``table_type``
     - VARCHAR(20)
     - NOT NULL
     - BEA table identifier. Unique. Check constraint:
       ``IN ('USE', 'MAKE', 'SUPPLY', 'TOTAL_REQ')``.
   * - ``description``
     - TEXT
     - NULL
     - Human-readable description of the table type.

**Allowed values for** ``table_type``:

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Value
     - Description
   * - ``USE``
     - Use table at Producers' prices (IOUse_Before_Redefinitions_PRO).
       Commodity use by industry. Source for direct requirements A[i,j].
       This is the primary value loaded by BEAIOLoader.
   * - ``MAKE``
     - Make table (IOMake_Before_Redefinitions_PRO).
       Industry-by-commodity output structure.
   * - ``SUPPLY``
     - Supply table. Total supply of each commodity from all sources
       (domestic production + imports).
   * - ``TOTAL_REQ``
     - Total requirements table (Leontief inverse) as published by BEA.
       Alternative to computing L = (I − A)⁻¹ internally.

**Populated by:** :class:`~babylon.data.bea.io_loader.BEAIOLoader`.


fact_bea_io_coefficient
------------------------

Sparse storage of BEA Input-Output direct requirements coefficients A[i,j]
by year and industry pair. Zero entries are omitted (sparse storage).

One row per (year, table_type, source_industry, target_industry) combination.
A[i,j] = dollar value of industry i's output required per dollar of industry
j's output.

.. list-table::
   :header-rows: 1
   :widths: 25 12 8 55

   * - Column
     - Type
     - Null
     - Description
   * - ``id``
     - INTEGER
     - NOT NULL
     - Auto-increment primary key.
   * - ``time_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_time.time_id``. Annual time record for the data year.
   * - ``table_type_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_bea_io_table_type.id``.
   * - ``source_industry_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_bea_industry.bea_industry_id``. Industry *i* in A[i,j].
       This is the industry whose output is *required as input*.
   * - ``target_industry_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_bea_industry.bea_industry_id``. Industry *j* in A[i,j].
       This is the industry whose output is *being produced*.
   * - ``coefficient``
     - FLOAT
     - NOT NULL
     - Direct requirements coefficient A[i,j]. Dollar value of industry i
       output required per dollar of industry j output. Range: [0, 1).
       Zero values are not stored (sparse storage).

**Unique constraint:** ``(time_id, table_type_id, source_industry_id, target_industry_id)``
named ``uq_bea_io_coeff``. Enables idempotent upsert on reload.

**Indexes:**

- ``idx_bea_io_coeff_time`` on ``time_id``
- ``idx_bea_io_coeff_table_type`` on ``table_type_id``
- ``idx_bea_io_coeff_source`` on ``source_industry_id``
- ``idx_bea_io_coeff_target`` on ``target_industry_id``

**Note on matrix semantics:** A[i,j] uses source=row, target=column convention.
``source_industry_id`` corresponds to row *i* (the input commodity industry);
``target_industry_id`` corresponds to column *j* (the using industry). To
reconstruct the full matrix for year Y:

.. code-block:: python

   coeffs = session.query(FactBEAIOCoefficient).filter(time_id=Y, table_type=USE)
   matrix[src_idx, tgt_idx] = coeff.coefficient

**Populated by:** :class:`~babylon.data.bea.io_loader.BEAIOLoader`.

----

CFS Area and FAF Flow Tables
==============================

dim_cfs_area
-------------

Census Commodity Flow Survey (CFS) geographic areas — the ~130 zones used by
the Bureau of Transportation Statistics for freight analysis.

CFS areas are aggregations of one or more counties. They provide a geographic
resolution between individual counties (~3,100) and states (51). The exact
count varies by FAF5 version; FAF5.7.1 uses approximately 132 zones.

.. list-table::
   :header-rows: 1
   :widths: 20 12 8 60

   * - Column
     - Type
     - Null
     - Description
   * - ``cfs_area_id``
     - INTEGER
     - NOT NULL
     - Auto-increment primary key.
   * - ``cfs_code``
     - VARCHAR(10)
     - NOT NULL
     - CFS Area code as string (e.g., ``"11"``, ``"119"``). Unique.
       Codes are bare integers (leading zeros stripped from FAF5 CSV).
   * - ``cfs_name``
     - VARCHAR(200)
     - NOT NULL
     - Human-readable CFS area name (e.g., ``"CFS Area 11"`` or
       ``"Boston, MA-NH CFS Area"`` if metadata is available).
   * - ``state_id``
     - INTEGER
     - NULL
     - Foreign key → ``dim_state.state_id``. The primary state for this
       CFS area. ``NULL`` if multi-state area or not yet mapped.

**Index:** ``idx_cfs_area_state`` on ``state_id``.

**Populated by:** :class:`~babylon.data.bts.faf_loader.FAFLoader` (on-demand,
one record per unique CFS code encountered in FAF5 CSV).


bridge_cfs_county
------------------

Many-to-many junction table mapping each CFS Area to the counties it contains,
with allocation weights. Weights represent what fraction of the CFS area's
freight is attributable to each constituent county.

.. list-table::
   :header-rows: 1
   :widths: 22 14 8 56

   * - Column
     - Type
     - Null
     - Description
   * - ``cfs_area_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_cfs_area.cfs_area_id``. Part of composite PK.
   * - ``county_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_county.county_id``. Part of composite PK.
   * - ``allocation_weight``
     - NUMERIC(10,8)
     - NOT NULL
     - Fraction of CFS area allocated to this county. Per CFS area, weights
       sum to the covered-tonnage share (≤ 1.0; 0.89–1.00 across the 132 FAF5
       zones in the PER-31 population). The residual is tonnage on modes the
       county factors do not allocate (air, intermodal, other) and stays at
       CFS-area grain.

**Primary key:** Composite ``(cfs_area_id, county_id)``.

**Allocation weight semantics:** To disaggregate a CFS-area-level value (e.g.,
imperial rent φ[a]) to county level, over the factor-covered modes:

.. code-block:: python

   county_value = phi_cfs_area * allocation_weight

The populated weights (PER-31, 2026-09-04) are a Derived 2022-tonnage-weighted
mean of the FAF5 mode × SCTG-G5 county factors, loaded by
``babylon_data.faf_bridge`` — not population or employment proxies. They are
deliberately not renormalized to 1.0: conservation holds only over the covered
modes (truck, rail, water, pipeline), so county totals reconcile with their
zone up to the reported covered share.


dim_sctg_commodity
-------------------

Standard Classification of Transported Goods (SCTG) commodity codes. The SCTG
classification is used by the Census Bureau's Commodity Flow Survey and the BTS
Freight Analysis Framework. FAF5 uses 2-digit SCTG codes (42 codes, numbered
01–43 with code 40 unused).

.. list-table::
   :header-rows: 1
   :widths: 20 12 8 60

   * - Column
     - Type
     - Null
     - Description
   * - ``sctg_id``
     - INTEGER
     - NOT NULL
     - Auto-increment primary key.
   * - ``sctg_code``
     - VARCHAR(5)
     - NOT NULL
     - Zero-padded 2-digit SCTG code (e.g., ``"01"``, ``"43"``). Unique.
   * - ``sctg_name``
     - VARCHAR(200)
     - NOT NULL
     - Commodity description (e.g., ``"SCTG 01"`` or full name if available).
   * - ``category``
     - VARCHAR(50)
     - NULL
     - Broad commodity category: ``'agriculture'``, ``'mining'``,
       ``'manufacturing'``, or ``NULL`` if uncategorized.
   * - ``strategic_value``
     - VARCHAR(20)
     - NULL
     - Strategic importance for simulation events.
       Check constraint: ``IN ('critical', 'high', 'medium', 'low') OR NULL``.

**Indexes:** ``idx_sctg_category`` on ``category``.

**Populated by:** :class:`~babylon.data.bts.faf_loader.FAFLoader` (on-demand,
one record per unique sctg2 code encountered in FAF5 CSV).


fact_faf_commodity_flow
------------------------

BTS FAF5 commodity flows at CFS Area geographic resolution. One row per
(origin, destination, commodity, mode, year) combination.

Distinct from ``fact_commodity_flow`` (county-level Census CFS data).
This table preserves FAF5's native ~130-area resolution rather than
disaggregating to counties.

.. list-table::
   :header-rows: 1
   :widths: 22 14 8 56

   * - Column
     - Type
     - Null
     - Description
   * - ``flow_id``
     - INTEGER
     - NOT NULL
     - Auto-increment primary key.
   * - ``origin_cfs_area_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_cfs_area.cfs_area_id``. Shipment origin CFS area.
   * - ``dest_cfs_area_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_cfs_area.cfs_area_id``. Shipment destination CFS area.
   * - ``sctg_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_sctg_commodity.sctg_id``. SCTG 2-digit commodity.
   * - ``source_id``
     - INTEGER
     - NOT NULL
     - Foreign key → ``dim_data_source.source_id``. Always ``BTS_FAF5``.
   * - ``year``
     - INTEGER
     - NOT NULL
     - Calendar year of the flow estimate (e.g., 2017, 2022).
   * - ``value_millions``
     - NUMERIC(14,2)
     - NULL
     - Estimated shipment value in millions of USD.
       ``NULL`` when the original CSV value was zero (sparse storage).
   * - ``tons_thousands``
     - NUMERIC(14,2)
     - NULL
     - Estimated shipment weight in thousands of short tons.
       ``NULL`` when original CSV value was zero.
   * - ``ton_miles_millions``
     - NUMERIC(14,2)
     - NULL
     - Estimated ton-miles in millions.
       ``NULL`` when original CSV value was zero.
   * - ``mode_code``
     - VARCHAR(10)
     - NULL
     - Transport mode code as string: ``'1'`` = truck, ``'2'`` = rail,
       ``'3'`` = water, ``'4'`` = air, ``'5'`` = pipeline.

**Indexes:**

- ``idx_faf_flow_origin`` on ``origin_cfs_area_id``
- ``idx_faf_flow_dest`` on ``dest_cfs_area_id``
- ``idx_faf_flow_sctg`` on ``sctg_id``
- ``idx_faf_flow_year`` on ``year``

**Sparsity note:** All three flow value fields use ``NULL`` for zero rather
than storing explicit zeros. This reduces storage significantly since FAF5
has many zero-flow origin-destination pairs for specific commodity codes and
modes. When reconstructing the O-D matrix, treat ``NULL`` as 0.

**Scale (2022 data):** ~2.49 million records, representing $18.7 trillion in
domestic commodity shipments across ~130 CFS areas, 42 SCTG codes, and 5
transport modes.

**Populated by:** :class:`~babylon.data.bts.faf_loader.FAFLoader`.

----

Related Documentation
=====================

- :ref:`tensor-hierarchy-concept` — Architectural rationale
- :ref:`tensor-hierarchy-reference` — Tensor type data dictionary
- :ref:`bea-io-tables` — BEA loader API and file inventory
- :ref:`faf-freight-data` — FAF loader API and file inventory
- :mod:`babylon.data.reference.schema` — SQLAlchemy ORM definitions
