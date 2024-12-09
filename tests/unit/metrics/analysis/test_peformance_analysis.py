import pytest
from statistics import mean
from typing import Dict, List

from tests.mocks.metrics_collector import MockMetricsCollector

@pytest.fixture
def metrics_collector() -> MockMetricsCollector:
    """Fixture providing a fresh MockMetricsCollector instance."""
    return MockMetricsCollector()

@pytest.fixture
def sample_memory_values() -> List[float]:
    """Fixture providing test memory usage values in bytes."""
    return [1000.0, 2000.0, 1500.0]

@pytest.fixture
def sample_latency_values() -> List[float]:
    """Fixture providing test latency values in milliseconds."""
    return [5.0, 10.0, 15.0]

class TestMemoryAnalysis:
    """Test suite for memory usage analysis functionality."""
    
    def test_memory_stats_calculation(
        self,
        metrics_collector: MockMetricsCollector,
        sample_memory_values: List[float]
    ) -> None:
        """Test that memory statistics are calculated correctly."""
        # Record sample memory values
        for value in sample_memory_values:
            metrics_collector.record_memory_usage(value)
            
        # Get memory statistics
        memory_stats = metrics_collector._analyze_memory_usage()
        
        # Verify statistics
        assert memory_stats["avg"] == pytest.approx(mean(sample_memory_values)), \
            "Average memory usage calculation incorrect"
        assert memory_stats["peak"] == max(sample_memory_values), \
            "Peak memory usage calculation incorrect"
        assert memory_stats["current"] == sample_memory_values[-1], \
            "Current memory usage value incorrect"

    def test_empty_memory_stats(self, metrics_collector: MockMetricsCollector) -> None:
        """Test handling of memory statistics when no data is recorded."""
        with pytest.raises(ValueError, match="No memory usage data available"):
            metrics_collector._analyze_memory_usage()

class TestLatencyAnalysis:
    """Test suite for query latency analysis functionality."""
    
    def test_latency_stats_calculation(
        self,
        metrics_collector: MockMetricsCollector,
        sample_latency_values: List[float]
    ) -> None:
        """Test that latency statistics are calculated correctly."""
        # Record sample latency values
        for latency in sample_latency_values:
            metrics_collector.record_query_latency(latency)
            
        # Get latency statistics
        latency_stats = metrics_collector._calculate_latency_stats()
        
        # Verify statistics
        assert "db_queries" in latency_stats, "Missing db_queries category"
        db_stats = latency_stats["db_queries"]
        
        assert db_stats["avg"] == pytest.approx(mean(sample_latency_values)), \
            "Average latency calculation incorrect"
        assert db_stats["min"] == min(sample_latency_values), \
            "Minimum latency calculation incorrect"
        assert db_stats["max"] == max(sample_latency_values), \
            "Maximum latency calculation incorrect"

    def test_empty_latency_stats(self, metrics_collector: MockMetricsCollector) -> None:
        """Test handling of latency statistics when no data is recorded."""
        with pytest.raises(ValueError, match="No latency data available"):
            metrics_collector._calculate_latency_stats()
