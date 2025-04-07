from tests.mocks.metrics_collector import MockMetricsCollector

class TestCacheMetrics:
    """Test suite for cache-specific metrics."""
    
    def test_cache_hit_recording(self, metrics_collector: MockMetricsCollector):
        """Test recording cache hits and misses."""
        # Record mix of hits and misses
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L1", False)
        
        # Verify counts
        hits = metrics_collector.metrics["cache_performance"]["hits"]["L1"]
        misses = metrics_collector.metrics["cache_performance"]["misses"]["L1"]
        assert hits == 2, "Should count cache hits correctly"
        assert misses == 1, "Should count cache misses correctly"
    
    def test_cache_level_separation(self, metrics_collector: MockMetricsCollector):
        """Test that cache levels are tracked separately."""
        # Record events for different cache levels
        metrics_collector.record_cache_event("L1", True)
        metrics_collector.record_cache_event("L2", True)
        metrics_collector.record_cache_event("L2", False)
        
        # Verify separate tracking
        l1_hits = metrics_collector.metrics["cache_performance"]["hits"]["L1"]
        l2_hits = metrics_collector.metrics["cache_performance"]["hits"]["L2"]
        l2_misses = metrics_collector.metrics["cache_performance"]["misses"]["L2"]
        
        assert l1_hits == 1, "Should track L1 cache hits separately"
        assert l2_hits == 1, "Should track L2 cache hits separately"
        assert l2_misses == 1, "Should track L2 cache misses separately"

    def test_initial_cache_state(self, metrics_collector: MockMetricsCollector):
        """Test that cache metrics are properly initialized."""
        # Verify initial state
        assert metrics_collector.metrics["cache_performance"]["hits"]["L1"] == 0, "L1 hits should start at 0"
        assert metrics_collector.metrics["cache_performance"]["hits"]["L2"] == 0, "L2 hits should start at 0"
        assert metrics_collector.metrics["cache_performance"]["misses"]["L1"] == 0, "L1 misses should start at 0"
        assert metrics_collector.metrics["cache_performance"]["misses"]["L2"] == 0, "L2 misses should start at 0"

    def test_cache_hit_rate_calculation(self, metrics_collector: MockMetricsCollector):
        """Test cache hit rate calculation in performance analysis."""
        # Record a mix of hits and misses
        metrics_collector.record_cache_event("L1", True)   # hit
        metrics_collector.record_cache_event("L1", True)   # hit
        metrics_collector.record_cache_event("L1", False)  # miss
        metrics_collector.record_cache_event("L2", True)   # hit
        metrics_collector.record_cache_event("L2", False)  # miss
        
        # Get performance analysis
        analysis = metrics_collector.analyze_performance()
        
        # Verify hit rates
        assert analysis["cache_hit_rate"]["L1"] == 2/3, "L1 cache hit rate should be calculated correctly"
        assert analysis["cache_hit_rate"]["L2"] == 1/2, "L2 cache hit rate should be calculated correctly"

    def test_empty_cache_hit_rate(self, metrics_collector: MockMetricsCollector):
        """Test cache hit rate calculation with no events."""
        analysis = metrics_collector.analyze_performance()
        
        # When no events are recorded, hit rate should be 0
        assert analysis["cache_hit_rate"]["L1"] == 0, "Empty L1 cache should have 0 hit rate"
        assert analysis["cache_hit_rate"]["L2"] == 0, "Empty L2 cache should have 0 hit rate"
