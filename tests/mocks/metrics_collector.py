from typing import Dict, List, Optional
from dataclasses import dataclass
import os
from datetime import datetime
from pathlib import Path
import pytest
from statistics import mean
from collections import Counter, deque

@dataclass
class PerformanceMetrics:
    """Data class for performance metrics results."""
    hot_objects: List[str]
    cache_hit_rate: Dict[str, float]
    avg_token_usage: float
    latency_stats: Dict[str, float]
    memory_profile: Dict[str, float]

class MockMetricsCollector:
    """Mock metrics collector for testing."""
    
    def __init__(self, log_dir: Optional[Path] = None):
        self.log_dir = log_dir
        self.current_session = {
            "start_time": datetime.now(),
            "total_objects": 0,
            "active_objects": 0,
            "cached_objects": 0
        }
        self.metrics = {
            "object_access": Counter(),
            "token_usage": deque(maxlen=1000),
            "cache_performance": {
                "hits": Counter(),
                "misses": Counter()
            },
            "latency": {
                "db_queries": deque(maxlen=100),
                "context_switches": deque(maxlen=100)
            },
            "memory_usage": deque(maxlen=1000),
            "errors": Counter(),  # Track error counts by type
            "failed_operations": [],  # Track failed operation details
            "contradiction_tracking": {
                "total": 0,
                "active": 0
            }
        }
        
        # Initialize default cache levels
        self.metrics["cache_performance"]["hits"]["L1"] = 0
        self.metrics["cache_performance"]["hits"]["L2"] = 0
        self.metrics["cache_performance"]["misses"]["L1"] = 0
        self.metrics["cache_performance"]["misses"]["L2"] = 0
        
    def record_object_access(self, object_id: str, context: str) -> None:
        """Record an object access."""
        self.metrics["object_access"][object_id] += 1
        self.current_session["total_objects"] += 1
        if context == "contradiction_system":
            self.metrics["contradiction_tracking"]["total"] += 1

    def record_metric(self, name: str, value: float, context: str = "", object_id: Optional[str] = None, context_level: Optional[str] = None) -> None:
        """Record a metric."""
        if name.startswith("error:"):
            self.metrics["errors"][context] += 1
            self.metrics["failed_operations"].append(context)
        if object_id:
            self.record_object_access(object_id, context_level or "")

    def record_cache_event(self, level: str, hit: bool) -> None:
        """Record a cache hit or miss."""
        if hit:
            self.metrics["cache_performance"]["hits"][level] += 1
        else:
            self.metrics["cache_performance"]["misses"][level] += 1

    def record_token_usage(self, tokens: int) -> None:
        """Record token usage."""
        self.metrics["token_usage"].append(tokens)

    def record_query_latency(self, latency: float) -> None:
        """Record query latency."""
        self.metrics["latency"]["db_queries"].append(latency)

    def record_context_switch(self, latency: float) -> None:
        """Record context switch latency."""
        self.metrics["latency"]["context_switches"].append(latency)

    def record_memory_usage(self, memory_mb: float) -> None:
        """Record memory usage."""
        self.metrics["memory_usage"].append(memory_mb)

    def _analyze_memory_usage(self) -> Dict[str, float]:
        """Analyze memory usage statistics."""
        if not self.metrics["memory_usage"]:
            raise ValueError("No memory usage data available")
            
        memory_values = list(self.metrics["memory_usage"])
        return {
            "avg": mean(memory_values),
            "peak": max(memory_values),
            "current": memory_values[-1]  # Last recorded value
        }

    def _calculate_latency_stats(self) -> Dict[str, Dict[str, float]]:
        """Calculate latency statistics."""
        stats = {}
        for metric_type in ["db_queries", "context_switches"]:
            values = list(self.metrics["latency"][metric_type])
            # Always include the metric type, even if there are no values
            stats[metric_type] = {
                "avg": mean(values) if values else 0.0,
                "min": min(values) if values else 0.0,
                "max": max(values) if values else 0.0,
                "values": values
            }
        return stats

    def analyze_performance(self) -> dict:
        """Analyze collected metrics."""
        hot_objects = {}
        for obj_id, count in self.metrics["object_access"].items():
            hot_objects[obj_id] = {"access_count": count}

        cache_hit_rate = {}
        for level in self.metrics["cache_performance"]["hits"].keys() | self.metrics["cache_performance"]["misses"].keys():
            hits = self.metrics["cache_performance"]["hits"][level]
            misses = self.metrics["cache_performance"]["misses"][level]
            total = hits + misses
            cache_hit_rate[level] = hits / total if total > 0 else 0

        memory_stats = self._analyze_memory_usage() if self.metrics["memory_usage"] else {"avg": 0, "peak": 0, "current": 0}
        latency_stats = self._calculate_latency_stats()

        return {
            "hot_objects": hot_objects,
            "cache_hit_rate": cache_hit_rate,
            "avg_token_usage": mean(self.metrics["token_usage"]) if self.metrics["token_usage"] else 0,
            "latency_stats": latency_stats,
            "memory_profile": memory_stats,
            "error_count": sum(self.metrics["errors"].values()),
            "failed_operations": self.metrics["failed_operations"],
            "contradiction_count": self.metrics["contradiction_tracking"]["total"]
        }

def test_mock_metrics_collector_initialization():
    collector = MockMetricsCollector()
    assert isinstance(collector.metrics["object_access"], Counter)
    assert isinstance(collector.metrics["token_usage"], deque)
    assert collector.metrics["cache_performance"]["hits"]["L1"] == 0
    assert collector.metrics["cache_performance"]["hits"]["L2"] == 0
    assert collector.metrics["cache_performance"]["misses"]["L1"] == 0
    assert collector.metrics["cache_performance"]["misses"]["L2"] == 0

def test_record_object_access():
    collector = MockMetricsCollector()
    collector.record_object_access("obj1", "test")
    collector.record_object_access("obj1", "test")
    collector.record_object_access("obj2", "test")
    
    assert collector.metrics["object_access"]["obj1"] == 2
    assert collector.metrics["object_access"]["obj2"] == 1

def test_record_cache_events():
    collector = MockMetricsCollector()
    collector.record_cache_event("L1", True)
    collector.record_cache_event("L1", False)
    collector.record_cache_event("L2", True)
    
    assert collector.metrics["cache_performance"]["hits"]["L1"] == 1
    assert collector.metrics["cache_performance"]["misses"]["L1"] == 1
    assert collector.metrics["cache_performance"]["hits"]["L2"] == 1
    assert collector.metrics["cache_performance"]["misses"]["L2"] == 0

def test_analyze_performance():
    collector = MockMetricsCollector()
    
    # Add some test data
    collector.record_object_access("hot_obj", "test")
    collector.record_object_access("hot_obj", "test")
    collector.record_cache_event("L1", True)
    collector.record_cache_event("L1", False)
    collector.record_token_usage(100)
    collector.record_token_usage(200)
    collector.record_query_latency(0.1)
    collector.record_query_latency(0.3)
    collector.record_memory_usage(50.0)
    collector.record_memory_usage(70.0)
    collector.record_metric("error:test", 1.0, "test_operation")
    
    results = collector.analyze_performance()
    
    assert "hot_obj" in results["hot_objects"]
    assert results["cache_hit_rate"]["L1"] == 0.5
    assert results["avg_token_usage"] == 150.0
    assert results["latency_stats"]["db_queries"]["avg"] == 0.2
    assert results["memory_profile"]["avg"] == 60.0
    assert results["memory_profile"]["peak"] == 70.0
    assert results["error_count"] == 1
    assert "test_operation" in results["failed_operations"]
