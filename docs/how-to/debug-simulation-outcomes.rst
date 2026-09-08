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
existing campaign data when comparing source revisions; V5 content refuses
weekly campaign presets rather than interpreting their ordinals as four-week
periods.

Explain the material change
---------------------------

Open the Bevy session with ``mise run play``. Hold the same completed period
and perspective when comparing campaigns. Trace an output through its opening
inputs, committed deliveries, production recipe, labor account, and closing
inventory. Distinguish foundation stocks, unavailable evidence, and a present
completed-period receipt with a real zero.

The Designed Michigan presets differ in the sheet-transfer travel time:
one period (four weeks) or three periods (twelve weeks). Their per-period
flow and hour budgets use the four-week interval. Initial stocks, people,
order totals, and physical recipes keep their original units. QCEW wages
remain observed weekly rates and do not become modeled payroll.

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
