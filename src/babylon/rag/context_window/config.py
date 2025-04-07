from dataclasses import dataclass
from babylon.config.base import BaseConfig

@dataclass
class ContextWindowConfig:
    """Configuration for the Context Window Management system.
    
    Attributes:
        max_token_limit: Maximum number of tokens allowed in the context window
        capacity_threshold: Percentage of capacity at which optimization should trigger
        prioritization_strategy: Strategy for content prioritization (relevance, recency, hybrid)
        min_content_importance: Minimum importance score for content to be kept in context
    """
    max_token_limit: int = 150000  # Default to 150k tokens
    capacity_threshold: float = 0.75  # Default to 75% capacity
    prioritization_strategy: str = "hybrid"  # Options: relevance, recency, hybrid
    min_content_importance: float = 0.2  # Minimum importance score to keep content
    
    @classmethod
    def from_base_config(cls) -> 'ContextWindowConfig':
        """Create context window config from BaseConfig."""
        return cls(
            max_token_limit=getattr(BaseConfig, 'CONTEXT_WINDOW_MAX_TOKENS', 150000),
            capacity_threshold=getattr(BaseConfig, 'CONTEXT_WINDOW_THRESHOLD', 0.75),
            prioritization_strategy=getattr(BaseConfig, 'CONTEXT_WINDOW_PRIORITY_STRATEGY', "hybrid"),
            min_content_importance=getattr(BaseConfig, 'CONTEXT_WINDOW_MIN_IMPORTANCE', 0.2),
        )
