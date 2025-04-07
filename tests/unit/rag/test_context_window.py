"""Tests for the Context Window Management component."""

import pytest
from datetime import datetime, timedelta
import time
from unittest.mock import MagicMock, patch

from babylon.rag.context_window.manager import ContextWindowManager
from babylon.rag.context_window.config import ContextWindowConfig
from babylon.rag.context_window.errors import CapacityExceededError, ContentInsertionError
from babylon.rag.context_window.token_counter import count_tokens


@pytest.fixture
def context_window_config():
    """Create a test configuration."""
    return ContextWindowConfig(
        max_token_limit=1000,  # Smaller for testing
        capacity_threshold=0.8,
        prioritization_strategy="hybrid",
        min_content_importance=0.2
    )


@pytest.fixture
def metrics_collector():
    """Create a mock metrics collector."""
    mock = MagicMock()
    return mock


@pytest.fixture
def lifecycle_manager():
    """Create a mock lifecycle manager."""
    mock = MagicMock()
    return mock


@pytest.fixture
def context_window(context_window_config, metrics_collector, lifecycle_manager):
    """Create a ContextWindowManager for testing."""
    manager = ContextWindowManager(
        config=context_window_config,
        metrics_collector=metrics_collector,
        lifecycle_manager=lifecycle_manager
    )
    return manager


def test_init(context_window, context_window_config):
    """Test initialization."""
    assert context_window.config == context_window_config
    assert context_window.total_tokens == 0
    assert context_window.content_count == 0
    assert context_window.capacity_percentage == 0.0


def test_add_content(context_window):
    """Test adding content to the context window."""
    result = context_window.add_content("test1", "This is test content", 10, 0.5)
    
    assert result is True
    assert context_window.total_tokens == 10
    assert context_window.content_count == 1
    assert context_window.capacity_percentage == 0.01  # 10/1000
    
    context_window.metrics_collector.record_token_usage.assert_called_with(10)


def test_get_content(context_window):
    """Test retrieving content from the context window."""
    content = "This is test content"
    context_window.add_content("test1", content, 10, 0.5)
    
    retrieved = context_window.get_content("test1")
    assert retrieved == content
    
    with pytest.raises(KeyError):
        context_window.get_content("nonexistent")


def test_remove_content(context_window):
    """Test removing content from the context window."""
    context_window.add_content("test1", "Content to remove", 15, 0.5)
    
    assert context_window.total_tokens == 15
    
    result = context_window.remove_content("test1")
    assert result is True
    assert context_window.total_tokens == 0
    assert context_window.content_count == 0
    
    result = context_window.remove_content("nonexistent")
    assert result is False


def test_capacity_management(context_window, context_window_config):
    """Test context window capacity management."""
    large_content = "X" * 700  # Will result in ~700 tokens
    context_window.add_content("large1", large_content, 700, 0.5)
    
    assert context_window.total_tokens == 700
    assert context_window.capacity_percentage == 0.7  # 700/1000
    
    large_content2 = "Y" * 200  # Will result in ~200 tokens
    
    context_window.add_content("large2", large_content2, 200, 0.3)
    
    assert context_window.total_tokens <= 1000
    assert context_window.capacity_percentage <= 1.0


def test_optimization(context_window):
    """Test explicit optimization of the context window."""
    context_window.add_content("low", "Low priority content", 200, 0.1)
    context_window.add_content("medium", "Medium priority content", 200, 0.5)
    context_window.add_content("high", "High priority content", 200, 0.9)
    
    assert context_window.total_tokens == 600
    
    result = context_window.optimize(target_tokens=400)
    
    assert result is True
    assert context_window.total_tokens <= 400
    
    with pytest.raises(KeyError):
        context_window.get_content("low")


def test_token_counting():
    """Test the token counting utility."""
    assert count_tokens("This is a test string") >= 5  # At least 5 tokens
    
    assert count_tokens(["item1", "item2", "item3"]) >= 3
    
    test_dict = {"key1": "value1", "key2": "value2"}
    assert count_tokens(test_dict) >= 4  # At least 4 tokens (2 keys + 2 values)


def test_error_handling(context_window):
    """Test error handling in the context window."""
    context_window.add_content("fill", "X" * 800, 800, 0.5)
    
    with pytest.raises(CapacityExceededError):
        context_window.add_content("overflow", "Y" * 500, 500, 0.1)


def test_content_priority(context_window):
    """Test content prioritization logic."""
    context_window.add_content("high", "High priority content", 100, 0.9)
    context_window.add_content("medium", "Medium priority content", 100, 0.5)
    context_window.add_content("low", "Low priority content", 100, 0.1)
    
    context_window.add_content("filler", "X" * 600, 600, 0.3)
    
    with pytest.raises(KeyError):
        context_window.get_content("low")
    
    assert context_window.get_content("high") == "High priority content"


def test_stats(context_window):
    """Test getting statistics from the context window."""
    context_window.add_content("test1", "Content 1", 100, 0.5)
    context_window.add_content("test2", "Content 2", 200, 0.7)
    
    stats = context_window.get_stats()
    
    assert stats["total_tokens"] == 300
    assert stats["content_count"] == 2
    assert stats["capacity_percentage"] == 0.3
    assert stats["content_added"] == 2
