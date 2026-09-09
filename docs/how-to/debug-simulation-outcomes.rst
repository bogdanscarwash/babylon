Debug Simulation Outcomes
=========================

Use the Rust runtime's committed reports to investigate a changed result,
a missing delivery, a staffing movement, or a slow advance. Retain the exact
campaign, content identity, source revision, and selected completed period.

Generate a bounded diagnostic report
------------------------------------

Start with the existing PostgreSQL service and schema. The reporter builds the
runtime, creates a fresh campaign, and records post-commit evidence:

.. code-block:: bash

   mise run sim:report
   mise run sim:report 130 3000 shared

The default run covers 15 four-week periods (60 weeks), crossing the annual
boundary at period 13. The second command covers ten modeled years: 130
periods, with a 3,000-second runtime timeout. A modeled year is 13 periods,
or 364 days. Use ``exclusive`` as the final argument only for a database whose
resource use belongs to this run; a campaign UUID alone does not isolate a
shared PostgreSQL service.

The report lives in a unique directory under ``reports/sim-runs``. It includes
``ticks.jsonl``, ``ticks.csv``, ``summary.json``, ``summary.txt``,
``diagnostics.json``, runtime stdout and stderr, and ``resources.json``.
The JSON rows disclose ``scope.tick_duration_days`` and the summary carries
that value from validated runtime evidence. The wrapper requires contiguous
commits, exact source and foundation identities, a restart every 13 periods,
and durable readback of the final successful period.

The embedded Michigan report is a bounded persistence diagnostic. Its fixed
parameters and exposed rule inventory define what it can demonstrate. It does
not establish completion of the Bevy material campaign or player agency.

Inspect an existing campaign
----------------------------

For a background Michigan run, inspect its recorded process and campaign tail:

.. code-block:: bash

   mise run sim:status
   mise run sim:probe
   tail -f .sim-pids/e2e.log

``sim:probe`` honors an explicit ``BABYLON_CAMPAIGN_ID`` or the worktree's
recorded default. ``sim:report`` always creates a fresh campaign. Preserve
existing campaign data when comparing source revisions. V6 content refuses
older presets without deleting or reinterpreting their saved state.

Explain the material change
---------------------------

Open the Bevy session with ``mise run play``. Hold the same completed period
and perspective when comparing campaigns. Trace an output through its opening
inputs, committed deliveries, production recipe, labor account, and closing
inventory. Distinguish foundation stocks, unavailable evidence, and a present
completed-period receipt with a real zero.

The Designed Standard and Delayed presets differ in sheet-transfer travel time:
one period (four weeks) or three periods (twelve weeks). Their per-period
flow and hour budgets use the four-week interval. Initial stocks, people,
order totals, and physical recipes keep their original units. QCEW wages
remain observed weekly rates and do not become modeled payroll.

Compare shared freight capacity
-------------------------------

Choose Shared freight — ample and Shared freight — constrained in New Campaign,
or launch either explicitly:

.. code-block:: bash

   mise run play -- --new --preset shared-freight-ample
   mise run play -- --new --preset shared-freight-constrained

Both campaigns use one-period travel, the same finite orders, and the same
opening production and workforce. Sheet metal and milled meal share a Designed
regional freight service. The shared capacity alone changes: 800 or 160 kg per
28-day period. Panel freight retains its independent capacity. The service
does not claim a physical road or border crossing.

In Production, select the sheet or meal chain. The shared-capacity reading
names both participating routes and shows the pool once. At completed period 1,
compare opening capacity, newly reserved quantity, remaining capacity, and each
order's requested and dispatched quantities. The ``unshipped`` field shows
the remaining backlog. Use ``Other participants`` to inspect the competing
chain. Capacity reservations are not physical arrivals.

.. list-table:: Baseline shared freight comparison
   :header-rows: 1
   :widths: 50 25 25

   * - Reading
     - Ample
     - Constrained
   * - Period-1 sheet dispatch
     - 320 kg
     - 120 kg
   * - Period-1 meal dispatch
     - 80 kg
     - 40 kg
   * - Period-1 remaining shared capacity
     - 400 kg
     - 0 kg
   * - Period-3 panel output
     - 32 panels
     - 12 panels
   * - Period-3 packaged-meal output
     - 80 kg
     - 40 kg

The opening orders total 600 kg of sheet and 200 kg of meal. At 160 kg
capacity their proportional shares are 120 and 40 kg. At 800 kg capacity,
supplier stock after period-1 production limits dispatch to 320 and 80 kg. Unused
shares stay unused.

For the order comparison, copy ``defines.toml`` and change only
``route.food_transfer.ORDERED_UNITS`` from 200 to 80. Create a new campaign with
``--new --preset shared-freight-constrained --defines /absolute/path/to/copy.toml``.
First dispatch becomes 141 kg of sheet and 18 kg of meal, with 1 kg left by flooring.
Editing the file cannot change a saved campaign.

Follow these dispatches to period-2 arrivals, then period-3 production. Compare
each site's employed and reserve counts and its ``Work request`` for the
next period. A temporary input shortage can reduce work and move people into reserve.
The full workforce account must still conserve people.

Compare saved campaigns at the same completed period. Return to a campaign
through Open and check its
continued identity and saved parameters. Hold an earlier period while advancing
to distinguish historical evidence from the current tail. Restricted knowledge
preview does not expose these material or capacity accounts.

Run the native discovery, comparison, and resume check at 1366×768 and
1920×1080. Automated receipts prove the recorded operations. The Director's
session supplies comprehension acceptance. This remains a partial PER-31
delivery within Gate 4.

Compare delivery time and opening stock
---------------------------------------

Run ``michigan_experiment`` for the PER-309 comparison.
It runs the admitted material replay path in memory. It reads production,
logistics, and staffing receipts. It needs no ``PostgreSQL`` service. Keep the
accepted baseline file unchanged. The example embeds it at compilation.

The four cases select Standard or Delayed delivery.
They set only ``process.panel_forming.OPENING_INPUT_UNITS`` to zero or 320 kg
of sheet.
``OPENING_PLANNED_BATCHES`` stays zero in all four cases.

Coordinate Cargo compilation and execution with the release session. Compile
from ``rust/`` when the host is available:

.. code-block:: bash

   cargo build -p babylon-persistence --example michigan_experiment --locked

After compilation, run the comparison with a fresh absolute output directory
owned by this task. Commit the prepared source first: measured runs need a
clean source tree and refuse a binary whose embedded harness sources differ
from the checkout. The parent of the new output directory must already exist.
Store artifacts outside the checkout or in ignored ``reports/``.

.. code-block:: bash

   cargo run -p babylon-persistence --example michigan_experiment --locked -- --output /absolute/task-owned-dir

The only experiment argument is ``--output DIR``. The fixed bound is four
cases of 16 four-week periods, with a 60-second execution limit after build
and at most 4 MiB of output. Replay uses the admitted seed ``319``.
It has no seed override or stochastic sweep. If the run fails or reaches a bound,
keep its failure evidence and do not treat partial output as a comparison.
Defer merging the exploration until release qualification finishes.

Read the four output files together:

* ``inputs.json`` retains canonical resolved numeric definitions and the fixed
  case matrix, including when a later transition fails.
* ``manifest.json`` records the resolved case definitions, source and campaign
  identities, output checksums, and execution bounds.
* ``periods.jsonl`` records process, route, and workforce evidence for each
  completed period.
* ``summary.json`` reports production milestones, constraint durations, and
  the food-chain comparison. Preserve absent milestones as absent. Do not
  substitute period zero.

The period records connect delivery, available work, production,
``unretained hours``, and the next period's staffing.

The example pins the accepted canonical numeric baseline. TOML formatting is
not part of that numeric identity. A changed baseline needs a new reviewed
experiment. The execution record includes a binary digest and checks embedded
harness sources against the clean checkout. It does not claim complete build
attestation.

``failure.json`` identifies the failed case and last recorded
completed period. Accept a comparison only with exit code zero, a complete
manifest, matching output checksums, and no ``failure.json``.
The watchdog can time out after the manifest write and still report a failure.

An absent completed production receipt means no prior production plan. The
input, labor and capacity ceilings describe the next opening and keep ties.
These diagnostics have the ``Derived`` evidence class. They apply to the
fixed topology of one process per site. They are not causal labels from the
engine.

A zero labor budget can follow from no ``material request``.
Compare the recorded ``material request`` with the next plan
before interpreting labor as an independent constraint. Dispatch ceilings
report finite orders separately from supplier stock and corridor capacity.

Check the Standard and Delayed pair without a buffer against the accepted delivery
comparison first. Then compare the stock effect within each delivery preset.
The 320 kg buffer provides one full period of panel input. The unchanged
zero opening plan still prevents panel production in period 1.

Keep the food chain as the unchanged control. Examine staffing as well as
completion time. A buffer can change a gap in work without eliminating a
later gap.

The bounded Rust contract tests reproduce these subassembly milestones:

.. list-table:: Subassembly milestones
   :header-rows: 1
   :widths: 35 25 20 20

   * - Delivery preset
     - Opening sheet at panel forming
     - First subassembly period
     - Period reaching 30 ``subassemblies``
   * - Standard
     - 0 kg
     - 5
     - 6
   * - Delayed
     - 0 kg
     - 7
     - 8
   * - Standard
     - 320 kg
     - 4
     - 5
   * - Delayed
     - 320 kg
     - 4
     - 7

Both buffered cases finish with 32 unsold panels at ``Macomb``. They start with
920 kg of material measured as metal input, compared with 600 kg without the
buffer. The fixed 60-panel order limits terminal subassembly output to 30 in
every case.
The buffer removes the first-output delay gap, but the completion
gap remains two periods. Buffered delayed subassembly production also causes
hires in periods 3 and 6, with a separation in period 5. The food branch stays
equal across all four cases.

This comparison adds an endowment of material.
It does not prove equal-resource efficiency or entertainment value.

Keep the emitted source and identity evidence with any measured comparison.
Investigate disagreement with these landmarks. The result describes the
Designed material circuit. Its in-memory receipts do not prove ``PostgreSQL``
durability, Bevy
comprehension, historical calibration, or a player milestone. See
:doc:`/reference/configuration` for all 56 fields, their units and validation.

Inspect performance
-------------------

Set ``BABYLON_TIMINGS=1`` when launching a material session. Its bounded stderr
record includes ``campaign``, ``period``, ``simulation_us``, ``preparation_us``,
``durable_write_publish_us``, and ``total_us``. Bevy separately records the
authenticated observer and Archive-card reads. Match timings by campaign and
completed period; a fast simulation stage cannot establish a fast visible
advance on its own.

Compare cold compilation, warm execution, database bootstrap, and UI response
separately. ``resources.json`` measures the runtime invocation and excludes
compilation and reporter overhead. Shared database deltas include concurrent
activity. Use the same source inputs and starting state for before/after
measurements, and report failures or unavailable measurements explicitly.

See :doc:`/concepts/architecture` for authority boundaries and
:doc:`/reference/ci-workflow` for development and release validation.
