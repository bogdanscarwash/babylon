Projection Registry (Constitution II.11)
========================================

This document is the constitutionally mandated specification for
Constitution II.11: *subsystem boundary interface contracts (views, RPC,
events) and the table-ownership registry*. It discharges the II.11
follow-up TODO recorded in ``CONSTITUTION.md``.

.. contents:: On this page
   :local:
   :depth: 2

The Contract
------------

Constitution II.11 requires that each subsystem own its persistence tables
and that **every cross-subsystem read go through a declared interface** — an
SQL view with an explicit contract, an RPC boundary, or an event stream.
Direct table access from outside the owning subsystem is prohibited; an
unowned table is undefined behaviour.

The projection layer (:mod:`babylon.projection`, Program 24 "The Hoist") is
the consumer of those declared interfaces on behalf of every client. The
registry it exposes — :data:`babylon.projection.registry.REGISTRY`, a tuple
of frozen :class:`~babylon.projection.registry.DeclaredView` records — is the
enumerable, introspectable form of the II.11 contract. Each record names one
Postgres view and pins four properties that make a read across the boundary
well-defined:

#. the **owning subsystem** of the tables the view reads;
#. the explicit **ORDER BY** that makes the projection deterministic;
#. the **columns** the view projects, and the subset exposed to full-text
   search; and
#. the frozen Pydantic **row-model** that hydrates a raw row into a validated
   object.

The registry is *data, not a connection*. Nothing in
:mod:`babylon.projection.registry` opens a database or executes SQL; it
records the shape of the contract so consumers (the county read-model, the
vault materializer, the fixture recorder) can introspect it without touching
Postgres.

Ownership Table
---------------

Every declared view, the subsystem that owns its underlying tables, the
deterministic ordering it must be read in, and the row-model that hydrates
it. This table is the normative registry; the code in
:mod:`babylon.projection.registry` is its executable mirror.

.. list-table:: Declared cross-subsystem read interfaces
   :header-rows: 1
   :widths: 26 24 26 24

   * - View
     - Owning subsystem
     - ORDER BY
     - Row-model
   * - ``v_hex_state_asof``
     - hex substrate (spec-089; hex res-7 is the only persisted source of
       truth, FR-019)
     - ``session_id, tick, h3_index``
     - :class:`~babylon.persistence.hex_state.DynamicHexState`
   * - ``v_county_value_aggregate``
     - hex substrate (spec-062 cross-scale aggregation; computed on read)
     - ``session_id, tick, county_fips``
     - ``CountyValueAggregate``
   * - ``v_state_value_aggregate``
     - hex substrate (spec-062 cross-scale aggregation; computed on read)
     - ``session_id, tick, state_fips``
     - ``StateValueAggregate``
   * - ``v_national_value_aggregate``
     - hex substrate (spec-062 cross-scale aggregation; computed on read)
     - ``session_id, tick, national_id``
     - ``NationalValueAggregate``
   * - ``v_global_phi_balance``
     - **ambiguous** — joins ``dynamic_external_node_state`` and
       ``boundary_flow_register``; no single owner is declared for the
       FR-044 conservation view
     - ``session_id, tick``
     - ``GlobalPhiBalance``
   * - ``v_national_trend``
     - game_session tick-summary read-model (spec-037 bootstrap +
       spec-061 FR-003 US4; single write path via
       ``GameSession.advance_tick``'s ``persist_tick_summary`` commit,
       T5 Unit U2 — "the wind is blowing")
     - ``session_id, tick``
     - ``NationalTrendView``

The row-models ``CountyValueAggregate``, ``StateValueAggregate``,
``NationalValueAggregate``, and ``GlobalPhiBalance`` live in
:mod:`babylon.persistence.postgres_aggregation` — the pre-existing,
II.11-branded typed facade that the registry **generalizes rather than
reinvents**. Their SQL definitions are canonical in
``src/babylon/persistence/migrations/0030_views_current.sql``.
``NationalTrendView`` lives in :mod:`babylon.projection.view_models` instead
— ``v_national_trend`` reads the game-session tick-summary tier, not the hex
substrate, so its row-model joins the dossier module rather than the
hex-aggregation facade. Its SQL definition is canonical in
``src/babylon/persistence/migrations/0038_tick_summary_trend.sql``.

Ambiguous ownership is recorded, never guessed
----------------------------------------------

Where a view reads tables owned by more than one subsystem and no governing
spec declares a single owner, II.11 is best served by recording the
ambiguity explicitly rather than inventing an owner. Such an entry sets
:attr:`~babylon.projection.registry.DeclaredView.ownership_ambiguous` to
``True`` and prefixes its ``owning_subsystem`` string with
:data:`~babylon.projection.registry.AMBIGUOUS_OWNER_PREFIX`. The model
validator requires the flag and the marker to agree, so an ambiguous entry
cannot silently masquerade as an owned one. ``v_global_phi_balance`` is the
sole ambiguous entry today: it joins the external-node subsystem's
``dynamic_external_node_state`` and the boundary-flow subsystem's
``boundary_flow_register``.

The ORDER BY determinism rule
-----------------------------

Constitution III.13 requires every materialization to be deterministic: two
runs over the same state must produce byte-identical output. A projection
read without an explicit ``ORDER BY`` leaves row order to the database, which
is free to vary it — a latent source of non-determinism in any page baked
from the projection. Therefore **every** :class:`DeclaredView` carries a
non-empty ``order_by`` clause, and the model rejects an empty one at
construction. The clause names a key that totally orders the view's rows
(``session_id, tick`` plus the entity key), so a consumer that reads with it
gets the same sequence every time.

Row-models versus dossiers
--------------------------

A registry row-model hydrates exactly one row of one SQL view — the c/v/s/k
sums of ``v_county_value_aggregate``, say. A *dossier* view-model such as
:class:`~babylon.projection.view_models.CountyView` is a different thing: it
composes fields drawn from several subsystems (value aggregate, per-county
Φ, survival calculus, the consciousness simplex, legitimacy) into one record
a client renders. Because a dossier crosses subsystems, every field a fog or
veil gate can withhold — and every field a run may simply not attribute — is
``Optional`` with honest ``None`` semantics: ``None`` means *absent*, never a
defaulted zero. Dossiers are assembled by the projection functions of
Program 24 P1 (the county read-model) and validated through the
``TypeAdapter`` hydrate helpers in
:mod:`babylon.projection.view_models`.

How a new view joins the registry
---------------------------------

To add a declared view:

#. Define (or reuse) the SQL view in a persistence migration, with an
   explicit column list.
#. Provide a frozen Pydantic row-model that hydrates one of its rows —
   reuse an existing model from :mod:`babylon.persistence` where the shape
   already exists.
#. Add a :class:`DeclaredView` to
   :data:`~babylon.projection.registry.REGISTRY` naming the view, its owning
   subsystem (or the ambiguity marker), a non-empty deterministic
   ``ORDER BY``, the full column list, the full-text-search subset, and the
   row-model.
#. Add a row to the ownership table above.

The contract tests in ``tests/unit/projection/test_registry.py`` then hold
the new entry to the same invariants as every other: a unique name, a
non-empty ``ORDER BY``, and full-text-search columns that are a subset of the
declared columns.
