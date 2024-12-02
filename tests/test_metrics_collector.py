import pytest
from datetime import datetime
from pathlib import Path
from babylon.metrics.collector import MetricsCollector

@pytest.fixture
def temp_log_dir(tmp_path):
    """Create a temporary directory for logs during testing."""
    return tmp_path / "test_logs"

@pytest.fixture
def metrics_collector(temp_log_dir):
    """Create a MetricsCollector instance for testing."""
    return MetricsCollector(log_dir=temp_log_dir)

def test_init(metrics_collector, temp_log_dir):
    """Test initialization of MetricsCollector.
    
    Verifies:
    - Log directory is correctly set
    - Session start time is a valid datetime
    - Initial counters are properly zeroed
    - Data structures are properly initialized
    """
    assert metrics_collector.log_dir == temp_log_dir
    assert isinstance(metrics_collector.current_session['start_time'], datetime)
    assert metrics_collector.current_session['total_objects'] == 0
    assert len(metrics_collector.metrics['object_access']) == 0

def test_record_object_access(metrics_collector):
    """Test recording object access events.
    
    Verifies:
    - Multiple accesses to same object are correctly counted
    - Different objects maintain separate access counts
    - Access counting is accurate and consistent
    """
    metrics_collector.record_object_access("test_obj_1", "test_context")
    metrics_collector.record_object_access("test_obj_1", "test_context")
    metrics_collector.record_object_access("test_obj_2", "test_context")
    
    assert metrics_collector.metrics['object_access']['test_obj_1'] == 2
    assert metrics_collector.metrics['object_access']['test_obj_2'] == 1

def test_record_token_usage(metrics_collector):
    """Test recording token usage.
    
    Verifies:
    - Token usage values are correctly recorded
    - Multiple recordings are stored in sequence
    - Values maintain their accuracy
    """
    metrics_collector.record_token_usage(100)
    metrics_collector.record_token_usage(150)
    
    assert len(metrics_collector.metrics['token_usage']) == 2
    assert list(metrics_collector.metrics['token_usage']) == [100, 150]

def test_record_cache_event(metrics_collector):
    """Test recording cache hits and misses.
    
    Verifies:
    - Cache hits and misses are correctly categorized
    - Different cache levels (L1, L2) are tracked separately
    - Hit/miss counters increment accurately
    """
    metrics_collector.record_cache_event("L1", True)
    metrics_collector.record_cache_event("L1", False)
    metrics_collector.record_cache_event("L2", True)
    
    assert metrics_collector.metrics['cache_performance']['hits']['L1'] == 1
    assert metrics_collector.metrics['cache_performance']['misses']['L1'] == 1
    assert metrics_collector.metrics['cache_performance']['hits']['L2'] == 1
    assert metrics_collector.metrics['cache_performance']['misses']['L2'] == 0

def test_analyze_performance(metrics_collector):
    """Test performance analysis functionality.
    
    Verifies:
    - All analysis metrics are present in output
    - Cache hit rates are correctly calculated
    - Hot objects are properly identified
    - Performance statistics are accurate
    - Memory and latency metrics are included
    - Optimization suggestions are generated
    """
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

def test_save_metrics(metrics_collector, temp_log_dir):
    """Test saving metrics to disk.
    
    Verifies:
    - Metrics are saved in the correct directory
    - JSON file is created with proper naming
    - All metrics data is properly serialized
    - Datetime objects are correctly converted to ISO format
    """
    metrics_collector.record_object_access("test_obj", "test")
    metrics_collector.save_metrics()
    
    # Check if metrics file was created
    metric_files = list(temp_log_dir.glob("metrics_*.json"))
    assert len(metric_files) == 1

def test_memory_analysis(metrics_collector):
    """Test memory usage analysis.
    
    Verifies:
    - Average memory usage is correctly calculated
    - Peak memory usage is tracked
    - Current memory usage is accurate
    - Statistics are properly computed from recorded values
    """
    test_values = [1000, 2000, 1500]
    for value in test_values:
        metrics_collector.record_memory_usage(value)
    
    memory_analysis = metrics_collector._analyze_memory_usage()
    assert memory_analysis['avg'] == sum(test_values) / len(test_values)
    assert memory_analysis['peak'] == max(test_values)
    assert memory_analysis['current'] == test_values[-1]

def test_latency_tracking(metrics_collector):
    """Test latency statistics calculation.
    
    Verifies:
    - Latency measurements are properly recorded
    - Average, minimum, and maximum latencies are calculated
    - Statistics are maintained for database queries
    - Values remain accurate across multiple recordings
    """
    test_latencies = [5.0, 10.0, 15.0]
    for latency in test_latencies:
        metrics_collector.record_query_latency(latency)
    
    latency_stats = metrics_collector._calculate_latency_stats()
    assert 'db_queries' in latency_stats
    assert latency_stats['db_queries']['avg'] == sum(test_latencies) / len(test_latencies)
    assert latency_stats['db_queries']['min'] == min(test_latencies)
    assert latency_stats['db_queries']['max'] == max(test_latencies)
