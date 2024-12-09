class TestMetricsRecording:
    """Test suite for metrics recording functionality."""
    
    def test_object_access_recording(self, metrics_collector):
        """Test recording and counting object accesses."""
        # Record multiple accesses
        metrics_collector.record_object_access("test_obj", "test")
        metrics_collector.record_object_access("test_obj", "test")
        
        # Verify count
        assert metrics_collector.access_records["test_obj"] == 2, (
            "Should correctly count multiple accesses to same object"
        )
        
        # Test new object
        metrics_collector.record_object_access("another_obj", "test")
        assert len(metrics_collector.access_records) == 2, (
            "Should track multiple distinct objects"
        )
    
    def test_token_usage_recording(self, metrics_collector):
        """Test recording token usage metrics."""
        test_tokens = [100, 150, 200]
        
        # Record token usage
        for tokens in test_tokens:
            metrics_collector.record_token_usage(tokens)
        
        assert len(metrics_collector.token_usage) == len(test_tokens), (
            "Should record all token usage entries"
        )
        assert metrics_collector.token_usage == test_tokens, (
            "Should preserve token usage values in order"
        )