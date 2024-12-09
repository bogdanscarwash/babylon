import pytest
from pathlib import Path
import time
from datetime import datetime, timedelta
import tempfile
import os
from datetime import datetime, timedelta
from typing import Tuple
from babylon.metrics.persistence import MetricsPersistence
from babylon.metrics.performance_metrics import SystemMetrics, AIMetrics, GameplayMetrics

@pytest.fixture
def metrics_db():
    """Provide a temporary database for metrics testing.
    
    Creates an isolated test database and ensures proper cleanup after tests.
    This helps prevent test pollution and ensures reproducible results.
    """
    temp_dir = tempfile.mkdtemp()
    db_path = os.path.join(temp_dir, "test_metrics.db")
    persistence = MetricsPersistence(db_path)
    
    yield persistence
    
    # Cleanup with error handling
    try:
        if os.path.exists(db_path):
            os.remove(db_path)
        os.rmdir(temp_dir)
    except Exception as e:
        print(f"Warning: Cleanup failed: {e}")

@pytest.fixture
def sample_metrics() -> Tuple[SystemMetrics, AIMetrics, GameplayMetrics]:
    """Provide sample metrics for testing.
    
    Creates a consistent set of test metrics with realistic values.
    Having standardized test data helps make tests more predictable
    and easier to understand.
    """
    system_metrics = SystemMetrics(
        timestamp=datetime.now().isoformat(),
        cpu_percent=50.0,
        memory_percent=60.0,
        swap_percent=30.0,
        disk_usage_percent=70.0,
        gpu_utilization=40.0,
        gpu_memory_percent=45.0
    )
    
    ai_metrics = AIMetrics(
        query_latency_ms=50.0,
        memory_usage_gb=1.5,
        token_count=1000,
        embedding_dimension=384,
        cache_hit_rate=0.95,
        anomaly_score=0.1,
        threshold_violations=[]
    )
    
    gameplay_metrics = GameplayMetrics(
        session_duration=3600.0,
        actions_per_minute=30.0,
        event_counts={"click": 100, "move": 200},
        contradiction_intensities={"economic": 0.7},
        user_choices={"path_a": 5}
    )
    
    return system_metrics, ai_metrics, gameplay_metrics


@pytest.fixture(scope="function")
def test_log_dir():
    """Create and manage a temporary log directory."""
    test_dir = Path("test_logs")
    test_dir.mkdir(exist_ok=True)
    
    yield test_dir
    
    # Cleanup with retry mechanism
    max_retries = 3
    for attempt in range(max_retries):
        try:
            time.sleep(0.1)  # Allow OS to release handles
            for file in test_dir.glob("*"):
                if file.is_file():
                    file.unlink()
            test_dir.rmdir()
            break
        except Exception as e:
            if attempt == max_retries - 1:
                raise RuntimeError(f"Failed to clean up after {max_retries} attempts: {e}")

@pytest.fixture(scope="function")
def metrics_persistence(test_log_dir):
    """Initialize MetricsPersistence with test database."""
    db_path = str(test_log_dir / "metrics.db")
    return MetricsPersistence(db_path)

@pytest.fixture(scope="function")
def sample_logs(test_log_dir):
    """Create sample log files with known dates and sizes."""
    test_files = []
    for days_ago in range(5):
        timestamp = datetime.now() - timedelta(days=days_ago)
        log_file = test_log_dir / f"metrics_{timestamp.strftime('%Y%m%d')}.log"
        with log_file.open('w') as f:
            f.write(f"Test log data for {days_ago} days ago")
        test_files.append(log_file)
    
    time.sleep(0.1)  # Allow file operations to complete
    return test_files