Installation
============

This guide covers how to install Babylon and set up your development environment.

Requirements
------------

- mise 2026.9.1 for the locked Python and uv toolchain
- ``rustup`` for the pinned Rust toolchain
- Debian window, audio, input and compiler libraries
- Docker Engine and ``docker compose`` for the default local Postgres database
- Git

Install the native Debian prerequisites first:

.. code-block:: bash

   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config git-lfs libssl-dev libpq-dev \
     libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev \
     libxcursor-dev libxi-dev libxrandr-dev libvulkan-dev mesa-vulkan-drivers

Installation Steps
------------------

1. Clone the repository:

   .. code-block:: bash

      git clone https://github.com/percy-raskova/babylon.git
      cd babylon

2. Install the pinned Python tools and locked dependencies:

   .. code-block:: bash

      mise install --locked
      mise run install

3. Install pre-commit hooks:

   .. code-block:: bash

      mise run hooks

4. Verify the installation:

   .. code-block:: bash

      mise run check

   Run the Rust check through the repository task:

   .. code-block:: bash

      mise run rust:check-no-docs

Open the Observer Game
----------------------

From the repository root:

.. code-block:: bash

   mise run play

The launcher builds the runtime and Bevy client with native Cargo, reuses a
reachable local database, and opens the saved campaign at its durable week.
On first use it creates a new campaign at week zero. The in-game menu provides
new and saved campaigns. It can compare committed weeks and start a scenario
with longer ``sheet-transfer`` durations. The window observes the durable runtime.

The Michigan map has 83 county QCEW baselines. The production scenario
has five Designed county-industry cohorts and a 16-week horizon, with 3D and
compact 2D views. Player interventions belong to Gate 5.

To keep saved worlds and open a new campaign:

.. code-block:: bash

   mise run play -- --new
   mise run play -- --new --preset delayed

Use ``--campaign UUID`` to reopen an exact campaign, or ``--no-build`` after
building the current source. The default database is ``babylon_test`` on
``127.0.0.1:5433``. A custom ``BABYLON_RUNTIME_DSN`` must point to a
reachable local database. The launcher passes writer credentials only to the
runtime and read credentials to the window.

Development Tools
-----------------

The project uses several development tools:

- **Ruff**: Linting and formatting
- **MyPy**: Static type checking
- **Pytest**: Testing framework
- **Cargo**: Rust formatting, linting, and tests
- **Commitizen**: Conventional commit messages

Running Tests
^^^^^^^^^^^^^

.. code-block:: bash

   # Run fast math/logic tests
   uv run pytest -m "not ai"

   # Run AI/narrative evaluation tests
   uv run pytest -m "ai"

   # Run a specific test
   uv run pytest tests/unit/test_foo.py::test_specific

Linting and Formatting
^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Check and fix linting issues
   uv run ruff check . --fix

   # Format code
   uv run ruff format .

   # Type check
   uv run mypy src
