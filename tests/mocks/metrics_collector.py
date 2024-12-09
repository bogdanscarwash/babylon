from typing import Dict, List, Optional
from dataclasses import dataclass

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
    
    def __init__(self):
        self.access_records: Dict[str, int] = {}
        self.cache_hits: Dict[str, int] = {"L1": 0, "L2": 0}
        self.cache_misses: Dict[str, int] = {"L1": 0, "L2": 0}
        self.token_usage: List[int] = []
        self.query_latencies: List[float] = []
        self.memory_usage: List[float] = []
        
    def record_object_access(self, object_id: str, context: str) -> None:
        """Record an object access."""
        if object_id not in self.access_records:
            self.access_records[object_id] = 0
        self.access_records[object_id] += 1

    def record_cache_event(self, level: str, hit: bool) -> None:
        """Record a cache hit or miss."""
        if hit:
            self.cache_hits[level] += 1
        else:
            self.cache_misses[level] += 1

    def record_token_usage(self, tokens: int) -> None:
        """Record token usage."""
        self.token_usage.append(tokens)

    def record_query_latency(self, latency: float) -> None:
        """Record query latency."""
        self.query_latencies.append(latency)

    def record_memory_usage(self, memory_mb: float) -> None:
        """Record memory usage."""
        self.memory_usage.append(memory_mb)

    def analyze_performance(self) -> dict:
        """Analyze collected metrics."""
        hot_objects = sorted(
            self.access_records.keys(),
            key=lambda x: self.access_records[x],
            reverse=True
        )

        cache_hit_rate = {}
        for level in ["L1", "L2"]:
            total = self.cache_hits[level] + self.cache_misses[level]
            cache_hit_rate[level] = self.cache_hits[level] / total if total > 0 else 0

        return {
            "hot_objects": hot_objects,
            "cache_hit_rate": cache_hit_rate,
            "avg_token_usage": sum(self.token_usage) / len(self.token_usage) if self.token_usage else 0,
            "latency_stats": {
                "context_switches": len(self.query_latencies) > 0,
                "avg_latency": sum(self.query_latencies) / len(self.query_latencies) if self.query_latencies else 0
            },
            "memory_profile": {
                "avg": sum(self.memory_usage) / len(self.memory_usage) if self.memory_usage else 0,
                "peak": max(self.memory_usage) if self.memory_usage else 0
            }
        }