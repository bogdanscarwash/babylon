"""Tests for state management and transitions."""

import pytest
from typing import Optional
from dataclasses import dataclass
from babylon.rag.lifecycle import LifecycleManager, ObjectState
from babylon.rag.exceptions import StateTransitionError


@dataclass
class MockObject:
    """Test object for lifecycle management."""
    id: str
    data: str
    state: ObjectState = ObjectState.INACTIVE
    last_accessed: Optional[float] = None
    last_modified: Optional[float] = None


def test_immediate_context_limit():
    """Test that immediate context respects its size limit."""
    manager = LifecycleManager()
    # Create 35 objects (above 30 limit)
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(35)]
    
    for obj in objects:
        manager.activate(obj)
    
    assert manager.immediate_context_size() <= 30
    # Verify excess objects moved to active cache
    assert manager.active_cache_size() >= 5


def test_working_set_sizes():
    """Test that working set size limits are enforced."""
    manager = LifecycleManager()
    
    # Create 600 objects (above total limit)
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(600)]
    
    for obj in objects:
        manager.activate(obj)
    
    # Verify tier limits
    assert manager.immediate_context_size() <= 30
    assert manager.active_cache_size() <= 200
    assert manager.background_context_size() <= 500
    
    # Verify total objects managed doesn't exceed max
    total = (manager.immediate_context_size() + 
             manager.active_cache_size() + 
             manager.background_context_size())
    assert total <= 730  # Max total (30 + 200 + 500)


def test_state_transition_validation():
    """Test that invalid state transitions are caught."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    # Cannot deactivate an already inactive object
    with pytest.raises(StateTransitionError):
        manager.deactivate(obj)
    
    # Cannot add to background if already in immediate
    manager.activate(obj)
    with pytest.raises(StateTransitionError):
        manager.add_to_background(obj)


def test_rapid_state_transitions():
    """Test system stability during rapid state transitions."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    # Perform rapid transitions
    for _ in range(100):
        manager.activate(obj)  # INACTIVE/ACTIVE/BACKGROUND -> IMMEDIATE
        manager.mark_inactive(obj)  # IMMEDIATE -> ACTIVE
        manager.add_to_background(obj)  # ACTIVE -> BACKGROUND
        manager.deactivate(obj)  # BACKGROUND -> INACTIVE
    
    # Verify object ends in a valid state
    assert obj.state in (
        ObjectState.INACTIVE,
        ObjectState.BACKGROUND,
        ObjectState.ACTIVE,
        ObjectState.IMMEDIATE
    )


def test_object_access_patterns():
    """Test that access patterns affect object placement."""
    manager = LifecycleManager()
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(50)]
    
    # Activate all objects
    for obj in objects:
        manager.activate(obj)
    
    # Frequently access first 10 objects
    for _ in range(10):
        for obj in objects[:10]:
            manager.get_object(obj.id)
    
    # Verify frequently accessed objects remain in higher tiers
    frequently_accessed = objects[:10]
    rarely_accessed = objects[10:]
    
    high_tier_count = sum(
        1 for obj in frequently_accessed
        if obj.state in (ObjectState.IMMEDIATE, ObjectState.ACTIVE)
    )
    assert high_tier_count >= 8  # Most frequently accessed objects should be in higher tiers
    
    low_tier_count = sum(
        1 for obj in rarely_accessed
        if obj.state == ObjectState.BACKGROUND
    )
    assert low_tier_count >= len(rarely_accessed) * 0.7  # Most rarely accessed objects should be in background
