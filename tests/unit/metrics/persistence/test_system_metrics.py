"""Unit tests for system metrics persistence operations.

Tests the functionality of system metrics storage and retrieval including:
- Basic CRUD operations
- Data validation
- Time range filtering
- Error handling
- Edge cases
"""

import pytest
from datetime import datetime, timedelta
import json
from typing import List

from babylon.metrics.performance_metrics import SystemMetrics
from babylon.exceptions import MetricsPersistenceError


class TestSystemMetricsPersistence:
    """Test suite for system metrics persistence operations."""

    def test_save_and_retrieve_system_metrics(self, metrics_db, sample_metrics):
        """Test basic save and retrieve operations for system metrics."""
        system_metrics, _, _ = sample_metrics
        
        # Save metrics
        metrics_db.save_system_metrics(system_metrics)
        
        # Retrieve and verify
        retrieved = metrics_db.get_system_metrics()
        assert len(retrieved) == 1, "Expected exactly one metric record"
        self._verify_system_metrics(retrieved[0], system_metrics)

    def test_save_multiple_system_metrics(self, metrics_db):
        """Test saving and retrieving multiple system metrics records."""
        base_time = datetime.now()
        metrics_list = []

        # Create and save multiple metrics
        for i in range(3):
            metrics = SystemMetrics(
                timestamp=(base_time + timedelta(minutes=i)).isoformat(),
                cpu_percent=50.0 + i,
                memory_percent=60.0 + i,
                swap_percent=30.0 + i,
                disk_usage_percent=70.0 + i,
                gpu_utilization=40.0 + i,
                gpu_memory_percent=45.0 + i
            )
            metrics_db.save_system_metrics(metrics)
            metrics_list.append(metrics)

        # Retrieve and verify all metrics
        retrieved = metrics_db.get_system_metrics()
        assert len(retrieved) == 3, "Expected three metric records"
        
        # Verify metrics are returned in chronological order
        for original, retrieved_metric in zip(metrics_list, retrieved):
            self._verify_system_metrics(retrieved_metric, original)

    @pytest.mark.parametrize("field,invalid_value,error_msg", [
        ("cpu_percent", -1.0, "cpu_percent must be between 0 and 100"),
        ("cpu_percent", 101.0, "cpu_percent must be between 0 and 100"),
        ("memory_percent", -1.0, "memory_percent must be between 0 and 100"),
        ("memory_percent", 150.0, "memory_percent must be between 0 and 100"),
        ("swap_percent", -5.0, "swap_percent must be between 0 and 100"),
        ("disk_usage_percent", 200.0, "disk_usage_percent must be between 0 and 100"),
        ("gpu_utilization", -10.0, "gpu_utilization must be between 0 and 100"),
        ("gpu_memory_percent", 120.0, "gpu_memory_percent must be between 0 and 100"),
    ])
    def test_system_metrics_validation(self, metrics_db, field, invalid_value, error_msg):
        """Test validation of system metrics data with various invalid values."""
        base_metrics = {
            "timestamp": datetime.now().isoformat(),
            "cpu_percent": 50.0,
            "memory_percent": 60.0,
            "swap_percent": 30.0,
            "disk_usage_percent": 70.0,
            "gpu_utilization": 40.0,
            "gpu_memory_percent": 45.0
        }
        
        # Update with invalid value
        base_metrics[field] = invalid_value
        
        with pytest.raises(ValueError, match=error_msg):
            metrics = SystemMetrics(**base_metrics)
            metrics_db.save_system_metrics(metrics)

    def test_time_range_filtering(self, metrics_db):
        """Test retrieving system metrics within a specific time range."""
        base_time = datetime.now()
        metrics_list = []

        # Create metrics at different times
        for i in range(5):
            metrics = SystemMetrics(
                timestamp=(base_time + timedelta(minutes=i*10)).isoformat(),
                cpu_percent=50.0 + i,
                memory_percent=60.0,
                swap_percent=30.0,
                disk_usage_percent=70.0,
                gpu_utilization=40.0,
                gpu_memory_percent=45.0
            )
            metrics_db.save_system_metrics(metrics)
            metrics_list.append(metrics)

        # Test different time ranges
        start_time = (base_time + timedelta(minutes=5)).isoformat()
        end_time = (base_time + timedelta(minutes=25)).isoformat()
        
        filtered = metrics_db.get_system_metrics(
            start_time=start_time,
            end_time=end_time
        )
        
        assert len(filtered) == 2, "Expected two metrics within time range"
        assert all(
            start_time <= m.timestamp <= end_time 
            for m in filtered
        ), "All metrics should be within time range"

    def test_edge_cases(self, metrics_db):
        """Test edge cases in system metrics operations."""
        # Test with minimum valid values
        min_metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=0.0,
            memory_percent=0.0,
            swap_percent=0.0,
            disk_usage_percent=0.0,
            gpu_utilization=0.0,
            gpu_memory_percent=0.0
        )
        metrics_db.save_system_metrics(min_metrics)

        # Test with maximum valid values
        max_metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=100.0,
            memory_percent=100.0,
            swap_percent=100.0,
            disk_usage_percent=100.0,
            gpu_utilization=100.0,
            gpu_memory_percent=100.0
        )
        metrics_db.save_system_metrics(max_metrics)

        # Verify both records were saved correctly
        retrieved = metrics_db.get_system_metrics()
        assert len(retrieved) == 2, "Expected two metric records"

    def test_error_handling(self, metrics_db):
        """Test error handling in system metrics operations."""
        # Test with invalid timestamp format
        with pytest.raises(ValueError):
            metrics = SystemMetrics(
                timestamp="not-a-timestamp",
                cpu_percent=50.0,
                memory_percent=60.0,
                swap_percent=30.0,
                disk_usage_percent=70.0,
                gpu_utilization=40.0,
                gpu_memory_percent=45.0
            )

        # Test with invalid time range
        with pytest.raises(ValueError):
            metrics_db.get_system_metrics(
                start_time="invalid-start",
                end_time="invalid-end"
            )

        # Test with end time before start time
        now = datetime.now()
        with pytest.raises(ValueError):
            metrics_db.get_system_metrics(
                start_time=(now + timedelta(hours=1)).isoformat(),
                end_time=now.isoformat()
            )

    def _verify_system_metrics(self, retrieved: SystemMetrics, original: SystemMetrics):
        """Helper method to verify system metrics equality.
        
        Args:
            retrieved: The metrics retrieved from the database
            original: The original metrics to compare against
        """
        assert retrieved.timestamp == original.timestamp
        assert retrieved.cpu_percent == original.cpu_percent
        assert retrieved.memory_percent == original.memory_percent
        assert retrieved.swap_percent == original.swap_percent
        assert retrieved.disk_usage_percent == original.disk_usage_percent
        assert retrieved.gpu_utilization == original.gpu_utilization
        assert retrieved.gpu_memory_percent == original.gpu_memory_percent

    # TODO: Add tests for:
    # - Database connection failure scenarios
    # - Concurrent access patterns
    # - Large dataset performance
    # - Metrics aggregation
    # - Data consistency checks
