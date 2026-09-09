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
existing campaign data when comparing source revisions. V7 content refuses
older presets without deleting or reinterpreting their saved state.

Explain the material change
---------------------------

Open the Bevy session with ``mise run play``. Hold the same completed period
and perspective when comparing campaigns. Start in World, whose default lens
shows modeled employed people.

The reserve lens shows unused modeled workforce.

Keep the observed QCEW employment and wage lenses separate. Those annual jobs
and source wages are not the circuit's workers or payroll.

For physical amounts, choose production, inventory, or inbound transit, then a
specific good and unit. Compare the same commodity across counties. Do not add
unlike outputs into one total. A reference-only county-sector cohort has no
modeled workforce account, which differs from an admitted account with zero
workers.

Select a county in World. Its side panel lists the county cohorts, with
Previous page and Next page when needed. Each page shows at most six.
Select a producer or merchant to open its relationships in Circuit.

Follow Upstream and Downstream links for suppliers and buyers.
Other participants / Shared freight links identify competitors for capacity.
These links are separate from the supplier list. Relationship and competitor
pages each show at most six groups or participants. Return to World with its
control or M to choose another county.

Return to World to highlight the selected cohort's shipment paths. Selecting
another county clears those paths until you choose one of its cohorts.
In World's drawer, choose Selected paths or Captured roads. The second layer
shows the campaign's captured physical route network behind the selected
shipment paths. It does not show every OSM road or draw all commodity flows.
Shared segments appear once, even when routes traverse them in opposite
directions. Tab and Enter reach both layer controls. Loading, failed, stale,
and restricted observations disclose no captured road geometry.

Trace a selected producer's output through opening inputs, its recipe, work,
movement, and closing inventory. If an owner has more than one process, choose
its Chart control for the process and output unit of interest. These processes
share inventory and workforce. Do not count the owner again for every recipe.

Open Readings beside the selected owner. Its heading retains subject, period,
output or handling, and modeled employed/reserve counts while the account
scrolls. Choose a section for its account.

* Flow: how did production and movement change stock?
* Freight: how much capacity did shipments reserve? What moved locally or
  by a timed route? Which owners share capacity?
* Work: how did handling or production hours compare with ``Work request``?
  How did employed and reserve counts change?
* Sources: which quantities carry the Designed evidence class? What observed
  QCEW evidence supports this owner's participation?

Distinguish foundation stocks, unavailable evidence, and a completed account
with a real zero. A completed reservation is not a physical arrival. The
account records local transfers separately from routed dispatch and arrival.

Tab reaches each section and its reading. Enter opens a section.
Page Up, Page Down, Home, and End scroll the focused reading.
A new subject or section starts at the top. Keyboard controls act on the
current campaign and observation. Loading or refused observations do not
license navigation through stale material details.

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

In Circuit, select the sheet or meal producer. The relationship panel summarizes
the shared pool beside the other-participant links. Open Readings and choose
Freight for both participating routes and their full account. At completed period 1,
compare opening capacity, newly reserved mass, remaining capacity, and each
order's requested and dispatched quantities. The ``unshipped`` field shows
the remaining backlog. Use ``Other participants`` to inspect the competing
chain.

Capacity reservations are not physical arrivals. Shared capacity uses exact
grams internally. The interface displays kilograms without losing fractional
mass. 800 kg is 800,000 grams and 160 kg is 160,000 grams. Dispatch and stock
keep their native good units.

.. list-table:: Regional shared freight regression landmarks
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
continued identity and saved parameters. Return Live before advancing, then
revisit an earlier period to distinguish its evidence from the current tail. Restricted knowledge
preview does not expose these material or capacity accounts.

Run the native discovery, comparison, and resume check at 1366×768 and
1920×1080. Automated receipts prove the recorded operations. The Director's
session supplies comprehension acceptance. This remains a partial PER-31
delivery within Gate 4.

Inspect merchants and statewide comparisons
-------------------------------------------

The campaign menu groups the four Statewide Michigan presets and the four
Regional proofs. Saved campaigns follow these groups. Choose Open to resume
the selected save, or Compare to read it beside the current campaign.

The statewide selections use the qualified physical road paths and frozen
interventions in :doc:`/reference/configuration`. The Mackinac shared freight
service changes from 100,000 kg to 1,000 kg per period. The packaging shortage
changes one Mackinac food producer's opening paper packaging from 160 kg to
80 kg. These Designed quantities give the finite game visible consequences.
they do not estimate real bridge throughput or industrial productivity.

Check saved campaigns against these committed engine readings.

The engine experiment alone does not certify saved campaigns
or Director comprehension.

.. list-table:: Statewide comparison landmarks. Outcomes: Derived
   :header-rows: 1
   :widths: 40 15 15 15 15

   * - Reading
     - Baseline
     - Freight only
     - Packaging only
     - Both
   * - Period 1 bridge dispatch, kg
     - 2,207
     - 709
     - 2,207
     - 709
   * - Period 1 Mackinac food output, kg
     - 1,600
     - 1,600
     - 800
     - 800
   * - Period 3 Mackinac food output, kg
     - 300
     - 100
     - 400
     - 100
   * - Period 3 Chippewa household wares, items
     - 8
     - 1
     - 8
     - 1
   * - Period 2 Chippewa manufacturing employed / reserve
     - 3 / 9
     - 1 / 11
     - 3 / 9
     - 1 / 11

The packaging shortage leaves 480 kg of grain and 160 kg of animal products
unused after period 1. Later packaging arrivals allow some catch-up. Compare
the full history: the period 3 food increase does not erase the earlier
shortfall.

In the freight case, 291 kg of first-period bridge capacity stays
unused. This includes the effects of simultaneous stock and capacity grants.
Rounding alone does not explain it. Reservations still equal actual dispatch
mass, and the allocator does not redistribute the unused grants.

Read the road display as a Designed freight network between county aggregates.
It admits motorway, trunk, primary, secondary, and tertiary roads and their
link classes. Residential, service, living-street, and unclassified roads
remain outside this game model. Read terminal markers as Designed county
attachments, not factory locations.

Physical routes still follow original node connections and admitted road
restrictions. Treat disconnected supply as a qualification failure, never as
an implied connection between nearby counties. See
:doc:`/reference/configuration` for the exact road-class policy.

For an admitted merchant account, read Flow as merchandise handling and
finite delivery rather than factory output. Reconcile its stock using opening
inventory, routed arrivals, local receipts, outbound shipments, local
transfers, end-buyer fulfillment, and closing inventory. Native commodity
identity and quantity must remain consistent through resale.

In Freight, distinguish these three movements:

Routed shipment
   It reserves shared mass capacity and stays in transit until its timed
   arrival. Captured road geometry describes the physical path. Schematic
   regional links do not claim a road.

County-local transfer
   It moves goods between different owners without a road stage or transit
   lot. The allocator fixes all outbound grants before these credits.
   The buyer cannot forward newly received local goods during the same close.

Local retail fulfillment
   It debits merchandise and advances a finite end-buyer order. Read it as
   delivery to end buyers. It records no household stock, consumption, or
   payment.

Read Work alongside a merchant's pending outbound orders. Handling has its
own mass and labor accounts. A zero employed count can coexist with
unmet handling work and a reserve available for later hiring. Check the
next opening and conservation of the whole workforce before attributing a
shortfall to transport.

For a qualified four-run comparison, read the same completed period in all
saved campaigns. Identify the single freight principal and opening packaging
stock changed by the captured definitions. Follow dispatch, arrival,
production or handling, end-buyer delivery, and employed/reserve differences.
The comparison does not advance either campaign. Count each workforce pool
once and compare production only within an exact good and unit.
Its bounded list of six shared freight accounts puts changed capacities
first, so the intervention remains visible among the statewide road services.

Read ``MODELED CAMPAIGN TOTALS`` for all disclosed owners, including those
outside the current relationship page. Owner and workforce identities must
match for these totals. For material totals, select an exact good and unit
in World's material lens. The comparison also shows delivery to end buyers
and unsold retail stock for that selection when the final-demand accounts
match. Missing or incompatible accounts make a total unavailable.

Open reconstructs the saved graph, source observations, recipes, paths, and
parameters. Editing or moving current source files cannot supply new facts to
that saved campaign. Check resume against its retained identities. Return Live
to advance, then revisit the earlier reading. Production-evidence V6 binds
the complete authorized reading, including absent versus completed-zero
accounts and ordered physical paths. Restricted previews disclose none of
these material accounts.

Record the exact source revision, campaign identities, completed periods, and
observations for a native session at 1366×768 and 1920×1080. Hosted and
automated evidence supports the Director's discovery, comparison, and resume
session. It does not supply comprehension acceptance. ADR260 records the
statewide boundary.

Qualify a statewide intervention
--------------------------------

Use ``statewide_experiment`` after physical path and commodity qualification.
It runs the same staffed material replay session for baseline, freight-only,
packaging-only, and combined candidates, each for 16 periods. It reads committed
receipts and does not write saved campaigns or change authored content.

Build from ``rust/`` when no other heavy gate is running:

.. code-block:: bash

   cargo build -p babylon-persistence --example statewide_experiment --locked
   target/debug/examples/statewide_experiment --help

Supply ``--defines``, ``--qualification``, and ``--physical`` with matching
TOML and decoded JSON sources. Select a captured ``--capacity-key`` and
``--food-process``. Set ``--constrained-grams`` below the baseline freight
capacity and ``--shortage-opening`` below that food producer's opening paper
packaging stock. The shortage may be zero. Use ``--output`` for a new absolute
JSON file path; the runner refuses an existing output.

Read ``qualified`` and ``witnesses`` in the report. Freight qualification needs
less dispatch through the selected capacity and later downstream differences
in both production and employed/reserve counts. Packaging qualification needs
a changed output at the selected food producer. A missing witness returns a
nonzero exit status while retaining the complete candidate report. Invalid
inputs produce no report. Reports must fit 64 MiB.

Choose a demonstration whose receipts explain affected and unaffected places.
More constrained freight can leave inputs at a supplier and change its own
production. A difference alone does not explain the chain of causes. Keep
quantities Designed and tune their effects for readable play. This check does
not estimate Michigan's real productivity or replace persisted and native
qualification.

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
regional experiment, whose authored topology has one process per site. They are
not causal labels from the engine.

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
:doc:`/reference/configuration` for the regional and statewide fields, their
units, and validation.

Inspect performance
-------------------

The full observer keeps one repeatable-read transaction while authenticating
the captured circuit. Its idle transaction limit is 120 seconds, matching
material runtime reads. SQL execution, lock and connection limits remain five
seconds. Report actual observation latency separately from these refusal limits.

Use ``BABYLON_POSTGRES_LIVE_FOCUS=statewide_synthetic mise run test:rust-postgres``
for the full-roster persistence regression. Once the canonical sources are
qualified, ``BABYLON_POSTGRES_LIVE_FOCUS=statewide_qualified mise run test:rust-postgres``
runs all four actual statewide presets through 16 periods in an owned disposable
database. The ordinary reader focus excludes this longer source qualification.
For hosted evidence, dispatch ``weekly-pg-integration.yml`` at the exact lane
revision with ``focus=statewide_qualified``. Manual runs check out that event's
commit. Scheduled runs check current ``dev``. This path runs no documentation
generation.

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
