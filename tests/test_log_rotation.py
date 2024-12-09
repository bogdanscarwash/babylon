import os
import time
import unittest
from datetime import datetime, timedelta
from pathlib import Path

from babylon.metrics.persistence import MetricsPersistence


class TestLogRotation(unittest.TestCase):
    def setUp(self):
        """Set up test environment."""
        # Create test directory and database
        self.test_dir = Path("test_logs")
        self.test_dir.mkdir(exist_ok=True)
        self.db_path = str(self.test_dir / "metrics.db")
        self.persistence = MetricsPersistence(self.db_path)

    def tearDown(self):
        """Clean up test environment."""
        try:
            # Give OS time to release file handles
            time.sleep(0.1)
            for file in self.test_dir.glob("*"):
                try:
                    if file.is_file():
                        file.unlink()
                except Exception as e:
                    print(f"Error deleting file {file}: {e}")
            self.test_dir.rmdir()
        except Exception as e:
            print(f"Error during cleanup: {e}")

    def test_log_rotation(self):
        """Test log file rotation based on size and time."""
        # Create multiple log files with different timestamps
        for days_ago in range(5):
            timestamp = datetime.now() - timedelta(days=days_ago)
            log_file = Path(self.db_path).parent / f"metrics_{timestamp.strftime('%Y%m%d')}.log"
            with log_file.open('w') as f:
                f.write(f"Test log data for {days_ago} days ago")

        # Wait for file operations to complete
        time.sleep(0.1)

        # Trigger rotation
        self.persistence.rotate_logs(max_age_days=3, max_size_mb=10)

        # Wait for rotation to complete
        time.sleep(0.1)

        # Count remaining log files (excluding rotated ones)
        remaining_logs = list(Path(self.db_path).parent.glob("metrics_2*.log"))
        self.assertLessEqual(len(remaining_logs), 3)

        if remaining_logs:  # Only check if there are remaining logs
            # Verify oldest remaining log is within threshold
            oldest_log = min(remaining_logs, key=lambda p: p.stat().st_mtime)
            age_days = (time.time() - oldest_log.stat().st_mtime) / (24 * 3600)
            self.assertLessEqual(age_days, 3)

    def test_size_based_rotation(self):
        """Test rotation based on file size."""
        # Create a large log file
        large_log = Path(self.db_path).parent / "metrics_large.log"
        with large_log.open("wb") as f:
            # Write 10MB of data (exactly at limit to trigger rotation)
            f.write(b"0" * (10 * 1024 * 1024))

        # Wait for file write to complete
        time.sleep(0.1)

        # Get initial file count
        initial_files = len(list(Path(self.db_path).parent.glob("metrics_*.log*")))

        # Trigger rotation
        self.persistence.rotate_logs(max_age_days=30, max_size_mb=10)

        # Wait for rotation to complete
        time.sleep(0.1)

        # Get final file count (including rotated files)
        final_files = len(list(Path(self.db_path).parent.glob("metrics_*.log*")))

        # Verify rotation occurred
        self.assertGreater(final_files, initial_files, "No rotation occurred")

        # Verify original file was rotated
        self.assertFalse(large_log.exists(), "Original file still exists")

        # Verify rotated file exists
        rotated_logs = list(Path(self.db_path).parent.glob("metrics_large_*.log"))
        self.assertEqual(len(rotated_logs), 1, "Expected exactly one rotated log file")

    def test_compression(self):
        """Test log compression during rotation."""
        # Create log file with highly compressible content
        log_file = Path(self.db_path).parent / "metrics.log"
        with log_file.open("w") as f:
            # Write 2MB of highly compressible data
            f.write("Test log data " * 100000)  # Each iteration is ~14 bytes

        # Wait for file write to complete
        time.sleep(0.1)

        # Get original size
        original_size = log_file.stat().st_size

        # Get initial compressed file count
        initial_compressed = len(list(Path(self.db_path).parent.glob("*.gz")))

        # Trigger rotation with compression
        self.persistence.rotate_logs(max_age_days=30, max_size_mb=1, compress=True)

        # Wait for compression to complete
        time.sleep(0.1)

        # Get final compressed file count
        final_compressed = len(list(Path(self.db_path).parent.glob("*.gz")))

        # Verify compression occurred
        self.assertGreater(final_compressed, initial_compressed, "No compression occurred")

        # Verify original file was removed
        self.assertFalse(log_file.exists(), "Original file still exists")

        # Verify compressed file exists and is smaller
        compressed_logs = list(Path(self.db_path).parent.glob("metrics_*.gz"))
        self.assertEqual(len(compressed_logs), 1, "Expected exactly one compressed log file")
        
        if compressed_logs:  # Only check if compression created a file
            compressed_size = compressed_logs[0].stat().st_size
            self.assertLess(compressed_size, original_size, "Compressed file is not smaller than original")


if __name__ == "__main__":
    unittest.main()
