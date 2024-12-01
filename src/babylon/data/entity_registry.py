from babylon.config import Config


class EntityRegistry:
    """Registry to maintain references to all game entities."""
    
    def __init__(self):
        self.entities = {}
        
    def register_entity(self, entity):
        """Register an entity in the registry."""
        self.entities[entity.id] = entity
        
    def get_entity(self, entity_id):
        """Retrieve an entity by its ID."""
        return self.entities.get(entity_id)
        
    def remove_entity(self, entity_id):
        """Remove an entity from the registry."""
        if entity_id in self.entities:
            del self.entities[entity_id]
