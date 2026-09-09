Architecture Boundary
=====================

``CONSTITUTION.md`` v4.2.0 governs the architecture. ``NORTH_STAR.md`` gives
the game direction and gate order. This page describes the live boundary after
the one-way PostgreSQL authority cutover.

System Boundary
---------------

Babylon has these primary boundaries:

#. A pure Rust engine judges one four-week tick.
#. Live Rust BSL rules control causal changes and finite material kernels.
#. Recognizers and events remain deterministic.
#. Executable shocks and player actions do not exist yet.
#. ``babylon-persistence`` owns authoritative game-managed PostgreSQL schema,
   writes, restart, and durability.
#. Python builds reference data and supplies current repository and operator
   tools. The frozen simulation and its mutable SQLite runtime are retired.
#. Bevy remains an administrative viewer with no player action.

One tick judges one fixed 28-day interval and produces one durable commit.
There are 13 periods in a modeled year; this is a 364-day simulation calendar,
not variable-length Gregorian months. V6 Michigan campaigns bind the interval
in their canonical content. Their authored TOML defines the work schedule,
recipes, workforce, stocks, orders, throughput, route durations, and a stop
horizon of 1 through 16 periods. The supplied values yield 160 Designed labor
hours per person per period and 16 periods (64 weeks). New reads the selected
file and stores its canonical values in the foundation. Open uses those saved
values. Admission refuses V1 through V5 campaigns and keeps their stored data.
The interval contract is ``contracts/simulation_interval_v1.yaml``.

The current Michigan material campaign admits an empty BSL rule set. Its
production, freight, and staffing run through the typed material transition.
The built-in ``production.bsl`` annual labor calibration remains a conformance
reference; it does not parameterize this campaign. Observed annual QCEW facts
and source weekly wages retain their original units.

Standard and Delayed keep independent freight capacities. The two shared
freight presets route sheet metal and milled meal through one Designed regional
kilogram pool, with 800 or 160 kg per period. Panel freight stays independent.
The existing material allocator reserves capacity once per corridor, unit,
and departure period, with proportional flooring and unused residuals.
Authenticated Production readings derive reservations and dispatch from
committed registers and receipts. They distinguish reserved capacity from
physical arrivals and link the competing chains to production and staffing.

Ordinary BSL rules derive and write world data through governed causal
operations. External shocks must not write downstream results directly.
AI can parse, retrieve, and narrate. AI does not judge a game rule.

Live Rust Path
--------------

The shipping engine path is:

``babylon-kernel``
   Deterministic types and contracts.

``babylon-graph``
   Native relations, hyperedges, and canonical hashes.

``babylon-bsl``
   The BSL lexer, parser, checker, typed finite-kernel analysis, exact
   forecasting, loader, and evaluator.

``babylon-tick``
   The four-week tick, replay identity, material state, and atomic publication.

``babylon-material-circuit``
   Physical production, routed freight, and conserved staffing transitions.

``babylon-persistence``
   Rust-owned PostgreSQL activation, campaign foundation, checkpoint restart,
   typed semantic rows, commit markers, and Archive dirty receipts.

``babylon-client``
   The Bevy administrative viewer.

Each four-week tick runs on detached state and buffers its events. The tick becomes
observable only after all rule, hash, and persistence boundaries succeed.
``GraphStateHash`` identifies graph bytes only. ``NominalWorldHash`` also binds
completed time, allocator cursors, and the governed phase-schedule digest.
``TickContentHashV1`` binds the identified replay result.
``ReplayTickSession`` publishes ``TickContentHashV1`` atomically. Replay
identity and campaign durability identity are separate typed inputs.

A finite kernel distributes exact ``Mass`` over one enum-ordered family of
bounded material effect bundles. It consumes one replay-keyed integer ticket
draw and applies only the selected bundle. The choice produces a separate
``ChoiceReceiptV1`` even when the selected bundle changes no material state.
Deterministic mechanics are the one-outcome case. Events own no authored
probability. The language assumes no independence between choices.

Authoritative Persistence
-------------------------

``babylon-runtime`` is the sole production composition root. It activates the
Rust schema and serves the live observer session through
``DurableMaterialRuntimeV3``. That runtime admits a V6 Michigan foundation,
judges one period, and commits ``CommittedMaterialTickEnvelopeV3`` containing
both graph and material evidence. Callers cannot submit a pre-judged report or
construct a second writer authority.

Material campaigns retain the V2 graph replay and schema-authority contracts
inside their V3 material envelope. Graph-only diagnostic campaigns exercise
``DurableReplayRuntimeV2`` separately. The observer session refuses those
campaigns and older material content. Refusal leaves stored data intact.

The epoch 8/9 predecessor ledger is append-only historical cutover evidence:

.. list-table::
   :header-rows: 1
   :widths: 20 25 55

   * - Ordinal
     - State
     - Meaning
   * - 1
     - ``prepared`` at epoch 8
     - Additive Rust schema and reference preparation completed.
   * - 2
     - ``rust_active`` at epoch 9
     - Legacy Python-managed relations were migrated or proved empty and
       retired. This row is the predecessor for the V2 authority transition.

The active authority ledger is
``babylon_meta.committed_tick_v2_authority_ledger``. Its only legal history is
``Prepared`` at epoch 10 followed by ``Active`` at epoch 11. Both rows bind the
active V2 cutover contract and epoch 11 reader migration. The active row also
binds the prepared-row digest, and the prepared row binds the epoch 9
predecessor-row digest.

The epoch 11 ``Active`` row is the final activation statement before
``COMMIT``. Activation is forward-only and idempotent. Runtime authority
reacquisition requires the exact two-row V2 ledger and its bound contract,
reader migration, and predecessor digests. The epoch 9 ``rust_active`` row
alone cannot reopen the writer.

.. Vale: these paragraphs preserve literal persistence and schema identifiers.
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

The runtime owns three schemas:

``babylon_ref``
   Immutable geography, H3 cohorts, and exact reference artifacts.

``babylon_state``
   Campaign foundation, typed graph and material rows, events, checkpoints,
   ``tick_event_v2`` and ``tick_event_field_v2``,
   ``tick_choice_receipt_v1`` with ``tick_choice_receipt_branch_v1`` and
   ``tick_choice_receipt_carrier_element_v1`` children, the commit marker
   ``tick_commit``, and
   ``archive_dirty_receipt_v1``.

``babylon_meta``
   The authority ledger plus typed campaign and navigation metadata.

One marker-last transaction writes the complete typed tick estate. It writes
choice receipts and choice-linked event metadata before a required full
checkpoint and one Archive dirty receipt. It then writes the commit marker.

The runtime acknowledges the tick only after ``COMMIT`` or exact
ambiguous-commit reconciliation. Retry and restart must reproduce the same
material envelope bytes. Material campaign markers record
``envelope_layout_version = 3``; material readers require that layout.
Graph-only diagnostic markers retain layout 2.

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES

Restart loads the campaign foundation or latest complete full checkpoint, then
replays a contiguous marker tail. A delta checkpoint is never a restore root.
Missing, duplicate, out-of-order, or digest-mismatched sections refuse before
the runtime resumes.

H3 Reader Boundary
------------------

Epoch 7 captured and proved the legacy H3 reader parity corpus. Epoch 9 has no
Python game-state reader edge and no compatibility projection. Rust installs
the exact reference cohort and Michigan dynamic foundation, then reads typed
relations directly.

Reference Data and Operator Tools
---------------------------------

PER-48 is decided. The one-way cutover is complete. Rust owns authoritative
game-managed Postgres. Python continues only in the roles declared below.

The retired Python simulation is available at ``p27-python-freeze``. BSL
citations read that tag; Rust owns the executable schedule, mechanics, replay
and persistence contracts. Python builds reference artifacts and supports
current repository and operator commands. Those tools do not adjudicate ticks
or read authoritative transition rows.

Client and Archive Boundary
---------------------------

The Bevy client still reads an administrative world view and displays the
nominal world hash. It does not submit a player intent.

Each committed tick emits an Archive dirty receipt. The Rust Archive worker
binds each receipt to an exact dirty batch, worker contract, and pinned
knowledge-grant snapshot. It publishes immutable county and place dossiers
with validated content and known citations. The scoped reader admits the
requested committed period, retained publication, and disclosed links together.
Global Archive progress cannot certify a selected page.

The runtime owns one Archive listener and worker. Empty Postgres notifications
signal committed tick markers and campaign enrollment. The listener registers
before reading durable work at startup and after reconnect. It drains retained
work through the existing worker. An idle notification timeout performs no
maintenance query. Notifications carry no world state or player intent.

One coordinator owns the V3 session control pipe and tick acknowledgements.
It flushes ``Committed`` before handling the resulting Archive progress. Bevy accepts
progress only for its acknowledged campaign and durable period, then refreshes
its scoped read. It does not poll for Archive maintenance.

Shutdown requests cooperative cancellation and observes actual worker
completion. A database connection that stays open beyond the existing process
deadline cannot claim successful shutdown.
ADR254 records this scheduling boundary. G5 adds player actions separately.

Event payloads contain observed or derived material facts, never probability.
Committed event metadata records the emitting rule and can carry an
automatically derived reference to the ``ChoiceReceiptV1`` that a finite
projection observed. Removing an event sink cannot change a material
trajectory.

Flow
----

This flow shows the current Michigan material campaign. Solid arrows are live.
Dashed arrows are later gate work.

.. Vale: the Mermaid block contains literal crate and schema identifiers.
.. vale off

.. mermaid::

   flowchart LR
       REF["babylon_ref"] --> TICK["Rust material tick"]
       DEFINES["Saved authored parameters"] --> TICK
       MATERIAL["Production, freight, staffing"] --> TICK
       EMPTY["Exact empty action batch"] --> TICK
       TICK --> IDENTIFIED["IdentifiedMaterialTickV3"]
       IDENTIFIED --> RUNTIME["DurableMaterialRuntimeV3"]
       RUNTIME --> STATE["babylon_state typed rows"]
       STATE --> RECEIPT["ChoiceReceiptV1 rows"]
       STATE --> MARKER["tick_commit"]
       STATE --> DIRTY["archive_dirty_receipt_v1"]
       MARKER --> VIEW["Bevy administrative viewer"]
       PY["Reference builders"] --> DATA["SQLite and Parquet artifacts"]
       DATA --> REF
       DIRTY --> ARCHIVE["Semantic Archive worker"]
       ARCHIVE -.-> CHOICE["Player decision"]

.. vale on

Invariants
----------

Tick identity
   Equal inputs produce equal graph, nominal-world, replay-tick, envelope, and
   typed semantic row bytes. Equal kernel instances produce equal allocation,
   draw, selection, and receipt bytes. Only the marker establishes durability.

Pure judgment
   Relation, BSL, and tick crates have no database dependency. Storage begins
   only after detached judgment succeeds.

Single authority
   No compatibility view, adapter, fallback, dual writer, dual storage, or
   runnable midpoint exists.

Native topology
   Hyperedges remain first-class public elements. Incidence data is an internal
   storage method.

Source honesty
   Each substantive value is ``Observed``, ``Derived``, ``Calibrated``, or
   ``Designed``.

Finite contingency
   Kernels select bounded material effects. Recognizers deterministically
   observe post-state. Exact event likelihood sums the branches that make the
   recognizer emit that event. Events never own probability.

Player relevance
   An administrative display cannot pass a game milestone. The persistence
   cutover is necessary infrastructure, not the playable decision loop.

Related Pages
-------------

- :doc:`/reference/persistence`
- :doc:`/concepts/persistence-architecture`
- :doc:`/concepts/topology`
- :doc:`/reference/bsl-language`
