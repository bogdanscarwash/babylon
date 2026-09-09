GUI Development Guide
=====================

This guide outlines the development standards and architecture for Babylon's
graphical user interface, "The Cockpit."

The interface is built using **PyQt6**, a cross-platform Python GUI framework
that enables high-performance real-time visualization of simulation state
with ECharts-based charts and H3 hexagonal map rendering.

Design Philosophy: Bunker Constructivism
----------------------------------------

The UI follows the "Bunker Constructivism" aesthetic—a blend of Soviet
Constructivism, brutalist industrial design, and Cold War bunker interfaces.
It conveys a sense of surveillance, decay, and urgency.

Visual Language
~~~~~~~~~~~~~~~

- **Metaphor**: A dark-mode engineering dashboard ("The Cockpit") for observing
  systemic collapse.
- **Palette**: High-contrast phosphor colors against absolute black.
- **Typography**: Monospace and industrial sans-serifs.
- **feel**: Mechanical, precise, and foreboding.

Color Palette
~~~~~~~~~~~~~

The design system is centrally defined in :py:class:`babylon.ui.design_system.BunkerPalette`.

.. list-table::
   :widths: 20 20 60
   :header-rows: 1

   * - Color Name
     - Hex
     - Semantic Usage
   * - **VOID**
     - ``#050505``
     - Background darkness. The canvas.
   * - **DATA_GREEN**
     - ``#39FF14``
     - Positive metrics, healthy systems, logs (INFO).
   * - **PHOSPHOR_BURN_RED**
     - ``#D40000``
     - Critical failures, alarms, logs (ERROR), Bourgeoisie class.
   * - **EXPOSED_COPPER**
     - ``#FFD700``
     - Warnings, degraded states, logs (WARN).
   * - **SILVER_DUST**
     - ``#C0C0C0``
     - Neutral text, labels, inactive elements.
   * - **DARK_METAL**
     - ``#404040``
     - Borders, grid lines, structural elements.
   * - **ROYAL_BLUE**
     - ``#4169E1``
     - Labor Aristocracy class indicator.

Architecture
------------

The UI is decoupled from the simulation engine via protocols. GUI code depends
**only** on ``SimulationState`` and ``SimulationControl`` protocols, never on
the ``Simulation`` implementation. This enables:

- Type-safe interfaces (mypy validates protocol usage)
- Mock implementations for testing
- Engine internals can evolve without breaking GUI

Protocol-Based Design
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   from babylon.protocols import SimulationState, SimulationControl

   def render_map(sim: SimulationState) -> None:
       """Render territories using read-only protocol."""
       snapshot = sim.get_snapshot()
       for tid, territory in snapshot.territories.items():
           color = profit_rate_to_color(territory.profit_rate)
           render_hexes(territory.hex_claims, color)

   def on_step_click(sim: SimulationControl) -> None:
       """Handle step button using control protocol."""
       sim.step()

The UI acts as a passive observer and controller, running in the main thread
while the simulation can tick independently.

Entry Point
~~~~~~~~~~~

The dashboard is launched via the ``babylon.ui.dashboard`` module:

.. code-block:: bash

   uv run python -m babylon.ui.dashboard

Or via mise:

.. code-block:: bash

   mise run ui

Core Components
~~~~~~~~~~~~~~~

1. **Narrative Feed**: A scrolling log window displaying semantic events from the
   ``NarrativeDirector``.
2. **Telemetry Plots**: Time-series ECharts graphs tracking key contradictions (e.g.,
   Imperial Rent vs. Stability).
3. **Control Panel**: Play/Pause/Step controls for the simulation loop.
4. **Topology Monitor**: (Planned) Network visualization of class/territory relations.

Development Standards
---------------------

1. **PyQt6 Best Practices**: Use signals and slots for communication between components.
   Avoid blocking the main thread with heavy CPU computations.
2. **Type Safety**: All UI code must be strictly typed.
3. **Design System**: Do not hardcode colors. Always import from ``babylon.ui.design_system``.

.. code-block:: python

   from babylon.ui.design_system import BunkerPalette

   # GOOD
   label.setStyleSheet(f"color: {BunkerPalette.PHOSPHOR_BURN_RED};")

   # BAD
   label.setStyleSheet("color: #FF0000;")

Reference
---------

- :doc:`/reference/simulation-protocols` - Protocol and snapshot API reference
- :py:mod:`babylon.protocols` - SimulationState and SimulationControl protocols
- :py:mod:`babylon.models.snapshots` - TerritoryState, HexState, SimulationSnapshot
- :py:mod:`babylon.ui.dashboard` - Main dashboard module
- :py:mod:`babylon.ui.design_system` - Color constants and style tokens
- `PyQt6 Documentation <https://www.riverbankcomputing.com/static/Docs/PyQt6/>`_
