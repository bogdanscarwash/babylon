from src.babylon.core.contradiction import ContradictionAnalysis
import pytest

class TestContradictionMetrics:
    """Test suite for contradiction-related metrics collection."""
    
    def test_contradiction_creation_metrics(
        self, 
        contradiction_analysis: ContradictionAnalysis, 
        sample_contradiction
    ):
        """Test metrics collection when creating contradictions."""
        contradiction_analysis.add_contradiction(sample_contradiction)
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        self._verify_contradiction_metrics(metrics, sample_contradiction.id)
    
    def test_contradiction_context_switches(
        self, 
        contradiction_analysis, 
        sample_contradiction
    ):
        """Test context switch tracking in contradiction analysis."""
        contradiction_analysis.add_contradiction(sample_contradiction)
        
        # Analyze the contradiction to trigger context switch
        contradiction_analysis.analyze_contradiction(sample_contradiction.id)
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        self._verify_context_switches(metrics)
    
    def test_contradiction_error_handling(
        self, 
        contradiction_analysis
    ):
        """Test metrics collection during error conditions."""
        with pytest.raises(ValueError):
            contradiction_analysis.analyze_contradiction("nonexistent_id")
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        assert metrics["error_count"] > 0, "Error should be recorded in metrics"
    
    def _verify_contradiction_metrics(
        self, 
        metrics: Dict[str, Any], 
        contradiction_id: str
    ):
        """Helper method to verify contradiction metrics."""
        assert contradiction_id in metrics["hot_objects"], (
            f"Contradiction {contradiction_id} should be in hot objects list"
        )
        
        # Verify any contradiction-specific metrics
        assert "contradiction_count" in metrics, (
            "Metrics should include contradiction count"
        )
    
    def _verify_context_switches(self, metrics: Dict[str, Any]):
        """Helper method to verify context switch metrics."""
        assert metrics["latency_stats"]["context_switches"], (
            "Context switches should be recorded"
        )
        
        assert "context_switch_count" in metrics, (
            "Metrics should include context switch count"
        )