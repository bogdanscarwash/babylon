Persistence Reference
=====================

``babylon-persistence``, composed by ``babylon-runtime``, owns authoritative
campaign state. The live observer session uses ``DurableMaterialRuntime``
with the current Michigan foundation. Python prepares reference artifacts and runs
operator tools; it has no campaign writer or transition reader.

Commands and Bootstrap
----------------------

Use the repository tasks from the checkout root:

.. code-block:: bash

   mise run db:bootstrap
   mise run play
   mise run sim:report

``db:bootstrap`` atomically constructs or verifies the current schema. It
validates the embedded H3 cohort and Michigan reference
foundation before database access, then installs the immutable reference
bundle. Fresh and current native schemas are the admitted starting states.
The retired Python database adoption, shadow backfill, and migration-prefix
modes are not available.

``play`` launches the durable observer session. Its runtime command is
``babylon-runtime session --stdio --defines PATH``. New reads and validates the
selected authored file before creating campaign rows. Open reconstructs the
campaign's saved values without reading that file.
The full foundation binds parameters, graph content, material bundles, staffing,
interval, and horizon. Unsupported content refuses without deleting the save.

``sim:report`` runs a current material campaign with deterministic identity.
Its default 15 periods exercise restart within the authored horizon. The runtime
refuses incompatible diagnostic databases and preserves their data.

Authority and Schema
--------------------

The runtime admits one complete schema. Fresh initialization holds an advisory
lock, constructs the schema atomically, and writes its identity last. Opening
or writing requires the expected identity, catalog structure, ownership, and
permitted reader-role grants. The runtime reconciles commit acknowledgement loss
through the exact committed state. It refuses incompatible or partially
initialized databases before mutation.

The authoritative schemas are:

``babylon_ref``
   Immutable geography, H3 cohorts, overlaps, and exact reference artifacts.

``babylon_state``
   Campaign foundations, graph and material state, events, choice receipts,
   checkpoints, commit markers, and Archive dirty receipts.

``babylon_meta``
   Authority and campaign/navigation metadata.

The complete schema includes material foundations, transitions, Archive, and
reader views. Reference-data installation and role grants are separate from
schema construction. The runtime admits only material commit layout 3.

Durable Material Runtime
------------------------

``DurableMaterialRuntime`` owns adjudication and commit. A new campaign
captures its graph foundation, complete material register, staffing authority,
and authored content identity in one foundation transaction. Opening a
campaign verifies those same stored components before reconstruction.

Each advance judges one 28-day period on detached state. The current Michigan
campaign has an empty BSL rule set; typed material production, routed freight,
and staffing determine its physical transition. The runtime stops at the saved
horizon, which can be 1 through 16 periods.

A caller cannot commit a pre-judged report. The runtime publishes an
acknowledgement only after a successful commit or exact reconciliation of an
ambiguous commit. Refused judgment does not advance the published session.

Transaction Boundary
--------------------

``CommittedMaterialTickEnvelope`` binds eight ordered families: the six typed
component families followed by the material register and material receipts.
It includes the exact action-batch source, graph evidence, events, choice
receipts, full checkpoint, and Archive dirty receipt.

The transaction writes the typed families and material state before the final
``babylon_state.tick_commit`` marker. Material markers carry
``envelope_layout_version = 3``. Material readers require that layout and the
exact component digests.

Collections use explicit positions or primary-key byte order. Numeric codecs
reject non-finite values and normalize negative zero. Retry reconstructs the
complete envelope and requires exact byte identity. Durability comes from the
commit marker, never a maximum tick over a state table.

Foundation, Restart, and Reads
-------------------------------

The foundation preserves the exact graph, world registers, resolver manifest,
prepared environment, replay identity, seed, content, and reference digests.
Current material admission decodes the saved canonical defines, rebuilds the complete
foundation, and compares its bytes. Editing or deleting an external TOML file
cannot change an existing campaign's parameters.

Restart verifies the foundation and a complete full checkpoint, reconstructs
its graph and material components, and authenticates the committed tail.
A delta checkpoint cannot be a restart root. Missing, inconsistent, or
noncanonical components refuse before the runtime resumes.

The full observer reads authenticated committed material evidence. The player
knowledge preview treats material parameters as opaque: it does not query the
hidden foundation bytes and returns no production or nominal-world projection.
Public campaign metadata alone cannot grant access to those values.

The production evidence digest binds route legs and freight-capacity accounts
alongside stocks, dispatch, arrivals, output, and staffing. Capacity readings
compare adjacent authenticated material registers against actual dispatch
receipts. Foundation has no completed reservation account. A completed period
can have a present account with zero dispatch. This projection uses the existing
SQL layout, material envelope, and session protocol.

The Archive dirty receipt participates in the envelope comparison. The Archive
worker can publish after the tick becomes durable, so the window reports its
progress separately. Restart does not consume historical Archive prose as
simulation input.

Verification
------------

Run the smallest applicable checks first and serialize heavy jobs:

.. code-block:: bash

   mise run rust:test:q -- -p babylon-persistence
   mise run test:rust-postgres

The PostgreSQL harness defaults to ``runtime_smoke``. It uses an immutable
pinned image, exact disposable container ownership, loopback admission, and
checked cleanup. Select one focus explicitly when its behavior changes:

.. code-block:: bash

   BABYLON_POSTGRES_LIVE_FOCUS=reference_integrity mise run test:rust-postgres
   BABYLON_POSTGRES_LIVE_FOCUS=runtime mise run test:rust-postgres
   BABYLON_POSTGRES_LIVE_FOCUS=archive mise run test:rust-postgres
   BABYLON_POSTGRES_LIVE_FOCUS=reader mise run test:rust-postgres
   BABYLON_POSTGRES_LIVE_FOCUS=client mise run test:rust-postgres

Main qualification and the weekly PostgreSQL workflow run all six focuses.
They retain reference integrity, rollback and ambiguous-commit reconciliation,
writer timeouts, runtime restart, Archive, authenticated reader, and live
client contracts. See :doc:`/reference/ci-workflow` for selection and reporting.

Contracts
---------

The current composition uses these contracts:

- ``contracts/current_schema.yaml``
- ``contracts/campaign_foundation_content.yaml``
- ``contracts/material_campaign_foundation_v2.yaml``
- ``contracts/committed_material_tick_v3.yaml``
- ``contracts/simulation_interval_v1.yaml``

Current format identities remain explicit. The runtime rejects unsupported inputs.
Independent semantic byte and refusal vectors remain executable. Superseded
implementations and contracts are recoverable through Git history.

See Also
--------

- :doc:`/concepts/architecture`
- :doc:`/reference/configuration`
- :doc:`/reference/determinism-contract`
