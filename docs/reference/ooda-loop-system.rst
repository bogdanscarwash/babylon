OODA Loop System Reference
==========================

Technical reference for the OODA Loop System (Feature 032).
Covers data types, turn resolution functions, action eligibility,
costs, consciousness effects, constraints, lifecycle capacity,
Layer 0 metabolism, Layer 3 propagation, NPC action selection,
and the OODASystem orchestrator.

**Import:**

.. code-block:: python

   from babylon.ooda import (
       # Data types
       Action,
       ActionCostModifier,
       ActionResult,
       InitiativeScore,
       OODAProfile,
       TurnResolution,
       # Cycle time and initiative
       compute_cycle_time,
       compute_initiative_score,
       resolve_action_order,
       compute_community_embeddedness,
       update_momentum,
       # Action eligibility
       ELIGIBILITY_MAP,
       check_eligibility,
       # Action costs
       compute_action_cost,
       # Action effects
       compute_consciousness_delta,
       resolve_action,
       # Constraints
       enforce_action_points,
       enforce_coordination_range,
       apply_autonomy_modifier,
       # Lifecycle
       compute_lifecycle_modifier,
       elder_legitimacy_bonus,
       # Layers
       process_layer0,
       process_layer3,
       # NPC
       select_npc_actions,
   )

.. contents:: On this page
   :local:
   :depth: 2


Data Types
----------

All models are frozen (immutable) Pydantic BaseModels from
``src/babylon/ooda/types.py``. These types flow through the turn
resolution pipeline and are discarded after Layer 3 propagation.

OODAProfile
~~~~~~~~~~~

Stored on organization graph nodes. Determines cycle time, action budget,
and coordination limits.

.. list-table::
   :header-rows: 1
   :widths: 25 15 10 50

   * - Field
     - Type
     - Default
     - Description
   * - ``sensor_latency``
     - ``int``
     - 1
     - Ticks of observation delay [0, 10].
   * - ``ideological_coherence``
     - ``float``
     - 0.5
     - How unified the org's worldview is [0, 1].
   * - ``analytical_capacity``
     - ``float``
     - 0.5
     - Ability to process information [0, 1].
   * - ``decision_mode``
     - ``DecisionMode``
     - DEMOCRATIC
     - How decisions are made (affects Decide phase time).
   * - ``bureaucratic_depth``
     - ``float``
     - 0.3
     - Layers of bureaucracy [0, 1].
   * - ``action_points``
     - ``int``
     - 3
     - Actions available per tick [0, 20].
   * - ``coordination_range``
     - ``int``
     - 1
     - Distinct territories targetable per tick [0, 100].
   * - ``autonomy``
     - ``float``
     - 0.5
     - Effectiveness-breadth tradeoff [0, 1].

Action
~~~~~~

A single organizational action proposed for one tick.

.. list-table::
   :header-rows: 1
   :widths: 25 15 10 50

   * - Field
     - Type
     - Default
     - Description
   * - ``org_id``
     - ``str``
     -
     - Acting organization ID (min length 1).
   * - ``action_type``
     - ``ActionType``
     -
     - What action to perform (21 values).
   * - ``target_id``
     - ``str``
     -
     - Target community, organization, or territory ID.
   * - ``action_point_cost``
     - ``int``
     - 1
     - AP cost after modifiers (minimum 1).
   * - ``cadre_labor_cost``
     - ``float``
     - 0.0
     - Forward-compatible: cadre hours required.
   * - ``sympathizer_labor_cost``
     - ``float``
     - 0.0
     - Forward-compatible: sympathizer hours.
   * - ``budget_cost``
     - ``float``
     - 0.0
     - Forward-compatible: monetary cost.

ActionResult
~~~~~~~~~~~~

Outcome of executing one action.

.. list-table::
   :header-rows: 1
   :widths: 25 20 10 45

   * - Field
     - Type
     - Default
     - Description
   * - ``action``
     - ``Action``
     -
     - The action that was executed.
   * - ``success``
     - ``bool``
     -
     - Whether the action succeeded.
   * - ``consciousness_delta``
     - ``ConsciousnessDelta | None``
     - ``None``
     - Consciousness effect on target community.
   * - ``direct_effects``
     - ``dict[str, Any]``
     - ``{}``
     - Action-type-specific effects (e.g., ``contestation_delta``).
   * - ``events_generated``
     - ``list[str]``
     - ``[]``
     - EventType values emitted.
   * - ``failure_reason``
     - ``str | None``
     - ``None``
     - Why the action failed (``None`` if success).

InitiativeScore
~~~~~~~~~~~~~~~

Computed per-tick ordering value.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Field
     - Type
     - Description
   * - ``org_id``
     - ``str``
     - Organization ID.
   * - ``score``
     - ``float``
     - Composite initiative score.
   * - ``speed_component``
     - ``float``
     - Contribution from OODA cycle time.
   * - ``institutional_component``
     - ``float``
     - Institutional bonus (state advantage).
   * - ``counterintel_component``
     - ``float``
     - Counter-intelligence capability.
   * - ``embeddedness_component``
     - ``float``
     - Community embeddedness.
   * - ``momentum_component``
     - ``float``
     - Recent success momentum.

**Invariant:** ``score = speed + institutional + counterintel + embeddedness + momentum``.

ActionCostModifier
~~~~~~~~~~~~~~~~~~

Cost adjustment for an action based on org-community relationship.

.. list-table::
   :header-rows: 1
   :widths: 20 15 65

   * - Field
     - Type
     - Description
   * - ``base_cost``
     - ``int``
     - Action type's default AP cost (>= 1).
   * - ``modifier``
     - ``float``
     - Multiplier (< 1.0 = discount, > 1.0 = surcharge).
   * - ``effective_cost``
     - ``int``
     - ``ceil(base_cost * modifier)``, minimum 1.
   * - ``reason``
     - ``str``
     - Human-readable explanation.

TurnResolution
~~~~~~~~~~~~~~

Complete processing of one tick's OODA resolution.

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Field
     - Type
     - Description
   * - ``tick``
     - ``int``
     - Which tick was resolved (>= 0).
   * - ``layer0_results``
     - ``list[ActionResult]``
     - Automatic metabolism results.
   * - ``initiative_order``
     - ``list[InitiativeScore]``
     - Sorted initiative scores (descending).
   * - ``action_phase_results``
     - ``list[ActionResult]``
     - All action results in execution order.
   * - ``layer3_effects``
     - ``dict[str, Any]``
     - Aggregated consequence propagation summary.


Enums
-----

DecisionMode
~~~~~~~~~~~~

How an organization makes decisions. Affects cycle time Decide phase.

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Value
     - Description
   * - ``AUTOCRATIC``
     - Single decision-maker. Fastest (base 1.0).
   * - ``DELEGATE``
     - Leader delegates to trusted subordinates (base 2.0).
   * - ``DEMOCRATIC``
     - Majority vote among members (base 3.0).
   * - ``CONSENSUS``
     - Full agreement required. Slowest (base 5.0).

ActionType
~~~~~~~~~~

21 action types available to organizations.

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Value
     - AP Cost
     - Description
   * - ``EDUCATE``
     - 1
     - Political education. Highest CI multiplier (1.2).
   * - ``AGITATE``
     - 1
     - Raise contestation. No direct CI effect.
   * - ``FUNDRAISE``
     - 1
     - Generate organizational resources.
   * - ``EMPLOY``
     - 1
     - Business auto-metabolism (Layer 0).
   * - ``SURVEIL``
     - 1
     - State surveillance. Low backfire (0.2).
   * - ``MAP_NETWORK``
     - 1
     - Intelligence mapping of social networks.
   * - ``PROPOSE_ALLIANCE``
     - 1
     - Diplomatic overture to another organization.
   * - ``DENOUNCE``
     - 1
     - Public condemnation.
   * - ``RECRUIT``
     - 2
     - Grow organizational membership.
   * - ``ORGANIZE``
     - 2
     - Build community structures. Triggers edge transition.
   * - ``PROPAGANDIZE``
     - 2
     - Media and messaging. High CI multiplier (0.8).
   * - ``PROVIDE_SERVICE``
     - 2
     - Serve-the-people programs. Tendency-split CI effect.
   * - ``REPRESS``
     - 2
     - State violence. High backfire (0.8).
   * - ``PROTEST``
     - 2
     - Public demonstration.
   * - ``COUNTER_INTEL``
     - 2
     - Counter-intelligence operations.
   * - ``ATTACK_INFRASTRUCTURE``
     - 2
     - Sabotage community infrastructure.
   * - ``ASSIMILATE``
     - 2
     - Institutional co-optation. Negative CI, LIBERAL tendency.
   * - ``STRIKE``
     - 3
     - Labor withdrawal.
   * - ``EXPROPRIATE``
     - 3
     - Seize property or resources.
   * - ``INFILTRATE``
     - 3
     - Intelligence infiltration of target organization.
   * - ``BUILD_INFRASTRUCTURE``
     - 3
     - Build community infrastructure.

Edge Types (Feature 032)
~~~~~~~~~~~~~~~~~~~~~~~~

Two edge types added to ``EdgeType`` for org-community relationships:

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Type
     - Direction
     - Description
   * - ``TRANSACTIONAL``
     - Organization -> Community
     - Service-for-support exchange. Instrumental relationship.
   * - ``SOLIDARISTIC``
     - Organization -> Community
     - Deep mutual commitment. Persists through hardship.


Cycle Time
----------

.. code-block:: python

   def compute_cycle_time(profile: OODAProfile, defines: OODADefines) -> float: ...

Compute total OODA cycle time from a profile. Four-phase additive model:

.. math::

   T_{cycle} = T_{observe} + T_{orient} + T_{decide} + T_{act}

See :doc:`/reference/ooda-coefficients` for coefficient details.

**Source:** ``src/babylon/ooda/cycle_time.py:17``


Initiative Scoring
------------------

compute_initiative_score
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def compute_initiative_score(
       org_id: str,
       cycle_time: float,
       jurisdiction: JurisdictionLevel | None,
       counter_intel_score: float,
       community_embeddedness: float,
       momentum: float,
       defines: OODADefines,
   ) -> InitiativeScore: ...

Compute initiative score for one organization. Five-component weighted sum.

**Source:** ``src/babylon/ooda/initiative.py:22``

resolve_action_order
~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def resolve_action_order(scores: list[InitiativeScore]) -> list[InitiativeScore]: ...

Sort organizations by initiative score (descending). Tiebreak by ``org_id``
(ascending, alphabetical).

**Source:** ``src/babylon/ooda/initiative.py:70``

compute_community_embeddedness
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def compute_community_embeddedness(org_id: str, graph: nx.DiGraph[str]) -> float: ...

Compute how embedded an organization is in its territories' communities.
Walks the org's ``territory_ids`` -> the ``social_class`` nodes TENANCY-linked
into each territory (the real Occupant -> Territory edge) -> each reachable
member's ``community_memberships`` list. Embeddedness is the share of
TENANCY-reachable members carrying at least one community membership;
bounded to [0, 1] by construction, 0.0 if the org has no territories or no
member is reachable.

This value is structurally 0.0 in every real game today: no production
writer populates ``SocialClass.community_memberships``
(``CommunitySystem.step`` no-ops every tick — seam registry
``src/babylon/sentinels/seam/registry.py:1969-1991``,
``liveness_class=STRUCTURALLY_IMPOSSIBLE``). Seeding that field is a
separate, owner-gated program; the function reads the real substrate shape
so the score activates the moment a producer exists.

**Source:** ``src/babylon/ooda/initiative.py:82``

update_momentum
~~~~~~~~~~~~~~~

.. code-block:: python

   def update_momentum(
       current_momentum: float,
       action_succeeded: bool,
       defines: OODADefines,
   ) -> float: ...

Update momentum after an action. Decays by ``momentum_decay`` per tick,
gains ``momentum_success_bonus`` on success.

**Source:** ``src/babylon/ooda/initiative.py:127``


Action Eligibility
------------------

ELIGIBILITY_MAP
~~~~~~~~~~~~~~~

.. code-block:: python

   ELIGIBILITY_MAP: dict[tuple[str, str], bool]

Frozen 21x4 eligibility matrix mapping ``(org_type_value, action_type_value)``
tuples to boolean availability. 12 actions are universal (available to all org
types). Remaining 9 are restricted by org type.

**Source:** ``src/babylon/ooda/action_eligibility.py``

check_eligibility
~~~~~~~~~~~~~~~~~

.. code-block:: python

   def check_eligibility(
       org_type: OrgType | str,
       action_type: ActionType | str,
       org_attrs: dict[str, Any] | None = None,
   ) -> bool: ...

Check if an organization type can perform an action. Consults ``ELIGIBILITY_MAP``
with three special-case overrides:

- **REPRESS**: non-state orgs with ``violence_capacity > 0`` can perform.
- **SURVEIL**: non-state orgs with ``surveillance_capacity > 0`` can perform.
- **ASSIMILATE**: PoliticalFaction/CivilSociety only if
  ``consciousness_tendency == LIBERAL`` AND ``is_institution == True``.

**Source:** ``src/babylon/ooda/action_eligibility.py``


Action Costs
------------

.. code-block:: python

   def compute_action_cost(
       action_type: ActionType,
       org_id: str,
       target_id: str,
       graph: nx.DiGraph[str],
       defines: OODADefines,
   ) -> ActionCostModifier: ...

Compute effective cost of an action with community modifiers. Three tiers:

1. **Embedded** (overlap > 0): discount.
2. **Contradiction axis**: 2.5x surcharge.
3. **Outsider**: 1.5x surcharge.

**Source:** ``src/babylon/ooda/action_costs.py:34``


Action Effects
--------------

compute_consciousness_delta
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def compute_consciousness_delta(
       org_attrs: dict[str, Any],
       target_community_id: str,
       action_type: ActionType,
       graph: nx.DiGraph[str],
       defines: OODADefines,
       org_defines: OrganizationDefines,
   ) -> ConsciousnessDelta | None: ...

Compute consciousness effect of an action on a target community. Returns
``None`` if the action has no CI effect (e.g., AGITATE).

**Steps:**

1. Look up action base from defines (zero = no effect).
2. Compute membership overlap and effective credibility.
3. Base delta = ``tendency_modifier * cadre * cohesion * credibility``.
4. Scale by action base.
5. EDUCATE contestation bonus (if contestation > threshold).
6. Clamp to ``max_ci_delta_per_tick``.

**Source:** ``src/babylon/ooda/action_effects.py:25``

resolve_action
~~~~~~~~~~~~~~

.. code-block:: python

   def resolve_action(
       action: Action,
       org_attrs: dict[str, Any],
       graph: nx.DiGraph[str],
       defines: OODADefines,
       org_defines: OrganizationDefines,
   ) -> ActionResult: ...

Resolve a single action, dispatching to specialized resolvers:

- **AGITATE**: no CI delta, returns ``contestation_delta`` in ``direct_effects``.
- **REPRESS/SURVEIL**: backfire raises target CI (REVOLUTIONARY tendency).
  Emits ``STATE_REPRESSION`` or ``STATE_SURVEILLANCE`` event.
- **ASSIMILATE**: negative CI delta, pushes LIBERAL tendency.
- **All others**: standard consciousness delta computation.

**Source:** ``src/babylon/ooda/action_effects.py:98``


Constraints
-----------

enforce_action_points
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def enforce_action_points(
       actions: list[Action],
       profile: OODAProfile,
   ) -> tuple[list[Action], list[ActionResult]]: ...

Greedily accept actions until AP budget is exhausted. Returns tuple of
(accepted actions, rejected ActionResults with ``failure_reason``).

**Source:** ``src/babylon/ooda/constraints.py:18``

enforce_coordination_range
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def enforce_coordination_range(
       actions: list[Action],
       profile: OODAProfile,
   ) -> tuple[list[Action], list[ActionResult]]: ...

Reject actions targeting more distinct territories than ``coordination_range``.
Actions targeting already-seen targets are always accepted.

**Source:** ``src/babylon/ooda/constraints.py:54``

apply_autonomy_modifier
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def apply_autonomy_modifier(
       num_distinct_targets: int,
       autonomy: float,
       defines: OODADefines,
   ) -> float: ...

Compute effectiveness modifier from the autonomy-breadth tradeoff.
Single target = 1.0 (no penalty). Multiple targets penalized proportionally.
Floor at 0.1.

**Source:** ``src/babylon/ooda/constraints.py:92``


Lifecycle Capacity
------------------

compute_lifecycle_modifier
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def compute_lifecycle_modifier(
       org_id: str,
       graph: nx.DiGraph[str],
       org_defines: OrganizationDefines,
   ) -> float: ...

Compute lifecycle-weighted capacity modifier. Youth = 0.0, Adult = 1.0,
Elder = ``elder_capacity_factor`` (default 0.2). Returns [0, 1].

Reuses :func:`babylon.domain.organizations.composition.lifecycle_composition` and
:func:`babylon.domain.organizations.composition.effective_capacity`.

**Source:** ``src/babylon/ooda/lifecycle_capacity.py:22``

elder_legitimacy_bonus
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def elder_legitimacy_bonus(
       org_id: str,
       graph: nx.DiGraph[str],
       org_defines: OrganizationDefines,
       ooda_defines: OODADefines,
   ) -> float: ...

Returns ``elder_legitimacy_multiplier`` (1.3) when organization has any elder
(D'-phase) members. Returns 1.0 otherwise.

**Source:** ``src/babylon/ooda/lifecycle_capacity.py:44``


Layer 0: Automatic Metabolism
-----------------------------

.. code-block:: python

   def process_layer0(
       graph: nx.DiGraph[str],
       services: ServiceContainer,
   ) -> list[ActionResult]: ...

Record automatic metabolism for Business organizations. Finds all
``_node_type="organization"`` nodes with ``org_type="business"``, generates
an EMPLOY ActionResult for each targeting their primary territory.

Runs before initiative ordering in the OODA system.

**Source:** ``src/babylon/ooda/layer0.py:22``


Layer 3: Consequence Propagation
---------------------------------

.. code-block:: python

   def process_layer3(
       action_results: list[ActionResult],
       graph: nx.DiGraph[str],
       defines: OODADefines,
   ) -> dict[str, Any]: ...

Propagate action consequences to communities. Five sub-processors mutate
the graph in place:

1. **Consciousness aggregation**: groups CI deltas by target community,
   calls :func:`babylon.domain.organizations.consciousness.aggregate_consciousness_effects`,
   updates ``collective_identity`` on community nodes.
2. **Heat propagation**: REPRESS actions add ``repress_heat_delta`` (0.15),
   SURVEIL actions add ``surveil_heat_delta`` (0.05) to community ``heat``.
   Capped at 1.0.
3. **Edge transitions**: ORGANIZE actions transition edges from
   ``TRANSACTIONAL`` to ``SOLIDARISTIC``.
4. **Infrastructure**: BUILD_INFRASTRUCTURE adds ``build_infrastructure_delta``
   (0.1), ATTACK_INFRASTRUCTURE subtracts ``attack_infrastructure_delta`` (0.1)
   from community ``infrastructure``. Clamped to [0, 1].
5. **Contestation**: AGITATE ``contestation_delta`` values from
   ``direct_effects`` are summed per community and applied to
   ``ideological_contestation``. Clamped to [0, 1].

Returns summary dict with keys: ``consciousness``, ``heat_updates``,
``edge_transitions``, ``infrastructure_updates``, ``contestation_updates``.

**Source:** ``src/babylon/ooda/layer3.py:24``


NPC Action Selection
--------------------

.. code-block:: python

   def select_npc_actions(
       org_id: str,
       org_attrs: dict[str, Any],
       target_id: str,
       defines: OODADefines,
   ) -> list[Action]: ...

Deterministic priority-based action selection for non-player organizations.
Greedily selects highest-priority eligible actions until AP budget is exhausted.

**Priority queues by org type:**

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Org Type
     - Priority (highest first)
   * - STATE_APPARATUS
     - SURVEIL, REPRESS, INFILTRATE, MAP_NETWORK, COUNTER_INTEL
   * - POLITICAL_FACTION
     - EDUCATE, ORGANIZE, AGITATE, RECRUIT, FUNDRAISE
   * - CIVIL_SOCIETY
     - PROVIDE_SERVICE, EDUCATE, ORGANIZE, FUNDRAISE, BUILD_INFRASTRUCTURE
   * - BUSINESS
     - EMPLOY, FUNDRAISE, DENOUNCE

AI-driven action selection is deferred to a future feature.

**Source:** ``src/babylon/ooda/npc_stub.py:50``


OODASystem
----------

.. code-block:: python

   from babylon.engine.systems.ooda import OODASystem

   system = OODASystem()
   system.name  # "ooda"
   system.step(graph, services, context)

The central orchestrator. Executes three-phase turn resolution each tick:

1. **Phase 1 -- Layer 0**: auto-metabolism for Business orgs.
2. **Phase 2 -- Action Phase**: compute initiative scores, sort descending,
   check for player actions in ``context.persistent_data["player_actions"]``,
   select NPC actions via ``select_npc_actions()``, resolve all actions.
3. **Phase 3 -- Layer 3**: propagate consequences via ``process_layer3()``.

Publishes ``ORGANIZATIONAL_ACTION`` event with summary statistics.

**Source:** ``src/babylon/engine/systems/ooda.py``


See Also
--------

- :doc:`/concepts/ooda-loop-system` -- Why the system works this way
- :doc:`/reference/ooda-coefficients` -- Every tunable coefficient
- :doc:`/reference/organizations` -- Organization model (Feature 031)
- :doc:`/reference/events` -- Event system reference
- :doc:`/reference/configuration` -- GameDefines configuration
- :py:mod:`babylon.ooda` -- Source code
