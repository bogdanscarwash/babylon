"""Error codes for Context Window Management (2100-2199)."""

from typing import Optional

CONTEXT_WINDOW_ERROR = 2100
TOKEN_COUNT_ERROR = 2101
CAPACITY_EXCEEDED_ERROR = 2102
OPTIMIZATION_FAILED_ERROR = 2103

CONTENT_PRIORITY_ERROR = 2110
CONTENT_REMOVAL_ERROR = 2111
CONTENT_INSERTION_ERROR = 2112

LIFECYCLE_INTEGRATION_ERROR = 2120
METRICS_COLLECTION_ERROR = 2121
CONFIGURATION_ERROR = 2122

class ContextWindowError(Exception):
    """Base exception for context window errors."""
    code = CONTEXT_WINDOW_ERROR
    
    def __init__(self, message: str, code: Optional[int] = None):
        self.code = code or self.__class__.code
        super().__init__(f"Error {self.code}: {message}")


class TokenCountError(ContextWindowError):
    """Error in token counting."""
    code = TOKEN_COUNT_ERROR


class CapacityExceededError(ContextWindowError):
    """Context window capacity exceeded."""
    code = CAPACITY_EXCEEDED_ERROR


class OptimizationFailedError(ContextWindowError):
    """Failed to optimize context window."""
    code = OPTIMIZATION_FAILED_ERROR


class ContentPriorityError(ContextWindowError):
    """Error in content prioritization."""
    code = CONTENT_PRIORITY_ERROR


class ContentRemovalError(ContextWindowError):
    """Error removing content from context window."""
    code = CONTENT_REMOVAL_ERROR


class ContentInsertionError(ContextWindowError):
    """Error inserting content into context window."""
    code = CONTENT_INSERTION_ERROR


class LifecycleIntegrationError(ContextWindowError):
    """Error integrating with lifecycle manager."""
    code = LIFECYCLE_INTEGRATION_ERROR


class MetricsCollectionError(ContextWindowError):
    """Error collecting metrics."""
    code = METRICS_COLLECTION_ERROR


class ConfigurationError(ContextWindowError):
    """Error in configuration."""
    code = CONFIGURATION_ERROR
