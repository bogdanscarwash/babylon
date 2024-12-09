import pytest
from tests.mocks.metrics_collector import MockMetricsCollector
from datetime import datetime
from pathlib import Path
from typing import Dict, Any



@pytest.fixture
def metrics_collector():
    """Provide a fresh metrics collector for each test."""
    return MockMetricsCollector()

@pytest.fixture
def populated_collector():
    """Provide a metrics collector with some pre-populated data."""
    collector = MockMetricsCollector()
    
    # Add some sample data
    collector.record_object_access("obj1", "test")
    collector.record_object_access("obj1", "test")
    collector.record_object_access("obj2", "test")
    
    collector.record_cache_event("L1", True)
    collector.record_cache_event("L1", False)
    collector.record_cache_event("L2", True)
    
    collector.record_token_usage(100)
    collector.record_token_usage(150)
    
    collector.record_query_latency(0.5)
    collector.record_query_latency(0.7)
    
    collector.record_memory_usage(1024.0)
    collector.record_memory_usage(2048.0)
    
    return collector

@pytest.fixture
def temp_log_dir(tmp_path: Path) -> Path:
    """Create temporary directory for metrics logs."""
    return tmp_path / "test_logs"

@pytest.fixture
def metrics_collector(temp_log_dir: Path) -> MockMetricsCollector:
    """Provide fresh MetricsCollector instance."""
    return MockMetricsCollector(log_dir=temp_log_dir)

@pytest.fixture
def sample_performance_data() -> Dict[str, Any]:
    """Provide consistent test data for performance analysis."""
    return {
        "hot_object": {
            "id": "hot_object",
            "access_count": 5
        },
        "token_usage": [100, 200, 300],
        "cache_events": {
            "L1": {"hits": 8, "misses": 2}  # 80% hit rate
        },
        "latencies": [5.0, 10.0, 15.0],  # ms
        "memory_usage": [500, 1000, 1500]  # MB
    }