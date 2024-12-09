from dataclasses import dataclass, field
from typing import Optional, Dict, List

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
    principal_contradiction: Optional['Contradiction']
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
