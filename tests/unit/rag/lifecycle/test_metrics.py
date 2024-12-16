"""Tests for performance metrics and monitoring."""

import pytest
import time
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


def test_performance_metrics():
    """Test that performance metrics are properly collected and reported."""
    manager = LifecycleManager()
    
    # Create and activate some objects
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(50)]
    for obj in objects:
        manager.activate(obj)
    
    # Deactivate some objects to get deactivation metrics
    for obj in objects[:10]:  # Deactivate first 10 objects
        manager.deactivate(obj)
    
    # Force some cache hits and misses
    for i in range(5):
        # Cache miss
        manager.get_object(f"nonexistent{i}")
        # Cache hit (from remaining active objects)
        manager.get_object(objects[20].id)
    
    # Get metrics
    metrics = manager.get_metrics()
    
    # Check operation counts
    assert metrics.activation_count == 50  # Initial activations
    assert metrics.deactivation_count == 10  # Explicit deactivations
    assert metrics.cache_miss_count == 5  # Nonexistent objects
    assert metrics.cache_hit_count == 5  # Successful retrievals
    assert metrics.tier_transition_count > 0  # Some objects should have been demoted
    
    # Check timing metrics
    assert metrics.avg_activation_time > 0
    assert metrics.avg_deactivation_time > 0
    
    # Check memory pressure stats
    assert 0 <= metrics.avg_memory_pressure <= 1.0
    
    # Check tier statistics
    assert metrics.immediate_context_usage <= 1.0  # Should be a percentage
    assert metrics.active_cache_usage <= 1.0
    assert metrics.background_context_usage <= 1.0


def test_cache_hit_tracking():
    """Test that cache hits and misses are properly tracked."""
    manager = LifecycleManager()
    
    # Create and activate an object
    obj = MockObject(id="test1", data="test data")
    manager.activate(obj)
    
    # Test cache hits from different tiers
    manager.get_object(obj.id)  # Hit in immediate
    manager.mark_inactive(obj)  # Move to active
    manager.get_object(obj.id)  # Hit in active + promotion
    manager.mark_inactive(obj)  # Move to active
    manager.add_to_background(obj)  # Move to background
    manager.get_object(obj.id)  # Hit in background + promotion
    
    # Test cache misses
    manager.get_object("nonexistent1")
    manager.get_object("nonexistent2")
    
    metrics = manager.get_metrics()
    assert metrics.cache_hit_count == 3
    assert metrics.cache_miss_count == 2


def test_tier_transition_tracking():
    """Test that tier transitions are properly counted."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    initial_transitions = manager.get_metrics().tier_transition_count
    
    # Each of these should count as a transition
    manager.activate(obj)  # INACTIVE -> IMMEDIATE
    manager.mark_inactive(obj)  # IMMEDIATE -> ACTIVE
    manager.add_to_background(obj)  # ACTIVE -> BACKGROUND
    manager.deactivate(obj)  # BACKGROUND -> INACTIVE
    
    metrics = manager.get_metrics()
    assert metrics.tier_transition_count == initial_transitions + 4


def test_memory_pressure_metrics():
    """Test that memory pressure metrics are properly tracked."""
    manager = LifecycleManager()
    
    # Set various pressure levels
    pressures = [0.1, 0.5, 0.9, 0.3]
    for pressure in pressures:
        manager.set_memory_pressure(pressure)
    
    metrics = manager.get_metrics()
    
    # Average should be the mean of all pressure values
    expected_avg = sum(pressures) / len(pressures)
    assert abs(metrics.avg_memory_pressure - expected_avg) < 0.01
    
    # Peak should be the highest pressure value
    assert metrics.peak_memory_pressure == max(pressures)


def test_tier_usage_metrics():
    """Test that tier usage percentages are correctly calculated."""
    manager = LifecycleManager()
    
    # Fill immediate context to 50% capacity
    objects = [
        MockObject(id=f"test{i}", data=f"data{i}")
        for i in range(15)  # 15 is 50% of default immediate limit (30)
    ]
    for obj in objects:
        manager.activate(obj)
    
    metrics = manager.get_metrics()
    assert 0.45 <= metrics.immediate_context_usage <= 0.55  # Allow small margin
    assert metrics.active_cache_usage == 0.0
    assert metrics.background_context_usage == 0.0
