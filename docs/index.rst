Babylon: The Fall of America
============================

Babylon is an **entertainment-first emergent political-economy game**. Babylon
is not a forecast and not a scientific reproduction. Theory constrains the
causal model but does not predetermine results.

Determinism proves computational identity, not scientific truth. Historical
cases test causal signatures and counterfactual behavior. The Bevy client is an
administrative viewer with no player action.

The project's executable gate contracts cover:

.. Vale: each protected item is a governed gate name.
.. vale Vale.Terms = NO
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

#. **PostgreSQL/H3/Archive decision-loop slice**

#. **Productive & distributive circuit**

#. **Player agency**

#. **COVID emergence benchmark**

.. vale Vale.Terms = YES
.. vale ste.UnapprovedWords = YES
.. vale ste.NounClusters = YES

Read the repository ``CONSTITUTION.md`` for the law. Read
``NORTH_STAR.md`` for the game direction and gate contracts.

System overview
---------------

Babylon uses one four-week simulation tick. Typed world data, BSL rules, and
material relations produce a new world and a stable hash. Rust commits that
world to Postgres and reconstructs saved campaigns from committed state.

Rust owns game judgment and world hashes. BSL has live rules but no executable
shocks. The live
Rust path uses ``babylon-kernel``, ``babylon-graph``, ``babylon-bsl``,
``babylon-tick``, ``babylon-persistence``, and ``babylon-client``.

The Bevy client shows the Michigan county map, production and staffing, committed
history, and saved-campaign comparisons. Enter advances one four-week period.
Space plays or pauses. The client has no player actions. Python supplies data
tools, model provisioning, operator commands, and the native process launcher.

.. Vale: this paragraph preserves literal persistence and schema identifiers.
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

Rust owns the ``babylon_ref``, ``babylon_state``, and ``babylon_meta`` campaign
boundary. The Archive and its restricted readers give cited observations to
the viewer. Reference SQLite and Parquet remain data build artifacts. Python
has no authoritative simulation or game-state writer.

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES

.. Vale: the next role contains a literal Sphinx document path.
.. vale ste.Ambiguity = NO

See :doc:`/concepts/architecture` for the boundary between live and planned
parts.

.. vale ste.Ambiguity = YES

First run
---------

Download the Linux preview from the `GitHub releases page
<https://github.com/percy-raskova/babylon/releases>`_. Its included ``README.md``
lists system requirements. Extract the archive and run ``./babylon``. The package
includes the native executables, their embedded assets, and the launcher. Its
local Docker database stores saves separately from a development checkout.

For development from source:

.. code-block:: bash

   git clone https://github.com/percy-raskova/babylon.git
   cd babylon
   mise trust
   mise install --locked
   mise run install
   mise run play

``mise run play`` builds and opens the durable native observer, starting its
local database when needed. See the root ``SETUP_GUIDE.md`` for system packages
and development installation details.

Manual contents
---------------

.. toctree::
   :maxdepth: 2
   :caption: Tutorials

   tutorials/index

.. toctree::
   :maxdepth: 2
   :caption: How-to guides

   how-to/index
   agents/governance

.. toctree::
   :maxdepth: 2
   :caption: Concepts

   concepts/index

.. toctree::
   :maxdepth: 2
   :caption: Reference

   reference/index

.. toctree::
   :maxdepth: 2
   :caption: API reference

   api/index

.. toctree::
   :maxdepth: 2
   :caption: Commentary

   commentary/index

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`

.. Vale: the next role contains a literal Sphinx reference name.
.. vale ste.UnapprovedWords = NO

* :ref:`search`

.. vale ste.UnapprovedWords = YES
