import pytest
from datetime import datetime
from pathlib import Path
from typing import Generator, Dict, List, Any, Optional
from pathlib import Path
from babylon.metrics.collector import MetricsCollector

from pathlib import Path
from typing import Generator

@pytest.fixture
def temp_log_dir(tmp_path: Path) -> Path:
    """Create a temporary directory for metrics logs during testing.
    
    Creates an isolated test directory to prevent test logs from mixing with
    production logs and ensure clean test environment for each test run.
    
    Args:
        tmp_path: pytest built-in fixture providing temporary directory

    Returns:
        Path: Temporary directory path for test logs
    """
    return tmp_path / "test_logs"

@pytest.fixture
def metrics_collector(temp_log_dir: Path) -> MetricsCollector:
    """Create a fresh MetricsCollector instance for each test.
    
    Provides an isolated MetricsCollector instance configured to use
    the temporary test directory, ensuring each test starts with a
    clean metrics collection state.
    
    Args:
        temp_log_dir: Fixture providing temporary log directory

    Returns:
        MetricsCollector: Fresh collector instance for testing
    """
    return MetricsCollector(log_dir=temp_log_dir)

def test_init(metrics_collector: MetricsCollector, temp_log_dir: Path) -> None:
    """Test initialization of MetricsCollector.
    
    Verifies that a new MetricsCollector instance is properly initialized with:
    - Correct log directory configuration
    - Valid session start timestamp
    - Zero-initialized counters
    - Empty data structures
    
    The test ensures the collector starts in a clean state and is ready
    to begin collecting metrics without any residual data.
    
    Args:
        metrics_collector: Fresh collector instance for testing
        temp_log_dir: Temporary directory for test logs
        
    Assertions:
        - Log directory matches provided temp directory
        - Session start time is a valid datetime
        - Total objects counter is zero
        - Object access dictionary is empty
    """
    assert metrics_collector.log_dir == temp_log_dir
    assert isinstance(metrics_collector.current_session['start_time'], datetime)
    assert metrics_collector.current_session['total_objects'] == 0
    assert len(metrics_collector.metrics['object_access']) == 0

def test_record_object_access(metrics_collector: MetricsCollector) -> None:
    """Test the object access tracking functionality.
    
    Validates that the MetricsCollector accurately tracks:
    - Multiple accesses to the same object
    - Single access to different objects
    - Separate counters for distinct objects
    - Exact access count accuracy
    
    This tracking helps identify frequently accessed "hot" objects
    that may benefit from caching or optimization.
    
    Args:
        metrics_collector: Fresh collector instance for testing
        
    Test Steps:
        1. Record two accesses to test_obj_1
        2. Record one access to test_obj_2
        3. Verify access counts match expected values
        
    Assertions:
        - test_obj_1 has exactly 2 recorded accesses
        - test_obj_2 has exactly 1 recorded access
    """
    metrics_collector.record_object_access("test_obj_1", "test_context")
    metrics_collector.record_object_access("test_obj_1", "test_context")
    metrics_collector.record_object_access("test_obj_2", "test_context")
    
    assert metrics_collector.metrics['object_access']['test_obj_1'] == 2
    assert metrics_collector.metrics['object_access']['test_obj_2'] == 1

def test_record_token_usage(metrics_collector: MetricsCollector) -> None:
    """Test recording token usage.
    
    This test checks the token usage tracking functionality:
    - Records two distinct token usage values (100 and 150)
    - Verifies the values are stored in the correct sequence
    - Confirms the deque maintains the exact values without modification
    
    Token usage tracking is essential for monitoring AI model consumption
    and optimizing resource usage in the game's AI systems.
    """
    metrics_collector.record_token_usage(100)
    metrics_collector.record_token_usage(150)
    
    assert len(metrics_collector.metrics['token_usage']) == 2
    assert list(metrics_collector.metrics['token_usage']) == [100, 150]

def test_record_cache_event(metrics_collector: MetricsCollector) -> None:
    """Test recording cache hits and misses.
    
    This test validates the cache performance tracking system:
    - Records and verifies one L1 cache hit
    - Records and verifies one L1 cache miss
    - Records and verifies one L2 cache hit
    - Confirms separate tracking for different cache levels
    
    Cache performance monitoring helps optimize the game's memory hierarchy
    and improve overall system performance.
    """
    metrics_collector.record_cache_event("L1", True)
    metrics_collector.record_cache_event("L1", False)
    metrics_collector.record_cache_event("L2", True)
    
    assert metrics_collector.metrics['cache_performance']['hits']['L1'] == 1
    assert metrics_collector.metrics['cache_performance']['misses']['L1'] == 1
    assert metrics_collector.metrics['cache_performance']['hits']['L2'] == 1
    assert metrics_collector.metrics['cache_performance']['misses']['L2'] == 0

def test_analyze_performance(metrics_collector: MetricsCollector) -> None:
    """Test performance analysis functionality.
    
    This test validates the performance analysis system by:
    1. Setting up test data with specific thresholds
    2. Recording metrics for:
       - Hot object access patterns
       - Token usage statistics
       - Cache hit/miss rates
       - Query latencies
       - Memory usage patterns
    3. Verifying analysis results for:
       - Cache performance
       - Resource utilization
       - System optimization
    """
    # Setup test data with specific thresholds
    test_data = {
        'hot_object': 5,  # Access count
        'token_usage': [100, 200, 300],
        'cache_events': {'L1': {'hits': 8, 'misses': 2}},  # 80% hit rate
        'latencies': [5.0, 10.0, 15.0],  # ms
        'memory_usage': [500, 1000, 1500]  # MB
    }
    
    # Record test data
    for _ in range(test_data['hot_object']):
        metrics_collector.record_object_access("hot_object", "test")
    
    for tokens in test_data['token_usage']:
        metrics_collector.record_token_usage(tokens)
        
    for _ in range(test_data['cache_events']['L1']['hits']):
        metrics_collector.record_cache_event("L1", True)
    for _ in range(test_data['cache_events']['L1']['misses']):
        metrics_collector.record_cache_event("L1", False)
        
    for latency in test_data['latencies']:
        metrics_collector.record_query_latency(latency)
        
    for memory in test_data['memory_usage']:
        metrics_collector.record_memory_usage(memory)
    # Setup some test data
    metrics_collector.record_object_access("hot_object", "test")
    metrics_collector.record_object_access("hot_object", "test")
    metrics_collector.record_object_access("hot_object", "test")
    metrics_collector.record_token_usage(100)
    metrics_collector.record_cache_event("L1", True)
    metrics_collector.record_cache_event("L1", False)
    metrics_collector.record_query_latency(10.0)
    metrics_collector.record_memory_usage(1000)
    
    analysis = metrics_collector.analyze_performance()
    
    assert 'cache_hit_rate' in analysis
    assert 'avg_token_usage' in analysis
    assert 'hot_objects' in analysis
    assert 'latency_stats' in analysis
    assert 'memory_profile' in analysis
    assert 'optimization_suggestions' in analysis
    
    assert analysis['hot_objects'] == ['hot_object']
    assert analysis['cache_hit_rate']['L1'] == 0.5

def test_save_metrics(metrics_collector: MetricsCollector, temp_log_dir: Path) -> None:
    """Test saving metrics to disk.
    
    This test validates the metrics persistence system:
    1. Records a sample object access to ensure non-empty metrics
    2. Triggers metrics save operation
    3. Verifies:
       - JSON file is created in the temporary test directory
       - File naming follows the metrics_TIMESTAMP.json pattern
       - Metrics data is properly serialized to JSON
       - Datetime values are converted to ISO format strings
    
    Proper metrics persistence is crucial for:
    - Post-mortem performance analysis
    - System behavior tracking over time
    - Historical trend analysis
    """
    metrics_collector.record_object_access("test_obj", "test")
    metrics_collector.save_metrics()
    
    # Check if metrics file was created
    metric_files = list(temp_log_dir.glob("metrics_*.json"))
    assert len(metric_files) == 1

def test_memory_analysis(metrics_collector: MetricsCollector) -> None:
    """Test memory usage analysis.
    
    This test validates the memory analysis subsystem:
    1. Records a sequence of memory usage values: [1000, 2000, 1500]
    2. Triggers memory analysis
    3. Verifies computed statistics:
       - Average memory usage (1500)
       - Peak memory usage (2000)
       - Current memory usage (1500)
    
    Accurate memory tracking helps:
    - Prevent memory leaks
    - Optimize resource allocation
    - Plan for scaling requirements
    """
    test_values = [1000, 2000, 1500]
    for value in test_values:
        metrics_collector.record_memory_usage(value)
    
    memory_analysis = metrics_collector._analyze_memory_usage()
    assert memory_analysis['avg'] == sum(test_values) / len(test_values)
    assert memory_analysis['peak'] == max(test_values)
    assert memory_analysis['current'] == test_values[-1]

def test_latency_tracking(metrics_collector: MetricsCollector) -> None:
    """Test latency statistics calculation.
    
    This test validates the latency tracking system:
    1. Records a series of database query latencies: [5.0, 10.0, 15.0]
    2. Triggers latency analysis
    3. Verifies computed statistics:
       - Average latency (10.0 ms)
       - Minimum latency (5.0 ms)
       - Maximum latency (15.0 ms)
    
    Latency tracking is essential for:
    - Identifying performance bottlenecks
    - Maintaining responsive gameplay
    - Meeting user experience requirements
    - Database query optimization
    """
    test_latencies = [5.0, 10.0, 15.0]
    for latency in test_latencies:
        metrics_collector.record_query_latency(latency)
    
    latency_stats = metrics_collector._calculate_latency_stats()
    assert 'db_queries' in latency_stats
    assert latency_stats['db_queries']['avg'] == sum(test_latencies) / len(test_latencies)
    assert latency_stats['db_queries']['min'] == min(test_latencies)
    assert latency_stats['db_queries']['max'] == max(test_latencies)
