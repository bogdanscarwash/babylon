from pathlib import Path
import time

class TestLogRotation:
    """Test suite for log rotation functionality."""
    
    # Constants for test configuration
    LARGE_FILE_SIZE_MB = 10
    COMPRESSIBLE_CONTENT = "Test log data " * 100000
    
    def test_time_based_rotation(self, metrics_persistence, sample_logs):
        """Test rotation of logs based on age."""
        # Trigger rotation
        metrics_persistence.rotate_logs(max_age_days=3, max_size_mb=100)
        time.sleep(0.1)
        
        # Verify results
        remaining_logs = list(Path(metrics_persistence.db_path).parent.glob("metrics_2*.log"))
        assert len(remaining_logs) <= 3, "Should keep only logs within age threshold"
        
        if remaining_logs:
            oldest_log = min(remaining_logs, key=lambda p: p.stat().st_mtime)
            age_days = (time.time() - oldest_log.stat().st_mtime) / (24 * 3600)
            assert age_days <= 3, f"Oldest log too old: {age_days} days"
    
    def test_size_based_rotation(self, metrics_persistence, test_log_dir):
        """Test rotation of logs based on file size."""
        # Create oversized log file
        large_log = self._create_large_log(
            test_log_dir,
            size_mb=self.LARGE_FILE_SIZE_MB
        )
        
        # Track initial state
        initial_count = len(list(test_log_dir.glob("metrics_*.log*")))
        
        # Trigger rotation
        metrics_persistence.rotate_logs(
            max_age_days=30,
            max_size_mb=self.LARGE_FILE_SIZE_MB
        )
        time.sleep(0.1)
        
        # Verify rotation occurred
        self._verify_rotation_results(
            test_log_dir,
            large_log,
            initial_count
        )
    
    def test_compression_during_rotation(self, metrics_persistence, test_log_dir):
        """Test compression of rotated log files."""
        # Create compressible log file
        log_file = test_log_dir / "metrics.log"
        self._write_compressible_content(log_file)
        
        original_size = log_file.stat().st_size
        initial_compressed = len(list(test_log_dir.glob("*.gz")))
        
        # Trigger rotation with compression
        metrics_persistence.rotate_logs(
            max_age_days=30,
            max_size_mb=1,
            compress=True
        )
        time.sleep(0.1)
        
        # Verify compression results
        self._verify_compression_results(
            test_log_dir,
            log_file,
            original_size,
            initial_compressed
        )
    
    def test_rotation_error_handling(self, metrics_persistence, test_log_dir):
        """Test error handling during rotation operations."""
        # Test with invalid directory permissions
        invalid_dir = test_log_dir / "invalid"
        invalid_dir.mkdir(mode=0o444)  # Read-only directory
        
        with pytest.raises(Exception):
            metrics_persistence.rotate_logs(
                max_age_days=1,
                max_size_mb=1
            )
    
    def _create_large_log(self, directory: Path, size_mb: int) -> Path:
        """Create a log file of specified size."""
        log_file = directory / "metrics_large.log"
        with log_file.open("wb") as f:
            f.write(b"0" * (size_mb * 1024 * 1024))
        time.sleep(0.1)
        return log_file
    
    def _write_compressible_content(self, file_path: Path) -> None:
        """Write highly compressible content to a file."""
        with file_path.open("w") as f:
            f.write(self.COMPRESSIBLE_CONTENT)
        time.sleep(0.1)
    
    def _verify_rotation_results(
        self,
        directory: Path,
        original_file: Path,
        initial_count: int
    ) -> None:
        """Verify results of a rotation operation."""
        final_count = len(list(directory.glob("metrics_*.log*")))
        assert final_count > initial_count, "No rotation occurred"
        assert not original_file.exists(), "Original file still exists"
        
        rotated_logs = list(directory.glob("metrics_large_*.log"))
        assert len(rotated_logs) == 1, "Expected exactly one rotated log file"
    
    def _verify_compression_results(
        self,
        directory: Path,
        original_file: Path,
        original_size: int,
        initial_compressed: int
    ) -> None:
        """Verify results of a compression operation."""
        final_compressed = len(list(directory.glob("*.gz")))
        assert final_compressed > initial_compressed, "No compression occurred"
        assert not original_file.exists(), "Original file still exists"
        
        compressed_logs = list(directory.glob("metrics_*.gz"))
        assert len(compressed_logs) == 1, "Expected exactly one compressed file"
        
        if compressed_logs:
            compressed_size = compressed_logs[0].stat().st_size
            assert compressed_size < original_size, (
                "Compressed file is not smaller than original"
            )
