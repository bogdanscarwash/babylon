from typing import Any

from babylon.core.entity import Entity
from babylon.metrics.collector import MetricsCollector


class EntityRegistry:
    """Registry to maintain references to all game entities."""

    def __init__(self, collection: Any = None):
        """Initialize a new EntityRegistry instance.

        Args:
            collection: ChromaDB collection for vector storage
        """
        self._entities: dict[str, Entity] = {}
        self.metrics = MetricsCollector()
        self.collection = collection

    def create_entity(self, type: str, role: str) -> Entity:
        """Create and register a new entity.

        Args:
            type: The type of entity to create
            role: The role of the entity

        Returns:
            The created Entity instance
        """
        entity = Entity(type=type, role=role)
        self._entities[entity.id] = entity

        if self.collection:
            # Add to ChromaDB with dummy embedding for testing
            self.collection.add(
                ids=[entity.id],
                embeddings=[[1.0] * 384],  # Dummy embedding
                metadatas={"type": type, "role": role},
            )

        return entity

    def update_entity(self, entity_id: str, **attributes) -> None:
        """Update an entity's attributes.

        Args:
            entity_id: ID of the entity to update
            **attributes: Attributes to update
        """
        if entity_id not in self._entities:
            raise ValueError(f"Entity {entity_id} not found")

        entity = self._entities[entity_id]
        for key, value in attributes.items():
            setattr(entity, key, value)

        if self.collection:
            # Update in ChromaDB
            self.collection.update(ids=[entity_id], metadatas={**attributes})

    def get_entity(self, entity_id: str) -> Entity | None:
        """Retrieve an entity by its ID.

        Args:
            entity_id: The unique identifier of the entity to retrieve

        Returns:
            The Entity instance if found, None if not found
        """
        if not entity_id:
            raise ValueError("Entity ID cannot be empty")

        entity = self._entities.get(entity_id)
        if entity:
            self.metrics.record_object_access(entity_id, "entity_registry")
        return entity

    def delete_entity(self, entity_id: str) -> None:
        """Delete an entity from the registry.

        Args:
            entity_id: ID of the entity to delete
        """
        if entity_id in self._entities:
            del self._entities[entity_id]

            if self.collection:
                # Delete from ChromaDB
                self.collection.delete(ids=[entity_id])
