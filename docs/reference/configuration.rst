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

Edit ``content/scenarios/michigan/defines.toml`` to tune the current circuit.
Its typed numeric entries specify work hours, process and freight throughput,
recipes, starting inventory, workforce, orders, and route delays. Comments state
units; these values are Designed inputs, separate from observed public data.
The runtime rejects missing or unknown keys, invalid values, incoherent
resources, and arithmetic overflow.

To use another file, pass ``--defines /absolute/path/to/defines.toml`` to the
launcher. Each New request reads and validates the selected file. The runtime
canonicalizes the effective values and captures them in the new campaign's
foundation identity. Editing or deleting that file later does not change the
parameters in an existing campaign. Open reconstructs its saved parameters.

Weekly capacities convert once into four-week budgets. People, stocks, order
totals, and per-batch recipes keep their declared units. Labor capacity derives
from the supplied workforce and work schedule; there is no separate hardcoded
schedule in the staffing engine. The old Python ``defines.yaml`` catalogue is
available in Git history and is not a configuration source for current play.

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
