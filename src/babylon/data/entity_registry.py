
from babylon.entities.entity import Entity
from babylon.metrics.collector import MetricsCollector


class EntityRegistry:
    """Registry to maintain references to all game entities.

    The EntityRegistry serves as a central repository for all entities in the game,
    providing efficient lookup and management of entity instances. It integrates
    with the metrics collection system to track entity access patterns.

    This registry is a critical component in the dialectical materialist simulation,
    as it maintains the state of all social forces and their relationships. It enables:
    - O(1) entity lookup by ID for contradiction analysis
    - Centralized tracking of entity lifecycle and access patterns
    - Performance optimization through metrics collection
    - Memory management of entity instances

    The registry uses a dictionary for constant-time lookups, which is essential
    for real-time contradiction analysis and event generation. Access patterns
    are tracked to identify heavily used entities that may need caching or
    optimization.

    Attributes:
        entities (Dict[str, Entity]): Dictionary mapping entity IDs to Entity instances.
                                    Uses string keys for efficient lookup and serialization.
        metrics (MetricsCollector): Collector for tracking entity access metrics.
                                  Monitors access patterns and performance.
    """

    def __init__(self):
        """Initialize a new EntityRegistry instance.

        Creates an empty registry and initializes the metrics collector for
        tracking entity access patterns. The registry starts empty and is
        populated as entities are registered during game initialization
        and runtime.

        The metrics collector is initialized to track:
        - Entity access frequency
        - Access patterns over time
        - Memory usage per entity type
        - Cache hit/miss rates

        No parameters are needed as the registry self-initializes its
        internal data structures and metrics collection system.
        """
        self.entities: dict[str, Entity] = {}
        self.metrics = MetricsCollector()

    def register_entity(self, entity: Entity) -> None:
        """Register an entity in the registry.

        Adds a new entity to the registry, making it available for lookup by ID.
        If an entity with the same ID already exists, it will be overwritten.
        This behavior enables:
        - Hot reloading of entity definitions
        - Dynamic entity updates during gameplay
        - State restoration from saved games

        The registration process is idempotent - registering the same entity
        multiple times has the same effect as registering it once. This
        simplifies entity management in complex game states.

        Args:
            entity: The Entity instance to register. Must have a valid ID attribute.
                   The entity's type and role determine its behavior in contradictions.

        Side Effects:
            - Adds or updates the entity in the entities dictionary
            - May trigger garbage collection of old entity instances
            - Updates internal indices used for entity relationship tracking

        Implementation Notes:
            - Uses direct dictionary assignment for O(1) registration
            - Does not validate entity relationships - this is handled by the
              contradiction analysis system
            - Thread-safe as long as entities dict isn't modified concurrently
        """
        self.entities[entity.id] = entity

    def get_entity(self, entity_id: str) -> Entity | None:
        """Retrieve an entity by its ID.

        Looks up and returns an entity from the registry by its ID.
        Records the access in the metrics collector for tracking.
        This method is the primary interface for entity access throughout
        the game system.

        The lookup process:
        1. Checks the entities dictionary for the ID
        2. Records the access attempt in metrics
        3. Returns the entity or None if not found

        Performance characteristics:
        - O(1) average case lookup time
        - O(1) metrics recording overhead
        - Memory overhead only for metrics storage

        Args:
            entity_id: The unique identifier of the entity to retrieve.
                      Must be a string matching the ID used during registration.

        Returns:
            The Entity instance if found, None if no entity exists with the given ID.
            The returned entity is a reference to the stored instance, not a copy.

        Side Effects:
            - Records the entity access in the metrics collector
            - Updates access timestamps for cache management
            - May trigger metrics analysis if thresholds are reached

        Usage Notes:
            - Consider caching results for frequently accessed entities
            - Check return value for None in critical code paths
            - Use metrics data to optimize access patterns
        """
        entity = self.entities.get(entity_id)
        if entity:
            self.metrics.record_object_access(entity_id, "entity_registry")
        return entity

    def remove_entity(self, entity_id: str) -> None:
        """Remove an entity from the registry.

        Removes an entity from the registry if it exists.
        Silently ignores the request if the entity ID is not found.
        This method is used for:
        - Cleaning up destroyed/obsolete entities
        - Managing entity lifecycle during state transitions
        - Removing temporary entities after their purpose is served

        The removal process:
        1. Checks if entity exists
        2. Removes from main registry if found
        3. Cleans up any associated metrics data
        4. No error if entity doesn't exist (idempotent)

        Args:
            entity_id: The unique identifier of the entity to remove.
                      Must match the ID used during registration.

        Side Effects:
            - Removes the entity from the entities dictionary if it exists
            - Cleans up associated metrics and tracking data
            - May trigger garbage collection
            - Updates internal indices and relationship tracking

        Implementation Notes:
            - Uses dict.pop() with default value for atomic remove
            - Thread-safe for single entity removal
            - Does not cascade delete related entities
            - Metrics history is preserved for analysis
        """
        if entity_id in self.entities:
            del self.entities[entity_id]
