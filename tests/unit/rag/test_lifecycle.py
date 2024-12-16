"""Integration tests for the RAG system's lifecycle management.

This file serves as a high-level entry point for lifecycle management tests.
Detailed test cases are organized in the lifecycle/ subdirectory:

- test_basic_operations.py: Basic lifecycle operations (creation, activation, deactivation)
- test_state_management.py: State transitions and working set management
- test_memory_pressure.py: Memory pressure handling and tier management
- test_metrics.py: Performance metrics and monitoring
- test_error_handling.py: Error cases and recovery scenarios
"""

import pytest
from typing import Optional
from dataclasses import dataclass
import time
from babylon.rag.lifecycle import LifecycleManager, ObjectState
from babylon.rag.exceptions import InvalidObjectError

# Import all tests to make them available when running pytest on this file
from .lifecycle.test_basic_operations import *
from .lifecycle.test_state_management import *
from .lifecycle.test_memory_pressure import *
from .lifecycle.test_metrics import *
from .lifecycle.test_error_handling import *


def test_lifecycle_integration():
    """Integration test exercising multiple aspects of lifecycle management."""
    manager = LifecycleManager()
    
    # Test basic operations with memory pressure
    objects = [MockObject(id=f"test{i}", data=f"data{i}") for i in range(50)]
    
    # Activate objects and verify tier management
    for obj in objects:
        manager.activate(obj)
    assert manager.immediate_context_size() <= 30
    
    # Apply memory pressure and verify system response
    manager.set_memory_pressure(0.8)
    assert manager.immediate_context_size() < 30
    
    # Test error handling during normal operations
    with pytest.raises(InvalidObjectError):
        manager.activate(InvalidMockObject(data="invalid"))
    
    # Verify metrics are being collected
    metrics = manager.get_metrics()
    assert metrics.activation_count == 50
    assert metrics.tier_transition_count > 0
    
    # Test recovery from memory pressure
    manager.set_memory_pressure(0.1)
    assert manager.immediate_context_size() > 15  # Should recover to higher capacity
    
    # Final state verification
    total_objects = (
        manager.immediate_context_size() +
        manager.active_cache_size() +
        manager.background_context_size()
    )
    assert total_objects == 50  # All objects should still be managed
