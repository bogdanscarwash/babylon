import pytest
import os
from tests.mocks.metrics_collector import MockMetricsCollector


class TestObjectTracking:
    """Test suite for object access tracking."""
    
    def test_single_object_multiple_access(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test multiple accesses to single object are counted correctly."""
        object_id = "test_obj_1"
        access_count = 3
        
        for _ in range(access_count):
            metrics_collector.record_object_access(object_id, "test_context")
        
        assert (metrics_collector.metrics["object_access"][object_id] 
                == access_count)
    
    def test_multiple_objects_tracking(
        self,
        metrics_collector: MockMetricsCollector
    ) -> None:
        """Test tracking of multiple distinct objects."""
        objects = {"obj1": 2, "obj2": 1, "obj3": 3}
        
        for obj_id, count in objects.items():
            for _ in range(count):
                metrics_collector.record_object_access(obj_id, "test_context")
        
        for obj_id, expected_count in objects.items():
            assert (metrics_collector.metrics["object_access"][obj_id] 
                    == expected_count)