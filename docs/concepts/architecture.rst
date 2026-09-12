Architecture Boundary
=====================

``CONSTITUTION.md`` v4.2.0 governs the architecture. ``NORTH_STAR.md`` gives
the game direction and gate order. This page describes the current Rust
implementation and persistence boundary.

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
not variable-length Gregorian months. Current Michigan campaigns bind the interval
in their canonical content. Their authored TOML defines the work schedule,
recipes, workforce, stocks, orders, throughput, route durations, and a stop
horizon of 1 through 16 periods. The supplied values yield 160 Designed labor
hours per person per period and 16 periods (64 weeks). Definitions
normalize regional and statewide authoring into one current content model.
New captures that model, the generated graph scenario, observed source cells,
and observed definitions. Statewide capture also retains resolved physical
paths, selected edge geometry, terminal attachments, and network authority.
Open reconstructs those saved facts without consulting changed source files
or rerouting. Admission refuses older Michigan content and keeps its stored
data. Development has no save migration or compatibility compiler.
The interval contract is ``contracts/simulation_interval_v1.yaml``.

The current Michigan material campaign admits an empty BSL rule set. Its
production, freight, local transfers, merchant handling, and staffing run
through the typed material transition.
The built-in ``production.bsl`` annual labor calibration remains a conformance
reference; it does not parameterize this campaign. Observed annual QCEW facts
and source weekly wages keep their original units.

Standard and Delayed keep independent freight capacities. The two shared
freight presets route sheet metal and milled meal through one Designed regional
kilogram pool, with 800 or 160 kg per period. Panel freight stays independent.
Native good quantities keep exact gram coefficients. The compiler emits
one budget for each active capacity principal and period. A timed journey
reserves its distinct principals through the existing proportional-floor
allocator, then debits actual movement and leaves unused residuals. The
regional service remains schematic. Statewide physical paths keep road
geometry separate from one-period travel and Designed capacity groups.
County terminal attachments do not identify factory locations.

The statewide roster admits source-supported commodity producers and physical
merchants. Processes at one owner share inventory and one workforce
pool. Source jobs and payroll remain context rather than modeled production
or labor. An added upstream process changes executable content without adding
another economic owner.

Merchants keep commodity identity while spending handling capacity and
labor. Routed arrivals precede outbound handling. County-local transfers
between owners debit the supplier and credit the buyer without a road stage,
transit lot, or physical-arrival receipt. The allocator fixes all outbound
grants before local credits, preventing recursive forwarding in the same close. Local
retail fulfillment completes finite end-buyer orders and creates no household
stock. It establishes neither consumption nor payment.

Authenticated Circuit readings derive stock movement, capacity reservations,
handling, and staffing from committed registers and receipts. They connect
shared-capacity competitors to output and workforce effects while preserving
the distinction between reservations, local transfers, and physical arrivals.
The roster and compiler do not certify a qualified statewide road graph or
the comparison's causal witnesses. ADR260 requires those checks and the
Director's comprehension session. Intervention identities and quantities
remain subject to that qualification.

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
   Physical production, routed freight, local transfers, merchant handling,
   finite final fulfillment, and conserved staffing transitions.

``babylon-persistence``
   Rust-owned PostgreSQL activation, campaign foundation, checkpoint restart,
   typed semantic rows, commit markers, and Archive dirty receipts.

``babylon-client``
   The Bevy administrative viewer.

Each four-week tick runs on detached state and buffers its events. The tick becomes
observable only after all rule, hash, and persistence boundaries succeed.
``GraphStateHash`` identifies graph bytes only. ``NominalWorldHash`` also binds
completed time, allocator cursors, and the governed phase-schedule digest.
``TickContentHash`` binds the identified replay result.
``ReplayTickSession`` publishes ``TickContentHash`` atomically. Replay
identity and campaign durability identity are separate typed inputs.

A finite kernel distributes exact ``Mass`` over one enum-ordered family of
bounded material effect bundles. It consumes one replay-keyed integer ticket
draw and applies only the selected bundle. The choice produces a separate
``ChoiceReceipt`` even when the selected bundle changes no material state.
Deterministic mechanics are the one-outcome case. Events own no authored
probability. The language assumes no independence between choices.

Authoritative Persistence
-------------------------

``babylon-runtime`` is the production composition root. It verifies the current
schema and serves the observer through ``DurableMaterialRuntime``. That runtime
captures one current Michigan foundation, judges one period, and commits graph
and material evidence together. Callers cannot submit a pre-judged report.

Production, freight, merchant handling, local transfers, retail fulfillment,
and staffing share one material state. Production updates that state's fields
through the common checked inventory operations. Staffing binds production or
merchant work to a conserved pool. ``ProductionEvidenceDigest`` binds the
complete authorized projection, sorting unordered rows while preserving event,
geometry, and physical path order.

Practice has one typed contract for actors, stable targets, authority, intents,
resource quotes, resolved batches, and ordered actions. The evidence driver
uses that contract directly. Seeded replay and sealed carrier keys govern BSL
and tick execution. Scenario and rule diagnostics use an explicit deterministic
identity and the production collect-and-apply adjudicator.

Fresh initialization constructs the complete current schema in one transaction,
under the advisory lock. Its schema identity is the final initialization write.
Admission verifies the exact schema, ownership, and permitted role grants.
Reference-data installation and reader-role provisioning have separate duties.
Both use the current schema. The runtime refuses incompatible databases before
mutation and preserves their data.

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

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES

Restart reconstructs the captured foundation and loads the latest complete
material checkpoint bound to the committed tail. A delta checkpoint is never a restore root.
Missing, duplicate, out-of-order, or digest-mismatched sections refuse before
the runtime resumes.

H3 Reader Boundary
------------------

Rust installs the exact reference cohort and Michigan dynamic foundation, then
reads typed relations directly. Python has no game-state reader or compatibility
projection. The consuming Rust paths check reference transport, hierarchy,
ordering, and current restricted readers.

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

World opens with the complete disclosed economy as a schematic network over
level county geography. All admitted owner cohorts remain visible, including isolated
cohorts. Commodity links preserve supplier and buyer identities; county-local
transfers and finite retail orders connect merchants to end buyers. Display
offsets separate aggregate owners without claiming factory locations.
Selecting a node highlights its direct links while retaining the wider network.
Industry filters retain the selected industry and its direct trading neighbors.

Modeled employed people, modeled reserve, and observed QCEW have separate
height lenses. Physical lenses need a specific good and
unit. They do not add unlike goods into an output total. Workforce totals
count each pool once.

A county selection leads to paged owner cohorts, then
Circuit detail with at most six incident relationship groups per page.
Shared-capacity competitors have their own navigation rather than appearing
as suppliers. Physical road layers reuse captured geometry and remain separate
from the schematic economy network. Both read the authenticated observation;
neither introduces a supplier graph or allocation engine of its own.

The Readings panel retains subject, period, output or handling, and workforce
context above its Flow, Freight, Work, and Sources sections. Foundation
accounts, unavailable evidence, and completed zero values remain distinct.
Comparison reads the same committed period in saved campaigns. It does not
advance either world. Restricted knowledge previews contain no material
projection or production-evidence digest.

Each committed tick emits an Archive dirty receipt. The Rust Archive worker
binds each receipt to an exact dirty batch, worker contract, and pinned
knowledge-grant snapshot. It publishes immutable county and place dossiers
with validated content and known citations. The scoped reader admits the
requested committed period, retained publication, and disclosed links together.
Global Archive progress cannot certify a selected page.

The runtime owns one Archive listener and worker. Empty Postgres notifications
signal committed tick markers and campaign creation. The listener registers
before reading durable work at startup and after reconnect. It drains retained
work through the existing worker. An idle notification timeout performs no
maintenance query. Notifications carry no world state or player intent.

One coordinator owns the session control pipe and tick acknowledgements.
It flushes ``Committed`` before handling the resulting Archive progress. Bevy accepts
progress only for its acknowledged campaign and durable period, then refreshes
its scoped read. It does not poll for Archive maintenance.

Shutdown requests cooperative cancellation and observes actual worker
completion. A database connection that stays open beyond the existing process
deadline cannot claim successful shutdown.
ADR254 records this scheduling boundary. G5 adds player actions separately.

Event payloads contain observed or derived material facts, never probability.
Committed event metadata records the emitting rule and can carry an
automatically derived reference to the ``ChoiceReceipt`` that a finite
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
       MATERIAL["Production, circulation, staffing"] --> TICK
       EMPTY["Exact empty action batch"] --> TICK
       TICK --> IDENTIFIED["IdentifiedMaterialTick"]
       IDENTIFIED --> RUNTIME["DurableMaterialRuntime"]
       RUNTIME --> STATE["babylon_state typed rows"]
       STATE --> RECEIPT["ChoiceReceipt rows"]
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
