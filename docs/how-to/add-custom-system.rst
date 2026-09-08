Add a Custom System
===================

This guide walks you through creating, registering, and testing custom
simulation systems to extend Babylon's mechanics.

Prerequisites
-------------

- Familiarity with :doc:`/concepts/simulation-systems`
- Understanding of NetworkX graph operations
- Python class implementation

Create a System Class
---------------------

Systems implement the ``System`` protocol. Here's a complete example—a
``PropagandaSystem`` that pushes ideology toward apolitical (0):

.. code-block:: python

   from babylon.kernel.system_protocol import System


   class PropagandaSystem:
       """System for ideological propaganda effects.

       Propaganda reduces class consciousness by pushing ideology
       toward 0 (apolitical center).
       """

       def __init__(self, effectiveness: float = 0.1):
           self.effectiveness = effectiveness

       def step(self, graph, services, context):
           for node_id, data in graph.nodes(data=True):
               if data.get("_node_type") != "social_class":
                   continue

               # Propaganda pushes ideology toward 0 (apolitical)
               current = data["ideology"]
               drift = -current * self.effectiveness
               graph.nodes[node_id]["ideology"] = current + drift

**Key requirements:**

1. Implement ``step(graph, services, context)`` method
2. Only mutate the ``graph`` parameter
3. Do not store state between ticks (stateless design)

Register Your System
--------------------

Systems are passed to the engine at construction time via constructor injection.
There is no runtime registration—the system list is immutable after creation.

.. code-block:: python

   from babylon.engine.simulation_engine import SimulationEngine, _DEFAULT_SYSTEMS
   from babylon.engine.systems import PropagandaSystem

   # Option 1: Start with the default system list
   # (recommended - see simulation_engine.py for canonical order)
   custom_systems = list(_DEFAULT_SYSTEMS)
   # Insert your system at the appropriate position
   custom_systems.insert(9, PropagandaSystem(effectiveness=0.05))  # Before SurvivalSystem

   # Option 2: Build from scratch (not recommended unless you know what you're doing)
   # from babylon.engine.systems import (
   #     VitalitySystem,
   #     TerritorySystem,
   #     ProductionSystem,
   #     SolidaritySystem,
   #     ImperialRentSystem,
   #     DecompositionSystem,
   #     ControlRatioSystem,
   #     MetabolismSystem,
   #     SurvivalSystem,
   #     StruggleSystem,
   #     ConsciousnessSystem,
   #     ContradictionSystem,
   # )
   # custom_systems = [
   #     VitalitySystem(),
   #     TerritorySystem(),
   #     ProductionSystem(),
   #     SolidaritySystem(),
   #     ImperialRentSystem(),
   #     DecompositionSystem(),
   #     ControlRatioSystem(),
   #     MetabolismSystem(),
   #     PropagandaSystem(effectiveness=0.05),  # Custom system
   #     SurvivalSystem(),
   #     StruggleSystem(),
   #     ConsciousnessSystem(),
   #     ContradictionSystem(),
   # ]

   # Pass systems at construction
   engine = SimulationEngine(systems=custom_systems)

**Order matters!** The system execution order encodes materialist causality:
economic base (production, extraction) → superstructure (consciousness, ideology).
See ``_DEFAULT_SYSTEMS`` in ``simulation_engine.py`` for the canonical ordering.

Use the Formula Registry
------------------------

For calculations that might need to be swapped or tested, use the
``FormulaRegistry`` from services:

.. code-block:: python

   class SurvivalSystem:
       def step(self, graph, services, context):
           formulas = services.formulas  # FormulaRegistry

           for node_id, data in graph.nodes(data=True):
               # Use registered formula (allows hot-swapping)
               P_S_A = formulas.calculate_acquiescence_probability(
                   wealth=data["wealth"],
                   subsistence=services.defines.survival.default_subsistence
               )
               graph.nodes[node_id]["P_acquiescence"] = P_S_A

**Benefits:**

- Testing with mock formulas
- Runtime formula changes for experimentation
- Consistent formula usage across systems

Emit Events
-----------

Systems can emit events for observers via the EventBus:

.. code-block:: python

   from babylon.engine import Event


   class RuptureEvent(Event):
       class_id: str
       tension_level: float


   class ContradictionSystem:
       def step(self, graph, services, context):
           for node_id, data in graph.nodes(data=True):
               if data["tension"] > rupture_threshold:
                   services.event_bus.emit(
                       RuptureEvent(
                           class_id=node_id,
                           tension_level=data["tension"]
                       )
                   )

Events enable loose coupling between systems and observers (like the
TopologyMonitor or narrative generators).

Test Your System
----------------

Systems are designed for isolated testing:

.. code-block:: python

   import pytest
   import networkx as nx
   from babylon.engine import ServiceContainer, EventBus
   from babylon.engine.formula_registry import FormulaRegistry


   @pytest.fixture
   def minimal_graph():
       G = nx.DiGraph()
       G.add_node(
           "C001",
           _node_type="social_class",
           wealth=100,
           organization=0.5,
           ideology=0.0
       )
       return G


   @pytest.fixture
   def services():
       from babylon.models import SimulationConfig
       from babylon.config.defines import GameDefines
       from babylon.persistence.database import DatabaseConnection

       return ServiceContainer(
           config=SimulationConfig(),
           database=DatabaseConnection(":memory:"),
           event_bus=EventBus(),
           formulas=FormulaRegistry(),
           defines=GameDefines(),
       )


   def test_propaganda_reduces_consciousness(minimal_graph, services):
       # Arrange
       minimal_graph.nodes["C001"]["ideology"] = 0.8
       system = PropagandaSystem(effectiveness=0.1)
       context = {"tick": 0}  # Context is a simple dict

       # Act
       system.step(minimal_graph, services, context)

       # Assert
       new_ideology = minimal_graph.nodes["C001"]["ideology"]
       assert new_ideology < 0.8  # Ideology moved toward 0
       assert new_ideology == pytest.approx(0.72)  # 0.8 - (0.8 * 0.1)

**Testing tips:**

- Create minimal graphs with only necessary nodes
- Test one behavior per test function
- Use ``pytest.approx()`` for floating-point comparisons

Debug Your System
-----------------

Add logging to trace system execution:

.. code-block:: python

   import logging


   class ConsciousnessSystem:
       def __init__(self):
           self.logger = logging.getLogger(__name__)

       def step(self, graph, services, context):
           self.logger.debug(f"Tick {context['tick']}: Processing consciousness")

           for node_id, data in graph.nodes(data=True):
               old_ideology = data["ideology"]
               # ... calculation ...
               new_ideology = data["ideology"]
               self.logger.debug(
                   f"  {node_id}: ideology {old_ideology:.2f} -> {new_ideology:.2f}"
               )

Enable debug logging:

.. code-block:: python

   import logging

   logging.getLogger("babylon.engine.systems").setLevel(logging.DEBUG)

Or via environment variable:

.. code-block:: bash

   export BABYLON_LOG_LEVEL=DEBUG

See Also
--------

- :doc:`/concepts/simulation-systems` - Why systems work this way
- :doc:`/reference/systems` - API reference for built-in systems
- :doc:`/reference/configuration` - Configure authored campaign parameters
- :py:mod:`babylon.engine.systems` - System implementations
