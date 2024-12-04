"""Performance metrics collection module.

This module provides comprehensive performance monitoring capabilities for the Babylon system:

Core Features:
- System resource monitoring (CPU, Memory, GPU, Disk)
- AI model performance tracking (latency, cache efficiency)
- Gameplay analytics collection
- Performance threshold validation
- Metric persistence and alerting

Usage:
    collector = MetricsCollector()
    
    # System metrics
    sys_metrics = collector.collect_system_metrics()
    
    # AI performance
    ai_metrics = collector.collect_ai_metrics(
        query_time=0.05,
        token_count=100,
        embedding_dim=384,
        cache_hits=90,
        cache_total=100,
        anomaly_score=0.1
    )
    
    # Log all metrics
    collector.log_metrics(
        system_metrics=sys_metrics,
        ai_metrics=ai_metrics
    )
"""

# Standard library imports
import json
import logging
import time
from datetime import datetime
from typing import Dict, Any, Optional, List, Tuple

# Third-party imports
import psutil
import GPUtil
from dataclasses import dataclass, asdict, field

# Configure module logger
logger = logging.getLogger(__name__)

logger = logging.getLogger(__name__)

@dataclass
class SystemMetrics:
    """System performance metrics.
    
    Tracks real-time system resource utilization including CPU, memory,
    disk, and GPU metrics when available.
    
    Attributes:
        timestamp: ISO format timestamp of metric collection
        cpu_percent: CPU utilization percentage (0-100)
        memory_percent: RAM utilization percentage (0-100)
        swap_percent: Swap space utilization percentage (0-100)
        disk_usage_percent: Disk space utilization percentage (0-100)
        gpu_utilization: GPU utilization percentage if available (0-100)
        gpu_memory_percent: GPU memory utilization if available (0-100)
    """
    timestamp: str
    cpu_percent: float
    memory_percent: float
    swap_percent: float
    disk_usage_percent: float
    gpu_utilization: Optional[float] = None
    gpu_memory_percent: Optional[float] = None
    
    def __post_init__(self):
        """Validate metric ranges."""
        for field_name in ['cpu_percent', 'memory_percent', 'swap_percent', 'disk_usage_percent']:
            value = getattr(self, field_name)
            if not 0 <= value <= 100:
                raise ValueError(f"{field_name} must be between 0 and 100")
        
        # Validate GPU metrics if present
        for field_name in ['gpu_utilization', 'gpu_memory_percent']:
            value = getattr(self, field_name)
            if value is not None and not 0 <= value <= 100:
                raise ValueError(f"{field_name} must be between 0 and 100")

@dataclass
class PerformanceThresholds:
    """Performance thresholds for monitoring.
    
    Defines acceptable ranges for various performance metrics.
    Exceeding these thresholds triggers warnings/alerts.
    
    Attributes:
        MAX_QUERY_LATENCY_MS: Maximum acceptable query response time
        MAX_MEMORY_USAGE_GB: Maximum acceptable memory usage
        MIN_CACHE_HIT_RATE: Minimum acceptable cache hit rate
        ALERT_INTERVAL_SEC: Minimum time between repeated alerts
    """
    MAX_QUERY_LATENCY_MS: float = 100.0
    MAX_MEMORY_USAGE_GB: float = 2.0
    MIN_CACHE_HIT_RATE: float = 0.90
    ALERT_INTERVAL_SEC: float = 300.0  # 5 minutes between repeated alerts
    
    def __post_init__(self):
        """Validate threshold values."""
        if self.MAX_QUERY_LATENCY_MS <= 0:
            raise ValueError("MAX_QUERY_LATENCY_MS must be positive")
        if self.MAX_MEMORY_USAGE_GB <= 0:
            raise ValueError("MAX_MEMORY_USAGE_GB must be positive")
        if not 0 <= self.MIN_CACHE_HIT_RATE <= 1:
            raise ValueError("MIN_CACHE_HIT_RATE must be between 0 and 1")
        if self.ALERT_INTERVAL_SEC <= 0:
            raise ValueError("ALERT_INTERVAL_SEC must be positive")

@dataclass
class AIMetrics:
    """AI system performance and behavior metrics.
    
    Tracks performance metrics related to AI model operations including
    query latency, memory usage, and cache efficiency.
    
    Attributes:
        query_latency_ms: Query response time in milliseconds
        memory_usage_gb: Memory usage in gigabytes
        token_count: Number of tokens processed
        embedding_dimension: Dimension of embedding vectors
        cache_hit_rate: Cache hit rate (0-1)
        anomaly_score: Anomaly detection score (0-1)
        threshold_violations: List of recent threshold violations
    """
    query_latency_ms: float
    memory_usage_gb: float
    token_count: int
    embedding_dimension: int
    cache_hit_rate: float
    anomaly_score: float
    threshold_violations: List[Tuple[str, float]] = field(default_factory=list)
    
    def __post_init__(self):
        """Validate metric ranges."""
        if self.query_latency_ms < 0:
            raise ValueError("query_latency_ms must be non-negative")
        if self.memory_usage_gb < 0:
            raise ValueError("memory_usage_gb must be non-negative")
        if self.token_count < 0:
            raise ValueError("token_count must be non-negative")
        if self.embedding_dimension <= 0:
            raise ValueError("embedding_dimension must be positive")
        if not 0 <= self.cache_hit_rate <= 1:
            raise ValueError("cache_hit_rate must be between 0 and 1")
        if not 0 <= self.anomaly_score <= 1:
            raise ValueError("anomaly_score must be between 0 and 1")
    
    def check_thresholds(self) -> Dict[str, bool]:
        """Check if metrics meet performance thresholds.
        
        Returns:
            Dict mapping metric names to boolean threshold compliance.
            
        Side Effects:
            Records threshold violations in threshold_violations list.
        """
        thresholds = PerformanceThresholds()
        results = {
            "query_latency": self.query_latency_ms <= thresholds.MAX_QUERY_LATENCY_MS,
            "memory_usage": self.memory_usage_gb <= thresholds.MAX_MEMORY_USAGE_GB,
            "cache_hit_rate": self.cache_hit_rate >= thresholds.MIN_CACHE_HIT_RATE
        }
        
        # Record violations with their values
        timestamp = datetime.now().isoformat()
        for metric, compliant in results.items():
            if not compliant:
                value = getattr(self, f"{metric}_ms" if metric == "query_latency" else 
                                    f"{metric}_gb" if metric == "memory_usage" else
                                    metric)
                self.threshold_violations.append((timestamp, metric, value))
        
        return results

@dataclass
class GameplayMetrics:
    """User gameplay and behavior metrics."""
    session_duration: float
    actions_per_minute: float
    event_counts: Dict[str, int]
    contradiction_intensities: Dict[str, float]
    user_choices: Dict[str, Any]

class MetricsCollector:
    """Collects and logs system, AI, and gameplay metrics."""

    def __init__(self, persist_metrics: bool = True, db_path: str = "metrics.db"):
        self.start_time = time.time()
        self.action_count = 0
        self.event_counts = {}
        self.user_choices = {}
        
        self.persist_metrics = persist_metrics
        if persist_metrics:
            from .persistence import MetricsPersistence
            self.persistence = MetricsPersistence(db_path)

    def collect_system_metrics(self) -> SystemMetrics:
        """Collect current system performance metrics."""
        metrics = SystemMetrics(
            timestamp=datetime.now().isoformat(),
            cpu_percent=psutil.cpu_percent(),
            memory_percent=psutil.virtual_memory().percent,
            swap_percent=psutil.swap_memory().percent,
            disk_usage_percent=psutil.disk_usage('/').percent,
            gpu_utilization=None,
            gpu_memory_percent=None
        )

        # Collect GPU metrics if available
        try:
            gpus = GPUtil.getGPUs()
            if gpus:
                metrics.gpu_utilization = gpus[0].load * 100
                metrics.gpu_memory_percent = gpus[0].memoryUtil * 100
        except Exception as e:
            logger.warning(f"Failed to collect GPU metrics: {e}")

        return metrics

    def collect_ai_metrics(self, query_time: float, token_count: int,
                         embedding_dim: int, cache_hits: int,
                         cache_total: int, anomaly_score: float) -> AIMetrics:
        """Collect AI system performance metrics."""
        memory_usage = psutil.Process().memory_info().rss / (1024 * 1024 * 1024)  # Convert to GB
        cache_hit_rate = cache_hits / max(cache_total, 1)
        
        metrics = AIMetrics(
            query_latency_ms=query_time * 1000,
            memory_usage_gb=memory_usage,
            token_count=token_count,
            embedding_dimension=embedding_dim,
            cache_hit_rate=cache_hit_rate,
            anomaly_score=anomaly_score
        )
        
        # Check if metrics meet thresholds
        threshold_results = metrics.check_thresholds()
        for metric, meets_threshold in threshold_results.items():
            if not meets_threshold:
                logger.warning(f"Performance threshold exceeded for {metric}")
                
        return metrics

    def collect_gameplay_metrics(self) -> GameplayMetrics:
        """Collect user gameplay metrics."""
        session_duration = time.time() - self.start_time
        actions_per_minute = (self.action_count / session_duration) * 60

        return GameplayMetrics(
            session_duration=session_duration,
            actions_per_minute=actions_per_minute,
            event_counts=self.event_counts.copy(),
            contradiction_intensities={},  # Populated by contradiction system
            user_choices=self.user_choices.copy()
        )

    def log_metrics(self, system_metrics: Optional[SystemMetrics] = None,
                   ai_metrics: Optional[AIMetrics] = None,
                   gameplay_metrics: Optional[GameplayMetrics] = None) -> None:
        """Log collected metrics in JSON format and persist if enabled."""
        metrics_data = {
            "timestamp": datetime.now().isoformat(),
            "system": asdict(system_metrics) if system_metrics else None,
            "ai": asdict(ai_metrics) if ai_metrics else None,
            "gameplay": asdict(gameplay_metrics) if gameplay_metrics else None
        }

        logger.info("Performance metrics", extra={"metrics": metrics_data})
        
        if self.persist_metrics:
            if system_metrics:
                self.persistence.save_system_metrics(system_metrics)
            if ai_metrics:
                self.persistence.save_ai_metrics(ai_metrics)
            if gameplay_metrics:
                self.persistence.save_gameplay_metrics(gameplay_metrics)

    def record_event(self, event_type: str) -> None:
        """Record occurrence of a game event."""
        self.event_counts[event_type] = self.event_counts.get(event_type, 0) + 1
        self.action_count += 1

    def record_user_choice(self, context: str, choice: Any) -> None:
        """Record a user's gameplay choice."""
        if context not in self.user_choices:
            self.user_choices[context] = []
        self.user_choices[context].append({
            "timestamp": datetime.now().isoformat(),
            "choice": choice
        })
