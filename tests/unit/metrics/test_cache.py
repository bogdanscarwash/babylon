from tests.unit.collectors import MockMetricsCollector

class TestCacheMetrics:
    """Test suite for cache-specific metrics."""
    
    def test_cache_hit_recording(self, metrics_collector: MockMetricsCollector):
        """Test recording cache hits and misses."""
        # Record mix of hits and misses
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L1", False)
        
        # Verify counts
        assert metrics_collector.cache_hits["L1"] == 2, (
            "Should count cache hits correctly"
        )
        assert metrics_collector.cache_misses["L1"] == 1, (
            "Should count cache misses correctly"
        )
    
    def test_cache_level_separation(self, metrics_collector: MockMetricsCollector):
        """Test that cache levels are tracked separately."""
        # Record events for different cache levels
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L2", True)
        metrics_collector.record_cache_event("L2", False)
        
        # Verify separate tracking
        assert metrics_collector.cache_hits["L1"] == 1, (
            "Should track L1 cache hits separately"
        )
        assert metrics_collector.cache_hits["L2"] == 1, (
            "Should track L2 cache hits separately"
        )
        assert metrics_collector.cache_misses["L2"] == 1, (
            "Should track L2 cache misses separately"
        )