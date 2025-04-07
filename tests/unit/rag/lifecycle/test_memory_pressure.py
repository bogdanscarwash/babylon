"""Tests for memory pressure handling and tier management."""

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
    last_modified: Optional[float] = None


def test_memory_pressure_detection():
    """Test that system detects and responds to memory pressure."""
    manager = LifecycleManager()
    
    # Create objects up to normal limits
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(30)]
    for obj in objects:
        manager.activate(obj)
    
    # Simulate high memory pressure
    manager.set_memory_pressure(0.9)  # 90% memory usage
    
    # System should reduce immediate context size
    assert manager.immediate_context_size() <= 15  # Expect ~50% reduction
    # Objects should be demoted rather than deactivated
    total_active = (manager.immediate_context_size() + 
                   manager.active_cache_size() + 
                   manager.background_context_size())
    assert total_active == 30  # All objects still in memory, just demoted


def test_extreme_memory_pressure():
    """Test system behavior under extreme memory pressure."""
    manager = LifecycleManager()
    
    # Create initial set of objects
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(100)]
    for obj in objects:
        manager.activate(obj)
    
    # Simulate extreme memory pressure (99%)
    manager.set_memory_pressure(0.99)
    
    # Verify system maintains minimum working set
    min_total = manager.immediate_context_size() + manager.active_cache_size()
    assert min_total > 0  # System should maintain some objects
    assert manager.immediate_context_size() <= 6  # ~20% of normal immediate limit


def test_memory_pressure_recovery():
    """Test system recovery after memory pressure is relieved."""
    manager = LifecycleManager()
    
    # Create initial objects
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(30)]
    for obj in objects:
        manager.activate(obj)
    
    # Record initial immediate context size
    initial_immediate = manager.immediate_context_size()
    
    # Apply memory pressure
    manager.set_memory_pressure(0.9)
    assert manager.immediate_context_size() < initial_immediate
    
    # Relieve memory pressure
    manager.set_memory_pressure(0.1)
    
    # Verify system recovers
    # Note: May not return exactly to initial size due to access patterns
    assert manager.immediate_context_size() >= initial_immediate * 0.8


def test_priority_handling():
    """Test that high priority objects remain in higher tiers under memory pressure."""
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


def test_gradual_pressure_adjustment():
    """Test that system adjusts gradually to increasing memory pressure."""
    manager = LifecycleManager()
    
    # Create initial set of objects
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(100)]
    for obj in objects:
        manager.activate(obj)
    
    # Record sizes at different pressure levels
    sizes = []
    for pressure in [0.2, 0.4, 0.6, 0.8]:
        manager.set_memory_pressure(pressure)
        sizes.append(manager.immediate_context_size())
    
    # Verify gradual reduction
    assert all(sizes[i] >= sizes[i + 1] for i in range(len(sizes) - 1))
    assert sizes[-1] > 0  # Should maintain some objects even at high pressure
