import pytest
from tests.mocks.metrics_collector import MockMetricsCollector
from datetime import datetime
from pathlib import Path
from typing import Dict, Any
from babylon.core.contradiction import ContradictionAnalysis
from babylon.data.entity_registry import EntityRegistry
from babylon.data.models.contradiction import Contradiction, Effect
from babylon.core.entity import Entity

@pytest.fixture
def metrics_collector(temp_log_dir):
    """Provide a fresh metrics collector for each test."""
    return MockMetricsCollector(log_dir=temp_log_dir)

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
def entity_registry() -> EntityRegistry:
    """Provide a fresh EntityRegistry instance."""
    return EntityRegistry()

@pytest.fixture
def contradiction_analysis(entity_registry: EntityRegistry, metrics_collector: MockMetricsCollector) -> ContradictionAnalysis:
    """Provide a fresh ContradictionAnalysis instance."""
    return ContradictionAnalysis(entity_registry, metrics_collector)

@pytest.fixture
def sample_contradiction() -> Contradiction:
    """Provide a sample contradiction for testing."""
    upper_class = Entity("Class", "Oppressor")  # Fixed: removed extra argument
    working_class = Entity("Class", "Oppressed")  # Fixed: removed extra argument
    
    return Contradiction(
        id="economic_inequality",
        name="Economic Inequality",
        description="Growing disparity between rich and poor.",
        entities=[upper_class, working_class],
        universality="Universal",
        particularity="Economic",
        principal_contradiction=None,
        principal_aspect=upper_class,
        secondary_aspect=working_class,
        antagonism="Antagonistic",
        intensity="Medium",
        state="Active",
        potential_for_transformation="High",
        conditions_for_transformation=["Revolutionary Movement"],
        resolution_methods={
            "Policy Reform": [
                Effect("upper_class", "wealth", "Decrease", 0.5, "Implement reforms")
            ],
            "Revolution": [
                Effect("upper_class", "wealth", "Decrease", 1.0, "Revolutionary change")
            ],
        },
        attributes={},
    )

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
