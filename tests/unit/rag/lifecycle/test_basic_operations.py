"""Tests for basic lifecycle operations."""

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


def test_object_deactivation():
    """Test that objects can be fully deactivated."""
    manager = LifecycleManager()
    obj = MockObject(id="test1", data="test data")
    
    manager.activate(obj)
    assert obj.state == ObjectState.IMMEDIATE
    
    manager.deactivate(obj)
    assert obj.state == ObjectState.INACTIVE


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
    obj = MockObject(id="test1", data="test data")
    
    # Put object in background
    manager.add_to_background(obj)
    assert obj.state == ObjectState.BACKGROUND
    
    # Promote to active
    manager.activate(obj)
    assert obj.state == ObjectState.IMMEDIATE
