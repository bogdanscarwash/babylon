"""Unit tests for MetricsPersistence class.

Tests the functionality of the MetricsPersistence class including:
- Database initialization and connection management
- Metrics storage and retrieval with time filtering
- Database rotation and compression
- Error handling and edge cases
"""

import os
import tempfile
import unittest
from datetime import datetime, timedelta
import json
import gzip
import sqlite3
from pathlib import Path
import stat

from babylon.metrics.performance_metrics import (
    AIMetrics,
    GameplayMetrics,
    SystemMetrics,
)
from babylon.metrics.persistence import MetricsPersistence
from babylon.exceptions import (
    DatabaseConnectionError,
    LogRotationError,
    MetricsPersistenceError,
)


class TestMetricsPersistence(unittest.TestCase):
    """Test suite for MetricsPersistence class."""

    def setUp(self):
        """Set up test environment before each test."""
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test_metrics.db")
        self.persistence = MetricsPersistence(self.db_path)
        self.base_time = datetime.now()

    def tearDown(self):
        """Clean up test environment after each test."""
        try:
            # Close any open connections
            if hasattr(self, 'persistence'):
                try:
                    with self.persistence._get_connection() as conn:
                        conn.close()
                except:
                    pass
            
            # Remove test database and rotated files
            for file in Path(self.temp_dir).glob("**/*.db*"):
                try:
                    # Reset file permissions before deletion
                    os.chmod(file, stat.S_IWRITE)
                    os.remove(file)
                except OSError:
                    pass
            
            # Remove readonly directory if it exists
            readonly_dir = os.path.join(self.temp_dir, "readonly")
            if os.path.exists(readonly_dir):
                # Reset directory permissions
                os.chmod(readonly_dir, stat.S_IWRITE | stat.S_IEXEC | stat.S_IREAD)
                for file in os.listdir(readonly_dir):
                    file_path = os.path.join(readonly_dir, file)
                    os.chmod(file_path, stat.S_IWRITE)
                    os.remove(file_path)
                os.rmdir(readonly_dir)
            
            os.rmdir(self.temp_dir)
        except Exception as e:
            print(f"Cleanup error: {e}")

    def create_test_system_metrics(self, time_offset_minutes=0) -> SystemMetrics:
        """Helper to create test system metrics with specified time offset."""
        timestamp = (self.base_time + timedelta(minutes=time_offset_minutes)).isoformat()
        return SystemMetrics(
            timestamp=timestamp,
            cpu_percent=50.0,
            memory_percent=60.0,
            swap_percent=30.0,
            disk_usage_percent=70.0,
            gpu_utilization=40.0,
            gpu_memory_percent=45.0,
        )

    def test_database_initialization(self):
        """Test database is properly initialized with required tables and indices."""
        with self.persistence._get_connection() as conn:
            # Check tables exist
            tables = conn.execute(
                "SELECT name FROM sqlite_master WHERE type='table'"
            ).fetchall()
            table_names = {table[0] for table in tables}
            expected_tables = {
                "system_metrics",
                "ai_metrics",
                "gameplay_metrics"
            }
            self.assertEqual(expected_tables, table_names & expected_tables)

            # Check indices exist
            indices = conn.execute(
                "SELECT name FROM sqlite_master WHERE type='index'"
            ).fetchall()
            index_names = {index[0] for index in indices}
            expected_indices = {
                "idx_system_metrics_timestamp",
                "idx_ai_metrics_timestamp",
                "idx_gameplay_metrics_timestamp"
            }
            self.assertEqual(expected_indices, index_names & expected_indices)

    def test_connection_error_handling(self):
        """Test database connection error handling."""
        # Create an invalid database path (directory that exists but is read-only)
        no_access_dir = os.path.join(self.temp_dir, "noaccess")
        os.makedirs(no_access_dir)
        
        # Create a dummy file and make it read-only
        db_path = os.path.join(no_access_dir, "metrics.db")
        with open(db_path, 'w') as f:
            f.write('')
        
        # Make the file read-only
        os.chmod(db_path, stat.S_IREAD)
        
        # Attempt to create persistence instance with read-only file
        with self.assertRaises(DatabaseConnectionError):
            persistence = MetricsPersistence(db_path)
            # Force connection attempt
            with persistence._get_connection():
                pass

    def test_save_and_retrieve_system_metrics_with_time_filtering(self):
        """Test saving and retrieving system metrics with time filtering."""
        # Create test metrics at different times
        metrics_1 = self.create_test_system_metrics(-60)  # 1 hour ago
        metrics_2 = self.create_test_system_metrics(-30)  # 30 mins ago
        metrics_3 = self.create_test_system_metrics(0)    # now

        # Save metrics
        for metrics in [metrics_1, metrics_2, metrics_3]:
            self.persistence.save_system_metrics(metrics)

        # Test time range filtering
        start_time = (self.base_time - timedelta(minutes=45)).isoformat()
        end_time = (self.base_time - timedelta(minutes=15)).isoformat()
        
        filtered_metrics = self.persistence.get_system_metrics(
            start_time=start_time,
            end_time=end_time
        )
        
        self.assertEqual(len(filtered_metrics), 1)
        self.assertEqual(filtered_metrics[0].timestamp, metrics_2.timestamp)

    def test_save_and_retrieve_ai_metrics_with_json_fields(self):
        """Test saving and retrieving AI metrics with JSON serialized fields."""
        metrics = AIMetrics(
            query_latency_ms=50.0,
            memory_usage_gb=1.5,
            token_count=1000,
            embedding_dimension=384,
            cache_hit_rate=0.95,
            anomaly_score=0.1,
            threshold_violations=["high_latency", "memory_usage"],
        )

        self.persistence.save_ai_metrics(metrics)

        retrieved = self.persistence.get_ai_metrics()
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].query_latency_ms, metrics.query_latency_ms)
        self.assertEqual(retrieved[0].threshold_violations, metrics.threshold_violations)

    def test_save_and_retrieve_gameplay_metrics_with_complex_data(self):
        """Test saving and retrieving gameplay metrics with complex nested data."""
        metrics = GameplayMetrics(
            session_duration=3600.0,
            actions_per_minute=30.0,
            event_counts={"click": 100, "move": 200, "nested": {"key": "value"}},
            contradiction_intensities={"economic": 0.7, "social": {"urban": 0.8}},
            user_choices={"path_a": 5, "nested": {"choice": True}},
        )

        self.persistence.save_gameplay_metrics(metrics)

        retrieved = self.persistence.get_gameplay_metrics()
        self.assertEqual(len(retrieved), 1)
        self.assertEqual(retrieved[0].event_counts, metrics.event_counts)
        self.assertEqual(
            retrieved[0].contradiction_intensities,
            metrics.contradiction_intensities
        )
        self.assertEqual(retrieved[0].user_choices, metrics.user_choices)

    def test_cleanup_old_metrics_with_verification(self):
        """Test cleanup of old metrics with verification of deleted records."""
        # Add old and new metrics
        old_metrics = self.create_test_system_metrics(-24*60)  # 24 hours ago
        new_metrics = self.create_test_system_metrics(0)       # now
        
        self.persistence.save_system_metrics(old_metrics)
        self.persistence.save_system_metrics(new_metrics)

        # Verify both metrics were saved
        all_metrics = self.persistence.get_system_metrics()
        self.assertEqual(len(all_metrics), 2)

        # Cleanup metrics older than 12 hours
        self.persistence.cleanup_old_metrics(days_to_keep=0.5)  # 12 hours

        # Verify only recent metrics remain
        remaining_metrics = self.persistence.get_system_metrics()
        self.assertEqual(len(remaining_metrics), 1)
        self.assertEqual(remaining_metrics[0].timestamp, new_metrics.timestamp)

    def test_database_rotation_with_compression(self):
        """Test database rotation with compression and recent data retention."""
        # Create test metrics
        metrics = self.create_test_system_metrics()
        self.persistence.save_system_metrics(metrics)

        # Get initial DB size
        initial_size = os.path.getsize(self.db_path)

        # Force database to be larger than rotation threshold
        with open(self.db_path, 'ab') as f:
            f.write(b'0' * (2 * 1024 * 1024))  # Add 2MB of data

        # Close existing connections before rotation
        with self.persistence._get_connection() as conn:
            conn.close()

        # Rotate logs with compression
        self.persistence.rotate_logs(max_size_mb=1, compress=True)

        # Verify rotation occurred
        self.assertTrue(os.path.exists(self.db_path))
        rotated_files = [
            f for f in os.listdir(self.temp_dir)
            if f.startswith("test_metrics_") and f.endswith(".db.gz")
        ]
        self.assertGreater(len(rotated_files), 0)

        # Verify compressed file exists and is smaller
        compressed_file = Path(self.temp_dir) / rotated_files[0]
        self.assertTrue(compressed_file.exists())
        self.assertLess(compressed_file.stat().st_size, initial_size)

        # Save new metrics after rotation
        new_metrics = self.create_test_system_metrics()
        self.persistence.save_system_metrics(new_metrics)

        # Verify we can still read metrics from new DB
        retrieved_metrics = self.persistence.get_system_metrics()
        self.assertEqual(len(retrieved_metrics), 1)

    def test_concurrent_access_handling(self):
        """Test handling of concurrent database access."""
        metrics = self.create_test_system_metrics()

        # Simulate concurrent access by holding connection
        with self.persistence._get_connection() as conn:
            # Try to save metrics while connection is held
            self.persistence.save_system_metrics(metrics)

            # Verify metrics were saved despite concurrent access
            cursor = conn.execute("SELECT COUNT(*) FROM system_metrics")
            count = cursor.fetchone()[0]
            self.assertEqual(count, 1)

    def test_error_handling_during_rotation(self):
        """Test error handling during database rotation."""
        # Create database file in temp directory first
        readonly_dir = os.path.join(self.temp_dir, "readonly")
        os.makedirs(readonly_dir)
        db_path = os.path.join(readonly_dir, "metrics.db")
        
        # Create and initialize database
        persistence = MetricsPersistence(db_path)
        
        # Add some data and make file large enough to trigger rotation
        metrics = self.create_test_system_metrics()
        persistence.save_system_metrics(metrics)
        
        # Force database to be larger than rotation threshold
        with open(db_path, 'ab') as f:
            f.write(b'0' * (2 * 1024 * 1024))  # Add 2MB of data
        
        # Close connection before changing permissions
        with persistence._get_connection() as conn:
            conn.close()
        
        # Make both directory and file read-only
        os.chmod(db_path, stat.S_IREAD)
        os.chmod(readonly_dir, stat.S_IREAD | stat.S_IEXEC)
        
        # Attempt rotation which should fail due to read-only permissions
        with self.assertRaises(LogRotationError):
            persistence.rotate_logs(max_size_mb=1)

    # TODO: Add tests for:
    # - Edge cases in time range filtering
    # - Database corruption scenarios
    # - Network filesystem scenarios
    # - Memory pressure scenarios


if __name__ == "__main__":
    unittest.main()
