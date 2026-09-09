.. _bea-department-mapping:

===========================================
BEA Industry to Marxian Department Mapping
===========================================

.. contents:: Table of Contents
   :local:
   :depth: 2

Complete reference for the mapping from BEA Summary-level industry codes to
the four Marxian departments. This mapping is loaded by
``DefaultDepartmentAggregator.get_default_mapping()`` from the TOML file
``src/babylon/economics/tensor_hierarchy/mappings/bea_to_department.toml``.

For the theoretical rationale behind the departments, see
:ref:`tensor-hierarchy-concept`. For the aggregation algorithm, see
:ref:`leontief-analysis`.

----

The Four Departments (Brief)
=============================

.. list-table::
   :header-rows: 1
   :widths: 10 25 65

   * - Code
     - Name
     - Description
   * - ``I``
     - Means of Production
     - Capital goods consumed productively. Mining, manufacturing (producer
       goods), construction, utilities, transport, finance, professional
       services, government.
   * - ``IIA``
     - Necessary Consumption
     - Wage goods required to reproduce labor power. Food, textiles, basic
       retail, basic lodging.
   * - ``IIB``
     - Luxury Consumption
     - Discretionary goods consumed by bourgeoisie and labor aristocracy.
       Consumer electronics, furniture, gambling, luxury retail.
   * - ``III``
     - Social Reproduction
     - Sectors producing labor power itself. Health care, education, social
       services, private households, religious organizations.

Theoretical sources: Marx (1885), Shaikh & Tonak (1994), Fortunati (1981).
See :ref:`tensor-hierarchy-concept` for full discussion of contested boundaries.

----

Department I: Means of Production
===================================

.. list-table::
   :header-rows: 1
   :widths: 15 35 50

   * - BEA Code
     - Industry Name
     - Classification Note
   * - ``1100A1``
     - Farms
     - Agricultural machinery, seeds, fertilizer are means of production.
   * - ``1130A1``
     - Forestry, fishing, and related activities
     - Raw material extraction; industrial inputs.
   * - ``211``
     - Oil and gas extraction
     - Energy inputs consumed productively.
   * - ``212``
     - Mining (except oil and gas)
     - Mineral extraction; producer goods.
   * - ``213``
     - Support activities for mining
     - Producer services enabling extraction.
   * - ``2200A0``
     - Utilities
     - Energy infrastructure; consumed by capital.
   * - ``23``
     - Construction
     - Fixed capital formation.
   * - ``3210A0``
     - Wood products
     - Industrial wood inputs.
   * - ``321``
     - Wood products (structural)
     - Overlap code for structural applications.
   * - ``324``
     - Petroleum and coal products
     - Industrial energy and chemical feedstocks.
   * - ``325``
     - Chemical products (industrial)
     - Industrial chemicals; producer goods dominant.
   * - ``326``
     - Plastics and rubber products
     - Industrial inputs for manufacturing.
   * - ``327``
     - Nonmetallic mineral products
     - Cement, glass, ceramics; construction inputs.
   * - ``331``
     - Primary metals
     - Steel, aluminum; capital goods input.
   * - ``332``
     - Fabricated metal products
     - Industrial hardware, structural metal.
   * - ``333``
     - Machinery
     - Capital equipment; core Dept I sector.
   * - ``3340A0``
     - Computer and electronic products
     - Producer computing; semiconductors, servers.
   * - ``335``
     - Electrical equipment and appliances
     - Industrial electrical equipment.
   * - ``3360A0``
     - Motor vehicles, bodies, trailers, parts
     - Commercial vehicles dominant (boundary case; see note below).
   * - ``3370A0``
     - Other transportation equipment
     - Aircraft, ships, railroad equipment; capital goods.
   * - ``3390A0``
     - Miscellaneous manufacturing
     - Instruments, industrial goods.
   * - ``4200``
     - Wholesale trade
     - Distribution infrastructure enabling capital circulation.
   * - ``481``
     - Air transportation
     - Producer logistics.
   * - ``482``
     - Rail transportation
     - Freight rail; capital circulation.
   * - ``483``
     - Water transportation
     - Bulk freight; capital circulation.
   * - ``484``
     - Truck transportation
     - Primary domestic freight mode.
   * - ``485``
     - Transit and ground passenger transportation
     - Worker transport; enables labor reproduction at scale.
   * - ``486``
     - Pipeline transportation
     - Energy infrastructure.
   * - ``487OS``
     - Other transportation and support activities
     - Port handling, air traffic control.
   * - ``493``
     - Warehousing and storage
     - Capital circulation infrastructure.
   * - ``5110A0``
     - Publishing industries (except internet)
     - Producer information goods.
   * - ``5120A0``
     - Motion picture and sound recording
     - Boundary case; placed in I as ideological capital production.
   * - ``5130A0``
     - Broadcasting and telecommunications
     - Communication infrastructure; producer services.
   * - ``5140A0``
     - Data processing, internet publishing
     - Digital infrastructure; producer computing.
   * - ``521CI``
     - Federal Reserve banks, credit intermediation
     - Financial capital circulation.
   * - ``523``
     - Securities, commodity contracts
     - Financial capital.
   * - ``524``
     - Insurance carriers
     - Risk-pooling for capital.
   * - ``525``
     - Funds, trusts, and other financial vehicles
     - Wealth management; financial capital.
   * - ``5311A0``
     - Rental and leasing services
     - Capital asset services.
   * - ``FIRE0``
     - Housing (owner-occupied residential)
     - Treated as capital (BEA imputed rental); see boundary note below.
   * - ``5411``
     - Legal services
     - Producer legal services.
   * - ``5412``
     - Accounting, tax preparation, bookkeeping
     - Producer financial services.
   * - ``5413``
     - Architectural, engineering, and related services
     - Capital project services.
   * - ``5414``
     - Specialized design services
     - Producer design.
   * - ``5415``
     - Computer systems design and related services
     - IT services for capital.
   * - ``5416``
     - Management, scientific, and technical consulting
     - Producer consulting.
   * - ``5417``
     - Scientific research and development
     - Innovation; creates future means of production.
   * - ``5418``
     - Advertising and related services
     - Realization function for capital.
   * - ``5419``
     - Other professional, scientific, and technical services
     - Producer services.
   * - ``5500A0``
     - Management of companies and enterprises
     - Coordination of capital.
   * - ``5611``
     - Administrative and support services
     - Business services.
   * - ``5612``
     - Waste management and remediation
     - Industrial waste handling.
   * - ``811``
     - Repair and maintenance
     - Capital goods maintenance.
   * - ``GSLE``
     - Federal government enterprises
     - State-owned productive capital (postal, Amtrak).
   * - ``GFG``
     - Federal general government
     - State coercive apparatus; enables capital accumulation.
   * - ``GSLG``
     - State and local general government
     - Local coercive apparatus; infrastructure.

----

Department IIa: Necessary Consumption
=======================================

Wage goods required to reproduce labor power—the minimum consumption bundle
the proletariat requires to continue working.

.. list-table::
   :header-rows: 1
   :widths: 15 35 50

   * - BEA Code
     - Industry Name
     - Classification Note
   * - ``311FT``
     - Food and beverage and tobacco products
     - Core wage goods; worker caloric reproduction.
   * - ``313TT``
     - Textile mills and textile product mills
     - Basic clothing; worker reproduction.
   * - ``315AL``
     - Apparel and leather and allied products
     - Basic clothing manufacturing.
   * - ``4400``
     - Retail trade
     - General retail; food and basic goods dominant. See boundary note.
   * - ``7211``
     - Traveler accommodation (basic lodging)
     - Budget hotels, motels; worker accommodation.
   * - ``FOOD``
     - Food services and drinking places
     - Basic food consumption; worker reproduction.

----

Department IIb: Luxury Consumption
=====================================

Discretionary goods consumed by bourgeoisie and labor aristocracy. These
sectors absorb surplus value without expanding the productive capacity of the
economy.

.. list-table::
   :header-rows: 1
   :widths: 15 35 50

   * - BEA Code
     - Industry Name
     - Classification Note
   * - ``334``
     - Computer and electronic products (consumer)
     - Consumer electronics, smartphones; discretionary.
   * - ``336``
     - Motor vehicles (consumer autos)
     - Consumer automobile aspect; boundary with Dept I (see note).
   * - ``337``
     - Furniture and related products
     - Discretionary household goods.
   * - ``339``
     - Miscellaneous manufacturing (consumer)
     - Consumer novelties, jewelry, toys.
   * - ``4481``
     - Clothing stores
     - Fashion retail; discretionary apparel.
   * - ``4521``
     - Department stores
     - General merchandise; upper-consumption bracket.
   * - ``4529``
     - Other general merchandise stores
     - Club stores; mixed but luxury-adjacent.
   * - ``4530``
     - Miscellaneous store retailers
     - Sporting goods, hobby, specialty retail.
   * - ``713``
     - Amusements, gambling, and recreation
     - Pure surplus value consumption.
   * - ``7211A0``
     - Hotels and motels (luxury travel)
     - Luxury lodging; bourgeois consumption.
   * - ``7212``
     - RV parks, recreational camps, rooming/boarding
     - Discretionary recreation.
   * - ``7213``
     - Dry cleaning and laundry
     - Bourgeois household services.
   * - ``AREP``
     - Arts, entertainment, recreation (consumer)
     - Cultural consumption; surplus value sink.

----

Department III: Social Reproduction
======================================

Sectors whose output is labor power itself—that produce, maintain, and
renew the capacity to work. Most labor in this department is performed as
unwaged domestic work (low *g*\ :sub:`33`), but the commodified fraction
appears in national accounts.

.. list-table::
   :header-rows: 1
   :widths: 15 35 50

   * - BEA Code
     - Industry Name
     - Classification Note
   * - ``621``
     - Ambulatory health care services
     - Outpatient care; worker health reproduction.
   * - ``622``
     - Hospitals
     - Inpatient care; emergency reproduction of labor power.
   * - ``623``
     - Nursing and residential care
     - Care for elderly/disabled; social reproduction.
   * - ``624``
     - Social assistance
     - Social services; labor power maintenance.
   * - ``6111``
     - Elementary and secondary schools
     - Credential/skill formation; core Dept III.
   * - ``6112``
     - Junior colleges
     - Postsecondary credential formation.
   * - ``6113``
     - Colleges, universities, and professional schools
     - Higher education; advanced credential formation.
   * - ``6114``
     - Business schools and computer training
     - Vocational skill formation.
   * - ``6115``
     - Technical and trade schools
     - Craft/technical skill formation.
   * - ``6116``
     - Other schools and instruction
     - Supplementary education.
   * - ``6117``
     - Educational support services
     - Administrative support for education sector.
   * - ``814``
     - Private households (domestic service workers)
     - Paid domestic labor; visible tip of unwaged iceberg.
   * - ``8139``
     - Religious, grant-making, civic organizations
     - Social cohesion; ideological reproduction.
   * - ``8131``
     - Religious organizations
     - Ideological reproduction; community social support.

----

Boundary Cases and Notes
==========================

**Motor vehicles (3360A0 in Dept I; 336 in Dept IIb):** Two BEA codes
appear in different departments. Code ``3360A0`` (manufacturing) is placed
in Department I because the industry produces both commercial vehicles and
consumer automobiles—and commercial use dominates by output value and
productive function. Code ``336`` (retail auto sales) is in Department IIb.

**Retail trade (4400 in Dept IIa) vs. specialty retail (4481, 4521, 4529,
4530 in Dept IIb):** General retail (BEA 4400) includes food retail, which
makes it primarily a wage-goods distributor. Luxury and discretionary
specialty retailers are separated into IIb with more specific codes.

**Owner-occupied housing (FIRE0 in Dept I):** The BEA treats imputed
owner-occupied rental income as capital output. Following Shaikh & Tonak,
housing is classified as capital (Dept I) rather than consumption. This
is a contested boundary: housing is *simultaneously* a consumption good and
a capital asset. The productive capital interpretation is the default.

**Future refinement:** The MVP mapping uses dominant-use classification
(one industry → one department). A more precise approach would use the
BEA commodity-by-industry bridge tables to fractionally allocate mixed
industries by output shares. This is noted in the TOML metadata but not yet
implemented.

----

How to Update the Mapping
==========================

The mapping is defined in:

.. code-block:: text

   src/babylon/economics/tensor_hierarchy/mappings/bea_to_department.toml

**TOML structure:**

.. code-block:: toml

   [departments]
   I   = ["1100A1", "1130A1", ...]   # List of BEA codes for Dept I
   IIA = ["311FT", "313TT", ...]     # List for Dept IIa
   IIB = ["334", "336", ...]         # List for Dept IIb
   III = ["621", "622", ...]         # List for Dept III

   [metadata]
   version = "1.0.0"
   date = "2026-02-26"
   bea_classification = "2007 NAICS-based BEA Summary level"
   reference = "..."
   notes = "..."

To add or move an industry:

1. Find the BEA Summary code in the BEA I-O Use table column headers.
2. Remove it from its current department list (if present).
3. Add it to the correct department list.
4. Add a note in ``[metadata].notes`` explaining the rationale.
5. Re-run tests: ``uv run pytest tests/unit/economics/tensor_hierarchy/``.

**How DefaultDepartmentAggregator reads it:**

.. code-block:: python

   aggregator = DefaultDepartmentAggregator()
   mapping = aggregator.get_default_mapping()
   # mapping: {"1100A1": "I", "311FT": "IIA", "334": "IIB", "621": "III", ...}

The aggregator reads the TOML at call time (not cached at import). BEA codes
not present in the mapping are silently excluded from aggregation. This is
intentional: unmapped industries contribute to the ~70-industry matrix but
not to the 4-department matrix.

**BEA code lookup:** The BEA classification (2007 NAICS-based Summary level)
is documented in the BEA I-O XLSX column headers and in the BEA interactive
data portal at `<https://www.bea.gov/industry/input-output-accounts-data>`_.

----

Related Documentation
=====================

- :ref:`tensor-hierarchy-concept` — Department theory and boundary case rationale
- :ref:`leontief-analysis` — How ~70 industries aggregate to 4 departments
- :ref:`bea-io-tables` — BEA I-O data format and file inventory
- :ref:`tensor-hierarchy-reference` — ``InterIndustryFlow`` tensor type
- :mod:`babylon.domain.economics.tensor_hierarchy.inter_industry` — ``DefaultDepartmentAggregator``
