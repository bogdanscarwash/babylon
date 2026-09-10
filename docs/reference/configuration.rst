Runtime and Tool Configuration
==============================

The Rust runtime owns game state and time. Python configuration supplies data
and operator tools. The frozen Python engine's ``GameDefines`` and optimization
configuration have been retired; old coefficient tables do not configure the
live Rust campaign.

Native session
--------------

``mise run play`` starts the runtime and Bevy client through
``tools/run_observer_session.py``. The launcher gives the runtime the writer
connection and gives the client separate observer and known-reader
capabilities.

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Input
     - Meaning
   * - ``BABYLON_RUNTIME_DSN``
     - PostgreSQL writer connection; defaults to the local test service on
       ``127.0.0.1:5433`` and database ``babylon_test``
   * - ``BABYLON_CAMPAIGN_ID``
     - Explicit campaign UUID when the command supports opening a campaign
   * - ``XDG_STATE_HOME``
     - Absolute state root for ``babylon/observer-campaign``, the launcher's
       continuation preference; defaults to ``~/.local/state``
   * - ``XDG_DATA_HOME``
     - Player data root; client logs use ``babylon/logs`` below it and default
       to ``~/.local/share/babylon/logs``
   * - ``RUST_LOG``
     - Rust tracing filter; the launcher also enables its session and client
       capture filters
   * - ``BABYLON_TIMINGS``
     - Set to ``1`` for bounded material-stage timing records on runtime stderr

The launcher accepts ``--campaign UUID``, ``--new``, ``--preset``,
``--defines PATH``, and ``--no-build``. Its ``--help`` output owns the exact
combinations. A new campaign
preserves existing worlds. V7 admission refuses older Michigan campaign content
and retains those saves without rewriting them.

Regional campaign selections remain Standard, Delayed, Shared freight — ample,
and Shared freight — constrained. The launcher accepts ``--preset standard``,
``delayed``, ``shared-freight-ample``, and ``shared-freight-constrained``, plus
the four statewide labels described below. Accepted labels do not certify
qualified statewide content.

Authored campaign values
~~~~~~~~~~~~~~~~~~~~~~~~

``content/scenarios/michigan/defines.toml`` uses definitions schema V3.
It contains the regional recipes and routes plus statewide commodity,
template, transport, and merchant tables. Operating quantities have the
``Designed`` evidence class. Observed QCEW establishments qualify participation.
Observed jobs and wages do not supply physical quantities or modeled workers.

The parser rejects unknown or missing fields and tables, fractional or
negative integers, files over 32,768 bytes, invalid units, and arithmetic
overflow. ``--defines PATH`` selects the numeric source for New. Canonical
numeric values ignore comments, whitespace, and table order. Source and
qualification pins also contribute to a statewide campaign's identity.

``MichiganCapturedContentV2`` retains normalized owners, recipes, stocks,
workforce, orders, explicit preset overrides, and observed source cells. It
also retains the generated graph scenario and observed definitions. Statewide
capture includes selected physical paths, deduplicated edge geometry, county
terminal attachments, vehicle profile, and network source identity. It excludes
the full routing matrix and unselected road graph.

``SectorBundleV2`` and ``StoredSectorBundleDefinesV4`` keep executable rows
and staffing authority around that capture. The captured-content and total
definitions limits are each 64 MiB. The generated graph source has a 1 MiB
limit. These are admission ceilings, not measured full-state performance
claims. Open reconstructs admitted saved authority without rereading changed
TOML, QCEW, or road files and without rerouting. Admission refuses unsupported
versions without deleting their stored data.

Regional parameters
^^^^^^^^^^^^^^^^^^^

Process values below follow this order: sheet rolling, panel forming,
subassembly making, meal milling, meal packaging. Corridor and route values
follow sheet transfer, panel transfer, food transfer. ``Consequential`` means
a parameter has a causal consumer. It does not promise that every valid
change changes output. ``Dormant`` means the selected preset does not consume
that value.

.. list-table:: Michigan parameter inventory. Evidence class: Designed
   :header-rows: 1
   :widths: 25 25 25 25

   * - Field
     - Baseline and units
     - Constraints and four-week conversion
     - Consumer and consequence
   * - ``SCHEMA_VERSION``
     - ``3``. Format version
     - Exactly ``3``
     - Canonical content admission. Fixed contract
   * - ``TICK_DURATION_DAYS``
     - ``28`` days
     - Exactly the Rust clock interval
     - Stored interval admission. Fixed contract
   * - ``HORIZON_PERIODS``
     - ``16`` four-week periods
     - Integer ``1..=16``. Unscaled
     - Session stop, inventory bounds, Circuit horizon. Consequential
   * - ``staffing.WORK_HOURS_PER_PERSON_WEEK``
     - ``40`` hours/person/week
     - Integer ``1..=168``. Becomes ``160`` hours/person/period
     - Opening labor budget and later staffing. Consequential
   * - ``process.*.BATCHES_PER_WEEK``
     - ``8, 8, 4, 4, 4`` batches/week
     - Positive. Becomes ``32, 32, 16, 16, 16`` batches/period
     - Capacity rows, plans and output. Consequential limits, frequently nonbinding
       for increases at baseline
   * - ``process.*.LABOR_HOURS_PER_BATCH``
     - ``100, 20, 40, 10, 20`` hours/batch
     - Positive. Unscaled. Joint staffing-demand bound
     - Recipe labor use and staffing demand. Consequential
   * - ``process.*.INPUT_UNITS_PER_BATCH``
     - ``10 kg, 10 kg, 2 panels, 5 kg, 5 kg`` per batch
     - Positive. Unscaled. Opening stock must cover its plan
     - Recipe consumption and plans that the input permits. Consequential
   * - ``process.*.OUTPUT_UNITS_PER_BATCH``
     - ``10 kg, 1 panel, 1 subassembly, 5 kg, 5 kg`` per batch
     - Positive. Unscaled. Horizon output must fit inventory
     - Production and downstream supply. Consequential
   * - ``process.*.OPENING_INPUT_UNITS``
     - ``600 kg, 0 kg, 0 panels, 200 kg, 0 kg``
     - Nonnegative. Unscaled. Joint inventory bound
     - Initial stock, production and later staffing demand. Consequential
   * - ``process.*.OPENING_PLANNED_BATCHES``
     - ``32, 0, 0, 16, 0`` batches
     - Nonnegative. Within opening input, labor and period capacity
     - Period-1 commitments only. Consequential
   * - ``process.*.EMPLOYED_PEOPLE``
     - ``20, 4, 4, 1, 2`` people
     - Positive. Unscaled. Joint workforce/hour bound
     - Opening workforce, labor and retention reference. Consequential
   * - ``process.*.RESERVE_PEOPLE``
     - ``0, 0, 0, 0, 0`` people
     - Nonnegative. Unscaled. Joint workforce/hour bound
     - Reserve display changes. Extra hiring supply is dormant for baseline
       output, consequential when demand exceeds the existing workforce
   * - ``corridor.*.UNITS_PER_WEEK``
     - ``80 kg, 8 panels, 20 kg`` per week
     - Positive. Becomes ``320 kg, 32 panels, 80 kg`` per period
     - Independent freight budgets in Standard and Delayed. Only panel capacity
       applies in the shared presets. Decreases can constrain dispatch
   * - ``route.*.TRAVEL_PERIODS``
     - ``1, 1, 1`` four-week periods
     - Positive 16-bit unsigned integer. Unscaled
     - Timed stages for Standard and both shared presets. Dormant in Delayed
   * - ``route.*.DELAYED_TRAVEL_PERIODS``
     - ``3, 1, 1`` four-week periods
     - 16-bit unsigned integer, at least normal travel. Unscaled
     - Timed stages for Delayed. Dormant in the other presets
   * - ``route.*.ORDERED_UNITS``
     - ``600 kg, 60 panels, 200 kg`` total
     - Positive. Unscaled. Joint buyer-inventory bound
     - Initial orders, backlog and shipment totals. Consequential
   * - ``regional_mass.KILOGRAM_GRAMS_PER_UNIT``
     - ``1000`` grams/kg
     - Exactly ``1000``. ``EVIDENCE_CLASS = "Designed"``
     - Converts kilogram freight to the shared gram principal
   * - ``regional_mass.PANEL_GRAMS_PER_UNIT``
     - ``10000`` grams/panel
     - Positive exact integer. Designed item mass
     - Converts panel freight while preserving its native throughput
   * - ``regional_mass.SUBASSEMBLY_GRAMS_PER_UNIT``
     - ``20000`` grams/subassembly
     - Positive exact integer. Designed item mass
     - Explicit mass for the native subassembly unit
   * - ``shared_freight.AMPLE_UNITS_PER_WEEK``
     - ``200 kg`` per week
     - Positive, at least constrained capacity. Becomes ``800 kg`` per period
     - One shared sheet and meal budget in Shared freight — ample
   * - ``shared_freight.CONSTRAINED_UNITS_PER_WEEK``
     - ``40 kg`` per week
     - Positive, no greater than ample capacity. Becomes ``160 kg`` per period
     - One shared sheet and meal budget in Shared freight — constrained

Weekly work hours and batch rates multiply by four once. Regional corridor
rates also multiply by four, then by the good's grams per native unit. Stocks,
plans, people, recipes, travel periods, and finite order totals keep their
native units. ``michigan_defines`` validates the values.
``michigan_material`` normalizes them. ``sector_bundle`` compiles the one
current material representation.

Regional opening plans must fit capacity, input stock, and employed labor
together. Workforce totals, available hours, and maximal staffing requests
must fit the exact graph-integer bound, ``2^53``. Inventory admission bounds
opening stock, horizon output, and incoming orders per site and good within
``u64``, including mass conversion. Travel can exceed the campaign horizon.
Admission does not promise arrival before the stop.

The five regional labor budgets remain ``3200, 640, 640, 160, 320`` hours per
period. Extra opening stock does not create a period-1 commitment when its
opening plan is zero. Standard and Delayed keep independent capacities.
The shared presets keep one Designed sheet-and-meal service of 800 or
160 kg per period, represented as 800,000 or 160,000 grams. Panel freight
retains 32 panels per period, represented as 320,000 grams. The service does
not identify a physical road or a Detroit–Windsor crossing.

``topology.json`` supplies regional identities and relationships. Numeric
values remain in TOML. The compiler emits each active capacity principal and
period budget once. Timed stages can share principals. The allocator floors
proportional shares, dispatches the smallest allowed grant, debits actual movement,
and leaves unused residuals. Capacity reservations remain distinct from
physical arrivals.

See :doc:`/how-to/debug-simulation-outcomes` for the regional comparisons.

Statewide authoring
^^^^^^^^^^^^^^^^^^^

The pinned commodity roster qualifies 397 county-sector owners: 83 agriculture
and forestry, 65 extractive mining, 83 manufacturing, 83 wholesale, and 83
retail. The other county-sector cells remain observed reference context.
The roster excludes support-only mining from executable production. Suppressed
jobs and wage metrics remain absent. Positive establishments still qualify
an eligible template.

The current catalog has 23 goods and 16 productive templates. Upstream
templates supply grain, animal products, logs, hydrocarbon feedstock, metal
ore, and mineral feedstock. Other templates produce prepared food, packaged
beverages, wood products, paper packaging, and industrial chemicals. Metal
stock, metal parts, machinery, electrical goods, and household wares complete
the catalog.

Recipes consume finite opening resources without regeneration.
Templates assigned to one owner share its inventory and workforce. Each
recipe keeps its typed inputs.

.. list-table:: Statewide table contracts. Evidence class: Designed
   :header-rows: 1
   :widths: 35 65

   * - Table and fields
     - Meaning and current authored values
   * - ``statewide.FINITE_ORDER_PERIODS``
     - ``4`` nominal production periods of finite orders. Integer
       ``1..=HORIZON_PERIODS``. Orders do not recur
   * - ``statewide.TERMINAL_ATTACHMENT_LIMIT_METERS``
     - ``50000`` meters. Positive limit for a Designed county
       terminal, not a factory location
   * - ``transport.ROAD_TRAVEL_PERIODS``
     - Exactly ``1`` period per road journey stage, regardless of the number
       of physical edges in its captured path
   * - ``transport.TRUCK_GROSS_WEIGHT_KG``, ``TRUCK_HEIGHT_MM``,
       ``TRUCK_WIDTH_MM``, ``TRUCK_LENGTH_MM``
     - ``40000`` kg, ``4000`` mm, ``2550`` mm, ``16500`` mm. Positive routing
       profile values. They do not set economic capacity
   * - ``transport.DEFAULT_MAXHEIGHT_MM``
     - ``4000`` mm. Designed height bound for OSM ``maxheight=default``.
       This bound does not measure physical clearance or economic capacity
   * - ``transport.ROAD_CAPACITY_GRAMS_PER_PERIOD``
     - ``100000000`` grams per shared capacity principal per period.
       Capacity groups and interventions need physical qualification
   * - ``transport.EXTRACTION_BUFFER_DEGREES_E7``
     - ``100000``, or 0.01 angular degree. A clipping buffer, not meters
   * - ``merchant.HANDLING_GRAMS_PER_PERIOD``
     - ``5000000`` grams per merchant per period. Separate from road capacity
   * - ``merchant.LABOR_HOURS_PER_KG``, ``LABOR_HOURS_PER_ITEM``
     - ``1`` hour/kg and ``10`` hours/item. Positive handling coefficients
   * - ``merchant.EMPLOYED_PEOPLE``, ``RESERVE_PEOPLE``
     - ``20`` employed and ``4`` reserve. Nonnegative integers with a positive,
       exactly representable total workforce
   * - ``commodity.<good>.UNIT``, ``GRAMS_PER_UNIT``, ``DISPOSITION``
     - Native ``kg`` or ``item``. Positive exact mass, with ``kg`` fixed at
       1000 grams. Disposition ``finite_opening`` or ``traded``
   * - ``template.<family>.OUTPUT_GOOD``, ``OUTPUT_UNITS_PER_BATCH``
     - One traded output matching the family key, with a positive native-unit
       quantity per batch
   * - ``template.<family>.INPUT_UNITS_PER_BATCH``, ``OPENING_INPUT_UNITS``
     - Maps keyed by the same 1 through 16 goods. Recipe coefficients are
       positive. Finite opening stocks are nonnegative
   * - ``template.<family>.BATCHES_PER_WEEK``, ``LABOR_HOURS_PER_BATCH``
     - Positive batch and labor rates. Current templates use four batches
       per week. Their recipes and labor coefficients remain per batch
   * - ``template.<family>.EMPLOYED_PEOPLE``, ``RESERVE_PEOPLE``
     - Current primary templates seed eight employed and four reserve people.
       An added process uses the owner's existing pool

``statewide``, ``transport``, ``merchant``, and ``regional_mass`` must declare
``EVIDENCE_CLASS = "Designed"``. Goods keep their native identity through
production, wholesale, retail, and final fulfillment. Merchant handling
consumes labor and mass capacity without manufacturing a new good. Local
inter-owner transfers have no road stage or transit lot. Local retail
fulfillment completes a finite end-buyer order without creating household
stock, consumption, payment, or revenue.

The Designed road-admission policy models freight between county aggregates
for the game. ``ROAD_CLASSES`` admits ``motorway``, ``trunk``, ``primary``,
``secondary``, ``tertiary``, and their ``_link`` classes. It excludes
``residential``, ``service``, ``living_street``, and ``unclassified`` roads.
Admitted paths still follow original OSM node connectivity, direction, access,
and turn restrictions.

An OSM ``maxheight=default`` record uses the captured Designed default height
bound. Explicit physical limits and access restrictions still apply. Other
height values without numbers remain excluded. Both Mackinac Bridge carriageways
and their toll-plaza approaches have original-source regression fixtures.

County terminals and their attachments remain Designed. They do not identify
factories or add physical road connections. Disconnected supply refuses
qualification. ``contracts/michigan_road_network_v1.yaml`` defines this scope.
The admission policy alone does not certify a qualified statewide network.

``[statewide.EXPERIMENT]`` supplies the qualified comparison changes. Regional
authoring can omit the table. Statewide New refuses it when absent, including
for the baseline selection. The admitted Mackinac crossing uses a Designed
shared freight service across both carriageways. Other selected road edges
share a service within each connected component of an OSM route reference,
road name, or unnamed way. These services do not claim observed traffic
capacity.

The baseline provides 100,000 kg per service per 28-day period. The freight
intervention changes only ``mackinac-bridge-freight`` to 1,000 kg. The input
intervention changes only the Mackinac manufacturing process
``26097-31-33-prepared_food`` from 160 kg to 80 kg of opening paper packaging.
The combined preset applies both changes.

.. list-table:: Statewide experiment fields
   :header-rows: 1
   :widths: 40 60

   * - Field
     - Admission constraint
   * - ``FREIGHT_CAPACITY_KEY``
     - A captured freight capacity identity
   * - ``CONSTRAINED_GRAMS_PER_PERIOD``
     - Positive grams below ``transport.ROAD_CAPACITY_GRAMS_PER_PERIOD``
   * - ``FOOD_PROCESS_KEY``
     - The captured process whose opening packaging stock changes
   * - ``PACKAGING_GOOD_KEY``
     - Exactly ``paper_packaging``
   * - ``SHORTAGE_OPENING_UNITS``
     - Nonnegative native units below the prepared-food template's opening
       paper-packaging stock

Statewide New reads three files beside the selected ``defines.toml``:
``statewide-sources.json``, ``statewide-qualification.json.gz``, and
``statewide-physical.json.gz``. The source manifest uses schema
``MichiganStatewideSourcesV1``. Its ``defines_sha256`` pins the raw TOML bytes,
including formatting. ``qualification_sha256`` and ``physical_network_sha256``
pin the compressed artifact bytes. The physical terminal source's
``defines_sha256`` must match the same TOML digest.

The manifest limit is 4096 bytes. Each compressed artifact and its decoded
content must fit 64 MiB. Missing files, mismatched hashes, invalid content,
or absent interventions refuse creation. Open uses the captured content and
does not enter this source loader.

The shipped physical artifact has a separate 2 MiB compressed publication
limit. The current capture uses 1,176,826 compressed bytes and 6,253,472 decoded
bytes. Its 1,245 transport services and 166 merchant handling services produce
22,576 opening period budgets across the 16-period horizon. The material
runtime permits at most 65,536 rows per family and a 64 MiB register. The
initial statewide register uses 2,170,588 bytes. Read these as representation
bounds, separate from Designed economic capacities.

The four statewide protocol selections are ``statewide-baseline``,
``statewide-freight-constraint``, ``statewide-packaging-shortage``, and
``statewide-both``. Their content identifiers end in ``-v7``. These selections
use the same qualified physical paths, finite orders and production parameters.
The launcher accepts all eight labels. Statewide creation requires the pinned
source siblings and explicit interventions. The source-backed engine experiment
establishes the freight and packaging effects; PostgreSQL, hosted, native and
Director acceptance require their separate evidence. ADR260 defines that
acceptance boundary.

Simulation Interval
~~~~~~~~~~~~~~~~~~~

The authoritative Rust clock uses one four-week simulation tick. These constants
are defined in ``babylon_kernel::clock`` and are not runtime overrides:

.. list-table::
   :header-rows: 1
   :widths: 35 15 50

   * - Constant
     - Value
     - Meaning
   * - ``DAYS_PER_TICK``
     - 28
     - Fixed simulation days in one transition and commit
   * - ``WEEKS_PER_TICK``
     - 4
     - Weeks represented by one period
   * - ``TICKS_PER_YEAR``
     - 13
     - Periods in a modeled 364-day year

V7 Michigan content validates ``TICK_DURATION_DAYS = 28`` in its authored
parameters and stores the duration in canonical foundation definitions.
Admission refuses a different duration or
an older weekly preset. Observed source units, including QCEW weekly wages,
retain their original meanings. Four-week flow budgets do not multiply opening
stocks, population counts, or per-batch recipes.

Reference tools and Python logging
----------------------------------

The retained ``babylon.config.base.BaseConfig`` loads ``.env`` and supports
``DEBUG``, ``TESTING``, ``DATABASE_URL``, ``LOG_LEVEL``, ``LOG_FORMAT``,
``LOG_DIR``, ``METRICS_ENABLED``, and ``METRICS_INTERVAL``. These values belong
to their Python consumers. ``DATABASE_URL`` defaults to a SQLite file under the
current working directory's ``data/sqlite`` directory; it is separate from the
Rust campaign's PostgreSQL connection.

``babylon.config.logging_config.setup_logging`` reads
``[tool.babylon.logging]`` from ``pyproject.toml`` and can accept an explicit
logging configuration file. The default handler writes rotating ``babylon.log``
and ``errors.log`` files under ``LOG_DIR``. The default directory is the
player data root's ``logs`` directory. Rust client logging uses its own tracing
subscriber and ``babylon-client.log``.

Operational commands
--------------------

``.mise.toml`` and its included task files define build, data, test, and
simulation commands. Inspect ``mise tasks`` and each command's ``--help`` for
current arguments. Run ``mise run sim:report`` for 15 four-week periods, or
``mise run sim:report 130`` for ten modeled years. The report wrapper restarts
at each 13-period boundary and records the actual runtime's interval in its
validated evidence.

See :doc:`/how-to/debug-simulation-outcomes` for report interpretation and
:doc:`/concepts/architecture` for the live authority boundary.
