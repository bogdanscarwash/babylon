Persistence Reference
=====================

``babylon-persistence``, composed by ``babylon-runtime``, owns authoritative
campaign state. The live observer session uses ``DurableMaterialRuntimeV3``
with a V5 Michigan foundation. Python prepares reference artifacts and runs
operator tools; it has no campaign writer or transition reader.

Commands and Bootstrap
----------------------

Use the repository tasks from the checkout root:

.. code-block:: bash

   mise run db:bootstrap
   mise run play
   mise run sim:report

``db:bootstrap`` constructs or verifies the current native schema and activates
its authority. It validates the embedded H3 cohort and Michigan reference
foundation before database access, then installs the immutable reference
bundle. Fresh and current native schemas are the admitted starting states.
The retired Python database adoption, shadow backfill, and migration-prefix
modes are not available.

``play`` launches the durable observer session. Its runtime command is
``babylon-runtime session --stdio --defines PATH``. New reads and validates the
selected authored file before installing session schemas or creating campaign
rows. Open reconstructs the campaign's saved values without reading that file.
The full foundation binds parameters, graph content, material bundles, staffing,
interval, and horizon. Unsupported content refuses without deleting the save.

``sim:report`` runs the separate graph-only diagnostic campaign. Its default
15 four-week periods cross the 13-period annual boundary and exercise restart.
``qa:michigan-rollover-smoke`` checks the same diagnostic rollover boundary.
These commands do not advance the material campaign shown in Bevy.

Authority and Schema
--------------------

The native constructor retains the exact construction SQL and checksum
history. The epoch 8/9 ``persistence_authority_ledger`` remains predecessor
evidence. Current authority is
``babylon_meta.committed_tick_v2_authority_ledger``:

#. ``Prepared`` at epoch 10 binds the epoch 9 predecessor, V2 cutover contract,
   and epoch 11 reader migration digests.
#. ``Active`` at epoch 11 binds those inputs and the exact prepared-row digest.

Activation writes its active row last. Reacquisition requires the exact two-row
ledger and its bound predecessor and contract digests. The epoch 9 row alone
cannot reopen the writer. Existing incompatible data is refused; there is no
Python upgrade or alternate writer path.

The authoritative schemas are:

``babylon_ref``
   Immutable geography, H3 cohorts, overlaps, and exact reference artifacts.

``babylon_state``
   Campaign foundations, graph and material state, events, choice receipts,
   checkpoints, commit markers, and Archive dirty receipts.

``babylon_meta``
   Authority and campaign/navigation metadata.

Material runtime installation builds on the active V2 graph schema. It adds
the material foundation and transition relations and admits material commit
layout 3. The graph-only diagnostic runtime retains layout 2.

Durable Material Runtime
------------------------

``DurableMaterialRuntimeV3`` owns adjudication and commit. A new campaign
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

``CommittedMaterialTickEnvelopeV3`` binds eight ordered families: the six typed
V2 component families followed by the material register and material receipts.
It includes the exact action-batch source, graph evidence, events, choice
receipts, full checkpoint, and Archive dirty receipt.

The transaction writes the typed families and material state before the final
``babylon_state.tick_commit`` marker. Material markers carry
``envelope_layout_version = 3``. Material readers require that layout and the
exact component digests; graph-only diagnostic markers retain layout 2.

Collections use explicit positions or primary-key byte order. Numeric codecs
reject non-finite values and normalize negative zero. Retry reconstructs the
complete envelope and requires exact byte identity. Durability comes from the
commit marker, never a maximum tick over a state table.

Foundation, Restart, and Reads
-------------------------------

The foundation preserves the exact graph, world registers, resolver manifest,
prepared environment, replay identity, seed, content, and reference digests.
V5 material admission decodes the saved canonical defines, rebuilds the complete
foundation, and compares its bytes. Editing or deleting an external TOML file
cannot change an existing campaign's parameters.

Restart verifies the foundation and a complete full checkpoint, reconstructs
its graph and material components, and replays any contiguous committed tail.
A delta checkpoint cannot be a restart root. Missing, inconsistent, or
noncanonical components refuse before the runtime resumes.

The full observer reads authenticated committed material evidence. The player
knowledge preview treats material parameters as opaque: it does not query the
hidden foundation bytes and returns no production or nominal-world projection.
Public campaign metadata alone cannot grant access to those values.

The Archive dirty receipt participates in the envelope comparison. The Archive
worker can publish after the tick becomes durable, so the window reports its
progress separately. Restart does not consume historical Archive prose as
simulation input.

Verification
------------

Run the smallest applicable checks first and serialize heavy jobs:

.. code-block:: bash

   uv run --frozen python tools/verify_rust_persistence_cutover_v2.py
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

- ``contracts/rust_persistence_cutover_v2.yaml``
- ``contracts/material_campaign_foundation_v2.yaml``
- ``contracts/committed_material_tick_v3.yaml``
- ``contracts/simulation_interval_v1.yaml``

Historical contracts and byte vectors retain their original names and layouts.
They provide codec and predecessor evidence, not permission to open old weekly
campaigns or run the retired Python migration path.

See Also
--------

- :doc:`/concepts/architecture`
- :doc:`/reference/configuration`
- :doc:`/reference/determinism-contract`
