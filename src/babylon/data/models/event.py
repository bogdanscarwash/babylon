"""Event model for handling game events and their effects."""

from typing import Any, List, Optional
from sqlalchemy import Column, Integer, String, JSON, ForeignKey
from sqlalchemy.orm import relationship
from babylon.data.database import Base


class Event(Base):
    """
    Represents a game event that can occur based on certain conditions.
    Events can have effects, triggers, and consequences.
    """
    __tablename__ = 'events'
    __table_args__ = {'extend_existing': True}  # Added to handle table redefinition

    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False)
    description = Column(String)
    type = Column(String)  # e.g., 'economic', 'political', 'social'
    effects = Column(JSON)  # List of effects when event occurs
    triggers = Column(JSON)  # Conditions that trigger the event
    consequences = Column(JSON)  # Follow-up events or effects
    escalation_paths = Column(JSON)  # Possible event escalations

    def __init__(
        self,
        name: str,
        description: str,
        event_type: str,
        effects: Optional[List[dict[str, Any]]] = None,
        triggers: Optional[List[dict[str, Any]]] = None,
        consequences: Optional[List[dict[str, Any]]] = None,
        escalation_paths: Optional[List[dict[str, Any]]] = None
    ) -> None:
        """
        Initialize a new event.

        Args:
            name: Name of the event
            description: Detailed description of the event
            event_type: Type of event (economic, political, social, etc.)
            effects: List of effects this event has when triggered
            triggers: List of conditions that trigger this event
            consequences: List of follow-up events or effects
            escalation_paths: List of possible event escalations
        """
        super().__init__()
        self.name = name
        self.description = description
        self.type = event_type
        self.effects = effects or []
        self.triggers = triggers or []
        self.consequences = consequences or []
        self.escalation_paths = escalation_paths or []

    def apply_effects(self, game_state: dict[str, Any]) -> None:
        """
        Apply all effects of this event to the game state.

        Args:
            game_state: Current game state to modify
        """
        for effect in self.effects:
            effect_type = effect.get('type')
            if effect_type == 'economic':
                self._apply_economic_effect(effect, game_state)
            elif effect_type == 'political':
                self._apply_political_effect(effect, game_state)
            elif effect_type == 'social':
                self._apply_social_effect(effect, game_state)

    def _apply_economic_effect(self, effect: dict[str, Any], game_state: dict[str, Any]) -> None:
        """Apply economic effects to the game state."""
        if 'economy' in game_state:
            economy = game_state['economy']
            if 'gdp_change' in effect:
                economy.gdp += effect['gdp_change']
            if 'unemployment_change' in effect:
                economy.unemployment_rate += effect['unemployment_change']

    def _apply_political_effect(self, effect: dict[str, Any], game_state: dict[str, Any]) -> None:
        """Apply political effects to the game state."""
        if 'politics' in game_state:
            politics = game_state['politics']
            if 'stability_change' in effect:
                politics.stability += effect['stability_change']
            if 'policy_changes' in effect:
                for policy in effect['policy_changes']:
                    politics.implement_policy(policy)

    def _apply_social_effect(self, effect: dict[str, Any], game_state: dict[str, Any]) -> None:
        """Apply social effects to the game state."""
        # TODO: Implement social effects when social systems are added
        pass

    def check_triggers(self, game_state: dict[str, Any]) -> bool:
        """
        Check if this event's triggers are met in the current game state.

        Args:
            game_state: Current game state to check against

        Returns:
            bool: True if all triggers are met, False otherwise
        """
        return all(self._evaluate_trigger(trigger, game_state) for trigger in self.triggers)

    def _evaluate_trigger(self, trigger: dict[str, Any], game_state: dict[str, Any]) -> bool:
        """
        Evaluate a single trigger condition.

        Args:
            trigger: Trigger condition to evaluate
            game_state: Current game state to check against

        Returns:
            bool: True if the trigger condition is met, False otherwise
        """
        trigger_type = trigger.get('type')
        if trigger_type == 'economic':
            return self._evaluate_economic_trigger(trigger, game_state)
        elif trigger_type == 'political':
            return self._evaluate_political_trigger(trigger, game_state)
        return False

    def _evaluate_economic_trigger(self, trigger: dict[str, Any], game_state: dict[str, Any]) -> bool:
        """Evaluate economic trigger conditions."""
        if 'economy' not in game_state:
            return False
        economy = game_state['economy']
        condition = trigger.get('condition', {})
        
        if 'min_gdp' in condition and economy.gdp < condition['min_gdp']:
            return False
        if 'max_unemployment' in condition and economy.unemployment_rate > condition['max_unemployment']:
            return False
        return True

    def _evaluate_political_trigger(self, trigger: dict[str, Any], game_state: dict[str, Any]) -> bool:
        """Evaluate political trigger conditions."""
        if 'politics' not in game_state:
            return False
        politics = game_state['politics']
        condition = trigger.get('condition', {})
        
        if 'min_stability' in condition and politics.stability < condition['min_stability']:
            return False
        if 'required_policies' in condition:
            required_policies = set(condition['required_policies'])
            active_policies = set(politics.active_policies.keys())
            if not required_policies.issubset(active_policies):
                return False
        return True
