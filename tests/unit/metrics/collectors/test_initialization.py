import os
from tests.mocks.metrics_collector import MockMetricsCollector
from datetime import datetime
import pytest


class TestMetricsCollectorInit:
    """Test suite for MetricsCollector initialization."""
    
    def test_collector_initialization(
        self,
        metrics_collector: MockMetricsCollector,
        temp_log_dir: str
    ) -> None:
        """Verify collector initializes with correct default state."""
        assert metrics_collector.log_dir == temp_log_dir
        assert isinstance(
            metrics_collector.current_session["start_time"],
            datetime
        )
        assert metrics_collector.current_session["total_objects"] == 0
        assert len(metrics_collector.metrics["object_access"]) == 0