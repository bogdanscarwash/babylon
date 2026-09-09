How to Work with the State Apparatus AI
=======================================

Goal-oriented guides for extending, tuning, integrating with, and
debugging the state apparatus AI subsystem (Feature 039).

.. contents:: On this page
   :local:
   :depth: 2


How to Add a New Sub-verb
--------------------------

To add a new sub-verb (e.g., ``EXTRADITE`` under REPRESS):

1. Add the enum member to ``StateActionType`` in
   ``src/babylon/models/enums.py``::

       EXTRADITE = "extradite"

2. Add it to the parent's frozenset in ``VERB_CHILDREN``
   (``src/babylon/models/entities/state_apparatus_ai.py``)::

       StateActionType.REPRESS: frozenset({
           StateActionType.SURVEIL,
           StateActionType.INFILTRATE,
           StateActionType.RAID,
           StateActionType.PROSECUTE,
           StateActionType.LIQUIDATE,
           StateActionType.EXTRADITE,  # new
       }),

3. Add a budget cost in ``_VERB_COSTS``
   (``src/babylon/ooda/state_ai/decision.py``)::

       StateActionType.EXTRADITE: 12.0,

4. Add a legitimacy cost in ``_LEGITIMACY_COSTS``
   (``src/babylon/ooda/state_ai/decision.py``)::

       StateActionType.EXTRADITE: 0.08,

5. Add it to the escalation ladder in ``StateApparatusAIDefines``
   (``src/babylon/config/defines.py``). Insert it at the appropriate
   position -- between PROSECUTE and LIQUIDATE for moderate-severity
   actions.

6. Write the effect resolution function. Create it in the appropriate
   module (``repress_effects.py`` for REPRESS sub-verbs)::

       def resolve_extradite(
           target_org: dict[str, Any],
           target_key_figure_id: str,
           destination: str,
           defines: StateApparatusAIDefines,
       ) -> tuple[dict[str, Any], float]:
           """Returns (updated_org, legitimacy_cost)."""
           ...

7. Export from ``src/babylon/ooda/state_ai/__init__.py``::

       __all__ = [
           ...
           "resolve_extradite",
       ]

8. Write tests. At minimum:

   - Unit test in ``tests/unit/state_ai/test_repress_effects.py``
     verifying the effect math.
   - Verify the enum, VERB_CHILDREN, and ``StateAction`` validator
     accept the new sub-verb.
   - Verify the escalation ladder includes the new sub-verb.

9. Run the full test suite::

       uv run pytest tests/unit/state_ai/ tests/contract/state_ai/ -v


How to Tune Faction Dynamics
-----------------------------

All faction dynamics parameters live in ``StateApparatusAIDefines``
(``src/babylon/config/defines.py``).

**To change how fast factions shift:**

Adjust ``max_faction_shift_per_tick`` (default: 0.05). Higher values
make the state more reactive to player actions; lower values create
more inertia. Range: ``[0.0, 0.2]``.

**To change when fascist convergence triggers:**

Adjust the three threshold fields:

- ``fascist_security_threshold`` (default: 0.4) -- SS weight needed
- ``fascist_settler_ci_threshold`` (default: 0.6) -- settler CI needed
- ``fascist_finance_ceiling`` (default: 0.25) -- FC weight ceiling

Lower ``fascist_security_threshold`` and ``fascist_settler_ci_threshold``
make convergence easier to trigger. Raise them to make fascism harder
to reach.

**To change how hard fascism is to exit:**

Adjust the reversion thresholds:

- ``reversion_ss_threshold`` (default: 0.25)
- ``reversion_ci_threshold`` (default: 0.30)

Lower values make reversion easier; higher values make fascism more
persistent.

**To verify your tuning:**

Run the faction dynamics contract tests::

    uv run pytest tests/contract/state_ai/test_faction_contract.py -v

These tests verify that convergence, reversion, and shift mechanics
produce correct behavior under the current configuration.


How to Adjust the REPRESS Pipeline
------------------------------------

The REPRESS pipeline has four sub-verbs, each with tunable parameters:

**INFILTRATE** (intelligence gathering via agents):

- ``infiltrate_informant_intel_rate`` -- intel gained per tick from
  informants (default: 0.05)
- ``infiltrate_provocateur_intel_rate`` -- intel from provocateurs
  (default: 0.03, lower because provocateurs provoke, not observe)
- ``infiltrate_mole_intel_rate`` -- intel from moles (default: 0.08,
  highest because moles have deepest access)
- ``infiltrate_detection_base_chance`` -- base probability of detection
  per tick (default: 0.1). PROVOCATEUR has 1.5x this chance.

**RAID** (direct action against organizations):

- ``raid_ci_radicalization_threshold`` -- CI level above which raids
  radicalize instead of suppress (default: 0.5). This is the
  consciousness dialectic pivot point.
- ``raid_ci_radicalization_boost`` -- CI increase when raids radicalize
  (default: 0.1)
- ``raid_ci_suppression_rate`` -- CI decrease when raids suppress
  (default: 0.15)
- ``raid_org_coherence_damage`` -- coherence damage per raid
  (default: 0.2)
- ``raid_key_figure_capture_base`` -- base capture probability for key
  figures (default: 0.3). Modified by force level multipliers.

**PROSECUTE** (legal action against individuals):

- ``prosecute_org_morale_damage`` -- organization morale impact
  (default: 0.1)
- ``prosecute_key_figure_removal_chance`` -- probability of removing
  targeted key figure (default: 0.6)
- ``prosecute_terrorism_charge_multiplier`` -- multiplier for TERRORISM
  charges (default: 1.5x all effects)

**LIQUIDATE** (elimination of key figures):

- ``liquidate_core_legitimacy_cost`` -- legitimacy cost in CORE
  territories (default: 0.15). Requires EMERGENCY_POWERS legislation.
- ``liquidate_periphery_legitimacy_cost`` -- legitimacy cost in
  PERIPHERY territories (default: 0.03)
- ``liquidate_deniability_threshold`` -- deniability level that halves
  legitimacy cost (default: 0.5)

Run the REPRESS contract tests after changes::

    uv run pytest tests/contract/state_ai/test_repress_contract.py -v


How to Use God Mode for Debugging
-----------------------------------

God mode exposes all state AI internals for development and testing.

**Enable god mode** by setting ``god_mode_enabled=True`` in
``StateApparatusAIDefines``::

    from babylon.config.defines import GameDefines, StateApparatusAIDefines

    defines = GameDefines()
    state_defines = defines.state_apparatus_ai.model_copy(
        update={"god_mode_enabled": True}
    )

**Read debug state** after action selection::

    from babylon.ooda.state_ai import RuleBasedStateAI

    ai = RuleBasedStateAI()
    actions = ai.select_action(
        org_id="fbi_001",
        faction_balance=balance,
        budget=budget,
        heat=0.6,
        defines=state_defines,
    )

    debug = ai.get_debug_state(
        defines=state_defines,
        faction_balance=balance,
        budget=budget,
        last_actions=actions,
    )
    # Returns dict with: dominant_faction, stability, legitimacy,
    # budget details, action list with costs

When ``god_mode_enabled=False``, ``get_debug_state()`` returns ``None``.


How to Integrate with State AI Events
---------------------------------------

The state AI defines six ``EventType`` members for downstream
consumers. To subscribe to state AI events:

1. Import the event types::

       from babylon.models.enums import EventType

2. Subscribe via the EventBus::

       event_bus.subscribe(
           EventType.STATE_ACTION_EXECUTED,
           your_handler,
       )
       event_bus.subscribe(
           EventType.FASCIST_CONVERGENCE,
           your_handler,
       )

Available event types:

- ``STATE_ACTION_EXECUTED`` -- any state verb resolved
- ``FASCIST_CONVERGENCE`` -- three-pillar conditions met
- ``FACTION_SHIFT`` -- faction weights changed
- ``THREAD_ESCALATION`` -- attention thread advanced phase
- ``LEGAL_FRAMEWORK_ENACTED`` -- new legislation created
- ``LEGAL_FRAMEWORK_REVOKED`` -- legislation revoked

.. note::

   These event types are declared but not yet emitted by any system as
   of the current implementation. If you need to consume these events,
   you will need to add emission calls at the appropriate resolution
   points. See :doc:`/reference/state-apparatus-ai` for the dispatch
   path.


How to Run the Integration Test
---------------------------------

The 52-tick integration test validates the full state AI lifecycle:
escalation, de-escalation, and convergence.

::

    uv run pytest tests/integration/test_state_ai_integration.py -v

This test:

1. Starts with Detroit 2010 defaults (FC=0.45, SS=0.30, SP=0.25)
2. Simulates increasing player heat over 52 ticks
3. Verifies verb selection shifts from CO_OPT through REPRESS
4. Verifies faction balance shifts toward SS under sustained heat
5. Verifies de-escalation when heat subsides
6. Validates budget constraints and attention thread allocation

To run all state AI tests (unit + contract + integration)::

    uv run pytest tests/unit/state_ai/ tests/contract/state_ai/ \
        tests/integration/test_state_ai_integration.py -v

Expected: 499 tests, all passing.


How to Read Player-Visible State Information
---------------------------------------------

The observability module provides functions that convert internal state
into player-appropriate representations:

**Observable actions** (what the player sees when the state acts)::

    from babylon.ooda.state_ai import create_observable_action

    obs = create_observable_action(action, territory_heat=0.7)
    # Returns: {verb, sub_verb, target_id, visible_intensity, territory_heat}

**Territory observables** (what the player sees about territory
conditions)::

    from babylon.ooda.state_ai import create_territory_observables

    obs = create_territory_observables(territory_dict)
    # Returns: {property_value_proxy, infrastructure_quality, heat,
    #           population, collective_identity}

**Counter-intelligence results** (tiered disclosure based on player
intel capability)::

    from babylon.ooda.state_ai import resolve_counter_intel

    intel = resolve_counter_intel(
        intel_success=0.6,
        faction_balance=balance,
        last_actions=actions,
        defines=defines,
    )
    # At 0.6: includes faction_balance + action details

Disclosure tiers:

- ``>= 0.0``: intel_level + visible actions (verb + target only)
- ``>= 0.3``: adds faction balance (rounded to 2 decimals)
- ``>= 0.6``: adds full action details (sub_verb, budget_cost, faction)
- ``>= 0.8``: adds full state (dominant_faction, stability, legitimacy)


How to Add a New Faction Shift Trigger
---------------------------------------

To add a new player action that shifts faction balance:

1. Add the action type key to ``_PLAYER_ACTION_SHIFTS`` in
   ``src/babylon/ooda/state_ai/faction_dynamics.py``::

       _PLAYER_ACTION_SHIFTS: dict[str, tuple[str, float]] = {
           "heat_generation": ("security_state", 0.03),
           "surviving_repression": ("security_state", -0.04),
           ...
           "your_new_trigger": ("finance_capital", -0.02),  # new
       }

   The tuple is ``(faction_to_shift, magnitude)``. Positive values
   increase that faction's weight; negative values decrease it.

2. Call ``apply_player_action_shift()`` from the resolution path where
   your trigger occurs::

       from babylon.ooda.state_ai import apply_player_action_shift

       new_balance = apply_player_action_shift(
           action_type="your_new_trigger",
           outcome="success",
           current_balance=current_balance,
           defines=defines,
       )

   Outcome ``"success"`` applies full magnitude; any other value
   applies ``0.5x``.

For material condition triggers (not player-initiated), use
``_MATERIAL_CONDITION_SHIFTS`` and ``apply_material_condition_shift()``
with the same pattern.


See Also
--------

- :doc:`/reference/state-apparatus-ai` -- Complete API reference
- :doc:`/concepts/state-apparatus-ai` -- Design rationale and
  architecture
