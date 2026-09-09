Sentinel error classes
======================

Six named failure classes, each with one or more sensors that find it. Five
classes are **advisory** and **local/on-demand** — their sensors print loudly,
they never gate CI. The sixth, public-surface baseline blindness (U7.11), is
wired as a real gate (``check:surface``, folded into ``mise run check`` and
CI, owner ruling 2026-07-19 — see that task's Files: block) because a scoped
test run cannot otherwise see a repo-wide public-surface baseline drift.
gate-blindness is the one exception to "one class, one sensor": it kept its
original advisory instrument (``check:coverage``) and gained a second, gating
instrument (``check:gate-coverage``, Task 3 of the qa:regression
modernization program) pointed at ``qa:regression``'s own scenario estate —
see that section for why.
Run one with ``mise run check:<name>`` or
``uv run python tools/sentinel_check.py <sensor>``.

Every finding renders in one line::

    [<error-class>] <symbol> @ <file>:<line> — <problem> | REMEDY: <remedy>

correct-but-inert
-----------------

A producer runs, its models validate, and nothing downstream changes because it
ran. Volume III's entire estate was in this state: eleven calculators executing
correctly at every year boundary, reaching nothing.

:Sensor: ``babylon.sentinels.liveness.checks.check_producers_are_not_inert``
:Registry: ``babylon.sentinels.liveness.registry.LIVENESS_ROWS``
:Run: ``mise run check:liveness``
:Remedy: wire one output to a production consumer, or retire the producer.

computed-but-never-consumed
---------------------------

One declared output has no production reader and no declared reason. ``Path A``
ground rent, ``FictitiousCapitalStock``, ``DEBT_SPIRAL_THRESHOLD`` and
``pole_readings`` were all here. Dormancy is legitimate — *undeclared* dormancy
is not.

:Sensor: ``babylon.sentinels.liveness.checks.check_outputs_have_readers``
:Run: ``mise run check:liveness``
:Remedy: add a real consumer, or set ``dormant_reason`` on the registry row.

gate-blindness
--------------

A gate is green and blind: its harness never executes the estate it claims to
guard. ``qa:regression`` — the project's Definition of Done — injected no
economics calculators at all, so its byte-identical baselines never ran a line
of Volumes I, II or III.

:Sensor: ``babylon.sentinels.coverage.checks.check_gate_estate_coverage``
:Registry: ``babylon.sentinels.coverage.registry.GATE_ESTATES``
:Run: ``mise run check:coverage``
:Remedy: build the harness's overrides from a committed deterministic fixture,
   or narrow the claim with ``exempt_keys`` **and** an ``exempt_reason``.

A second instrument points this same class at ``qa:regression`` itself: the
2026-07-19 U9 inertness episode proved the byte-identical gate can stay green
over a dead feature when no canonical scenario exercises it. ``check:gate-coverage``
statically proves every one of the engine's 30 Systems is either evidenced by a
scenario or declared a reviewable ``CoverageGap`` row in
``tools/regression_scenarios.py`` — and, unlike the advisory instrument above,
it **gates** (owner ruling 2026-07-19 precedent, same as ``check:surface``).

:Sensor: ``babylon.sentinels.gate_coverage.checks.check_union_covers_all_systems``
   (plus ``check_declared_names_exist`` and ``check_bundle_evidence``)
:Run: ``mise run check:gate-coverage`` — gates, folded into ``check:sentinels-static``
   and CI.
:Remedy: add a real ``SystemEvidence`` row backed by a spot-run or a committed
   baseline, or a ``CoverageGap`` row with a true reason + remediation.

intensive-aggregation
---------------------

An unweighted mean of a rate, ratio, share, balance or index across space or
class. The aggregate profit rate is ``Σs / Σ(c+v)``, never ``mean(rᵢ)``: the
unweighted form lets a four-hundred-person county swing a national threshold as
hard as Wayne County.

:Sensor: ``babylon.sentinels.aggregation.checks.check_no_unweighted_intensive_means``
:Registry: ``babylon.sentinels.aggregation.registry``
:Run: ``mise run check:aggregation-intensive``
:Remedy: aggregate numerator and denominator separately, or declare an
   ``AggregationExemption`` with the reason equal weighting is materially right.

undeclared-coupling
-------------------

The coupling graph is a claim about the code, and claims drift. Checked in
**both** directions: an edge with no real read behind it (declared-but-absent),
and a real read with no edge declaring it (present-but-undeclared). Four reserved
edges sat dormant for months; ``momentum_coupling`` was a real dependency nobody
had written down.

:Sensor: ``babylon.sentinels.coupling.checks`` (both directions)
:Registry: ``babylon.sentinels.coupling.registry.MEASUREMENT_DEPENDENCIES``
:Run: ``mise run check:coupling``
:Remedy: wire the dependency the edge claims, delete the edge, or declare the
   edge that the real read already implies.

public-surface baseline blindness
----------------------------------

A scoped, per-task test run cannot see repo-wide public-surface baselines: an
``__all__`` edited without its pinned ``EXPECTED_*_PUBLIC`` baseline reds only
the full gate, never a scoped run. Live specimen: ``CapitalVolumeIIIDefines``
added to ``babylon.config.defines.__all__`` (U2.3) without a matching edit to
``EXPECTED_DEFINES_PUBLIC``.

:Sensor: ``babylon.sentinels.surface.checks.check_pinned_surfaces``
:Registry: ``babylon.sentinels.surface.registry.PINNED_SURFACES``
:Run: ``mise run check:surface`` — unlike the other five, this one **gates**
   (folded into ``mise run check`` and CI, owner ruling 2026-07-19; U7.11).
:Remedy: edit the baseline frozenset in the same commit that changed
   ``__all__``.
