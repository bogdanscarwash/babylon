# Performance Metrics Error Codes

## System Metrics (1000-1999)
- 1000: Invalid CPU usage value
- 1001: Invalid memory usage value 
- 1002: Invalid disk usage value
- 1003: GPU metrics collection failed
- 1004: System metrics validation failed

## Metrics Collection (1500-1599)
- 1500: Failed to record metric
- 1501: Invalid metric name
- 1502: Invalid metric value
- 1503: Metric validation failed
- 1504: Context validation failed
- 1505: Failed to save metrics to disk
- 1506: Failed to load metrics from disk
- 1507: Failed to analyze metrics
- 1508: Failed to generate suggestions
- 1509: Failed to calculate statistics

## AI Metrics (2000-2999)
- 2000: Query latency exceeds threshold
- 2001: Memory usage exceeds threshold
- 2002: Cache hit rate below threshold
- 2003: Invalid embedding dimension
- 2004: Token count validation failed

## Gameplay Metrics (3000-3999)
- 3000: Invalid session duration
- 3001: Event count validation failed
- 3002: User choice validation failed
- 3003: Contradiction intensity validation failed

## Collection Errors (4000-4999)
- 4000: Metrics collection failed
- 4001: Persistence operation failed
- 4002: Alert threshold exceeded
- 4003: Metric validation failed

## Integration Errors (5000-5999)
- 5000: ChromaDB operation failed
- 5001: Entity registry operation failed
- 5002: Concurrent operation failed
- 5003: Persistence verification failed

## Backup Errors (6000-6999)
- 6000: Backup creation failed
- 6001: Backup verification failed
- 6002: Insufficient disk space
- 6003: Backup path inaccessible
- 6004: Backup metadata corruption
- 6005: Backup compression failed
- 6006: Restore operation failed
- 6007: Backup integrity check failed
- 6008: Backup cleanup failed
- 6009: Concurrent backup conflict

Backup errors occur during backup/restore operations and include:
- Space validation failures (6002)
- Access permission issues (6003) 
- Data integrity problems (6004, 6007)
- Operation failures (6000, 6005, 6006)
- Resource cleanup issues (6008)
