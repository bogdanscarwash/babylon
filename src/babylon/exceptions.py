"""Custom exceptions for the Babylon application."""

class BabylonError(Exception):
    """Base exception class for all Babylon-specific errors."""
    def __init__(self, message: str, error_code: str = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)

class DatabaseError(BabylonError):
    """Raised when database operations fail."""
    pass

class EntityError(BabylonError):
    """Base class for entity-related errors."""
    pass

class EntityNotFoundError(EntityError):
    """Raised when an entity cannot be found."""
    pass

class EntityValidationError(EntityError):
    """Raised when entity validation fails."""
    pass

class ConfigurationError(BabylonError):
    """Raised when there are configuration issues."""
    pass

class GameStateError(BabylonError):
    """Raised when game state becomes invalid."""
    pass

class BackupError(BabylonError):
    """Raised when backup/restore operations fail."""
    pass
