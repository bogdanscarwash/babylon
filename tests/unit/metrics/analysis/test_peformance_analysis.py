from tests.mocks.metrics_collector import MockMetricsCollector
from typing import Dict, List

class TestPerformanceAnalysis:
    """Test suite for performance analysis functionality."""
    
    def test_memory_analysis(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test memory usage statistics calculation."""
        test_values = [1000, 2000, 1500]
        for value in test_values:
            metrics_collector.record_memory_usage(value)

        memory_stats = metrics_collector._analyze_memory_usage()
        self._verify_memory_stats(memory_stats, test_values)
    
    def test_latency_analysis(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test latency statistics calculation."""
        test_latencies = [5.0, 10.0, 15.0]
        for latency in test_latencies:
            metrics_collector.record_query_latency(latency)

        latency_stats = metrics_collector._calculate_latency_stats()
        self._verify_latency_stats(latency_stats, test_latencies)
    
    def _verify_memory_stats(
        self,
        stats: Dict[str, float],
        values: List[float]
    ) -> None:
        """Helper method to verify memory statistics."""
        assert stats["avg"] == sum(values) / len(values)
        assert stats["peak"] == max(values)
        assert stats["current"] == values[-1]
    
    def _verify_latency_stats(
        self,
        stats: Dict[str, Dict[str, float]],
        values: List[float]
    ) -> None:
        """Helper method to verify latency statistics."""
        assert "db_queries" in stats
        db_stats = stats["db_queries"]
        assert db_stats["avg"] == sum(values) / len(values)
        assert db_stats["min"] == min(values)
        assert db_stats["max"] == max(values)