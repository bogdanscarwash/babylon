import pytest
import os
from tests.mocks.metrics_collector import MockMetricsCollector

class TestCachePerformance:
    """Test suite for cache performance monitoring."""
    
    def test_cache_hit_recording(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test recording of cache hits."""
        metrics_collector.record_cache_event("L1", True)
        assert (metrics_collector.metrics["cache_performance"]["hits"]["L1"] 
                == 1)
    
    def test_cache_miss_recording(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test recording of cache misses."""
        metrics_collector.record_cache_event("L2", False)
        assert (metrics_collector.metrics["cache_performance"]["misses"]["L2"] 
                == 1)
    
    def test_hit_rate_calculation(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test calculation of cache hit rates."""
        # Record 8 hits and 2 misses for 80% hit rate
        for _ in range(8):
            metrics_collector.record_cache_event("L1", True)
        for _ in range(2):
            metrics_collector.record_cache_event("L1", False)
            
        analysis = metrics_collector.analyze_performance()
        assert analysis["cache_hit_rate"]["L1"] == 0.8