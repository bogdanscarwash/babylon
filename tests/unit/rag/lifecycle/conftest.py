"""Shared fixtures for lifecycle management tests."""

import pytest
from typing import Optional
from dataclasses import dataclass
from babylon.rag.lifecycle import ObjectState


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


@pytest.fixture
def mock_object():
    """Create a basic mock object for testing."""
    return MockObject(id="test1", data="test data")


@pytest.fixture
def invalid_mock_object():
    """Create an invalid mock object for testing."""
    return InvalidMockObject(data="test data")


@pytest.fixture
def mock_objects(count: int = 10):
    """Create a list of mock objects for testing."""
    return [
        MockObject(id=f"test{i}", data=f"data{i}")
        for i in range(count)
    ]
