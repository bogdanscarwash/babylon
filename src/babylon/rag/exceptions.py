"""Custom exceptions for the RAG system.

Defines RAG-specific exceptions that integrate with the main Babylon error hierarchy.
See ERROR_CODES.md for detailed documentation of error code ranges and usage.
"""

from babylon.utils.exceptions import BabylonError


class RagError(BabylonError):
    """Base class for RAG system errors.

    Error Code Ranges:
    - RAG_001 to RAG_099: General RAG system errors
    - RAG_100 to RAG_199: Lifecycle management errors
    - RAG_200 to RAG_299: Embedding errors
    - RAG_300 to RAG_399: Query/retrieval errors
    - RAG_400 to RAG_499: Pre-embeddings errors
    """

    pass


class LifecycleError(RagError):
    """Base class for lifecycle management errors.

    Error Code Ranges:
    - RAG_100 to RAG_119: Object validation errors
    - RAG_120 to RAG_139: State transition errors
    - RAG_140 to RAG_159: Memory management errors
    - RAG_160 to RAG_179: Cache consistency errors
    """

    pass


class InvalidObjectError(LifecycleError):
    """Error raised when an object is invalid or missing required attributes.

    Common Error Codes:
    - RAG_101: Missing required attribute
    - RAG_102: Invalid attribute type
    - RAG_103: Invalid object state
    - RAG_104: Object validation failed
    """

    def __init__(
        self,
        message: str,
        error_code: str = "RAG_101",
        field_name: str | None = None,
        current_value: any = None,
    ) -> None:
        self.field_name = field_name
        self.current_value = current_value
        details = {"field_name": field_name, "current_value": str(current_value)}
        super().__init__(message, error_code, details)


class StateTransitionError(LifecycleError):
    """Error raised when an invalid state transition is attempted.

    Common Error Codes:
    - RAG_121: Invalid state transition
    - RAG_122: State transition not allowed
    - RAG_123: Object in wrong state
    - RAG_124: State transition validation failed
    """

    def __init__(
        self,
        message: str,
        error_code: str = "RAG_121",
        current_state: str | None = None,
        target_state: str | None = None,
    ) -> None:
        self.current_state = current_state
        self.target_state = target_state
        details = {"current_state": current_state, "target_state": target_state}
        super().__init__(message, error_code, details)


class CorruptStateError(LifecycleError):
    """Error raised when internal state corruption is detected.

    Common Error Codes:
    - RAG_161: Object in multiple contexts
    - RAG_162: Inconsistent state detected
    - RAG_163: Cache corruption
    - RAG_164: Reference integrity violation
    """

    def __init__(
        self,
        message: str,
        error_code: str = "RAG_161",
        affected_objects: list[str] | None = None,
    ) -> None:
        self.affected_objects = affected_objects or []
        details = {"affected_objects": self.affected_objects}
        super().__init__(message, error_code, details)


class PreEmbeddingError(RagError):
    """Base class for pre-embedding errors.
    
    Error Code Ranges:
    - RAG_400 to RAG_419: Preprocessing errors
    - RAG_420 to RAG_439: Chunking errors
    - RAG_440 to RAG_459: Cache management errors
    """
    
    pass


class PreprocessingError(PreEmbeddingError):
    """Error raised during content preprocessing.
    
    Common Error Codes:
    - RAG_401: Content too short
    - RAG_402: Content too long
    - RAG_403: Invalid content format
    - RAG_404: Language detection failed
    """
    
    def __init__(
        self,
        message: str,
        error_code: str = "RAG_401",
        content_id: str | None = None,
    ) -> None:
        self.content_id = content_id
        details = {"content_id": content_id}
        super().__init__(message, error_code, details)


class ChunkingError(PreEmbeddingError):
    """Error raised during content chunking.
    
    Common Error Codes:
    - RAG_421: Empty content
    - RAG_422: Invalid chunk size
    - RAG_423: Invalid chunking strategy
    - RAG_424: Chunking failed
    """
    
    def __init__(
        self,
        message: str,
        error_code: str = "RAG_421",
        content_id: str | None = None,
    ) -> None:
        self.content_id = content_id
        details = {"content_id": content_id}
        super().__init__(message, error_code, details)
