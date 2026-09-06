Babylon: The Fall of America
============================

Babylon is an **entertainment-first emergent political-economy game**. Babylon
is not a forecast and not a scientific reproduction. Theory constrains the
causal model but does not predetermine results.

Determinism proves computational identity, not scientific truth. Historical
cases test causal signatures and counterfactual behavior. The Bevy client is an
administrative viewer with no player action.

The next four executable gates are:

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

Read the repository ``CONSTITUTION.md`` v4.1.0 for the law. Read
``NORTH_STAR.md`` for the game direction and gate contracts.

System overview
---------------

Babylon is a causal sandbox with a fixed four-week tick. Typed world data, rules,
and feedback produce a new world, an in-memory Rust ``TickReport``, and a stable
hash. Persisted replay remains in the frozen Python path.

Rust owns game judgment and world hashes. BSL has live rules but no executable
shocks. The live
Rust path uses ``babylon-kernel``, ``babylon-graph``, ``babylon-bsl``,
``babylon-tick``, and ``babylon-client``.

The Bevy client can show the county atlas and move ticks forward. The client has no
game verbs. The Python engine is a frozen behavioral reference and remains
useful for port contracts and data periphery.

.. Vale: this paragraph preserves literal persistence and schema identifiers.
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

Python supplies mutable SQLite replay, atomic Postgres persistence, and
partial ``babylon_meta`` state. The full v4 Rust three-schema campaign boundary
and Archive decision cycle have not landed.

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES

.. Vale: the next role contains a literal Sphinx document path.
.. vale ste.Ambiguity = NO

See :doc:`/concepts/architecture` for the boundary between live and planned
parts.

.. vale ste.Ambiguity = YES

First run
---------

.. code-block:: bash

   git clone https://github.com/percy-raskova/babylon.git
   cd babylon
   mise trust
   mise run setup
   mise run reference:python-smoke
   mise run check

``mise run reference:python-smoke`` starts the frozen Python reference smoke
test. It does not start a playable Bevy game. See the root ``SETUP_GUIDE.md``
for installation help.

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
