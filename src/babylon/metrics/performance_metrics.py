"""Performance metrics collection module.

Collects system performance metrics including:
- CPU/Memory/GPU utilization
- AI model performance data
- Gameplay analytics
- User behavior patterns
"""

import psutil
import logging
import json
import time
from typing import Dict, Any, Optional
import GPUtil
from dataclasses import dataclass, asdict
from datetime import datetime

logger = logging.getLogger(__name__)

@dataclass
class SystemMetrics:
    """System performance metrics."""
    timestamp: str
    cpu_percent: float
    memory_percent: float
    swap_percent: float
    disk_usage_percent: float
    gpu_utilization: Optional[float]
    gpu_memory_percent: Optional[float]

@dataclass
class AIMetrics:
    """AI system performance and behavior metrics."""
    model_latency_ms: float
    token_count: int
    embedding_dimension: int
    cache_hit_rate: float
    anomaly_score: float

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

    def __init__(self):
        self.start_time = time.time()
        self.action_count = 0
        self.event_counts = {}
        self.user_choices = {}

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

    def collect_ai_metrics(self, model_latency: float, token_count: int,
                         embedding_dim: int, cache_hits: int,
                         cache_total: int, anomaly_score: float) -> AIMetrics:
        """Collect AI system performance metrics."""
        return AIMetrics(
            model_latency_ms=model_latency * 1000,
            token_count=token_count,
            embedding_dimension=embedding_dim,
            cache_hit_rate=cache_hits / max(cache_total, 1),
            anomaly_score=anomaly_score
        )

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
        """Log collected metrics in JSON format."""
        metrics_data = {
            "timestamp": datetime.now().isoformat(),
            "system": asdict(system_metrics) if system_metrics else None,
            "ai": asdict(ai_metrics) if ai_metrics else None,
            "gameplay": asdict(gameplay_metrics) if gameplay_metrics else None
        }

        logger.info("Performance metrics", extra={"metrics": metrics_data})

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
