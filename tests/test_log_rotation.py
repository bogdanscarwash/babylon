import time
import unittest
from datetime import datetime, timedelta
from pathlib import Path

from babylon.metrics.persistence import MetricsPersistence


class TestLogRotation(unittest.TestCase):
    def setUp(self):
        self.log_dir = Path("test_logs")
        self.log_dir.mkdir(exist_ok=True)
        self.persistence = MetricsPersistence(str(self.log_dir / "metrics.db"))

    def tearDown(self):
        for file in self.log_dir.glob("*"):
            file.unlink()
        self.log_dir.rmdir()

    def test_log_rotation(self):
        """Test log file rotation based on size and time."""
        # Create multiple log files with different timestamps
        for days_ago in range(5):
            timestamp = datetime.now() - timedelta(days=days_ago)
            log_file = self.log_dir / f"metrics_{timestamp.strftime('%Y%m%d')}.log"
            log_file.write_text(f"Test log data for {days_ago} days ago")

        # Trigger rotation
        self.persistence.rotate_logs(max_age_days=3, max_size_mb=10)

        # Verify only recent logs remain
        remaining_logs = list(self.log_dir.glob("metrics_*.log"))
        self.assertLessEqual(len(remaining_logs), 3)

        # Verify oldest remaining log is within threshold
        oldest_log = min(remaining_logs, key=lambda p: p.stat().st_mtime)
        age_days = (time.time() - oldest_log.stat().st_mtime) / (24 * 3600)
        self.assertLessEqual(age_days, 3)

    def test_size_based_rotation(self):
        """Test rotation based on file size."""
        # Create a large log file
        large_log = self.log_dir / "metrics_large.log"
        with large_log.open("wb") as f:
            f.write(b"0" * (11 * 1024 * 1024))  # 11MB

        # Trigger rotation
        self.persistence.rotate_logs(max_age_days=30, max_size_mb=10)

        # Verify file was rotated
        self.assertFalse(large_log.exists())
        rotated_logs = list(self.log_dir.glob("metrics_large.log.*"))
        self.assertEqual(len(rotated_logs), 1)

    def test_compression(self):
        """Test log compression during rotation."""
        # Create log file
        log_file = self.log_dir / "metrics.log"
        log_file.write_text("Test log data" * 1000)

        # Trigger rotation with compression
        self.persistence.rotate_logs(max_age_days=30, max_size_mb=10, compress=True)

        # Verify compressed file exists
        compressed_logs = list(self.log_dir.glob("metrics.log.*.gz"))
        self.assertEqual(len(compressed_logs), 1)
