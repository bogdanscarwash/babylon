How-To Guides
=============

Task-oriented guides that help you accomplish specific goals. These assume
you already understand the basics and need to solve a particular problem.

.. note::

   **Looking for tutorials?** See :doc:`/tutorials/index` for learning-oriented
   guides that build foundational knowledge.

Extending the Simulation
------------------------

.. toctree::
   :maxdepth: 1

   add-custom-system
   /reference/configuration

Guides for extending and customizing the simulation mechanics.

**Add a Custom System**
   Create, register, and test custom simulation systems to model new
   mechanics like propaganda, sanctions, or environmental effects.

**Tune Authored Campaign Parameters**
   Edit the current Michigan TOML parameters before creating a campaign.
   The configuration reference defines the units, validation, and saved-value
   boundary. Existing campaigns retain their own parameters.

Reference Data
--------------

.. toctree::
   :maxdepth: 1

   reference-data-pipeline

**Add or Change Reference Data (parquet-canonical pipeline)**
   Add tables or ingest rows through the source-only pipeline: parquet +
   ``schema.sql`` are canonical, the SQLite reference DB is a deterministic
   build product, and loaders run against scratch copies via
   ``tools/loader_to_sources.py`` (ADR098).

State Apparatus AI (Feature 039)
---------------------------------

.. toctree::
   :maxdepth: 1

   state-apparatus-ai

**Work with the State Apparatus AI**
   Add new sub-verbs, tune faction dynamics, adjust the REPRESS pipeline,
   use god mode for debugging, integrate with state AI events, read
   player-visible state information, add faction shift triggers, and
   run the 52-tick integration test.

Debugging & Analysis
--------------------

.. toctree::
   :maxdepth: 1

   debug-simulation-outcomes

**Debug Simulation Outcomes**
   Inspect committed Rust reports, explain material accounts, and measure
   four-week simulation and database work with the existing diagnostic tasks.

GUI Development
---------------

.. toctree::
   :maxdepth: 1

   gui-development

**GUI Development Plan**
   Build visualization and user interface features using NiceGUI.
   Covers the phased approach from basic displays to full interactivity.

Getting Started
---------------

.. toctree::
   :maxdepth: 1

   setup-dev-environment

**Set Up a Development Environment**
   Complete setup guide for Linux, macOS, and Windows. Windows users get
   step-by-step WSL 2 installation and VSCode Remote integration.

Contributing
------------

.. toctree::
   :maxdepth: 1

   contribute
   run-ci-locally

**Submit a Pull Request**
   Complete workflow from branching to merge. Includes branch naming,
   commit conventions, and handling CI failures.

**Run CI Locally**
   Test CI checks before pushing using direct commands, mise tasks,
   or ``gh act`` for full workflow simulation.

Coming Soon
-----------

These guides are planned for future development:

- **Optimize RAG** - Improve AI narrative generation performance
- **Build PDF Documentation** - Generate PDF books from Sphinx docs

See Also
--------

- :doc:`/tutorials/index` - Learning-oriented guides for newcomers
- :doc:`/concepts/index` - Deep explanations of design decisions
- :doc:`/reference/index` - Quick lookup for APIs and settings
