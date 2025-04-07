from dataclasses import dataclass, field
from typing import Optional, Dict, List, Any

from babylon.core.entity import Entity


@dataclass
class Effect:
    """Represents an effect that can be applied to an entity."""

    target_id: str
    attribute: str
    operation: str  # "Increase" or "Decrease"
    magnitude: float
    description: str


@dataclass
class Contradiction:
    """Represents a dialectical contradiction in the game system."""

    id: str
    name: str
    description: str
    entities: List[Entity]
    universality: str  # "Universal" or "Particular"
    particularity: str  # Domain specific (e.g., "Economic", "Political")
    principal_contradiction: Optional["Contradiction"]
    principal_aspect: Entity
    secondary_aspect: Entity
    antagonism: str  # "Primary" or "Secondary"
    intensity: str  # "Low", "Medium", "High"
    state: str  # "Active", "Resolved", "Latent"
    potential_for_transformation: str  # "Low", "Medium", "High"
    conditions_for_transformation: List[str]
    resolution_methods: Dict[str, List[Effect]]
    attributes: Dict[str, any] = field(default_factory=dict)
    intensity_history: List[float] = field(default_factory=list)

    @property
    def intensity_value(self) -> float:
        """Get numerical value for intensity."""
        intensity_map = {"Low": 0.0, "Medium": 0.5, "High": 1.0}
        return intensity_map.get(self.intensity, 0.0)

    def update_intensity(self, game_state: Dict[str, Any]) -> None:
        """Update contradiction intensity based on game state.

        Args:
            game_state: Current game state containing relevant metrics
        """
        # Default implementation - can be overridden by specific contradiction types
        if self.particularity == "Economic":
            gini_coefficient = game_state.get("economy", {}).get("gini_coefficient", 0)
            if gini_coefficient >= 0.6:
                self.intensity = "High"
            elif gini_coefficient >= 0.4:
                self.intensity = "Medium"
            else:
                self.intensity = "Low"
        elif self.particularity == "Political":
            stability_index = game_state.get("politics", {}).get("stability_index", 1)
            if stability_index <= 0.2:
                self.intensity = "High"
            elif stability_index <= 0.3:
                self.intensity = "Medium"
            else:
                self.intensity = "Low"
        else:
            # Default to current intensity if no specific rules
            pass

        # Record intensity value in history
        self.intensity_history.append(self.intensity_value)
