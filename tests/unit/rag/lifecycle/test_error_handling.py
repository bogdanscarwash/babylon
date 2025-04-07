"""Tests for error handling and recovery scenarios."""

import pytest
from typing import Optional
from dataclasses import dataclass
from babylon.rag.lifecycle import LifecycleManager, ObjectState
from babylon.rag.exceptions import (
    InvalidObjectError,
    StateTransitionError,
    CorruptStateError
)


@dataclass
class MockObject:
    """Test object for lifecycle management."""
    id: str
    data: str
    state: ObjectState = ObjectState.INACTIVE
    last_accessed: Optional[float] = None
    last_modified: Optional[float] = None


@dataclass
class InvalidMockObject:
    """Test object missing required attributes."""
    data: str  # Missing id field


def test_invalid_object_handling():
    """Test that system properly handles invalid objects."""
    manager = LifecycleManager()
    
    # Test object without required id attribute
    invalid_obj = InvalidMockObject(data="test data")
    with pytest.raises(InvalidObjectError):
        manager.activate(invalid_obj)
    
    # Test object with invalid state
    corrupt_obj = MockObject(id="corrupt", data="test data")
    corrupt_obj.state = "INVALID_STATE"  # type: ignore
    with pytest.raises(InvalidObjectError):
        manager.activate(corrupt_obj)
    
    # Test None object
    with pytest.raises(InvalidObjectError):
        manager.activate(None)  # type: ignore


def test_corrupt_state_recovery():
    """Test that system can detect and recover from corrupt internal state."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    # Simulate corruption by directly modifying internal state
    manager.activate(obj)
    manager._immediate_context[obj.id] = obj
    manager._active_cache[obj.id] = obj  # Object incorrectly in multiple tiers
    
    # System should detect corruption during next operation
    with pytest.raises(CorruptStateError):
        manager.activate(MockObject(id="test2", data="data2"))
    
    # System should recover by cleaning up corrupt state
    assert manager.immediate_context_size() + manager.active_cache_size() == 1
    assert obj.id in manager._immediate_context or obj.id in manager._active_cache


def test_invalid_state_transitions():
    """Test handling of invalid state transitions."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    # Test invalid transitions from INACTIVE
    with pytest.raises(StateTransitionError):
        manager.deactivate(obj)  # Can't deactivate inactive object
    
    # Test invalid transitions from IMMEDIATE
    manager.activate(obj)
    with pytest.raises(StateTransitionError):
        manager.add_to_background(obj)  # Can't directly move from IMMEDIATE to BACKGROUND
    
    # Test invalid transitions from BACKGROUND
    obj2 = MockObject(id="test2", data="test data")
    manager.add_to_background(obj2)
    with pytest.raises(StateTransitionError):
        manager.mark_inactive(obj2)  # Can't mark_inactive from BACKGROUND


def test_memory_pressure_validation():
    """Test validation of memory pressure values."""
    manager = LifecycleManager()
    
    # Test invalid pressure values
    with pytest.raises(ValueError):
        manager.set_memory_pressure(-0.1)  # Below minimum
    
    with pytest.raises(ValueError):
        manager.set_memory_pressure(1.1)  # Above maximum


def test_duplicate_activation():
    """Test handling of duplicate object activation."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    # First activation should succeed
    manager.activate(obj)
    initial_metrics = manager.get_metrics()
    
    # Second activation of same object should not cause issues
    manager.activate(obj)
    final_metrics = manager.get_metrics()
    
    # Should only count as one transition
    assert final_metrics.tier_transition_count == initial_metrics.tier_transition_count


def test_error_recovery_consistency():
    """Test that error recovery maintains system consistency."""
    manager = LifecycleManager()
    
    # Create a set of valid objects
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(10)]
    for obj in objects:
        manager.activate(obj)
    
    # Try operations with invalid objects interspersed
    for i in range(5):
        # Valid operation
        manager.get_object(objects[i].id)
        
        # Invalid operation that should not affect system state
        try:
            invalid_obj = InvalidMockObject(data=f"invalid{i}")
            manager.activate(invalid_obj)
        except InvalidObjectError:
            pass
        
        # System should maintain consistency
        assert manager.get_object(objects[i].id) is not None
    
    # Verify final system state is consistent
    total_objects = (manager.immediate_context_size() + 
                    manager.active_cache_size() + 
                    manager.background_context_size())
    assert total_objects == 10  # Should still have all valid objects
