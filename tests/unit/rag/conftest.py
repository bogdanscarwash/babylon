"""Test fixtures for RAG system tests."""

import pytest
from typing import Optional
from dataclasses import dataclass
from babylon.rag.lifecycle import ObjectState, LifecycleManager


@pytest.fixture
def mock_object():
    """Create a mock object for testing."""
    @dataclass
    class MockObject:
        id: str
        data: str
        state: ObjectState = ObjectState.INACTIVE
        last_accessed: Optional[float] = None
    
    return MockObject


@pytest.fixture
def lifecycle_manager():
    """Create a fresh lifecycle manager for testing."""
    return LifecycleManager()


@pytest.fixture
def populated_manager(lifecycle_manager, mock_object):
    """Create a lifecycle manager pre-populated with test objects."""
    # Add some immediate context objects
    for i in range(10):
        obj = mock_object(id=f"immediate{i}", data=f"immediate_data{i}")
        lifecycle_manager.activate(obj, priority=2)
    
    # Add some active cache objects
    for i in range(50):
        obj = mock_object(id=f"active{i}", data=f"active_data{i}")
        lifecycle_manager.activate(obj, priority=1)
    
    # Add some background objects
    for i in range(100):
        obj = mock_object(id=f"background{i}", data=f"background_data{i}")
        lifecycle_manager.add_to_background(obj)
    
    return lifecycle_manager
