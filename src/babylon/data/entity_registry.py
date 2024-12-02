from typing import Dict, Optional
from babylon.entities.entity import Entity
from babylon.metrics.collector import MetricsCollector

class EntityRegistry:
    """Registry to maintain references to all game entities.
    
    The EntityRegistry serves as a central repository for all entities in the game,
    providing efficient lookup and management of entity instances. It integrates
    with the metrics collection system to track entity access patterns.
    
    Attributes:
        entities (Dict[str, Entity]): Dictionary mapping entity IDs to Entity instances
        metrics (MetricsCollector): Collector for tracking entity access metrics
    """
    
    def __init__(self):
        """Initialize a new EntityRegistry instance.
        
        Creates an empty registry and initializes the metrics collector for
        tracking entity access patterns.
        """
        self.entities: Dict[str, Entity] = {}
        self.metrics = MetricsCollector()
        
    def register_entity(self, entity: Entity) -> None:
        """Register an entity in the registry.
        
        Adds a new entity to the registry, making it available for lookup by ID.
        If an entity with the same ID already exists, it will be overwritten.
        
        Args:
            entity: The Entity instance to register
            
        Side Effects:
            - Adds or updates the entity in the entities dictionary
        """
        self.entities[entity.id] = entity
        
    def get_entity(self, entity_id: str) -> Optional[Entity]:
        """Retrieve an entity by its ID.
        
        Looks up and returns an entity from the registry by its ID.
        Records the access in the metrics collector for tracking.
        
        Args:
            entity_id: The unique identifier of the entity to retrieve
            
        Returns:
            The Entity instance if found, None if no entity exists with the given ID
            
        Side Effects:
            - Records the entity access in the metrics collector
        """
        entity = self.entities.get(entity_id)
        if entity:
            self.metrics.record_object_access(entity_id, "entity_registry")
        return entity
        
    def remove_entity(self, entity_id: str) -> None:
        """Remove an entity from the registry.
        
        Removes an entity from the registry if it exists.
        Silently ignores the request if the entity ID is not found.
        
        Args:
            entity_id: The unique identifier of the entity to remove
            
        Side Effects:
            - Removes the entity from the entities dictionary if it exists
        """
        if entity_id in self.entities:
            del self.entities[entity_id]
