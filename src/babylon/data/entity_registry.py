from typing import Dict, Optional
from babylon.entities.entity import Entity

class EntityRegistry:
    """Registry to maintain references to all game entities."""
    
    def __init__(self):
        self.entities: Dict[str, Entity] = {}
        
    def register_entity(self, entity: Entity) -> None:
        """Register an entity in the registry."""
        self.entities[entity.id] = entity
        
    def get_entity(self, entity_id: str) -> Optional[Entity]:
        """Retrieve an entity by its ID."""
        return self.entities.get(entity_id)
        
    def remove_entity(self, entity_id: str) -> None:
        """Remove an entity from the registry."""
        if entity_id in self.entities:
            del self.entities[entity_id]
