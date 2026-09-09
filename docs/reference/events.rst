Frozen Python Event System Reference
====================================

This reference documents the frozen Python ``EventBus`` and Pydantic event-model
surface retained as historical behavioral evidence. It does not define live
Rust/BSL authored payloads, ``TickPayloadV2``, or committed-event provenance.
The canonical live BSL emit registry is
``docs/reference/event-schema-registry.toml``. For the current conceptual
boundary, see :doc:`/concepts/event-system`.

Overview
--------

The frozen Python simulation emits typed Pydantic events during each tick. The
:class:`EventBus` captures the events and converts them to typed models. The
simulation persists them in :attr:`WorldState.events`. This enables:

- AI narrative generation from structured data
- Event-driven analysis and visualization
- Replay and counterfactual scenarios

**Key Components:**

- :py:mod:`babylon.models.events` - Pydantic event models
- :py:mod:`babylon.models.enums` - EventType enum
- :py:mod:`babylon.kernel.event_bus` - Pub/sub event bus
- :py:mod:`babylon.engine.simulation_engine` - Event conversion

EventType Enum
--------------

The :class:`EventType` enum categorizes all frozen Python events by type. It
defines the vocabulary below by category:

**Economic Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``SURPLUS_EXTRACTION``
     - Imperial rent extracted from worker via EXPLOITATION edge
   * - ``IMPERIAL_SUBSIDY``
     - Subsidy paid to comprador state (converts to repression)
   * - ``ECONOMIC_CRISIS``
     - Pool depleted below critical threshold, triggers crisis response
   * - ``SUPERWAGE_CRISIS``
     - Core cannot afford super-wages (triggers LA decomposition)
   * - ``PERIPHERAL_REVOLT``
     - Periphery severs EXPLOITATION edges when P(S|R) > P(S|A)

**Consciousness Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``CONSCIOUSNESS_TRANSMISSION``
     - Consciousness flows via SOLIDARITY edge
   * - ``MASS_AWAKENING``
     - Class consciousness crosses threshold
   * - ``SOLIDARITY_AWAKENING``
     - Entity enters active struggle (deprecated, use MASS_AWAKENING)

**Struggle Events (George Floyd Dynamic):**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``EXCESSIVE_FORCE``
     - State violence spark event (stochastic police brutality)
   * - ``UPRISING``
     - Mass insurrection triggered by spark + agitation
   * - ``SOLIDARITY_SPIKE``
     - Solidarity infrastructure built through shared struggle
   * - ``POWER_VACUUM``
     - Comprador insolvency triggers George Jackson bifurcation
   * - ``REVOLUTIONARY_OFFENSIVE``
     - Organized labor seizes opportunity during power vacuum
   * - ``FASCIST_REVANCHISM``
     - Core workers react with nationalism during power vacuum

**Vitality Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``ENTITY_DEATH``
     - Entity starved (wealth < consumption_needs)
   * - ``POPULATION_DEATH``
     - Probabilistic mortality from inequality
   * - ``POPULATION_ATTRITION``
     - Grinding Attrition deaths from coverage deficit

**Terminal Crisis Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``CLASS_DECOMPOSITION``
     - Labor aristocracy splits into enforcers + internal proletariat
   * - ``CONTROL_RATIO_CRISIS``
     - Prisoners exceed guard capacity (ratio inverted)
   * - ``TERMINAL_DECISION``
     - System bifurcates to revolution or genocide

**Topology Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``RUPTURE``
     - Contradiction tension reached 1.0
   * - ``PHASE_TRANSITION``
     - Percolation threshold crossed (gaseous/transitional/liquid/solid)

**Metabolism Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``ECOLOGICAL_OVERSHOOT``
     - Consumption exceeds biocapacity (O > 1.0)

**Endgame Events:**

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - EventType
     - Description
   * - ``ENDGAME_REACHED``
     - Game-ending condition met (victory, collapse, or fascism)

Event Class Hierarchy
---------------------

Events form a type hierarchy with shared base classes:

.. mermaid::

   classDiagram
       SimulationEvent <|-- EconomicEvent
       SimulationEvent <|-- SuperwageCrisisEvent
       SimulationEvent <|-- ClassDecompositionEvent
       SimulationEvent <|-- ControlRatioCrisisEvent
       SimulationEvent <|-- TerminalDecisionEvent
       SimulationEvent <|-- ConsciousnessEvent
       SimulationEvent <|-- StruggleEvent
       SimulationEvent <|-- ContradictionEvent
       SimulationEvent <|-- TopologyEvent
       SimulationEvent <|-- EndgameEvent

       EconomicEvent <|-- ExtractionEvent
       EconomicEvent <|-- SubsidyEvent
       EconomicEvent <|-- CrisisEvent

       ConsciousnessEvent <|-- TransmissionEvent
       ConsciousnessEvent <|-- MassAwakeningEvent

       StruggleEvent <|-- SparkEvent
       StruggleEvent <|-- UprisingEvent
       StruggleEvent <|-- SolidaritySpikeEvent

       ContradictionEvent <|-- RuptureEvent

       TopologyEvent <|-- PhaseTransitionEvent

       class SimulationEvent {
           <<frozen>>
           +tick: int
           +timestamp: datetime
       }
       class EconomicEvent {
           +amount: Currency
       }
       class ConsciousnessEvent {
           +target_id: str
       }
       class StruggleEvent {
           +node_id: str
       }
       class ContradictionEvent {
           +edge: str
       }
       class TopologyEvent {
           +percolation_ratio: float
           +num_components: int
       }

Base Event Model
----------------

SimulationEvent
~~~~~~~~~~~~~~~

All events inherit from :class:`SimulationEvent`:

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Discriminator for event category
   * - ``tick``
     - ``int``
     - Simulation tick when event occurred
   * - ``timestamp``
     - ``datetime | None``
     - Optional wall-clock time

**Model Configuration:**

- ``frozen=True``: Events are immutable after creation
- ``extra="forbid"``: No additional fields allowed

Economic Events
---------------

Events related to value extraction and imperial rent dynamics.

EconomicEvent (Base)
~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``amount``
     - ``Currency``
     - Value transferred in this event

ExtractionEvent
~~~~~~~~~~~~~~~

Emitted when imperial rent is extracted via EXPLOITATION edge.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``SURPLUS_EXTRACTION``
   * - ``source_id``
     - ``str``
     - Worker node ID (extracted from)
   * - ``target_id``
     - ``str``
     - Owner node ID (extracted to)
   * - ``amount``
     - ``Currency``
     - Amount of rent extracted
   * - ``mechanism``
     - ``str``
     - Extraction mechanism (default: "imperial_rent")

SubsidyEvent
~~~~~~~~~~~~

Emitted when imperial subsidy is paid via CLIENT_STATE edge.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``IMPERIAL_SUBSIDY``
   * - ``source_id``
     - ``str``
     - Core bourgeoisie node ID
   * - ``target_id``
     - ``str``
     - Comprador state node ID
   * - ``amount``
     - ``Currency``
     - Subsidy amount
   * - ``repression_boost``
     - ``float``
     - Increase to target's repression capacity

CrisisEvent
~~~~~~~~~~~

Emitted when economic crisis conditions are detected.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``ECONOMIC_CRISIS``
   * - ``pool_ratio``
     - ``float``
     - Current imperial rent pool ratio
   * - ``aggregate_tension``
     - ``float``
     - Total system tension
   * - ``decision``
     - ``str``
     - Bourgeoisie response (e.g., "AUSTERITY")
   * - ``wage_delta``
     - ``float``
     - Change in wage rates

Terminal Crisis Events
----------------------

Events from the Carceral Equilibrium (Terminal Crisis Dynamics).

SuperwageCrisisEvent
~~~~~~~~~~~~~~~~~~~~

Emitted when the imperial rent pool cannot pay super-wages.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``SUPERWAGE_CRISIS``
   * - ``payer_id``
     - ``str``
     - Core bourgeoisie who can't pay
   * - ``receiver_id``
     - ``str``
     - Labor aristocracy not receiving wages
   * - ``desired_wages``
     - ``float``
     - Amount of wages that were needed
   * - ``available_pool``
     - ``float``
     - Amount available (zero or negative)

ClassDecompositionEvent
~~~~~~~~~~~~~~~~~~~~~~~

Emitted when Labor Aristocracy splits into two fractions.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``CLASS_DECOMPOSITION``
   * - ``original_id``
     - ``str``
     - Labor aristocracy entity that split
   * - ``enforcer_fraction``
     - ``float``
     - Fraction that became enforcers (default 0.15)
   * - ``proletariat_fraction``
     - ``float``
     - Fraction that became internal proletariat (0.85)

ControlRatioCrisisEvent
~~~~~~~~~~~~~~~~~~~~~~~

Emitted when prisoners exceed guard control capacity.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``CONTROL_RATIO_CRISIS``
   * - ``prisoner_population``
     - ``int``
     - Size of prisoner/surplus population
   * - ``enforcer_population``
     - ``int``
     - Size of enforcer/guard population
   * - ``control_ratio``
     - ``float``
     - Prisoners per enforcer
   * - ``capacity_threshold``
     - ``float``
     - Maximum ratio enforcers can handle

TerminalDecisionEvent
~~~~~~~~~~~~~~~~~~~~~

Emitted when system bifurcates to final outcome.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``TERMINAL_DECISION``
   * - ``outcome``
     - ``str``
     - Either "revolution" or "genocide"
   * - ``avg_organization``
     - ``float``
     - Average organization level of prisoners
   * - ``revolution_threshold``
     - ``float``
     - Threshold above which revolution occurs

Consciousness Events
--------------------

Events related to ideology drift and consciousness transmission.

ConsciousnessEvent (Base)
~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``target_id``
     - ``str``
     - Node whose consciousness is affected

TransmissionEvent
~~~~~~~~~~~~~~~~~

Emitted when consciousness is transmitted via SOLIDARITY edge.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``CONSCIOUSNESS_TRANSMISSION``
   * - ``source_id``
     - ``str``
     - Transmitting node ID
   * - ``target_id``
     - ``str``
     - Receiving node ID
   * - ``delta``
     - ``float``
     - Consciousness change magnitude
   * - ``solidarity_strength``
     - ``float``
     - Edge strength facilitating transmission

MassAwakeningEvent
~~~~~~~~~~~~~~~~~~

Emitted when a node's consciousness crosses the awakening threshold.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``MASS_AWAKENING``
   * - ``target_id``
     - ``str``
     - Awakened node ID
   * - ``old_consciousness``
     - ``float``
     - Consciousness before awakening
   * - ``new_consciousness``
     - ``float``
     - Consciousness after awakening
   * - ``triggering_source``
     - ``str``
     - Node that triggered awakening

Struggle Events
---------------

Events related to the Agency Layer (George Floyd Dynamic).

StruggleEvent (Base)
~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``node_id``
     - ``str``
     - Node where struggle occurs

SparkEvent (frozen Python)
~~~~~~~~~~~~~~~~~~~~~~~~~~

The frozen Python reference emits ``SparkEvent`` when an ``EXCESSIVE_FORCE``
event occurs. Its ``spark_probability`` field records the probability used by
that historical producer.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``EXCESSIVE_FORCE``
   * - ``node_id``
     - ``str``
     - Target of state violence
   * - ``repression``
     - ``float``
     - Repression level at node
   * - ``spark_probability``
     - ``float``
     - Probability recorded by the frozen Python producer

Live AJ BSL ``EXCESSIVE_FORCE`` payload
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The live Rust path does not construct ``SparkEvent`` and does not author event
probability. Under Amendment AJ, the adjacent ``struggle/spark-recognizer``
emits the following payload from
``rust/crates/babylon-tick/content/rules/struggle-spark.bsl``. The canonical
schema row is in ``docs/reference/event-schema-registry.toml``.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - BSL type
     - Description
   * - ``subject``
     - ``NodeRef``
     - Social-class carrier observed by the recognizer
   * - ``repression``
     - ``Intensity``
     - Repression faced by the carrier
   * - ``backfire``
     - ``Intensity``
     - Post-state agitation backfire
   * - ``incident-tick``
     - ``Int``
     - Tick recorded by the selected material outcome

The live BSL schema requires all four fields. ``node-id`` and
``spark-probability`` remain documented in the registry only as optional
``builder-only`` evidence for the frozen Python builder. They are absent from
the live authored payload. ``emitting_rule`` and the automatically derived
``choice_receipt_ref`` belong to committed-event metadata outside that payload.

UprisingEvent
~~~~~~~~~~~~~

Emitted when conditions trigger revolt.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``UPRISING``
   * - ``node_id``
     - ``str``
     - Node where uprising occurs
   * - ``trigger``
     - ``str``
     - What triggered uprising (e.g., "spark", "p_rev")
   * - ``agitation``
     - ``float``
     - Agitation level at trigger
   * - ``repression``
     - ``float``
     - Repression level at trigger

SolidaritySpikeEvent
~~~~~~~~~~~~~~~~~~~~

Emitted when solidarity infrastructure is built during uprising.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``SOLIDARITY_SPIKE``
   * - ``node_id``
     - ``str``
     - Node gaining solidarity
   * - ``solidarity_gained``
     - ``float``
     - Amount of solidarity added
   * - ``edges_affected``
     - ``int``
     - Number of edges strengthened
   * - ``triggered_by``
     - ``str``
     - Cause (e.g., "uprising")

Vitality Events
---------------

Events from the VitalitySystem (mortality and extinction).

.. note::

   These events currently have EventType enum values but no dedicated
   Pydantic model classes. They are emitted as raw Event objects.

**ENTITY_DEATH:**

Emitted when an entity is fully extinct (population = 0 or starvation).

Payload fields: ``entity_id``, ``wealth``, ``consumption_needs``, ``s_bio``,
``s_class``, ``cause`` ("extinction", "starvation", or "wealth_threshold")

**POPULATION_ATTRITION:**

Emitted when deaths occur from coverage deficit (Grinding Attrition phase).

Payload fields: ``entity_id``, ``deaths``, ``remaining_population``, ``attrition_rate``

**POPULATION_DEATH:**

Emitted for probabilistic mortality from inequality (deprecated, use POPULATION_ATTRITION).

Contradiction Events
--------------------

Events related to tension accumulation and rupture.

ContradictionEvent (Base)
~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``edge``
     - ``str``
     - Edge identifier (format: "source->target")

RuptureEvent
~~~~~~~~~~~~

Emitted when tension on an edge reaches 1.0.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``RUPTURE``
   * - ``edge``
     - ``str``
     - Edge that ruptured

Topology Events
---------------

Events related to network structure and phase transitions.

TopologyEvent (Base)
~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``percolation_ratio``
     - ``Probability``
     - L_max / total_nodes
   * - ``num_components``
     - ``int``
     - Number of disconnected components

PhaseTransitionEvent
~~~~~~~~~~~~~~~~~~~~

Emitted when percolation ratio crosses a threshold boundary.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``PHASE_TRANSITION``
   * - ``previous_state``
     - ``str``
     - Previous phase ("gaseous", "transitional", "liquid", "solid")
   * - ``new_state``
     - ``str``
     - New phase after transition
   * - ``largest_component_size``
     - ``int``
     - Size of giant component (L_max)
   * - ``cadre_density``
     - ``float``
     - Ratio of cadre to sympathizers [0, 1]
   * - ``is_resilient``
     - ``bool | None``
     - Resilience test result (if available)

**4-Phase State Model:**

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - State
     - Threshold
     - Meaning
   * - Gaseous
     - ``ratio < 0.1``
     - Atomized, no coordination capacity
   * - Transitional
     - ``0.1 <= ratio < 0.5``
     - Emerging structure, unstable
   * - Liquid
     - ``ratio >= 0.5, cadre < 0.5``
     - Mass movement (weak ties)
   * - Solid
     - ``ratio >= 0.5, cadre >= 0.5``
     - Vanguard party (strong ties)

Metabolism Events
-----------------

Events from the MetabolismSystem (ecological overshoot).

**ECOLOGICAL_OVERSHOOT:**

Emitted when consumption exceeds biocapacity (overshoot ratio > 1.0).

Payload fields: ``overshoot_ratio``, ``total_consumption``, ``total_biocapacity``

Endgame Events
--------------

Events signaling game-ending conditions.

EndgameEvent
~~~~~~~~~~~~

Emitted when a game-ending condition is met.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``event_type``
     - ``EventType``
     - Always ``ENDGAME_REACHED``
   * - ``outcome``
     - ``GameOutcome``
     - The outcome that ended the simulation

**GameOutcome Values:**

- ``REVOLUTIONARY_VICTORY``: Proletarian revolution succeeded
- ``ECOLOGICAL_COLLAPSE``: Metabolic rift has become fatal
- ``FASCIST_CONSOLIDATION``: Fascism has consolidated power
- ``IN_PROGRESS``: Simulation still running (not an endgame)

Event Lifecycle
---------------

Events flow through the system in this sequence:

1. **Emission**: Systems call ``event_bus.publish(type, tick, payload)``
2. **Collection**: ``EventBus.get_history()`` returns all events for tick
3. **Conversion**: ``_convert_bus_event_to_pydantic()`` creates typed models
4. **Persistence**: Events appended to ``WorldState.events``
5. **Observation**: AI/narrative layer reads typed events

**Observer Events (Sprint 3.3):**

Observers like :class:`TopologyMonitor` emit events after the tick completes.
These are collected via ``get_pending_events()`` and injected into the
*next* tick's ``WorldState.events`` via ``persistent_context['_observer_events']``.

Factory Methods
---------------

The :class:`DomainFactory` provides factory methods for creating events in tests:

.. code-block:: python

   from tests.factories.domain import DomainFactory

   factory = DomainFactory()

   # Create events
   extraction = factory.create_extraction_event(tick=1, amount=0.1)
   subsidy = factory.create_subsidy_event(tick=1, amount=0.05)
   transmission = factory.create_transmission_event(tick=1, delta=0.1)
   spark = factory.create_spark_event(tick=1, repression=0.8)
   uprising = factory.create_uprising_event(tick=1, agitation=0.7)
   phase = factory.create_phase_transition_event(
       tick=1,
       previous_state="gaseous",
       new_state="liquid",
   )

Assertions
----------

The :class:`BabylonAssert` class provides semantic assertions for events:

.. code-block:: python

   from tests.assertions import Assert
   from babylon.models.events import ExtractionEvent

   # Assert event exists
   Assert(world_state).has_event(ExtractionEvent)

   # Assert event with specific fields
   Assert(world_state).has_event(
       ExtractionEvent,
       source_id="C001",
       target_id="C002",
   )

   # Assert event count
   Assert(world_state).has_events_count(3)

   # Assert amount condition
   Assert(world_state).has_event(ExtractionEvent, amount_gt=0.0)

See Also
--------

- ``docs/reference/event-schema-registry.toml`` - Canonical live BSL emit
  payload registry
- :doc:`/concepts/event-system` - Current event architecture
- :doc:`/reference/topology` - TopologyMonitor and phase transitions
- :doc:`/reference/systems` - Systems that emit events
- :py:mod:`babylon.kernel.event_bus` - Frozen Python event bus implementation
