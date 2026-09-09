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
preserves existing worlds. V5 admission refuses older weekly campaign content.

Authored campaign values
~~~~~~~~~~~~~~~~~~~~~~~~

``content/scenarios/michigan/defines.toml`` contains 56 required numeric fields:
The campaign has three fields and staffing has one. Each of five
processes has eight fields. Each of three corridors has four fields.

All fields have the ``Designed`` evidence class.
Observed QCEW jobs and wages do not supply these quantities. The parser rejects
unknown or missing fields and tables, fractional or negative values, files over
32,768 bytes, invalid resources, and arithmetic overflow.

The launcher accepts ``--defines /absolute/path/to/defines.toml``. Each New
request validates that file and stores canonical values in the campaign
foundation. Comments, whitespace, and table order do not change their identity.

The stored envelope also captures observed definitions, sector bundles, and
staffing. Open verifies and reconstructs this saved authority without reading
the mutable TOML file. Even a value unused by the selected delivery preset
remains part of the stored identity.

The inventory below groups repeated fields. Process values follow source order:
sheet rolling, panel forming, subassembly making, meal milling, meal packaging.
Corridor values follow sheet transfer, panel transfer, food transfer. Each
``process.*`` row covers five fields. Each ``corridor.*`` row covers three.

``Consequential`` denotes a live causal consumer within the supported horizon.
It does not assert that every valid change changes output.
``Dormant`` denotes a value that the selected run does not consume. No field
is redundant across all supported campaigns according to the current audit.

.. list-table:: Michigan parameter inventory. Evidence class: Designed
   :header-rows: 1
   :widths: 25 25 25 25

   * - Field
     - Baseline and units
     - Constraints and four-week conversion
     - Consumer and consequence
   * - ``SCHEMA_VERSION``
     - ``1``. Format version
     - Exactly ``1``
     - Canonical content admission. Fixed contract
   * - ``TICK_DURATION_DAYS``
     - ``28`` days
     - Exactly the Rust clock interval
     - Stored interval admission. Fixed contract
   * - ``HORIZON_PERIODS``
     - ``16`` four-week periods
     - Integer ``1..=16``. Unscaled
     - Session stop, inventory bounds, Production horizon. Consequential
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
     - Freight capacity and arrival timing. Increases are redundant at baseline
       supply, decreases can constrain dispatch
   * - ``corridor.*.TRAVEL_PERIODS``
     - ``1, 1, 1`` four-week periods
     - Positive 16-bit unsigned integer. Unscaled
     - Route leg for Standard. Consequential there, dormant in Delayed
   * - ``corridor.*.DELAYED_TRAVEL_PERIODS``
     - ``3, 1, 1`` four-week periods
     - 16-bit unsigned integer, at least normal travel. Unscaled
     - Route leg for Delayed. Consequential there, dormant in Standard
   * - ``corridor.*.ORDERED_UNITS``
     - ``600 kg, 60 panels, 200 kg`` total
     - Positive. Unscaled. Joint buyer-inventory bound
     - Initial orders, backlog and shipment totals. Consequential

Only weekly work hours, process throughput, and corridor throughput multiply
by four. The authored stock, plans, people, recipes, travel periods, and order
totals keep their units. ``michigan_defines`` owns typed validation.
``michigan_material`` compiles the values. ``sector_bundle`` supplies the
stored foundation and material rows. Rust material production, logistics,
planning, and staffing produce the receipts consumed by the Production view.

Opening plans must fit capacity, input stock, and employed labor together.
Workforce totals, total available hours, and maximal staffing requests must fit
the exact graph-integer bound, ``2^53``. Inventory checks bound opening stock,
largest horizon output, and incoming order totals per site and good within
``u64``. Travel can exceed the campaign horizon. Admission does not promise
that goods will arrive before the stop.

All five baseline labor budgets equal full-capacity labor demand:
``3200, 640, 640, 160, 320`` hours/period. These constraints interact.
Raising throughput alone need not raise production. Extra opening stock
does not create a period-1 commitment when its opening plan remains zero.

With the other baseline values fixed, raising process or corridor capacity
does not raise output. Milling can demand more labor while its one-person
pool still limits production. Added reserves change the reserve account, but
baseline requests never exceed the workforce already supplied. These are
conditional results, not reasons to remove the underlying fields.

Equal panel and food travel values in both presets do not make their fields
redundant. The old Python ``defines.yaml`` catalog remains in Git history
and is not a source for current play.

See :doc:`/how-to/debug-simulation-outcomes` for the bounded delivery-time and
opening-stock comparison.

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

V5 Michigan content validates ``TICK_DURATION_DAYS = 28`` in its authored
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
