"""Custom exceptions for metrics subsystem."""

class MetricsError(Exception):
    """Base exception for metrics-related errors."""
    pass

class DatabaseConnectionError(MetricsError):
    """Raised when database connection fails."""
    pass

class MetricsPersistenceError(MetricsError):
    """Raised when metrics cannot be saved/loaded."""
    pass

class LogRotationError(MetricsError):
    """Raised when log rotation fails."""
    pass
