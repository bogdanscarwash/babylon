# Performance Metrics Logging

## Overview

The metrics system uses structured logging with JSON formatting to enable:
- Easy parsing and analysis
- Integration with monitoring tools
- Historical trend analysis
- Alert generation

## Log Levels

- DEBUG: Detailed debugging information
- INFO: Regular metrics collection
- WARNING: Threshold violations
- ERROR: Collection/validation failures
- CRITICAL: System-level failures

## Log Format

Each log entry contains:
```json
{
    "timestamp": "ISO-8601 timestamp",
    "level": "log level",
    "metrics": {
        "system": {
            "cpu_percent": float,
            "memory_percent": float,
            ...
        },
        "ai": {
            "query_latency_ms": float,
            "cache_hit_rate": float,
            ...
        },
        "gameplay": {
            "session_duration": float,
            "actions_per_minute": float,
            ...
        }
    },
    "threshold_violations": [
        {
            "metric": "metric name",
            "value": float,
            "threshold": float,
            "timestamp": "ISO-8601 timestamp"
        }
    ]
}
```

## Configuration

```python
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('metrics.log'),
        logging.StreamHandler()
    ]
)
```

## Best Practices

1. Always include timestamps
2. Use structured data
3. Include context with errors
4. Rotate log files
5. Monitor log volume
