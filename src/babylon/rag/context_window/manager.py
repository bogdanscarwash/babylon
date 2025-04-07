from collections import Counter, defaultdict
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple, Union
import logging
from heapq import heappush, heappop

from babylon.metrics.collector import MetricsCollector
from babylon.rag.context_window.config import ContextWindowConfig
from babylon.rag.context_window.errors import (
    CapacityExceededError, ContentInsertionError, ContentPriorityError, 
    ContentRemovalError, OptimizationFailedError, TokenCountError
)

class ContextWindowManager:
    """Manages the token usage and content prioritization in the RAG context window.
    
    The ContextWindowManager ensures that the total token usage stays within the limits
    of the AI model while prioritizing the most relevant content. It implements:
    
    1. Token counting and tracking
    2. Content prioritization based on relevance
    3. Automatic optimization when approaching limits
    4. Integration with metrics collection
    
    Attributes:
        config: Configuration for the context window
        metrics_collector: Collector for performance metrics
        lifecycle_manager: Optional manager for object lifecycles
    """
    
    def __init__(
        self, 
        config: Optional[ContextWindowConfig] = None,
        metrics_collector: Optional[MetricsCollector] = None,
        lifecycle_manager: Any = None
    ):
        """Initialize the context window manager.
        
        Args:
            config: Configuration for token limits and optimization thresholds
            metrics_collector: Collector for performance metrics
            lifecycle_manager: Optional manager for object lifecycles
        """
        self.config = config or ContextWindowConfig.from_base_config()
        self.metrics_collector = metrics_collector
        self.lifecycle_manager = lifecycle_manager
        
        self._content = {}  # id -> content mapping
        self._token_counts = {}  # id -> token count mapping
        self._priority_queue = []  # (priority, id) heap for quick access to lowest priority
        self._priority_lookup = {}  # id -> priority mapping for quick updates
        self._access_counts = Counter()  # id -> access count for recency tracking
        self._last_access = {}  # id -> datetime of last access
        
        self._total_tokens = 0
        
        self._content_added = 0
        self._content_removed = 0
        self._optimizations_performed = 0
        
        self.logger = logging.getLogger(__name__)
    
    @property
    def total_tokens(self) -> int:
        """Get the total number of tokens in the context window."""
        return self._total_tokens
    
    @property
    def capacity_percentage(self) -> float:
        """Get the current percentage of capacity used."""
        if self.config.max_token_limit > 0:
            return self._total_tokens / self.config.max_token_limit
        return 0.0
    
    @property
    def content_count(self) -> int:
        """Get the number of content items in the context window."""
        return len(self._content)
    
    def add_content(self, content_id: str, content: Any, token_count: int, importance: float = 0.5) -> bool:
        """Add content to the context window.
        
        Args:
            content_id: Unique identifier for the content
            content: The content to add
            token_count: Number of tokens in the content
            importance: Importance score for the content (0.0 to 1.0)
            
        Returns:
            True if content was added successfully, False if optimization was required
            
        Raises:
            ContentInsertionError: If content could not be added
            CapacityExceededError: If capacity is exceeded and optimization fails
        """
        if content_id in self._content:
            self._remove_content(content_id)
        
        if token_count > self.config.max_token_limit:
            raise CapacityExceededError(
                f"Content with {token_count} tokens exceeds maximum limit of {self.config.max_token_limit}."
            )
        
        new_total = self._total_tokens + token_count
        would_exceed_max = new_total > self.config.max_token_limit
        would_exceed_threshold = new_total > (self.config.max_token_limit * self.config.capacity_threshold)
        
        if would_exceed_max:
            raise CapacityExceededError(
                f"Cannot add content with {token_count} tokens. " 
                f"Current usage: {self._total_tokens}/{self.config.max_token_limit} tokens."
            )
        
        optimized = False
        if would_exceed_threshold:
            optimized = self._optimize_context_window(needed_tokens=token_count)
            if not optimized:
                raise CapacityExceededError(
                    f"Cannot add content with {token_count} tokens after optimization. " 
                    f"Current usage: {self._total_tokens}/{self.config.max_token_limit} tokens."
                )
        
        try:
            priority = self._calculate_priority(content_id, importance)
            
            self._content[content_id] = content
            self._token_counts[content_id] = token_count
            self._priority_lookup[content_id] = priority
            heappush(self._priority_queue, (priority, content_id))
            self._total_tokens += token_count
            
            self._access_counts[content_id] += 1
            self._last_access[content_id] = datetime.now()
            
            self._content_added += 1
            
            if self.metrics_collector:
                self.metrics_collector.record_token_usage(self._total_tokens)
            
            self.logger.info(f"Added content {content_id} with {token_count} tokens. Total: {self._total_tokens}")
            return True
        except Exception as e:
            raise ContentInsertionError(f"Failed to add content {content_id}: {str(e)}")
    
    def get_content(self, content_id: str) -> Any:
        """Get content from the context window and update its priority.
        
        Args:
            content_id: Unique identifier for the content
            
        Returns:
            The requested content
            
        Raises:
            KeyError: If content_id is not found
        """
        if content_id not in self._content:
            raise KeyError(f"Content {content_id} not found in context window")
        
        self._access_counts[content_id] += 1
        self._last_access[content_id] = datetime.now()
        
        current_priority = self._priority_lookup[content_id] 
        new_priority = self._calculate_priority(content_id, current_priority)
        
        if abs(new_priority - current_priority) > 0.1:
            self._priority_lookup[content_id] = new_priority
        
        return self._content[content_id]
    
    def remove_content(self, content_id: str) -> bool:
        """Remove content from the context window.
        
        Args:
            content_id: Unique identifier for the content
            
        Returns:
            True if content was removed, False if not found
            
        Raises:
            ContentRemovalError: If an error occurs during removal
        """
        try:
            return self._remove_content(content_id)
        except Exception as e:
            raise ContentRemovalError(f"Failed to remove content {content_id}: {str(e)}")
    
    def _remove_content(self, content_id: str) -> bool:
        """Internal method to remove content without error handling."""
        if content_id not in self._content:
            return False
        
        content = self._content.pop(content_id)
        token_count = self._token_counts.pop(content_id)
        self._priority_lookup.pop(content_id)
        
        self._total_tokens -= token_count
        
        self._content_removed += 1
        
        if self.metrics_collector:
            self.metrics_collector.record_token_usage(self._total_tokens)
        
        self.logger.info(f"Removed content {content_id} with {token_count} tokens. Total: {self._total_tokens}")
        return True
    
    def optimize(self, target_tokens: Optional[int] = None) -> bool:
        """Optimize the context window to reduce token usage.
        
        Args:
            target_tokens: Target number of tokens to reduce to, defaults to threshold
            
        Returns:
            True if optimization was successful, False otherwise
            
        Raises:
            OptimizationFailedError: If optimization fails
        """
        try:
            return self._optimize_context_window(target_tokens=target_tokens)
        except Exception as e:
            raise OptimizationFailedError(f"Failed to optimize context window: {str(e)}")
    
    def _optimize_context_window(self, target_tokens: Optional[int] = None, needed_tokens: int = 0) -> bool:
        """Internal method to optimize the context window without error handling.
        
        Args:
            target_tokens: Target number of tokens to reduce to
            needed_tokens: Number of tokens needed for a new addition
            
        Returns:
            True if optimization was successful, False otherwise
        """
        if target_tokens is None:
            target_tokens = int(self.config.max_token_limit * self.config.capacity_threshold)
        
        current = self._total_tokens
        needed_reduction = (current + needed_tokens) - target_tokens
        
        if needed_reduction <= 0:
            return True
        
        self._rebuild_priority_queue()
        
        tokens_freed = 0
        items_removed = 0
        removed_ids = []
        
        while tokens_freed < needed_reduction and self._priority_queue:
            priority, content_id = heappop(self._priority_queue)
            
            if content_id not in self._content:
                continue
            
            importance = -priority  # Convert back to positive importance score
            if importance > self.config.min_content_importance and len(self._priority_queue) > 0:
                continue
                
            token_count = self._token_counts[content_id]
            removed = self._remove_content(content_id)
            
            if removed:
                tokens_freed += token_count
                items_removed += 1
                removed_ids.append(content_id)
                
                if self.lifecycle_manager and hasattr(self.lifecycle_manager, "update_object_state"):
                    try:
                        self.lifecycle_manager.update_object_state(content_id, "REMOVED_FROM_CONTEXT")
                    except Exception as e:
                        self.logger.error(f"Failed to update lifecycle state for {content_id}: {str(e)}")
        
        self._optimizations_performed += 1
        
        if self.metrics_collector:
            self.metrics_collector.record_token_usage(self._total_tokens)
            self.logger.info(
                f"Context window optimization: removed {items_removed} items, freed {tokens_freed} tokens"
            )
        
        self.logger.info(
            f"Optimized context window: removed {items_removed} items, "
            f"freed {tokens_freed} tokens. New total: {self._total_tokens}"
        )
        
        if tokens_freed < needed_reduction:
            return False
            
        return True
    
    def _calculate_priority(self, content_id: str, importance: float) -> float:
        """Calculate priority score for content.
        
        Lower priority scores will be removed first.
        
        Args:
            content_id: Content identifier
            importance: Base importance score (0.0 to 1.0)
            
        Returns:
            Priority score where higher means more important to keep
        """
        access_count = self._access_counts.get(content_id, 0)
        last_access = self._last_access.get(content_id, datetime.min)
        
        now = datetime.now()
        seconds_since_access = (now - last_access).total_seconds()
        recency_score = 1.0 / (1.0 + seconds_since_access / 3600)  # Normalize to hours
        
        frequency_score = min(1.0, access_count / 10)  # Cap at 10 accesses
        
        if self.config.prioritization_strategy == "relevance":
            final_score = 0.7 * importance + 0.2 * recency_score + 0.1 * frequency_score
        elif self.config.prioritization_strategy == "recency":
            final_score = 0.2 * importance + 0.7 * recency_score + 0.1 * frequency_score
        else:  # hybrid (default)
            final_score = 0.4 * importance + 0.4 * recency_score + 0.2 * frequency_score
        
        return -final_score
    
    def _rebuild_priority_queue(self) -> None:
        """Rebuild the priority queue to remove stale entries."""
        updated_queue = []
        for content_id, priority in self._priority_lookup.items():
            if content_id in self._content:
                heappush(updated_queue, (priority, content_id))
        self._priority_queue = updated_queue
    
    def get_stats(self) -> Dict[str, Any]:
        """Get statistics about the context window."""
        return {
            "total_tokens": self._total_tokens,
            "capacity_percentage": self.capacity_percentage,
            "content_count": self.content_count,
            "content_added": self._content_added,
            "content_removed": self._content_removed,
            "optimizations_performed": self._optimizations_performed,
            "token_limit": self.config.max_token_limit,
            "capacity_threshold": self.config.capacity_threshold,
            "prioritization_strategy": self.config.prioritization_strategy
        }
    
    def count_tokens(self, content: str) -> int:
        """Count the number of tokens in a string.
        
        This is a simple implementation that could be enhanced with a proper tokenizer.
        
        Args:
            content: String content to count tokens for
            
        Returns:
            Estimated token count
        """
        try:
            words = content.split()
            return int(len(words) * 1.3) + 1  # +1 to avoid zero
        except Exception as e:
            raise TokenCountError(f"Failed to count tokens: {str(e)}")
