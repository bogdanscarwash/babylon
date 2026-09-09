Declared Synthetic Data
========================

The closed registry of every sanctioned synthetic/fallback data source on a
Babylon production code path — what fakes data, why that is legitimate, and
what guards it so the fake value can never reach a production run unrecorded.

.. contents:: On this page
   :local:
   :depth: 2

The Invariant
-------------

Babylon's engine and web layers contain a small, deliberate set of synthetic
or fallback data sources: a mock API bridge for dev/test without Postgres,
hardcoded graceful-degradation defaults for unwired economics calculators,
and hand-authored (not empirically-sourced) scenario fixtures. Each of these
is legitimate *only* because something guards it — a ``DEBUG`` gate, a
fallback-tally counter the run manifest surfaces, or the self-evident fact of
being a deterministic fixture that is never mistaken for reference data.

**No synthetic value reaches a production run unrecorded.** That is the
invariant this document and its companion sentinel enforce. Concretely:
every sanctioned source is declared in
:data:`babylon.sentinels.synthetic.registry.SYNTHETIC_SOURCES` alongside the
symbol that guards it, and
:mod:`babylon.sentinels.synthetic.checks` statically proves both symbols
still exist on every fast-gate run. A guard that is renamed, moved, or
deleted reds the gate even if the fake-data source itself is untouched — the
failure mode this closes is *"the source is still here, but nothing is
watching it anymore."* A newly discovered synthetic/fallback source on a
production path must be declared here, with a real guard, to pass review —
the registry is meant to stay **closed**, not merely additive.

This mirrors the pattern the data-coverage sentinel
(:mod:`babylon.sentinels.coverage`) already established for *missing*
reference data; this registry is its mirror image for *fabricated* data.

Registry
--------

Five sources are currently declared (Phase F ghost-data audit, verified
2026-07-16). A targeted ``rg`` sweep of ``src/`` and ``web/`` for
``fallback``/``placeholder``/``synthetic``/``stub``/``mock`` on non-test
production paths found no *unguarded* synthetic source beyond these five —
every other hit was either a SQL/template ``?``/``{...}`` placeholder or a
comment about one of the sources below.

.. list-table::
   :header-rows: 1
   :widths: 18 30 30 22

   * - Source
     - What it fakes
     - Guard
     - Guard kind
   * - ``StubEngineBridge``
     - The full ``EngineBridge`` query/action surface — deterministic,
       realistic-looking mock session/tick/map/doctrine data
     - ``_get_bridge()`` refuses it outside ``DEBUG``
     - DEBUG gate
   * - ``_DEFAULT_EMPLOYMENT``
     - Per-county employment headcount (100,000) when no real
       ``employment_source`` is wired
     - Wired-source override + in-code III.11 comment
     - Documented default (**not** tallied — see below)
   * - ``TickDynamicsSystem._compute_national_params`` fallbacks
     - National ``gamma_basket``/``gamma_III`` (0.68 / 0.33) when their
       calculators are unwired or return no data
     - ``EconomicsFallbackTally`` counters, surfaced in the run manifest
     - Fallback instrumentation
   * - ``TwoNodeScenario``
     - A minimal 2-node ``WorldState`` with fixed illustrative parameters
     - ``Scenario`` ABC + name-collision auto-registry; explicit
       ``scenario=`` selection only
     - Deterministic seed
   * - ``WayneCountyScenario`` (``"detroit"``)
     - The Wayne County tri-county ``WorldState``: hand-classified H3 hexes
       and fixed illustrative entities
     - Same ``Scenario`` ABC guard as above
     - Deterministic seed

StubEngineBridge — the dev/test mock bridge
--------------------------------------------

**Where:** ``web/game/stub_bridge.py:748``, class ``StubEngineBridge``.
Serves the same response envelope as the real ``EngineBridge`` — e.g. the
4-entity Wayne County class roster built by ``_make_wayne_county_entities``
— so the Django app and frontend can run without PostgreSQL or a live
engine (module docstring, ``stub_bridge.py:1-13``).

**The guard:** ``web/game/api.py``'s ``_get_bridge()`` (line 70) lazily
instantiates the singleton bridge. When no real bridge has been initialized
(no call to ``init_bridge()``/``GameConfig.ready()``) it checks
``django.conf.settings.DEBUG``:

.. code-block:: python

   if not django_settings.DEBUG:
       raise ImproperlyConfigured(
           "EngineBridge not initialized and DEBUG is off — refusing to serve the "
           "StubEngineBridge's fabricated data through the production API "
           "(Seam Sensor 3 provenance / Constitution III.11). Initialize a real "
           "bridge via init_bridge() / GameConfig.ready() with a persistence layer."
       )
   ...
   _bridge_instance = StubEngineBridge()

DEBUG off refuses the stub outright; DEBUG on falls back to it with a logged
warning. The error message's "Seam Sensor 3 provenance" naming borrows the
seam sentinel's vocabulary (:mod:`babylon.sentinels.seam.provenance`,
"proves the frontend is not promised a field the backend never sends") but
is its own runtime check, not a call into that module — Sensor 3's actual
static AST check targets a different seam (the map-emitter/frontend-type
diff). This guard's own dynamic proof is
``tests/unit/web/test_stub_bridge_guard.py``:
``test_stub_bridge_refused_with_debug_off`` /
``test_stub_bridge_allowed_with_debug_on``. Bridge *parity* (the stub
exposes the same ``get_*`` methods/signatures as the real bridge, so it
cannot silently drift out of shape) is a separate guard:
``tests/unit/web/test_stub_bridge_parity.py``.

Economics graceful-degradation defaults
------------------------------------------

**Where:** ``src/babylon/domain/economics/tick/initializer.py:44``,
``_DEFAULT_EMPLOYMENT: float = 100_000.0`` — a per-county employment
headcount used by ``DefaultTickInitializer`` to seed the very first tick's
county states, and mirrored (as an inline literal, not a shared reference)
at three further call sites:
``tick/system/__init__.py:434`` (``_bootstrap_county_states``, reading an
existing graph node), ``tick/system/__init__.py:700``
(``_compute_county_states``, when there is no ``prev`` state), and
``tick/graph_bridge.py:262`` (``from_graph``).

**The guard:** ``TickDynamicsSystem._compute_county_states``
(``tick/system/__init__.py:629``) prefers a wired real source over the
default:

.. code-block:: python

   # Real per-county headcount from a wired employment_source (QCEW
   # county rollup), symmetric with capital_stock above; falls back to
   # the documented 100k default when unwired or the county-year is
   # absent (Constitution III.11 graceful degradation).
   employment = prev.employment if prev else 100_000.0
   if services.employment_source is not None:
       emp_result = services.employment_source.get_county_total_employment(fips, year)
       if emp_result and isinstance(emp_result, (int, float)):
           employment = float(emp_result)

``ServiceContainer.employment_source`` (``src/babylon/engine/services.py:176``)
is the override point; its docstring makes the fallback explicit: *"None =>
the tick pipeline keeps its documented 100k graceful-degradation default."*
Every call site of the literal carries the same in-code III.11 citation, so
the default is never unexplained where it appears.

**Verified gap (2026-07-16):** unlike the ``gamma_basket``/``gamma_III``
fallbacks below, this default is **not** one of the counted fields in
``EconomicsFallbackTally`` — it is documented-in-place but not tallied into
the run manifest's ``economics_fallbacks`` block. The background brief for
this audit phase described the employment default as covered by "its
economics_fallbacks tally"; that is not accurate as of this writing. This
document records the gap rather than silently repeating the inaccurate
claim (Verifiability: never document what does not exist in code) or fixing
it (out of scope for a docs-only registry phase) — an
``employment_source_none``-style counter, symmetric with
``gamma_basket_calculator_none``, would close it.

The MELT/gamma_basket/gamma_III fallback tally
--------------------------------------------------

**Where:** ``TickDynamicsSystem._compute_national_params``
(``tick/system/__init__.py:442``). ``gamma_basket_raw`` (0.68) and
``gamma_III_raw`` (0.33) are substituted when ``basket_calculator`` /
``gamma_calculator`` is unwired (``None``) or returns no data. MELT's
``tau`` has no literal-substitution fallback in this path: an unavailable
MELT result aborts the whole annual pipeline for the tick via an early
``return None`` (and, one level up, ``TickDynamicsSystem.step()`` skips the
pipeline entirely — a ``logger.debug()``-only no-op, uncounted by any tally
field — when ``services.melt_calculator`` is ``None`` at all,
``tick/system/__init__.py:151``).

**The guard:** ``EconomicsFallbackTally`` (``src/babylon/engine/services.py:28``),
a fresh instance per ``ServiceContainer`` (``economics_fallbacks`` field,
default-factory-constructed, ``services.py:196``). Every substitution
increments a named counter:

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Counter
     - Fires when
   * - ``record_melt_unavailable()``
     - ``melt_calculator`` is wired but returns no usable value (tick pipeline aborts)
   * - ``record_gamma_basket_calculator_none()``
     - ``basket_calculator`` is ``None`` (0.68 substituted)
   * - ``record_gamma_iii_calculator_none()``
     - ``gamma_calculator`` is ``None`` (0.33 substituted)
   * - ``record_gamma_iii_returned_none()``
     - ``gamma_calculator`` is wired but returns no data (0.33 substituted)

``observe_wiring()`` additionally snapshots wired-vs-``None`` status for all
three calculators on every observation. ``EconomicsFallbackTally.to_dict()``
exposes the counters to frozen-reference diagnostics and tests. It is not a
Rust runtime or persistence-authority surface.

Seed scenario fixtures
--------------------------

**Where:** ``src/babylon/engine/scenarios/`` — a package of 6 registered
``Scenario`` subclasses (``base.py:41``, ``_SCENARIO_REGISTRY`` at
``base.py:38``), including ``TwoNodeScenario``
(``scenarios/two_node.py:20``) and ``WayneCountyScenario``
(``scenarios/wayne_county.py:20``, the scenario the ``"wayne"``/``"detroit"``
aliases resolve to). Each delegates ``build()`` to a legacy free function
(``_legacy.py`` / ``_legacy_wayne.py``) for byte-equality with the
pre-Bundle-2 baseline. The data itself is hand-authored, not
empirically-sourced: fixed illustrative parameters
(``worker_wealth=0.5``, ``extraction_efficiency=0.8``, ...) for
``two_node``; H3 hexes classified by a hardcoded bounding-box rule
(``_classify_wayne_hex``) for ``wayne_county`` — neither reads the
reference database.

**The guard:** these are deterministic, self-documented fixtures, never
substituted for reference data. Two mechanisms keep it that way:

- **Explicit selection only.** ``resolve_scenario()``
  (``web/game/engine_bridge.py:5745``) is the single entry point every
  API/CLI path routes through and raises ``ValueError`` on an unknown
  scenario name — there is no silent fallback to a default scenario for a
  typo or unrecognized identifier (``fix/seed-scenario-loud``,
  ``project/execution/briefs/fix-seed-scenario-loud.md``).
- **Name-collision detection.** ``Scenario.__init_subclass__``
  (``scenarios/base.py:73-89``) raises ``ValueError`` at import time if two
  subclasses register the same ``name`` — a fixture can never silently
  shadow another under one name.

Verifying the registry stays closed
----------------------------------------

:mod:`babylon.sentinels.synthetic` (layer 0.5, same rank as
:mod:`babylon.config`) is the fifth member of the
:mod:`babylon.sentinels` family (alongside Seam, Data-Coverage, Partition,
and the dynamic Determinism/Round-Trip/Economic-Conservation sentinels — see
``ai/decisions/`` for the family's ADRs). Its check,
``check_sources_and_guards_exist()``, parses each
registry row's ``source_file`` and ``guard_file`` with :mod:`ast` — never
importing ``web`` or the engine, which sit above the sentinels' layer-0.5
import boundary — and asserts both the fabricating symbol and its guarding
symbol still exist. A missing symbol on *either* side reds the gate; a
missing/unparseable declared file is a distinct infrastructure failure
(``SentinelCheckError``, exit 2), never silently swallowed into a false
pass.

Run it directly:

.. code-block:: bash

   uv run python tools/sentinel_check.py synthetic --check

This sensor proves static **coherence** (the named symbols exist), not each
guard's runtime behavior — that is what the dynamic test named in each
registry row's ``invariant`` field (e.g.
``tests/unit/web/test_stub_bridge_guard.py``) is for. The sensor's own test
suite is ``tests/unit/sentinels/test_synthetic_registry_check.py``.

Adding a newly discovered source
-------------------------------------

A synthetic or fallback value found on a new production code path is
sanctioned only once it has a real guard AND a registry row naming both. Add
a :class:`~babylon.sentinels.synthetic.registry.SyntheticSource` entry to
:data:`~babylon.sentinels.synthetic.registry.SYNTHETIC_SOURCES` with
``source_file``/``source_symbol`` (the fake), ``guard_file``/``guard_symbol``
(what stops it reaching production unrecorded), a ``guard_kind`` from the
closed vocabulary, and update this document's registry table. Without a row,
the sensor cannot see the source at all — it is a *closed*, opt-in registry,
not a scan that discovers new fakes on its own.

See Also
------------

- :mod:`babylon.sentinels.coverage` — the mirror-image sentinel for
  *missing* reference data (the ``NoDataSentinel`` degradation path).
- ``CONSTITUTION.md`` III.11 (Loud Failure), VIII.12 (no disarmed
  guardrail).
- ``project/execution/briefs/fix-seed-scenario-loud.md`` — the scenario
  entry-point loud-failure fix this document's *Seed scenario fixtures*
  section relies on.
