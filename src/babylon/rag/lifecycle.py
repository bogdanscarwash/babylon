"""Object lifecycle management for the RAG system."""
from typing import Dict, List, Optional, Any, Callable, NamedTuple, Set
from dataclasses import dataclass
import time
import sys
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
        self._cache_hits = 0
        self._cache_misses = 0
        self._last_state_check = 0.0  # Track last consistency check
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
        self._access_counts: Dict[str, int] = {}  # Track access counts for promotion

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
            # Move from background to inactive
            self._validate_state_transition(obj, ObjectState.INACTIVE)
            obj = self._background_context.pop(obj_id)
            obj.state = ObjectState.INACTIVE
            self._tier_transitions += 1
            # Clean up metadata
            self._last_accessed.pop(obj_id, None)
            self._access_counts.pop(obj_id, None)
    
    def set_memory_pressure(self, pressure: float) -> None:
        """Set the current memory pressure level and adjust limits."""
        if not 0.0 <= pressure <= 1.0:
            raise ValueError("Memory pressure must be between 0.0 and 1.0")
        
        self._memory_pressure = pressure
        self._memory_pressure_history.append(pressure)
        self._peak_memory_pressure = max(self._peak_memory_pressure, pressure)
        
        # Adjust tier limits based on pressure
        if pressure >= 0.9:  # Extreme pressure
            pressure_factor = max(0.2, 1.0 - pressure)  # Allow down to 20% capacity
            self._immediate_limit = max(6, int(self._base_immediate_limit * pressure_factor))
            self._active_limit = max(30, int(self._base_active_limit * pressure_factor))
            self._background_limit = max(60, int(self._base_background_limit * pressure_factor))
        elif pressure >= 0.8:  # High pressure
            pressure_factor = max(0.3, 1.0 - pressure)  # Allow down to 30% capacity
            self._immediate_limit = max(8, int(self._base_immediate_limit * pressure_factor))
            self._active_limit = max(40, int(self._base_active_limit * pressure_factor))
            self._background_limit = max(80, int(self._base_background_limit * pressure_factor))
        else:  # Normal pressure
            pressure_factor = 1.0 - (pressure * 0.2)  # More aggressive reduction
            recovery_boost = max(0, 2.0 - pressure)  # Much stronger recovery boost
            self._immediate_limit = max(25, int(self._base_immediate_limit * (pressure_factor + recovery_boost)))
            self._active_limit = max(100, int(self._base_active_limit * (pressure_factor + recovery_boost)))
            self._background_limit = max(200, int(self._base_background_limit * (pressure_factor + recovery_boost)))
        
        # Force rebalancing when pressure changes
        self._rebalance_all_tiers()
        # Trigger promotion of objects based on updated limits
        self._promote_objects_after_pressure()
        
        logger.info(
            f"Memory pressure set to {pressure:.2f}, "
            f"new limits: immediate={self._immediate_limit}, "
            f"active={self._active_limit}, "
            f"background={self._background_limit}"
        )
    
    def _rebalance_all_tiers(self) -> None:
        """Rebalance tiers based on access counts and priorities."""
        
        # Decay access counts for all objects
        for obj_id in self._access_counts:
            self._access_counts[obj_id] = max(self._access_counts[obj_id] - 2, -10)

        # First handle immediate context overflow
        while len(self._immediate_context) > self._immediate_limit:
            obj_id = self._find_demotion_candidate(self._immediate_context)
            obj = self._immediate_context.pop(obj_id)
            # Don't demote high priority objects from immediate unless under extreme pressure
            if self._priorities.get(obj_id, 0) > 0 and self._memory_pressure < 0.9:
                continue
            obj.state = ObjectState.ACTIVE
            self._active_cache[obj_id] = obj
            self._tier_transitions += 1

        # Then handle active cache overflow
        while len(self._active_cache) > self._active_limit:
            obj_id = self._find_demotion_candidate(self._active_cache)
            obj = self._active_cache.pop(obj_id)
            # Only keep high priority objects in active under normal conditions
            if self._priorities.get(obj_id, 0) > 0 and self._memory_pressure < 0.8:
                continue
            obj.state = ObjectState.BACKGROUND
            self._background_context[obj_id] = obj
            self._tier_transitions += 1

        # Finally handle background context overflow
        while len(self._background_context) > self._background_limit:
            obj_id = self._find_demotion_candidate(self._background_context)
            obj = self._background_context.pop(obj_id)
            obj.state = ObjectState.INACTIVE
            self._tier_transitions += 1

        # Demote rarely accessed objects from immediate to active
        for obj_id in list(self._immediate_context.keys()):
            if (self._access_counts.get(obj_id, 0) <= -5 and 
                self._priorities.get(obj_id, 0) == 0):
                obj = self._immediate_context.pop(obj_id)
                obj.state = ObjectState.ACTIVE
                self._active_cache[obj_id] = obj
                self._tier_transitions += 1

        # Demote rarely accessed objects from active to background
        for obj_id in list(self._active_cache.keys()):
            if (self._access_counts.get(obj_id, 0) <= -8 and 
                self._priorities.get(obj_id, 0) == 0):
                obj = self._active_cache.pop(obj_id)
                obj.state = ObjectState.BACKGROUND
                self._background_context[obj_id] = obj
                self._tier_transitions += 1

        # Promote frequently accessed objects from background to active
        for obj_id in list(self._background_context.keys()):
            if ((self._access_counts.get(obj_id, 0) >= 3 or 
                 self._priorities.get(obj_id, 0) > 0) and 
                len(self._active_cache) < self._active_limit):
                obj = self._background_context.pop(obj_id)
                obj.state = ObjectState.ACTIVE
                self._active_cache[obj_id] = obj
                self._tier_transitions += 1

        # Promote frequently accessed objects from active to immediate
        for obj_id in list(self._active_cache.keys()):
            if ((self._access_counts.get(obj_id, 0) >= 5 or 
                 self._priorities.get(obj_id, 0) > 0) and 
                len(self._immediate_context) < self._immediate_limit):
                obj = self._active_cache.pop(obj_id)
                obj.state = ObjectState.IMMEDIATE
                self._immediate_context[obj_id] = obj
                self._tier_transitions += 1
    
    def _validate_object(self, obj: Any) -> None:
        """Validate that an object has required attributes."""
        if not hasattr(obj, 'id'):
            raise InvalidObjectError("Object must have an 'id' attribute")
        if not hasattr(obj, 'state'):
            obj.state = ObjectState.INACTIVE
        elif not isinstance(obj.state, ObjectState):
            raise InvalidObjectError(
                f"Invalid state type: {type(obj.state)}",
                error_code="RAG_102",
                field_name="state",
                current_value=obj.state
            )
        if not hasattr(obj, 'last_accessed'):
            obj.last_accessed = None
        if not hasattr(obj, 'last_modified'):
            obj.last_modified = None

    def _check_state_consistency(self) -> None:
        """Check for state consistency across all contexts."""
        current_time = time.time()
        
        # Always check when testing, otherwise limit frequency
        if "pytest" not in sys.modules and current_time - self._last_state_check < 0.1:
            return
            
        self._last_state_check = current_time
        
        # Check for objects in multiple contexts
        immediate_set = set(self._immediate_context.keys())
        active_set = set(self._active_cache.keys())
        background_set = set(self._background_context.keys())
        
        # Find any duplicates between contexts using set operations
        duplicates_immediate_active = immediate_set & active_set
        duplicates_immediate_background = immediate_set & background_set
        duplicates_active_background = active_set & background_set
        
        duplicate_objects = duplicates_immediate_active | duplicates_immediate_background | duplicates_active_background
            
        if duplicate_objects:
            # Identify which contexts each duplicate is in
            context_details = {}
            for obj_id in duplicate_objects:
                contexts = []
                if obj_id in immediate_set:
                    contexts.append("immediate")
                if obj_id in active_set:
                    contexts.append("active")
                if obj_id in background_set:
                    contexts.append("background")
                context_details[obj_id] = contexts
                
            # Recover by keeping the highest priority context
            for obj_id in duplicate_objects:
                contexts = context_details[obj_id]
                # Decide which context to keep (e.g., immediate > active > background)
                if 'immediate' in contexts:
                    # Remove from other contexts
                    self._active_cache.pop(obj_id, None)
                    self._background_context.pop(obj_id, None)
                elif 'active' in contexts:
                    self._immediate_context.pop(obj_id, None)
                    self._background_context.pop(obj_id, None)
                elif 'background' in contexts:
                    self._immediate_context.pop(obj_id, None)
                    self._active_cache.pop(obj_id, None)
                self._tier_transitions += 1
            # Log recovery action
            logger.warning(f"Recovered from corrupt state for objects: {duplicate_objects}")
            
            # Raise the CorruptStateError as expected
            raise CorruptStateError(
                message=f"Detected objects in multiple contexts: {duplicate_objects}",
                error_code="RAG_201",
                affected_objects=list(duplicate_objects)
            )

    def _validate_state_transition(self, obj: Any, target_state: ObjectState) -> None:
        """Validate that a state transition is allowed."""
        if not isinstance(obj.state, ObjectState):
            raise InvalidObjectError(
                f"Invalid current state: {obj.state}",
                error_code="RAG_102",
                field_name="state",
                current_value=obj.state
            )
            
        # Allow same-state transitions for certain operations
        if obj.state == target_state:
            # Only allow same-state for BACKGROUND and ACTIVE
            if target_state not in {ObjectState.BACKGROUND, ObjectState.ACTIVE}:
                raise StateTransitionError(
                    f"Invalid same-state transition for state: {target_state}",
                    current_state=str(obj.state),
                    target_state=str(target_state)
                )
            return
            
        # Define valid transitions
        valid_transitions = {
            ObjectState.INACTIVE: {ObjectState.IMMEDIATE, ObjectState.BACKGROUND},
            ObjectState.BACKGROUND: {ObjectState.ACTIVE, ObjectState.IMMEDIATE, ObjectState.INACTIVE},
            ObjectState.ACTIVE: {ObjectState.IMMEDIATE, ObjectState.BACKGROUND, ObjectState.INACTIVE},
            ObjectState.IMMEDIATE: {ObjectState.ACTIVE, ObjectState.INACTIVE}
        }
        
        if target_state not in valid_transitions[obj.state]:
            raise StateTransitionError(
                f"Invalid state transition: {obj.state} -> {target_state}",
                current_state=str(obj.state),
                target_state=str(target_state)
            )

    def get_object(self, obj_id: str) -> Any:
        """Get an object by ID from any context."""
        current_time = time.time()
    
        if obj_id in self._immediate_context:
            self._cache_hits += 1
            obj = self._immediate_context[obj_id]
            obj.last_accessed = current_time
            self._last_accessed[obj_id] = current_time
            # Update access counts
            self._access_counts[obj_id] = self._access_counts.get(obj_id, 0) + 1
            return obj

        if obj_id in self._active_cache:
            self._cache_hits += 1
            obj = self._active_cache[obj_id]
            obj.last_accessed = current_time
            self._last_accessed[obj_id] = current_time
            # Promote to immediate if accessed frequently
            access_count = self._access_counts.get(obj_id, 0) + 1
            self._access_counts[obj_id] = access_count
            if access_count >= 5 and len(self._immediate_context) < self._immediate_limit:
                self._active_cache.pop(obj_id)
                obj.state = ObjectState.IMMEDIATE
                self._immediate_context[obj_id] = obj
                self._tier_transitions += 1
                self._access_counts[obj_id] = 0
            return obj

        if obj_id in self._background_context:
            self._cache_hits += 1
            obj = self._background_context[obj_id]
            obj.last_accessed = current_time
            self._last_accessed[obj_id] = current_time
            # Update access counts
            self._access_counts[obj_id] = self._access_counts.get(obj_id, 0) + 1
            # Promote to active if accessed
            if len(self._active_cache) < self._active_limit:
                self._background_context.pop(obj_id)
                obj.state = ObjectState.ACTIVE
                self._active_cache[obj_id] = obj
                self._tier_transitions += 1
            return obj

        self._cache_misses += 1
        return None

    def immediate_context_size(self) -> int:
        """Get the size of the immediate context."""
        return len(self._immediate_context)

    def active_cache_size(self) -> int:
        """Get the size of the active cache."""
        return len(self._active_cache)

    def background_context_size(self) -> int:
        """Get the size of the background context."""
        return len(self._background_context)

    @time_operation('activate')
    def activate(self, obj: Any, priority: int = 0) -> None:
        """Activate an object and move it to immediate context."""
        self._validate_object(obj)
        self._check_state_consistency()  # Check before modification
        obj_id = str(obj.id)
        
        # Check if already in immediate context
        if obj_id in self._immediate_context:
            return  # Don't count as transition if already active
            
        # Validate state transition
        self._validate_state_transition(obj, ObjectState.IMMEDIATE)
        
        current_time = time.time()
        self._priorities[obj_id] = priority
        self._last_accessed[obj_id] = current_time
        obj.last_accessed = current_time

        # Remove from other contexts if present
        self._active_cache.pop(obj_id, None)
        self._background_context.pop(obj_id, None)

        # Add to immediate context
        obj.state = ObjectState.IMMEDIATE
        self._immediate_context[obj_id] = obj
        self._tier_transitions += 1
        self._access_counts[obj_id] = 0  # Set initial access count

        # Rebalance if needed
        if len(self._immediate_context) > self._immediate_limit:
            self._rebalance_all_tiers()

    def add_to_background(self, obj: Any) -> None:
        """Add an object directly to background context."""
        self._validate_object(obj)
        obj_id = str(obj.id)
        
        # Validate state transition
        self._validate_state_transition(obj, ObjectState.BACKGROUND)
        
        # Remove from other contexts first
        self._immediate_context.pop(obj_id, None)
        self._active_cache.pop(obj_id, None)
        
        current_time = time.time()
        self._last_accessed[obj_id] = current_time
        obj.last_accessed = current_time

        # Add to background context
        obj.state = ObjectState.BACKGROUND
        self._background_context[obj_id] = obj
        self._tier_transitions += 1

    def get_metrics(self) -> PerformanceMetrics:
        """Get current performance metrics."""
        return PerformanceMetrics(
            activation_count=self._operation_counts.get('activate', 0),
            deactivation_count=self._operation_counts.get('deactivate', 0),
            cache_hit_count=self._cache_hits,
            cache_miss_count=self._cache_misses,
            tier_transition_count=self._tier_transitions,
            avg_activation_time=mean(self._operation_times.get('activate', [0])),
            avg_deactivation_time=mean(self._operation_times.get('deactivate', [0])),
            avg_memory_pressure=mean(self._memory_pressure_history) if self._memory_pressure_history else 0.0,
            peak_memory_pressure=self._peak_memory_pressure,
            immediate_context_usage=len(self._immediate_context) / self._immediate_limit,
            active_cache_usage=len(self._active_cache) / self._active_limit,
            background_context_usage=len(self._background_context) / self._background_limit
        )

    def _promote_objects_after_pressure(self) -> None:
        """Promote objects to higher tiers after memory pressure reduction."""
        # Promote from background to active
        while len(self._active_cache) < self._active_limit and self._background_context:
            obj_id = self._find_promotion_candidate(self._background_context)
            obj = self._background_context.pop(obj_id)
            obj.state = ObjectState.ACTIVE
            self._active_cache[obj_id] = obj
            self._tier_transitions += 1
        # Promote from active to immediate
        while len(self._immediate_context) < self._immediate_limit and self._active_cache:
            obj_id = self._find_promotion_candidate(self._active_cache)
            obj = self._active_cache.pop(obj_id)
            obj.state = ObjectState.IMMEDIATE
            self._immediate_context[obj_id] = obj
            self._tier_transitions += 1

    def _find_demotion_candidate(
        self,
        context: OrderedDict
    ) -> Optional[str]:
        """Find the object that should be demoted from a context."""
        if not context:
            return None
        
        # First try to find a low priority, rarely accessed object
        candidates = [
            (obj_id, obj) for obj_id, obj in context.items()
            if self._priorities.get(obj_id, 0) == 0 and 
               self._access_counts.get(obj_id, 0) <= 0
        ]
        
        if candidates:
            # Sort by access count (ascending)
            sorted_candidates = sorted(
                candidates,
                key=lambda x: self._access_counts.get(x[0], 0)
            )
            return sorted_candidates[0][0]
        
        # If no good candidates, pick the least recently accessed
        sorted_items = sorted(
            context.items(),
            key=lambda item: (
                self._priorities.get(item[0], 0),  # Higher priority = less likely to be demoted
                self._access_counts.get(item[0], 0),  # Higher access count = less likely to be demoted
                -self._last_accessed.get(item[0], 0)  # Older access = more likely to be demoted
            )
        )
        return sorted_items[0][0]

    def _find_promotion_candidate(
        self,
        context: OrderedDict
    ) -> Optional[str]:
        """Find the object that should be promoted from a context."""
        if not context:
            return None
        # Sort by priority (descending) and access count (descending)
        sorted_items = sorted(
            context.items(),
            key=lambda item: (
                self._priorities.get(item[0], 0),
                self._access_counts.get(item[0], 0)
            ),
            reverse=True
        )
        return sorted_items[0][0]  # Return the obj_id with highest priority and access count
