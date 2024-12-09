class TestMetricsAnalysis:
    """Test suite for metrics analysis functionality."""
    
    def test_performance_analysis(self, populated_collector):
        """Test the performance analysis results."""
        analysis = populated_collector.analyze_performance()
        
        # Test hot objects analysis
        assert "obj1" == analysis["hot_objects"][0], (
            "Most frequently accessed object should be first"
        )
        
        # Test cache hit rate calculation
        l1_rate = analysis["cache_hit_rate"]["L1"]
        assert l1_rate == 0.5, "L1 cache hit rate should be 50%"
        
        # Test average calculations
        assert analysis["avg_token_usage"] == 125.0, (
            "Should calculate correct average token usage"
        )
        
        # Test memory profile
        assert analysis["memory_profile"]["peak"] == 2048.0, (
            "Should identify correct peak memory usage"
        )