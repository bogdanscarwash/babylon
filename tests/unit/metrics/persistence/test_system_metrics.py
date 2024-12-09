import pytest
from datetime import datetime
from tests.unit.metrics.persistence.conftest import SystemMetrics, metrics_db

class TestSystemMetricsPersistence:
    """Test suite for system metrics persistence operations."""
    
    def test_save_and_retrieve_system_metrics(self, metrics_db: metrics_db, sample_metrics: SystemMetrics):
        """Test the full cycle of saving and retrieving system metrics."""
        system_metrics, _, _ = sample_metrics
        
        # Save metrics
        metrics_db.save_system_metrics(system_metrics)
        
        # Retrieve and verify
        retrieved = metrics_db.get_system_metrics()
        self._verify_system_metrics(retrieved[0], system_metrics)
    
    def test_system_metrics_validation(self, metrics_db):
        """Test validation of system metrics data."""
        # Test with invalid values
        with pytest.raises(ValueError):
            invalid_metrics = SystemMetrics(
                timestamp=datetime.now().isoformat(),
                cpu_percent=-50.0,  # Invalid negative value
                memory_percent=60.0,
                swap_percent=30.0,
                disk_usage_percent=70.0,
                gpu_utilization=40.0,
                gpu_memory_percent=45.0
            )
            metrics_db.save_system_metrics(invalid_metrics)
    
    def _verify_system_metrics(self, retrieved: SystemMetrics, original: SystemMetrics):
        """Helper method to verify system metrics equality."""
        assert retrieved.cpu_percent == original.cpu_percent
        assert retrieved.memory_percent == original.memory_percent
        assert retrieved.swap_percent == original.swap_percent
        assert retrieved.disk_usage_percent == original.disk_usage_percent
        assert retrieved.gpu_utilization == original.gpu_utilization
        assert retrieved.gpu_memory_percent == original.gpu_memory_percent