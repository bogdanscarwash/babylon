"""Unit tests for the RAG system's object lifecycle management."""

import pytest
from typing import Optional
from dataclasses import dataclass
from babylon.rag.lifecycle import LifecycleManager, ObjectState


@dataclass
class MockObject:
    """Test object for lifecycle management."""
    
    id: str
    data: str
    state: ObjectState = ObjectState.INACTIVE
    last_accessed: Optional[float] = None


def test_lifecycle_manager_creation():
    """Test that lifecycle manager is created with correct initial state."""
    manager = LifecycleManager()
    assert manager.immediate_context_size() == 0
    assert manager.active_cache_size() == 0
    assert manager.background_context_size() == 0


def test_object_activation():
    """Test that objects can be activated and moved to immediate context."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    manager.activate(obj)
    assert obj.state == ObjectState.IMMEDIATE
    assert manager.immediate_context_size() == 1


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


def test_object_demotion():
    """Test that least recently used objects are demoted properly."""
    manager = LifecycleManager()
    obj1 = MockObject(id="test1", data="data1")
    obj2 = MockObject(id="test2", data="data2")
    
    manager.activate(obj1)
    manager.activate(obj2)
    
    # Force demotion of obj1
    manager.mark_inactive(obj1)
    
    assert obj1.state == ObjectState.ACTIVE
    assert obj2.state == ObjectState.IMMEDIATE


def test_background_promotion():
    """Test that background objects can be promoted when accessed."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="data1")
    
    # Put object in background
    manager.add_to_background(obj)
    assert obj.state == ObjectState.BACKGROUND
    
    # Promote to active
    manager.activate(obj)
    assert obj.state == ObjectState.IMMEDIATE


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


def test_object_deactivation():
    """Test that objects can be fully deactivated."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="data1")
    
    manager.activate(obj)
    assert obj.state == ObjectState.IMMEDIATE
    
    manager.deactivate(obj)
    assert obj.state == ObjectState.INACTIVE


def test_priority_handling():
    """Test that high priority objects remain in higher tiers."""
    manager = LifecycleManager()
    high_priority = MockObject(id="high", data="important")
    low_priority = MockObject(id="low", data="routine")
    
    manager.activate(high_priority, priority=1)
    manager.activate(low_priority, priority=0)
    
    # Force memory pressure by exceeding both immediate and active limits
    for i in range(250):  # Above immediate (30) + active (200) limits
        manager.activate(MockObject(id=f"filler{i}", data="filler"))
    
    # High priority should remain in immediate or active
    assert high_priority.state in (ObjectState.IMMEDIATE, ObjectState.ACTIVE)
    # Low priority should be demoted to background
    assert low_priority.state == ObjectState.BACKGROUND


# TODO: Add tests for:
# - Concurrent access patterns
# - Memory pressure handling
# - Cache invalidation
# - Performance metrics collection
# - Error handling for corrupt/invalid objects
