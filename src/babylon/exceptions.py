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
    All other Babylon-specific exceptions inherit from this class to provide
    consistent error handling and reporting throughout the application.
    
    The error code system follows a standardized pattern:
    - First 2-4 letters indicate the subsystem (e.g., DB, ENT, CFG)
    - Underscore separator
    - 3 digits for specific error type
    
    Attributes:
        message (str): Human-readable error description
        error_code (Optional[str]): Machine-readable error code (e.g., "DB_001")
        
    Example:
        try:
            raise BabylonError(
                message="Failed to initialize game state",
                error_code="GAME_001"
            )
        except BabylonError as e:
            logger.error(f"{e.error_code}: {e.message}")
            
    Error Code Ranges:
        - GAME_001-099: Core game system errors
        - DB_001-099: Database operation errors
        - ENT_001-099: Entity management errors
        - CFG_001-099: Configuration errors
        - BACKUP_001-099: Backup/restore errors
    """
    def __init__(self, message: str, error_code: str = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)

class DatabaseError(BabylonError):
    """Raised when database operations fail.
    
    Used for errors related to ChromaDB operations including:
    - Connection failures (DB_001-019)
    - Query execution errors (DB_020-039)
    - Data integrity issues (DB_040-059)
    - Backup/restore operations (DB_060-079)
    - Performance issues (DB_080-099)
    
    Attributes:
        message (str): Detailed error description
        error_code (str): DB_XXX format code
        
    Example:
        try:
            result = chroma_client.query(...)
        except Exception as e:
            raise DatabaseError(
                message=f"Query failed: {str(e)}",
                error_code="DB_021"
            )
            
    Common Error Codes:
        DB_001: Connection failed
        DB_002: Authentication failed
        DB_020: Query syntax error
        DB_040: Data corruption detected
        DB_060: Backup creation failed
    """
    pass

class EntityError(BabylonError):
    """Base class for entity-related errors.
    
    Parent class for all exceptions related to game entities and their lifecycle:
    - Entity creation/deletion (ENT_001-019)
    - Entity state management (ENT_020-039)
    - Entity relationships (ENT_040-059)
    - Entity validation (ENT_060-079)
    - Entity persistence (ENT_080-099)
    
    This class provides common functionality for entity-specific errors while
    maintaining the error code hierarchy. Child classes should use appropriate
    error code ranges from the ENT_XXX namespace.
    
    Example:
        try:
            entity.update_state(new_state)
        except ValidationError as e:
            raise EntityError(
                message=f"Invalid entity state: {e}",
                error_code="ENT_022"
            )
            
    Common Error Codes:
        ENT_001: Creation failed
        ENT_020: Invalid state transition
        ENT_040: Invalid relationship
        ENT_060: Validation failed
        ENT_080: Persistence failed
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
