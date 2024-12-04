"""Custom exceptions for the Babylon application.

This module defines a comprehensive hierarchy of custom exceptions used throughout 
the application. The hierarchy is designed to provide specific error types while 
maintaining a common base class for all Babylon-specific errors.
"""

from typing import Optional, Any, List

Exception Hierarchy:
    BabylonError                  # Root exception for all Babylon errors
    - DatabaseError               # Database operations and connectivity issues
    - EntityError                 # Base class for entity-related errors
        - EntityNotFoundError     # Entity lookup/access failures
        - EntityValidationError   # Entity data validation failures
    - ConfigurationError          # Configuration loading and validation issues
    - GameStateError             # Game state consistency and transition errors
    - BackupError                # Backup/restore operation failures

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
        - GAME_001 to GAME_099: Core game system errors
        - DB_001 to DB_099: Database operation errors
        - ENT_001 to ENT_099: Entity management errors
        - CFG_001 to CFG_099: Configuration errors
        - BACKUP_001 to BACKUP_099: Backup/restore errors
    """
    def __init__(self, message: str, error_code: str = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)

class DatabaseError(BabylonError):
    """Raised when database operations fail.
    
    Used for errors related to ChromaDB operations including:
    - Connection failures (DB_001 to DB_019)
    - Query execution errors (DB_020 to DB_039)
    - Data integrity issues (DB_040 to DB_059)
    - Backup/restore operations (DB_060 to DB_079)
    - Performance issues (DB_080 to DB_099)
    
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
    - Entity creation/deletion (ENT_001 to ENT_019)
    - Entity state management (ENT_020 to ENT_039)
    - Entity relationships (ENT_040 to ENT_059)
    - Entity validation (ENT_060 to ENT_079)
    - Entity persistence (ENT_080 to ENT_099)
    
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
    """Raised when an entity cannot be found in the registry.
    
    This exception is used for all entity lookup failures including:
    - Entity ID not found in registry (ENT_404)
    - Entity was previously deleted (ENT_410)
    - Reference to non-existent entity (ENT_450)
    - Entity type mismatch (ENT_460)
    
    Error Code Ranges:
        ENT_404 to ENT_409: Basic lookup failures
        ENT_410 to ENT_419: Deleted entity access
        ENT_450 to ENT_459: Reference resolution errors
        ENT_460 to ENT_469: Type/role mismatch errors
    
    Attributes:
        message (str): Detailed error description
        error_code (str): ENT_XXX format code
        entity_id (Optional[str]): ID of the entity that wasn't found
        
    Example:
        try:
            entity = registry.get_entity(entity_id)
            if not entity:
                raise EntityNotFoundError(
                    message=f"Entity {entity_id} not found in registry",
                    error_code="ENT_404",
                    entity_id=entity_id
                )
        except EntityNotFoundError as e:
            logger.error(f"Entity lookup failed: {e.message} ({e.error_code})")
            # Handle missing entity appropriately
            
    Common Error Codes:
        ENT_404: Entity not found in registry
        ENT_410: Attempted access to deleted entity
        ENT_450: Invalid entity reference
        ENT_460: Entity type mismatch
    """
    def __init__(self, message: str, error_code: str, entity_id: Optional[str] = None):
        super().__init__(message, error_code)
        self.entity_id = entity_id
    pass

class EntityValidationError(EntityError):
    """Raised when entity validation fails during creation or updates.
    
    This exception handles all validation failures including:
    - Data type validation (ENT_601 to ENT_619)
    - Value range validation (ENT_620 to ENT_639)
    - Required field validation (ENT_640 to ENT_659)
    - State transition validation (ENT_660 to ENT_679)
    - Relationship validation (ENT_680 to ENT_699)
    
    The validation system ensures entities maintain consistency with:
    - Data type constraints
    - Value range limits
    - Required attribute presence
    - Valid state transitions
    - Relationship integrity rules
    
    Attributes:
        message (str): Detailed validation error description
        error_code (str): ENT_XXX format code
        field_name (Optional[str]): Name of the invalid field
        current_value: Current invalid value
        allowed_values: List or description of allowed values
        
    Example:
        try:
            if not (0 <= power_value <= 100):
                raise EntityValidationError(
                    message="Power value must be between 0 and 100",
                    error_code="ENT_621",
                    field_name="power",
                    current_value=power_value,
                    allowed_values="0-100"
                )
        except EntityValidationError as e:
            logger.error(
                f"Validation failed: {e.message} "
                f"(Field: {e.field_name}, "
                f"Value: {e.current_value}, "
                f"Allowed: {e.allowed_values})"
            )
            
    Common Error Codes:
        ENT_601: Invalid data type
        ENT_621: Value out of range
        ENT_641: Missing required field
        ENT_661: Invalid state transition
        ENT_681: Invalid relationship
    """
    def __init__(self, message: str, error_code: str, field_name: Optional[str] = None,
                 current_value: Any = None, allowed_values: Any = None):
        super().__init__(message, error_code)
        self.field_name = field_name
        self.current_value = current_value
        self.allowed_values = allowed_values
    pass

class ConfigurationError(BabylonError):
    """Raised when configuration loading or validation fails.
    
    This exception handles all configuration-related errors including:
    - Environment variable issues (CFG_001 to CFG_019)
    - Configuration file problems (CFG_020 to CFG_039)
    - Value validation failures (CFG_040 to CFG_059)
    - Dependency configuration (CFG_060 to CFG_079)
    - Runtime reconfiguration (CFG_080 to CFG_099)
    
    The configuration system validates:
    - Required settings presence
    - Value type correctness
    - Value range constraints
    - Setting dependencies
    - Configuration consistency
    
    Attributes:
        message (str): Detailed configuration error description
        error_code (str): CFG_XXX format code
        setting_name (Optional[str]): Name of the problematic setting
        current_value (Optional[Any]): Current invalid value
        required_type (Optional[type]): Expected type for the setting
        
    Example:
        try:
            if 'SECRET_KEY' not in os.environ:
                raise ConfigurationError(
                    message="Missing required SECRET_KEY environment variable",
                    error_code="CFG_001",
                    setting_name="SECRET_KEY"
                )
        except ConfigurationError as e:
            logger.error(
                f"Configuration error: {e.message} "
                f"(Setting: {e.setting_name})"
            )
            sys.exit(1)
            
    Common Error Codes:
        CFG_001: Missing environment variable
        CFG_020: Configuration file not found
        CFG_040: Invalid setting value
        CFG_060: Missing dependency configuration
        CFG_080: Invalid runtime reconfiguration
    """
    def __init__(self, message: str, error_code: str, setting_name: Optional[str] = None,
                 current_value: Any = None, required_type: Optional[type] = None):
        super().__init__(message, error_code)
        self.setting_name = setting_name
        self.current_value = current_value
        self.required_type = required_type
    pass

class GameStateError(BabylonError):
    """Raised when game state consistency or transitions fail.
    
    This exception handles all game state errors including:
    - State consistency violations (GAME_001 to GAME_019)
    - Invalid state transitions (GAME_020 to GAME_039)
    - Rule enforcement failures (GAME_040 to GAME_059)
    - System synchronization issues (GAME_060 to GAME_079)
    - Resource management problems (GAME_080 to GAME_099)
    
    The game state system ensures:
    - State consistency across systems
    - Valid state transitions
    - Rule compliance
    - System synchronization
    - Resource integrity
    
    Attributes:
        message (str): Detailed state error description
        error_code (str): GAME_XXX format code
        current_state (Optional[str]): Current invalid state
        expected_state (Optional[str]): Expected valid state
        affected_systems (Optional[List[str]]): Impacted game systems
        
    Example:
        try:
            if not self._validate_state_transition(current_state, new_state):
                raise GameStateError(
                    message="Invalid state transition attempted",
                    error_code="GAME_021",
                    current_state=current_state,
                    expected_state=new_state,
                    affected_systems=['economy', 'politics']
                )
        except GameStateError as e:
            logger.error(
                f"Game state error: {e.message} "
                f"(Current: {e.current_state}, "
                f"Expected: {e.expected_state}, "
                f"Systems: {', '.join(e.affected_systems)})"
            )
            
    Common Error Codes:
        GAME_001: State consistency violation
        GAME_021: Invalid state transition
        GAME_041: Rule violation detected
        GAME_061: System synchronization failure
        GAME_081: Resource integrity error
    """
    def __init__(self, message: str, error_code: str, current_state: Optional[str] = None,
                 expected_state: Optional[str] = None, affected_systems: Optional[List[str]] = None):
        super().__init__(message, error_code)
        self.current_state = current_state
        self.expected_state = expected_state
        self.affected_systems = affected_systems or []
    pass

class BackupError(BabylonError):
    """Raised when backup or restore operations fail.
    
    This exception handles all backup/restore errors including:
    - Backup creation failures (BACKUP_001 to BACKUP_019)
    - Restore validation errors (BACKUP_020 to BACKUP_039)
    - Storage space issues (BACKUP_040 to BACKUP_059)
    - File system errors (BACKUP_060 to BACKUP_079)
    - Data integrity problems (BACKUP_080 to BACKUP_099)
    
    The backup system ensures:
    - Reliable state preservation
    - Data integrity verification
    - Space management
    - Atomic operations
    - Version control
    
    Attributes:
        message (str): Detailed backup error description
        error_code (str): BACKUP_XXX format code
        backup_path (Optional[str]): Path to backup location
        required_space (Optional[int]): Required storage space in bytes
        available_space (Optional[int]): Available storage space in bytes
        
    Example:
        try:
            if available_space < required_space:
                raise BackupError(
                    message="Insufficient disk space for backup",
                    error_code="BACKUP_041",
                    backup_path="/path/to/backup",
                    required_space=required_space,
                    available_space=available_space
                )
        except BackupError as e:
            logger.error(
                f"Backup failed: {e.message} "
                f"(Path: {e.backup_path}, "
                f"Required: {e.required_space}, "
                f"Available: {e.available_space})"
            )
            
    Common Error Codes:
        BACKUP_001: Backup creation failed
        BACKUP_021: Restore validation failed
        BACKUP_041: Insufficient storage space
        BACKUP_061: File system error
        BACKUP_081: Data integrity error
    """
    def __init__(self, message: str, error_code: str, backup_path: Optional[str] = None,
                 required_space: Optional[int] = None, available_space: Optional[int] = None):
        super().__init__(message, error_code)
        self.backup_path = backup_path
        self.required_space = required_space
        self.available_space = available_space
    pass
