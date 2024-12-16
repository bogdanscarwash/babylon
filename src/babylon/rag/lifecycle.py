"""Object lifecycle management for the RAG system."""
from typing import Dict, List, Optional, Any, Callable, NamedTuple, Set
from dataclasses import dataclass
import time
from enum import Enum, auto
import logging
from collections import OrderedDict
from statistics import mean
from functools import wraps

from babylon.rag.exceptions import (
    InvalidObjectError,
    StateTransitionError,
    CorruptStateError
)

logger = logging.getLogger(__name__)


def time_operation(metric_name: str):
    """Decorator to time operations and update metrics."""
    def decorator(func):
        @wraps(func)
        def wrapper(self, *args, **kwargs):
            start_time = time.perf_counter()  # Use perf_counter for higher precision
            result = func(self, *args, **kwargs)
            duration = time.perf_counter() - start_time
            
            # Update timing metrics
            times = self._operation_times.setdefault(metric_name, [])
            times.append(duration)
            
            # Update operation count
            self._operation_counts[metric_name] = (
                self._operation_counts.get(metric_name, 0) + 1
            )
            
            return result
        return wrapper
    return decorator


class ObjectState(Enum):
    """States an object can be in within the lifecycle system."""
    
    INACTIVE = auto()  # Object is not in memory
    BACKGROUND = auto()  # In background context (300-500 objects)
    ACTIVE = auto()  # In active cache (100-200 objects)
    IMMEDIATE = auto()  # In immediate context (20-30 objects)


class PerformanceMetrics(NamedTuple):
    """Performance metrics for the lifecycle manager."""
    
    # Operation counts
    activation_count: int
    deactivation_count: int
    cache_hit_count: int
    cache_miss_count: int
    tier_transition_count: int
    
    # Timing metrics (in seconds)
    avg_activation_time: float
    avg_deactivation_time: float
    
    # Memory pressure stats
    avg_memory_pressure: float
    peak_memory_pressure: float
    
    # Tier usage (as percentages of capacity)
    immediate_context_usage: float
    active_cache_usage: float
    background_context_usage: float


class LifecycleManager:
    """Manages object lifecycles and working set tiers."""
    
    def __init__(self):
        """Initialize the lifecycle manager."""
        # Use OrderedDict to maintain access order
        self._immediate_context: OrderedDict[str, Any] = OrderedDict()
        self._active_cache: OrderedDict[str, Any] = OrderedDict()
        self._background_context: OrderedDict[str, Any] = OrderedDict()
        self._priorities: Dict[str, int] = {}
        
        # Base size limits for each tier
        self._base_immediate_limit = 30
        self._base_active_limit = 200
        self._base_background_limit = 500
        
        # Current size limits (adjusted by memory pressure)
        self._immediate_limit = self._base_immediate_limit
        self._active_limit = self._base_active_limit
        self._background_limit = self._base_background_limit
        
        # Memory pressure tracking (0.0 to 1.0)
        self._memory_pressure = 0.0
        self._memory_pressure_history: List[float] = []
        self._peak_memory_pressure = 0.0
        
        # Access timestamps
        self._last_accessed: Dict[str, float] = {}
        
        # Performance metrics
        self._operation_times: Dict[str, List[float]] = {}
        self._operation_counts: Dict[str, int] = {}
        self._tier_transitions = 0
        
    @time_operation('deactivate')
    def deactivate(self, obj: Any) -> None:
        """Fully deactivate an object, removing it from all contexts."""
        self._validate_object(obj)
        if obj.state == ObjectState.INACTIVE:
            raise StateTransitionError(
                message="Cannot deactivate an already inactive object",
                error_code="RAG_122",
                current_state=str(obj.state),
                target_state=str(ObjectState.INACTIVE)
            )
        
        self._check_state_consistency()
        
        obj_id = str(obj.id)
        
        # Track tier transition if object was in any tier
        if (obj_id in self._immediate_context or
            obj_id in self._active_cache or
            obj_id in self._background_context):
            self._tier_transitions += 1
        
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
        self._validate_object(obj)
        self._check_state_consistency()
        
        obj_id = str(obj.id)
        current_time = time.time()
        
        if obj_id in self._immediate_context:
            # Move from immediate to active
            self._validate_state_transition(obj, ObjectState.ACTIVE)
            obj = self._immediate_context.pop(obj_id)
            self._active_cache[obj_id] = obj
            obj.state = ObjectState.ACTIVE
            self._tier_transitions += 1
            self._last_accessed[obj_id] = current_time
        elif obj_id in self._active_cache:
            # Move from active to background
            self._validate_state_transition(obj, ObjectState.BACKGROUND)
            obj = self._active_cache.pop(obj_id)
            self._background_context[obj_id] = obj
            obj.state = ObjectState.BACKGROUND
            self._tier_transitions += 1
            self._last_accessed[obj_id] = current_time
        elif obj_id in self._background_context:
            # Cannot mark inactive from background
            raise StateTransitionError(
                message="Cannot mark_inactive from BACKGROUND state",
                error_code="RAG_123",
                current_state=str(obj.state),
                target_state=str(ObjectState.INACTIVE)
            )
    
    def set_memory_pressure(self, pressure: float) -> None:
        """Set the current memory pressure level and adjust limits."""
        if not 0.0 <= pressure <= 1.0:
            raise ValueError("Memory pressure must be between 0.0 and 1.0")
        
        self._memory_pressure = pressure
        self._memory_pressure_history.append(pressure)
        self._peak_memory_pressure = max(self._peak_memory_pressure, pressure)
        
        # Adjust tier limits based on pressure
        if pressure >= 0.9:  # Extreme pressure
            pressure_factor = max(0.1, 1.0 - pressure)  # Allow down to 10% capacity
            self._immediate_limit = max(4, int(self._base_immediate_limit * pressure_factor))
            self._active_limit = max(15, int(self._base_active_limit * pressure_factor))
            self._background_limit = max(30, int(self._base_background_limit * pressure_factor))
        elif pressure >= 0.8:  # High pressure
            pressure_factor = max(0.2, 1.0 - pressure)  # Allow down to 20% capacity
            self._immediate_limit = max(6, int(self._base_immediate_limit * pressure_factor))
            self._active_limit = max(20, int(self._base_active_limit * pressure_factor))
            self._background_limit = max(50, int(self._base_background_limit * pressure_factor))
        else:  # Normal pressure
            pressure_factor = 1.0 - (pressure * 0.2)  # More gradual reduction
            recovery_boost = max(0, 0.2 - pressure)  # Boost capacity during recovery
            self._immediate_limit = max(8, int(self._base_immediate_limit * (pressure_factor + recovery_boost)))
            self._active_limit = max(25, int(self._base_active_limit * (pressure_factor + recovery_boost)))
            self._background_limit = max(75, int(self._base_background_limit * (pressure_factor + recovery_boost)))
        
        # Force rebalancing when pressure changes
        self._rebalance_all_tiers()
        
        logger.info(
            f"Memory pressure set to {pressure:.2f}, "
            f"new limits: immediate={self._immediate_limit}, "
            f"active={self._active_limit}, "
            f"background={self._background_limit}"
        )
    
    def _rebalance_all_tiers(self) -> None:
        """Force rebalancing of all tiers based on current limits."""
        current_time = time.time()
        old_threshold = current_time - 1800  # 30 minutes
        
        # Move excess objects from immediate to active
        while len(self._immediate_context) > self._immediate_limit:
            obj_id = self._find_demotion_candidate(self._immediate_context)
            obj = self._immediate_context.pop(obj_id)
            obj.state = ObjectState.ACTIVE
            self._active_cache[obj_id] = obj
            self._tier_transitions += 1
        
        # Move excess objects from active to background
        while len(self._active_cache) > self._active_limit:
            obj_id = self._find_demotion_candidate(self._active_cache)
            obj = self._active_cache.pop(obj_id)
            obj.state = ObjectState.BACKGROUND
            self._background_context[obj_id] = obj
            self._tier_transitions += 1
        
        # Move old objects from active to background
        for obj_id in list(self._active_cache.keys()):
            last_access = self._last_accessed.get(obj_id, 0)
            if current_time - last_access > old_threshold:
                obj = self._active_cache.pop(obj_id)
                self._background_context[obj_id] = obj
                obj.state = ObjectState.BACKGROUND
                self._tier_transitions += 1
        
        # Move excess objects from background to inactive
        while len(self._background_context) > self._background_limit:
            obj_id = self._find_demotion_candidate(self._background_context)
            obj = self._background_context.pop(obj_id)
            obj.state = ObjectState.INACTIVE
            self._tier_transitions += 1
    
    def _find_demotion_candidate(
        self,
        context: OrderedDict,
        age_threshold: Optional[float] = None
    ) -> Optional[str]:
        """Find the object that should be demoted from a context."""
        lowest_priority = float('inf')
        oldest_access = float('inf')
        candidate = None
        
        for obj_id in context:
            priority = self._priorities.get(obj_id, 0)
            last_access = self._last_accessed.get(obj_id, 0)
            
            # If age threshold is provided and object is old enough, it's a candidate
            if age_threshold and last_access < age_threshold:
                return obj_id
            
            # First check priority
            if priority < lowest_priority:
                lowest_priority = priority
                oldest_access = last_access
                candidate = obj_id
            # If same priority, check access time
            elif priority == lowest_priority and last_access < oldest_access:
                oldest_access = last_access
                candidate = obj_id
        
        return candidate or next(iter(context))  # Fallback to first object if no candidate
