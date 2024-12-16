"""Object lifecycle management for the RAG system."""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass
import time
from enum import Enum, auto
import logging
from collections import OrderedDict

logger = logging.getLogger(__name__)


class ObjectState(Enum):
    """States an object can be in within the lifecycle system."""
    
    INACTIVE = auto()  # Object is not in memory
    BACKGROUND = auto()  # In background context (300-500 objects)
    ACTIVE = auto()  # In active cache (100-200 objects)
    IMMEDIATE = auto()  # In immediate context (20-30 objects)


class LifecycleManager:
    """Manages object lifecycles and working set tiers.
    
    The LifecycleManager maintains three tiers of object storage:
    - Immediate context: 20-30 most recently/frequently accessed objects
    - Active cache: 100-200 objects that are actively being used
    - Background context: 300-500 objects that provide broader context
    
    Objects move between tiers based on:
    - Access patterns
    - Priority levels
    - Memory pressure
    - System state
    """
    
    def __init__(self):
        """Initialize the lifecycle manager."""
        # Use OrderedDict to maintain access order
        self._immediate_context: OrderedDict[str, Any] = OrderedDict()
        self._active_cache: OrderedDict[str, Any] = OrderedDict()
        self._background_context: OrderedDict[str, Any] = OrderedDict()
        self._priorities: Dict[str, int] = {}
        
        # Size limits for each tier
        self._immediate_limit = 30
        self._active_limit = 200
        self._background_limit = 500
        
        # Access timestamps
        self._last_accessed: Dict[str, float] = {}
    
    def immediate_context_size(self) -> int:
        """Get number of objects in immediate context."""
        return len(self._immediate_context)
    
    def active_cache_size(self) -> int:
        """Get number of objects in active cache."""
        return len(self._active_cache)
    
    def background_context_size(self) -> int:
        """Get number of objects in background context."""
        return len(self._background_context)
    
    def activate(self, obj: Any, priority: int = 0) -> None:
        """Activate an object and move it to immediate context.
        
        Args:
            obj: The object to activate
            priority: Priority level (higher values = higher priority)
        """
        obj_id = str(obj.id)
        self._priorities[obj_id] = priority
        self._last_accessed[obj_id] = time.time()
        
        # Remove from other contexts if present
        self._active_cache.pop(obj_id, None)
        self._background_context.pop(obj_id, None)
        
        # Add to immediate context
        self._immediate_context[obj_id] = obj
        obj.state = ObjectState.IMMEDIATE
        
        # Handle overflow
        self._balance_tiers()
    
    def deactivate(self, obj: Any) -> None:
        """Fully deactivate an object, removing it from all contexts."""
        obj_id = str(obj.id)
        
        # Remove from all contexts
        self._immediate_context.pop(obj_id, None)
        self._active_cache.pop(obj_id, None)
        self._background_context.pop(obj_id, None)
        
        # Update state
        obj.state = ObjectState.INACTIVE
        
        # Clean up metadata
        self._priorities.pop(obj_id, None)
        self._last_accessed.pop(obj_id, None)
    
    def mark_inactive(self, obj: Any) -> None:
        """Mark an object as inactive, moving it to a lower tier."""
        obj_id = str(obj.id)
        
        if obj_id in self._immediate_context:
            # Move from immediate to active
            obj = self._immediate_context.pop(obj_id)
            self._active_cache[obj_id] = obj
            obj.state = ObjectState.ACTIVE
        elif obj_id in self._active_cache:
            # Move from active to background
            obj = self._active_cache.pop(obj_id)
            self._background_context[obj_id] = obj
            obj.state = ObjectState.BACKGROUND
    
    def add_to_background(self, obj: Any) -> None:
        """Add an object directly to background context."""
        obj_id = str(obj.id)
        self._background_context[obj_id] = obj
        obj.state = ObjectState.BACKGROUND
        self._last_accessed[obj_id] = time.time()
        self._balance_tiers()
    
    def _balance_tiers(self) -> None:
        """Balance objects across tiers based on limits and priorities."""
        # Handle immediate context overflow
        while len(self._immediate_context) > self._immediate_limit:
            # Find lowest priority, least recently accessed object
            obj_id = self._find_demotion_candidate(self._immediate_context)
            obj = self._immediate_context.pop(obj_id)
            self._active_cache[obj_id] = obj
            obj.state = ObjectState.ACTIVE
        
        # Handle active cache overflow
        while len(self._active_cache) > self._active_limit:
            obj_id = self._find_demotion_candidate(self._active_cache)
            obj = self._active_cache.pop(obj_id)
            self._background_context[obj_id] = obj
            obj.state = ObjectState.BACKGROUND
        
        # Handle background context overflow
        while len(self._background_context) > self._background_limit:
            obj_id = self._find_demotion_candidate(self._background_context)
            obj = self._background_context.pop(obj_id)
            obj.state = ObjectState.INACTIVE
    
    def _find_demotion_candidate(self, context: OrderedDict) -> str:
        """Find the object that should be demoted from a context."""
        lowest_priority = float('inf')
        oldest_access = float('inf')
        candidate = None
        
        for obj_id in context:
            priority = self._priorities.get(obj_id, 0)
            last_access = self._last_accessed.get(obj_id, 0)
            
            # First check priority
            if priority < lowest_priority:
                lowest_priority = priority
                oldest_access = last_access
                candidate = obj_id
            # If same priority, check access time
            elif priority == lowest_priority and last_access < oldest_access:
                oldest_access = last_access
                candidate = obj_id
        
        return candidate

    # TODO: Add methods for:
    # - Batch operations
    # - Memory pressure handling
    # - Cache invalidation
    # - Performance metrics collection
    # - Error handling for corrupt/invalid objects
