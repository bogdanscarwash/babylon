from typing import Dict, Any
from src.babylon.core.contradiction import ContradictionAnalysis
from src.babylon.data.models.contradiction import Contradiction
import pytest

class TestContradictionMetrics:
    """Test suite for contradiction-related metrics collection."""
    
    def test_contradiction_creation_metrics(
        self, 
        contradiction_analysis: ContradictionAnalysis, 
        sample_contradiction: Contradiction
    ):
        """Test metrics collection when creating contradictions.
        
        Verifies that:
        1. The contradiction is added to hot objects
        2. Contradiction count is incremented
        3. Access metrics are recorded
        """
        contradiction_analysis.add_contradiction(sample_contradiction)
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        self._verify_contradiction_metrics(metrics, sample_contradiction.id)
    
    def test_contradiction_context_switches(
        self, 
        contradiction_analysis: ContradictionAnalysis, 
        sample_contradiction: Contradiction
    ):
        """Test context switch tracking in contradiction analysis.
        
        Verifies that:
        1. Context switches are recorded during analysis
        2. Switch count is incremented
        3. Latency stats include context switch times
        """
        contradiction_analysis.add_contradiction(sample_contradiction)
        
        # Create game state with required economic data
        game_state = {
            "economy": {
                "gini_coefficient": 0.5  # Will trigger intensity update
            }
        }
        
        # Update the contradiction to trigger context switch
        contradiction_analysis._update_contradiction(sample_contradiction, game_state)
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        self._verify_context_switches(metrics)
    
    def test_contradiction_error_handling(
        self, 
        contradiction_analysis: ContradictionAnalysis
    ):
        """Test metrics collection during error conditions.
        
        Verifies that:
        1. Errors are properly recorded in metrics
        2. Error count is incremented
        3. Failed operations are tracked
        """
        with pytest.raises(AttributeError):
            # Try to update a nonexistent contradiction
            contradiction_analysis._update_contradiction(None, {})
        
        metrics = contradiction_analysis.metrics.analyze_performance()
        assert metrics["error_count"] > 0, "Error should be recorded in metrics"
        assert "failed_operations" in metrics, "Failed operations should be tracked"
    
    def test_multiple_contradiction_tracking(
        self,
        contradiction_analysis: ContradictionAnalysis,
        sample_contradiction: Contradiction
    ):
        """Test tracking metrics for multiple contradictions.
        
        Verifies that:
        1. Multiple contradictions are properly tracked
        2. Access patterns are recorded
        3. Performance impact is measured
        """
        # Add same contradiction multiple times
        for _ in range(3):
            contradiction_analysis.add_contradiction(sample_contradiction)
            
        metrics = contradiction_analysis.metrics.analyze_performance()
        assert metrics["contradiction_count"] == 3, (
            "Should track correct number of contradictions"
        )
        assert metrics["hot_objects"][sample_contradiction.id]["access_count"] >= 3, (
            "Should track multiple accesses to same contradiction"
        )
    
    def _verify_contradiction_metrics(
        self, 
        metrics: Dict[str, Any], 
        contradiction_id: str
    ):
        """Helper method to verify contradiction metrics.
        
        Args:
            metrics: Performance metrics dictionary
            contradiction_id: ID of contradiction being verified
        """
        assert contradiction_id in metrics["hot_objects"], (
            f"Contradiction {contradiction_id} should be in hot objects list"
        )
        
        assert "contradiction_count" in metrics, (
            "Metrics should include contradiction count"
        )
        
        assert metrics["contradiction_count"] > 0, (
            "Contradiction count should be incremented"
        )
        
        assert metrics["hot_objects"][contradiction_id]["access_count"] > 0, (
            "Access count should be tracked for contradiction"
        )
    
    def _verify_context_switches(self, metrics: Dict[str, Any]):
        """Helper method to verify context switch metrics.
        
        Args:
            metrics: Performance metrics dictionary
        """
        assert "latency_stats" in metrics, (
            "Metrics should include latency stats"
        )
        
        assert "context_switches" in metrics.get("latency_stats", {}), (
            "Latency stats should include context switches"
        )
        
        assert len(metrics["latency_stats"]["context_switches"]["values"]) > 0, (
            "Should have recorded at least one context switch"
        )
