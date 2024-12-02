from __future__ import annotations
from typing import Any, Callable, Dict, List, Optional, Union

class Entity:
    def __init__(
        self,
        entity_id: str,
        entity_type: str,
        role: str
    ) -> None:
        self.entity_id: str = entity_id
        self.entity_type: str = entity_type
        self.role: str = role

class Effect:
    def __init__(
        self,
        target: Any,
        attribute: str,
        modification_type: str,
        value: Union[int, float],
        description: str
    ) -> None:
        self.target: Any = target
        self.attribute: str = attribute
        self.modification_type: str = modification_type
        self.value: Union[int, float] = value
        self.description: str = description

    def apply(self, game_state: Dict[str, Any]) -> None:
        pass

class Attribute:
    def __init__(
        self,
        name: str,
        value: Any
    ) -> None:
        self.name: str = name
        self.value: Any = value

class Contradiction:
    MAX_INTENSITY: float = 100.0

    def __init__(
        self,
        id: str,
        name: str,
        description: str,
        entities: List[Entity],
        universality: str,
        particularity: str,
        principal_contradiction: Optional[Contradiction],
        principal_aspect: Entity,
        secondary_aspect: Entity,
        antagonism: str,
        intensity: str,
        state: str,
        potential_for_transformation: str,
        conditions_for_transformation: List[str],
        resolution_methods: Dict[str, List[Effect]],
        attributes: Dict[str, Any] = {},
        selected_resolution_method: Optional[str] = None,
        intensity_value: float = 0.0,
        intensity_history: Optional[List[float]] = None,
        update_intensity: Optional[Callable[[Contradiction, Dict[str, Any]], None]] = None
    ) -> None:
        self.id: str = id
        self.name: str = name
        self.description: str = description
        self.entities: List[Entity] = entities
        self.universality: str = universality
        self.particularity: str = particularity
        self.principal_contradiction: Optional[Contradiction] = principal_contradiction
        self.principal_aspect: str = principal_aspect
        self.secondary_aspect: str = secondary_aspect
        self.antagonism: str = antagonism
        self.intensity: str = intensity
        self.state: str = state
        self.potential_for_transformation: str = potential_for_transformation
        self.conditions_for_transformation: List[str] = conditions_for_transformation
        self.resolution_methods: Dict[str, List[Effect]] = resolution_methods
        self.attributes: Dict[str, Any] = attributes
        self.selected_resolution_method: Optional[str] = selected_resolution_method
        self.intensity_value: float = intensity_value
        self.intensity_history: List[float] = intensity_history if intensity_history is not None else []
        self.update_intensity: Optional[Callable[[Contradiction, Dict[str, Any]], None]] = update_intensity

    def is_resolvable(self) -> bool:
        return self.potential_for_transformation > 0.5

    def transform(self, new_state: str) -> None:
        self.state = new_state
