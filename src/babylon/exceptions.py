"""Custom exceptions for the Babylon application.

This module defines a comprehensive hierarchy of custom exceptions used throughout 
the application. The hierarchy is designed to provide specific error types while 
maintaining a common base class for all Babylon-specific errors.

Exception Hierarchy:
    BabylonError                  # Root exception for all Babylon errors
    ├── DatabaseError            # Database operations and connectivity issues
    ├── EntityError             # Base class for entity-related errors
    │   ├── EntityNotFoundError    # Entity lookup/access failures
    │   └── EntityValidationError  # Entity data validation failures
    ├── ConfigurationError      # Configuration loading and validation issues
    ├── GameStateError         # Game state consistency and transition errors
    └── BackupError           # Backup/restore operation failures

Each exception includes:
- message: A human-readable error description
- error_code: A machine-readable error code (e.g., "DB_001")
- Additional context through inheritance and stack traces

Usage Example:
    try:
        entity = registry.get_entity(entity_id)
        if not entity:
            raise EntityNotFoundError(
                message=f"Entity {entity_id} not found",
                error_code="ENT_404"
            )
    except EntityNotFoundError as e:
        logger.error(f"Entity lookup failed: {e.message} ({e.error_code})")
        # Handle the error appropriately

Error Code Convention:
    - DB_XXX: Database-related errors
    - ENT_XXX: Entity-related errors
    - CFG_XXX: Configuration errors
    - GAME_XXX: Game state errors
    - BACKUP_XXX: Backup/restore errors

Integration with Logging:
    All exceptions integrate with the logging system to provide:
    - Structured error information
    - Error codes for filtering and analysis
    - Stack traces for debugging
    - Correlation IDs for request tracking
"""

class BabylonError(Exception):
    """Base exception class for all Babylon-specific errors.
    
    This class serves as the root of the Babylon exception hierarchy.
    All other Babylon-specific exceptions inherit from this class.
    
    Attributes:
        message (str): Human-readable error description
        error_code (Optional[str]): Machine-readable error code (e.g., "DB_001")
        
    Example:
        raise BabylonError("Failed to initialize game state", "GAME_001")
    """
    def __init__(self, message: str, error_code: str = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)

class DatabaseError(BabylonError):
    """Raised when database operations fail.
    
    Used for errors related to:
    - Connection failures
    - Query execution errors
    - Data integrity issues
    - Backup/restore operations
    
    Example:
        raise DatabaseError("Failed to connect to ChromaDB", "DB_001")
    """
    pass

class EntityError(BabylonError):
    """Base class for entity-related errors.
    
    Parent class for all exceptions related to game entities:
    - Entity creation/deletion
    - Entity state management
    - Entity relationships
    """
    pass

class EntityNotFoundError(EntityError):
    """Raised when an entity cannot be found.
    
    Used when:
    - Looking up entities by ID
    - Accessing deleted entities
    - Resolving entity references
    
    Example:
        raise EntityNotFoundError(f"Entity {entity_id} not found", "ENT_404")
    """
    pass

class EntityValidationError(EntityError):
    """Raised when entity validation fails.
    
    Used for:
    - Invalid attribute values
    - Missing required fields
    - Constraint violations
    - State transition errors
    
    Example:
        raise EntityValidationError("Invalid power value: must be 0-100", "ENT_VAL_001")
    """
    pass

class ConfigurationError(BabylonError):
    """Raised when there are configuration issues.
    
    Used for:
    - Missing environment variables
    - Invalid configuration values
    - Configuration conflicts
    - Initialization failures
    
    Example:
        raise ConfigurationError("Missing required SECRET_KEY", "CFG_001")
    """
    pass

class GameStateError(BabylonError):
    """Raised when game state becomes invalid.
    
    Used for:
    - Inconsistent game state
    - Invalid state transitions
    - Rule violations
    - System synchronization errors
    
    Example:
        raise GameStateError("Invalid event sequence detected", "GAME_002")
    """
    pass

class BackupError(BabylonError):
    """Raised when backup/restore operations fail.
    
    Used for:
    - Backup creation failures
    - Restore validation errors
    - Disk space issues
    - File system errors
    
    Example:
        raise BackupError("Insufficient disk space for backup", "BACKUP_001")
    """
    pass
