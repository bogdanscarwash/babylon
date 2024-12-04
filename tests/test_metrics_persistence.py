import unittest
import tempfile
import os
from datetime import datetime, timedelta
from src.babylon.metrics.persistence import MetricsPersistence
from src.babylon.metrics.performance_metrics import (
    SystemMetrics,
    AIMetrics,
    GameplayMetrics
)

class TestMetricsPersistence(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test_metrics.db")
        self.persistence = MetricsPersistence(self.db_path)

    def tearDown(self):
        if os.path.exists(self.db_path):
            os.remove(self.db_path)
        os.rmdir(self.temp_dir)

    def test_save_and_retrieve_system_metrics(self):
        metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=50.0,
            memory_percent=60.0,
            swap_percent=30.0,
            disk_usage_percent=70.0,
            gpu_utilization=40.0,
            gpu_memory_percent=45.0
        )
        
        self.persistence.save_system_metrics(metrics)
        retrieved = self.persistence.get_system_metrics()
        
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].cpu_percent, metrics.cpu_percent)
        self.assertEqual(retrieved[0].memory_percent, metrics.memory_percent)

    def test_save_and_retrieve_ai_metrics(self):
        metrics = AIMetrics(
            query_latency_ms=50.0,
            memory_usage_gb=1.5,
            token_count=1000,
            embedding_dimension=384,
            cache_hit_rate=0.95,
            anomaly_score=0.1,
            threshold_violations=[]
        )
        
        self.persistence.save_ai_metrics(metrics)
        retrieved = self.persistence.get_ai_metrics()
        
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].query_latency_ms, metrics.query_latency_ms)
        self.assertEqual(retrieved[0].memory_usage_gb, metrics.memory_usage_gb)

    def test_save_and_retrieve_gameplay_metrics(self):
        metrics = GameplayMetrics(
            session_duration=3600.0,
            actions_per_minute=30.0,
            event_counts={"click": 100, "move": 200},
            contradiction_intensities={"economic": 0.7},
            user_choices={"path_a": 5}
        )
        
        self.persistence.save_gameplay_metrics(metrics)
        retrieved = self.persistence.get_gameplay_metrics()
        
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].session_duration, metrics.session_duration)
        self.assertEqual(retrieved[0].actions_per_minute, metrics.actions_per_minute)
        self.assertEqual(retrieved[0].event_counts, metrics.event_counts)

    def test_cleanup_old_metrics(self):
        # Add old metrics
        old_date = (datetime.now() - timedelta(days=40)).isoformat()
        metrics = SystemMetrics(
            timestamp=old_date,
            cpu_percent=50.0,
            memory_percent=60.0,
            swap_percent=30.0,
            disk_usage_percent=70.0
        )
        self.persistence.save_system_metrics(metrics)
        
        # Add recent metrics
        recent_metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=55.0,
            memory_percent=65.0,
            swap_percent=35.0,
            disk_usage_percent=75.0
        )
        self.persistence.save_system_metrics(recent_metrics)
        
        # Cleanup old metrics
        self.persistence.cleanup_old_metrics(days_to_keep=30)
        
        # Verify only recent metrics remain
        retrieved = self.persistence.get_system_metrics()
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].cpu_percent, recent_metrics.cpu_percent)

if __name__ == '__main__':
    unittest.main()
