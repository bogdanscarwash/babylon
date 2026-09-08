Source and API boundaries
=========================

Rust owns simulation, committed game state, and the native viewer. The
authoritative implementations live under ``rust/crates/`` in the repository:

.. list-table:: Native implementation
   :header-rows: 1
   :widths: 30 70

   * - Crate
     - Responsibility
   * - ``babylon-kernel``
     - Deterministic values, identities, and arithmetic.
   * - ``babylon-graph``
     - Typed material relations and graph state.
   * - ``babylon-bsl``
     - BSL parsing, validation, and rule execution.
   * - ``babylon-tick``
     - Causal phase order and four-week transitions.
   * - ``babylon-persistence``
     - Postgres bootstrap, committed campaigns, Archive, and session protocol.
   * - ``babylon-client``
     - Bevy observer, production/history views, and restricted native readers.

Language-neutral byte and behavior contracts live in ``contracts/``. New
Michigan campaigns read ``content/scenarios/michigan/defines.toml``. Existing
campaigns use their stored configuration. See :doc:`/concepts/architecture`
for live authority and :doc:`/versioning` for release identity.

Python periphery
----------------

The retained ``src/babylon/`` package supports data preparation and operator
work. It has no authoritative game transitions or game-state writer.

* ``babylon.data`` provides reference artifact and geographic data helpers.
* ``babylon.reference`` provides reference SQLite and BEA ingestion utilities.
* ``babylon.intelligence`` provides corpus manifests and model provisioning.
* ``babylon.config`` provides operator paths and logging configuration.
* ``babylon.cli`` provides operator commands.

``tools/run_observer_session.py`` launches the native runtime and client with
separate database capabilities. Both the development command and the downloadable
package use that launcher. Retired Python engine, model, and formula interfaces
are available through Git history. They are not part of the current manual.
