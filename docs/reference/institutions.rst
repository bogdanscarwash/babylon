Institution Reference
=====================

Technical reference for the Institution Base Model (Feature 040).
Covers entity models, supporting models, event types, pure functions,
graph queries, enums, graph integration, and configuration.

**Import:**

.. code-block:: python

   from babylon.models.entities.institution import (
       Institution,
       InternalBalanceOfForces,
       ReproductionMechanism,
       SpawningBlueprint,
       InstitutionOrgRelation,
       FactionShiftEvent,
       ReproductionEvent,
       BonapartistModeEvent,
   )

   from babylon.domain.institution import (
       structural_selectivity,
       update_internal_balance,
       hegemonic_fraction_effect,
   )

.. contents:: On this page
   :local:
   :depth: 2


Entity Models
-------------

Institution
~~~~~~~~~~~

Third-layer entity between substrate (SocialClass, Territory, Community)
and agents (Organizations). Frozen (immutable). Graph node type:
``_node_type="institution"``.

.. list-table::
   :header-rows: 1
   :widths: 25 20 10 45

   * - Field
     - Type
     - Default
     - Description
   * - ``id``
     - ``str``
     -
     - Unique institution identifier (min length 1).
   * - ``name``
     - ``str``
     -
     - Human-readable name (min length 1).
   * - ``apparatus_type``
     - ``ApparatusType``
     -
     - Althusserian classification (RSA, ISA, or Economic).
   * - ``social_function``
     - ``SocialFunction``
     -
     - Population need served by this institution.
   * - ``class_inscription``
     - ``ClassInscription``
     - BOURGEOIS
     - Which class the institution serves.
   * - ``internal_balance``
     - ``InternalBalanceOfForces``
     -
     - Factional weight distribution.
   * - ``action_modifiers``
     - ``dict[str, float]``
     - ``{}``
     - Override structural selectivity modifiers (ActionType -> multiplier).
   * - ``budget``
     - ``float``
     - 0.0
     - Available resources (>= 0).
   * - ``fixed_asset_territory_ids``
     - ``list[str]``
     - ``[]``
     - Territories with fixed infrastructure.
   * - ``legal_authorities``
     - ``frozenset[str]``
     - ``frozenset()``
     - Legal powers held (e.g. "arrest", "tax", "legislate").
   * - ``personnel_capacity``
     - ``int``
     - 0
     - Maximum personnel count (>= 0).
   * - ``formalization_level``
     - ``float``
     - 0.5
     - Degree of bureaucratic formalization [0, 1].
   * - ``institutional_inertia``
     - ``float``
     - 0.5
     - Resistance to rapid change [0, 1].
   * - ``legitimacy``
     - ``float``
     - 0.5
     - Public perceived legitimacy [0, 1].
   * - ``housed_org_ids``
     - ``list[str]``
     - ``[]``
     - Organization IDs housed within this institution.
   * - ``territory_ids``
     - ``list[str]``
     - ``[]``
     - Territories where institution operates.
   * - ``jurisdiction``
     - ``frozenset[str] | None``
     - ``None``
     - Jurisdiction scope. Only valid for ``RSA_`` apparatus types.
   * - ``lifecycle_function``
     - ``LifecyclePhase | None``
     - ``None``
     - D-P-D' lifecycle phase assignment.
   * - ``reproduction``
     - ``ReproductionMechanism``
     -
     - Self-perpetuation mechanisms.
   * - ``spawning_blueprints``
     - ``list[SpawningBlueprint]``
     - ``[]``
     - Templates for replacement Organizations.

**Validators:**

- ``jurisdiction`` must be ``None`` for non-RSA apparatus types.
- All ``action_modifiers`` values must be > 0.0.
- ``internal_balance`` faction weights must sum to 1.0 (tolerance +/- 0.01).


Supporting Models
-----------------

InternalBalanceOfForces
~~~~~~~~~~~~~~~~~~~~~~~

Factional weight distribution within an institution. Three ruling-class
fractions compete for hegemony. Weights always sum to 1.0.

.. list-table::
   :header-rows: 1
   :widths: 30 15 10 45

   * - Field
     - Type
     - Default
     - Description
   * - ``liberal_technocratic``
     - ``float``
     -
     - Weight of Liberal-Technocratic faction [0, 1].
   * - ``revanchist_fascist``
     - ``float``
     -
     - Weight of Revanchist-Fascist faction [0, 1].
   * - ``institutionalist_bonapartist``
     - ``float``
     -
     - Weight of Institutionalist-Bonapartist faction [0, 1].
   * - ``internal_contestation``
     - ``float``
     - 0.0
     - How actively factional warfare is occurring [0, 1].

**Computed fields:**

- ``hegemonic_fraction`` (``RulingClassFraction``): Faction with highest weight.

**Validators:**

- Faction weights must sum to 1.0 (tolerance: ``0.99 <= total <= 1.01``).

ReproductionMechanism
~~~~~~~~~~~~~~~~~~~~~

Self-perpetuation capacity of an institution. Tracks formal mechanisms
for institutional reproduction.

.. list-table::
   :header-rows: 1
   :widths: 30 15 10 45

   * - Field
     - Type
     - Default
     - Description
   * - ``recruitment_pipeline``
     - ``bool``
     - ``False``
     - Has formal member intake process.
   * - ``training_program``
     - ``bool``
     - ``False``
     - Has formal training/socialization.
   * - ``succession_protocol``
     - ``bool``
     - ``False``
     - Has leadership succession plan.
   * - ``budget_independence``
     - ``float``
     - 0.0
     - Fraction of budget from own sources [0, 1].
   * - ``legal_self_perpetuation``
     - ``bool``
     - ``False``
     - Has legal mandate to exist.

**Computed fields:**

- ``reproduction_capacity`` (``float``): Composite score.
  Formula: ``(sum(bools) / 4) * 0.7 + budget_independence * 0.3``.
  The four boolean mechanisms contribute 70%, budget independence 30%.

SpawningBlueprint
~~~~~~~~~~~~~~~~~

Template for replacement Organization creation. Stored on institutions
to define how replacements are created when housed Organizations are
destroyed.

.. list-table::
   :header-rows: 1
   :widths: 30 20 10 40

   * - Field
     - Type
     - Default
     - Description
   * - ``org_type``
     - ``OrgType``
     -
     - Organization category to spawn.
   * - ``default_class_character``
     - ``ClassCharacter``
     -
     - Initial class character for spawned org.
   * - ``base_attributes``
     - ``dict[str, Any]``
     - ``{}``
     - Additional attributes for spawned org.

InstitutionOrgRelation
~~~~~~~~~~~~~~~~~~~~~~

Relationship between institution and housed Organization. Tracks
material and political dimensions of housing.

.. list-table::
   :header-rows: 1
   :widths: 30 20 10 40

   * - Field
     - Type
     - Default
     - Description
   * - ``institution_id``
     - ``str``
     -
     - Parent institution ID (min length 1).
   * - ``organization_id``
     - ``str``
     -
     - Housed organization ID (min length 1).
   * - ``resource_provision``
     - ``float``
     - 0.0
     - Fraction of institution resources provided [0, 1].
   * - ``legal_cover``
     - ``bool``
     - ``False``
     - Whether institution provides legal protection.
   * - ``legitimacy_transfer``
     - ``float``
     - 0.0
     - How much institutional legitimacy transfers [0, 1].
   * - ``action_oversight``
     - ``float``
     - 0.0
     - How much institution constrains org actions [0, 1].
   * - ``factional_alignment``
     - ``RulingClassFraction | None``
     - ``None``
     - Which ruling-class faction the org aligns with.


Event Types
-----------

Three event types are returned by institution functions. These are data
objects, not emitted directly -- callers decide when and how to emit them
via the EventBus.

FactionShiftEvent
~~~~~~~~~~~~~~~~~

Returned by ``update_internal_balance()`` when the hegemonic fraction changes.
Maps to ``EventType.INSTITUTION_FACTION_SHIFT``.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``institution_id``
     - ``str``
     - Institution whose hegemonic fraction changed.
   * - ``old_fraction``
     - ``RulingClassFraction``
     - Previous hegemonic fraction.
   * - ``new_fraction``
     - ``RulingClassFraction``
     - New hegemonic fraction.
   * - ``weights``
     - ``dict[str, float]``
     - Updated faction weights.

ReproductionEvent
~~~~~~~~~~~~~~~~~

Returned when an institution spawns a replacement Organization.
Maps to ``EventType.INSTITUTION_REPRODUCTION``.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``institution_id``
     - ``str``
     - Institution that spawned the org.
   * - ``spawned_org_type``
     - ``OrgType``
     - Type of organization spawned.
   * - ``blueprint``
     - ``SpawningBlueprint``
     - Blueprint used for spawning.

BonapartistModeEvent
~~~~~~~~~~~~~~~~~~~~

Returned by ``update_internal_balance()`` when the BONAPARTIST weight
crosses the threshold while other fractions are below the exclusion
threshold. Maps to ``EventType.INSTITUTION_BONAPARTIST_MODE``.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Field
     - Type
     - Description
   * - ``institution_id``
     - ``str``
     - Institution entering Bonapartist mode.
   * - ``bonapartist_weight``
     - ``float``
     - Current BONAPARTIST faction weight [0, 1].


Pure Functions
--------------

All three functions are stateless. They take data in and return data out.
No EventBus dependency.

**Import:**

.. code-block:: python

   from babylon.domain.institution import (
       structural_selectivity,
       update_internal_balance,
       hegemonic_fraction_effect,
   )

structural_selectivity
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def structural_selectivity(
       institution: Institution,
       action_type: ActionType,
       defaults: dict[str, dict[str, float]],
   ) -> float: ...

Computes the cost modifier for an action within an institution.
Lookup order:

1. ``institution.action_modifiers[action_type.value]`` -- institution-level override.
2. ``defaults[apparatus_type.value][action_type.value]`` -- apparatus-type defaults.
3. ``1.0`` -- no modifier (fallback).

Returns a multiplier: < 1.0 means cheaper, > 1.0 means more expensive.

The ``defaults`` argument is typically ``InstitutionDefines.default_action_modifiers``.

update_internal_balance
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def update_internal_balance(
       balance: InternalBalanceOfForces,
       crisis_intensity: float,
       legitimacy: float,
       external_threat: float,
       alpha: float = 0.05,
       bonapartist_threshold: float = 0.4,
       bonapartist_exclusion_threshold: float = 0.35,
       institution_id: str = "unknown",
   ) -> tuple[InternalBalanceOfForces, list[FactionShiftEvent | BonapartistModeEvent]]: ...

Alpha-smoothed factional balance shift:

- Rising ``crisis_intensity`` drives REVANCHIST weight up (repression impulse).
- Falling ``legitimacy`` weakens LIBERAL weight (consent breaks down).
- Rising ``external_threat`` drives BONAPARTIST weight up (self-preservation).

Deltas::

   revanchist_delta  = alpha * crisis_intensity
   liberal_delta     = -alpha * (1.0 - legitimacy)
   bonapartist_delta = alpha * external_threat

After applying deltas, weights are clamped to [0, 1] and renormalized to
sum to 1.0. Contestation is computed as ``min(1.0, 1.0 - max_weight + 0.1)``.

Returns a tuple of (new balance, events). Events may include:

- ``FactionShiftEvent`` if the hegemonic fraction changed.
- ``BonapartistModeEvent`` if BONAPARTIST weight exceeds ``bonapartist_threshold``
  and both other fractions are below ``bonapartist_exclusion_threshold``.

hegemonic_fraction_effect
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   def hegemonic_fraction_effect(
       fraction: RulingClassFraction,
   ) -> dict[str, Any]: ...

Returns OODA modifier hints based on the hegemonic fraction:

.. list-table::
   :header-rows: 1
   :widths: 30 30 20 20

   * - Fraction
     - Preferred Actions
     - Escalation Reluctance
     - Strategy
   * - LIBERAL_TECHNOCRATIC
     - ``[ASSIMILATE]``
     - 0.7
     - Consent-based rule
   * - REVANCHIST_FASCIST
     - ``[REPRESS]``
     - 0.2
     - Naked repression
   * - INSTITUTIONALIST_BONAPARTIST
     - ``[SURVEIL]``
     - 0.5
     - Self-preservation

Raises ``ValueError`` if ``fraction`` is not a valid ``RulingClassFraction``.


Enums
-----

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Enum
     - Values
   * - ``ApparatusType``
     - RSA: ``RSA_EXECUTIVE``, ``RSA_MILITARY``, ``RSA_POLICE``,
       ``RSA_JUDICIAL``, ``RSA_CARCERAL``.
       ISA: ``ISA_EDUCATIONAL``, ``ISA_RELIGIOUS``, ``ISA_FAMILY``,
       ``ISA_LEGAL``, ``ISA_POLITICAL``, ``ISA_COMMUNICATIONS``,
       ``ISA_CULTURAL``.
       Economic: ``ECONOMIC_PRODUCTIVE``, ``ECONOMIC_FINANCIAL``,
       ``ECONOMIC_EXTRACTIVE``.
   * - ``SocialFunction``
     - ``EMPLOYMENT``, ``EDUCATION``, ``WORSHIP``, ``POLICING``,
       ``HEALTHCARE``, ``CARE``, ``ADJUDICATION``, ``COMMUNICATION``,
       ``LEGISLATION``, ``INCARCERATION``, ``MILITARY_DEFENSE``,
       ``FINANCIAL_INTERMEDIATION``
   * - ``ClassInscription``
     - ``BOURGEOIS``, ``PROLETARIAN``, ``CONTESTED``
   * - ``RulingClassFraction``
     - ``LIBERAL_TECHNOCRATIC``, ``REVANCHIST_FASCIST``,
       ``INSTITUTIONALIST_BONAPARTIST``
   * - ``LifecyclePhase``
     - ``D_DEPENDENT``, ``P_PRODUCTIVE``, ``D_PRIME_DEPENDENT``

**Edge types** (added to ``EdgeType``):

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Type
     - Direction
     - Description
   * - ``HOUSES``
     - Institution -> Organization
     - Institution houses and shapes an Organization.

**Event types** (added to ``EventType``):

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Type
     - Description
   * - ``INSTITUTION_FACTION_SHIFT``
     - Hegemonic fraction changed within an institution.
   * - ``INSTITUTION_REPRODUCTION``
     - Institution spawned a replacement Organization.
   * - ``INSTITUTION_BONAPARTIST_MODE``
     - Bonapartist threshold crossed.


Graph Integration
-----------------

Institutions are stored on ``WorldState`` and participate in the
``to_graph()``/``from_graph()`` round-trip:

.. code-block:: python

   from babylon.models.world_state import WorldState

   world = WorldState(
       tick=0,
       institutions={"doj": department_of_justice},
       institution_relations=[doj_fbi_relation],
   )

   graph = world.to_graph()
   # graph.nodes["doj"]["_node_type"] == "institution"

   reconstructed = WorldState.from_graph(graph, tick=0)
   assert isinstance(reconstructed.institutions["doj"], Institution)

``to_graph()`` creates ``HOUSES`` edges from each institution to its
``housed_org_ids``.

**frozenset handling:** ``model_dump()`` converts ``frozenset`` to ``list``.
``from_graph()`` converts back to ``frozenset`` for ``legal_authorities``
and ``jurisdiction`` fields.


InstitutionDefines
------------------

Tunable parameters in ``GameDefines.institution``:

.. list-table::
   :header-rows: 1
   :widths: 40 10 50

   * - Parameter
     - Default
     - Description
   * - ``alpha_smoothing_rate``
     - 0.05
     - [S] Per-call smoothing rate for factional balance shifts.
   * - ``bonapartist_threshold``
     - 0.4
     - [S] BONAPARTIST weight above which Bonapartist mode triggers.
   * - ``bonapartist_exclusion_threshold``
     - 0.35
     - [S] Other fractions must be below this for Bonapartist mode.
   * - ``default_action_modifiers``
     - (see below)
     - [S] Default action cost modifiers per ApparatusType.

**Default action modifiers:**

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Apparatus Type
     - Modifiers (ActionType -> multiplier)
   * - ``rsa_executive``
     - propagandize: 0.5, repress: 0.8, educate: 1.5
   * - ``rsa_police``
     - repress: 0.6, surveil: 0.7, educate: 2.0
   * - ``rsa_judicial``
     - surveil: 0.5, repress: 1.2
   * - ``rsa_military``
     - repress: 0.4, attack_infrastructure: 0.5
   * - ``rsa_carceral``
     - repress: 0.5, educate: 2.5
   * - ``isa_educational``
     - educate: 0.7, recruit: 0.8, repress: 2.0
   * - ``isa_religious``
     - educate: 0.8, recruit: 0.7, repress: 2.5
   * - ``isa_family``
     - educate: 0.9, recruit: 1.5
   * - ``isa_communications``
     - propagandize: 0.5, agitate: 0.6, repress: 2.0
   * - ``isa_cultural``
     - educate: 0.8, propagandize: 0.7, repress: 2.0
   * - ``isa_legal``
     - (none)
   * - ``isa_political``
     - (none)
   * - ``economic_productive``
     - employ: 0.5, fundraise: 0.7, repress: 1.5
   * - ``economic_financial``
     - fundraise: 0.4, employ: 0.8
   * - ``economic_extractive``
     - fundraise: 0.5, attack_infrastructure: 0.8


Deprecation Notes
-----------------

The ``Organization.is_institution`` and ``Organization.institutional_persistence``
fields (Feature 031) are deprecated. Use ``Institution.formalization_level``
and ``Institution.institutional_inertia`` instead. Constructing an Organization
with these fields set emits ``DeprecationWarning``.

The legacy JSON Schema at ``src/babylon/schemas/entities/institution.schema.json``
is superseded by the Pydantic ``Institution`` model and will be removed in a
future version.


See Also
--------

- :doc:`/concepts/institution-model` -- Why institutions exist as a separate layer
- :doc:`/reference/organizations` -- Organization Base Model reference
- :doc:`/reference/data-models` -- SocialClass, Territory, and graph structure
- :doc:`/reference/state-apparatus-ai` -- State Apparatus AI reference
- :doc:`/reference/ooda-loop-system` -- OODA Loop System reference
- :py:mod:`babylon.domain.institution` -- Pure function source code
- :py:mod:`babylon.models.entities.institution` -- Entity model source code
