from typing import Any


class Politics:
    """
    Represents the political system and its various components.
    Handles political simulation including power relations, factions, and stability.
    """

    def __init__(self) -> None:
        """Initialize the political system with default values."""
        # Core political indicators
        self.stability: float = 0.5  # Political stability (0.0 to 1.0)
        self.legitimacy: float = 0.7  # Regime legitimacy (0.0 to 1.0)
        self.democracy_index: float = 0.6  # Level of democratic institutions
        self.repression_level: float = 0.3  # State repression level
        
        # Power relations
        self.power_relations: dict[str, float] = {
            'ruling_class': 0.8,  # Dominant class power
            'working_class': 0.3,  # Working class power
            'military': 0.6,  # Military influence
            'bureaucracy': 0.5  # State bureaucracy power
        }
        
        # Political dynamics
        self.class_consciousness: float = 0.4  # Working class consciousness
        self.social_cohesion: float = 0.6  # Social unity level
        self.resistance_potential: float = 0.3  # Potential for organized resistance
        
        # Active policies and their effects
        self.active_policies: dict[str, dict[str, Any]] = {}

    def update(self) -> None:
        """
        Update the political state based on current conditions and events.
        This method should be called each game tick to simulate political changes.
        """
        self._update_stability()
        self._update_power_relations()
        self._update_class_dynamics()
        self._evaluate_policies()

    def _update_stability(self) -> None:
        """Update political stability based on current conditions."""
        # Calculate stability factors
        legitimacy_factor = self.legitimacy * 0.3
        democracy_factor = self.democracy_index * 0.2
        repression_effect = -self.repression_level * 0.2
        resistance_effect = -self.resistance_potential * 0.3
        
        # Update stability
        stability_change = (
            legitimacy_factor +
            democracy_factor +
            repression_effect +
            resistance_effect
        )
        
        self.stability = max(0.0, min(1.0, self.stability + stability_change * 0.1))

    def _update_power_relations(self) -> None:
        """Process changes in power relations between political actors."""
        # Update working class power based on consciousness
        consciousness_effect = self.class_consciousness * 0.1
        self.power_relations['working_class'] = max(0.0, min(1.0,
            self.power_relations['working_class'] + consciousness_effect
        ))
        
        # Update ruling class power based on economic control
        self.power_relations['ruling_class'] = max(0.0, min(1.0,
            self.power_relations['ruling_class'] - consciousness_effect * 0.5
        ))
        
        # Military influence affects stability
        military_effect = (self.power_relations['military'] - 0.5) * 0.1
        self.stability = max(0.0, min(1.0, self.stability + military_effect))

    def _update_class_dynamics(self) -> None:
        """Update class consciousness and resistance potential."""
        # Class consciousness grows with inequality and repression
        consciousness_growth = (
            self.repression_level * 0.2 +
            (1 - self.legitimacy) * 0.1
        )
        self.class_consciousness = max(0.0, min(1.0,
            self.class_consciousness + consciousness_growth * 0.1
        ))
        
        # Resistance potential based on consciousness and repression
        self.resistance_potential = (
            self.class_consciousness * 0.6 +
            self.repression_level * 0.4
        ) * (1 - self.legitimacy)

    def _evaluate_policies(self) -> None:
        """Evaluate the effects of current political policies."""
        for policy_id, policy in self.active_policies.items():
            if policy['type'] == 'reform':
                self.legitimacy += policy.get('legitimacy_effect', 0) * 0.1
                self.democracy_index += policy.get('democracy_effect', 0) * 0.1
            elif policy['type'] == 'repression':
                self.repression_level += policy.get('repression_effect', 0) * 0.1
                self.legitimacy -= policy.get('legitimacy_cost', 0) * 0.1

        # Ensure values stay within bounds
        self.legitimacy = max(0.0, min(1.0, self.legitimacy))
        self.democracy_index = max(0.0, min(1.0, self.democracy_index))
        self.repression_level = max(0.0, min(1.0, self.repression_level))

    def add_faction(self, name: str, initial_power: float) -> None:
        """
        Add a new political faction to the system.

        Args:
            name: Name of the faction
            initial_power: Initial power level (0.0 to 1.0)
        """
        if name not in self.power_relations:
            self.power_relations[name] = max(0.0, min(1.0, initial_power))

    def implement_policy(self, policy: dict[str, Any]) -> None:
        """
        Implement a new political policy.

        Args:
            policy: Dictionary containing policy parameters and effects
        """
        policy_id = policy.get('id')
        if policy_id and policy_id not in self.active_policies:
            self.active_policies[policy_id] = policy
            
            # Immediate effects
            if 'immediate_effects' in policy:
                effects = policy['immediate_effects']
                self.stability += effects.get('stability_effect', 0)
                self.legitimacy += effects.get('legitimacy_effect', 0)
                self.democracy_index += effects.get('democracy_effect', 0)

    def calculate_unrest(self) -> float:
        """
        Calculate current level of political unrest.

        Returns:
            float: Unrest level (0.0 to 1.0)
        """
        return (
            (1 - self.stability) * 0.4 +
            (1 - self.legitimacy) * 0.3 +
            self.resistance_potential * 0.3
        )

    def get_political_state(self) -> dict[str, Any]:
        """
        Get current political state indicators.

        Returns:
            dict[str, Any]: Dictionary of current political indicators
        """
        return {
            'stability': self.stability,
            'legitimacy': self.legitimacy,
            'democracy_index': self.democracy_index,
            'repression_level': self.repression_level,
            'class_consciousness': self.class_consciousness,
            'resistance_potential': self.resistance_potential,
            'power_relations': self.power_relations.copy(),
            'unrest_level': self.calculate_unrest()
        }
