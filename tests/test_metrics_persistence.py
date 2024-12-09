import os
import tempfile
import unittest
from datetime import datetime, timedelta

from babylon.metrics.performance_metrics import (
    AIMetrics,
    GameplayMetrics,
    SystemMetrics,
)
from babylon.metrics.persistence import MetricsPersistence


class TestMetricsPersistence(unittest.TestCase):
    def setUp(self):
        """Initialize test environment."""
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test_metrics.db")
        self.persistence = MetricsPersistence(self.db_path)

    def tearDown(self):
        """Clean up test environment."""
        try:
            if os.path.exists(self.db_path):
                os.remove(self.db_path)
            os.rmdir(self.temp_dir)
        except Exception as e:
            print(f"Cleanup error: {e}")

    def test_save_and_retrieve_system_metrics(self):
        """Test saving and retrieving system metrics."""
        try:
            # Create test metrics
            metrics = SystemMetrics(
                timestamp=datetime.now().isoformat(),
                cpu_percent=50.0,
                memory_percent=60.0,
                swap_percent=30.0,
                disk_usage_percent=70.0,
                gpu_utilization=40.0,
                gpu_memory_percent=45.0,
            )

            # Save metrics
            try:
                self.persistence.save_system_metrics(metrics)
            except Exception as e:
                self.fail(f"Failed to save metrics: {e!s}")

            # Retrieve and verify metrics
            try:
                retrieved = self.persistence.get_system_metrics()
                self.assertEqual(len(retrieved), 1, "Expected exactly one metric record")
                self.assertEqual(retrieved[0].cpu_percent, metrics.cpu_percent)
                self.assertEqual(retrieved[0].memory_percent, metrics.memory_percent)
            except Exception as e:
                self.fail(f"Failed to retrieve metrics: {e!s}")

        except ValueError as e:
            self.fail(f"Invalid metric values: {e!s}")

    def test_save_and_retrieve_ai_metrics(self):
        """Test saving and retrieving AI metrics."""
        metrics = AIMetrics(
            query_latency_ms=50.0,
            memory_usage_gb=1.5,
            token_count=1000,
            embedding_dimension=384,
            cache_hit_rate=0.95,
            anomaly_score=0.1,
            threshold_violations=[],
        )

        self.persistence.save_ai_metrics(metrics)

        retrieved = self.persistence.get_ai_metrics()
        self.assertEqual(len(retrieved), 1, "Expected exactly one AI metric record")
        self.assertEqual(retrieved[0].query_latency_ms, metrics.query_latency_ms)
        self.assertEqual(retrieved[0].memory_usage_gb, metrics.memory_usage_gb)
        self.assertEqual(retrieved[0].token_count, metrics.token_count)
        self.assertEqual(retrieved[0].embedding_dimension, metrics.embedding_dimension)
        self.assertEqual(retrieved[0].cache_hit_rate, metrics.cache_hit_rate)
        self.assertEqual(retrieved[0].anomaly_score, metrics.anomaly_score)

    def test_save_and_retrieve_gameplay_metrics(self):
        """Test saving and retrieving gameplay metrics."""
        metrics = GameplayMetrics(
            session_duration=3600.0,
            actions_per_minute=30.0,
            event_counts={"click": 100, "move": 200},
            contradiction_intensities={"economic": 0.7},
            user_choices={"path_a": 5},
        )

        self.persistence.save_gameplay_metrics(metrics)

        retrieved = self.persistence.get_gameplay_metrics()
        self.assertEqual(len(retrieved), 1, "Expected exactly one gameplay metric record")
        self.assertEqual(retrieved[0].session_duration, metrics.session_duration)
        self.assertEqual(retrieved[0].actions_per_minute, metrics.actions_per_minute)
        self.assertEqual(retrieved[0].event_counts, metrics.event_counts)
        self.assertEqual(retrieved[0].contradiction_intensities, metrics.contradiction_intensities)
        self.assertEqual(retrieved[0].user_choices, metrics.user_choices)

    def test_cleanup_old_metrics(self):
        """Test cleanup of old metrics."""
        # Add old metrics
        old_date = (datetime.now() - timedelta(days=40)).isoformat()
        old_metrics = SystemMetrics(
            timestamp=old_date,
            cpu_percent=50.0,
            memory_percent=60.0,
            swap_percent=30.0,
            disk_usage_percent=70.0,
            gpu_utilization=40.0,
            gpu_memory_percent=45.0,
        )
        self.persistence.save_system_metrics(old_metrics)

        # Add recent metrics
        recent_metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=55.0,
            memory_percent=65.0,
            swap_percent=35.0,
            disk_usage_percent=75.0,
            gpu_utilization=45.0,
            gpu_memory_percent=50.0,
        )
        self.persistence.save_system_metrics(recent_metrics)

        # Verify both metrics were saved
        all_metrics = self.persistence.get_system_metrics()
        self.assertEqual(len(all_metrics), 2, "Expected two metric records before cleanup")

        # Cleanup old metrics
        self.persistence.cleanup_old_metrics(days_to_keep=30)

        # Verify only recent metrics remain
        retrieved = self.persistence.get_system_metrics()
        self.assertEqual(len(retrieved), 1, "Expected exactly one metric record after cleanup")
        self.assertEqual(retrieved[0].cpu_percent, recent_metrics.cpu_percent)
        self.assertEqual(retrieved[0].memory_percent, recent_metrics.memory_percent)
        self.assertEqual(retrieved[0].swap_percent, recent_metrics.swap_percent)
        self.assertEqual(retrieved[0].disk_usage_percent, recent_metrics.disk_usage_percent)
        self.assertEqual(retrieved[0].gpu_utilization, recent_metrics.gpu_utilization)
        self.assertEqual(retrieved[0].gpu_memory_percent, recent_metrics.gpu_memory_percent)


if __name__ == "__main__":
    unittest.main()
