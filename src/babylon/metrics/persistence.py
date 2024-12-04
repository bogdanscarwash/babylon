"""Metric persistence module.

Provides functionality to store and retrieve performance metrics using SQLite.
Implements efficient storage and querying of time-series metric data with 
support for data retention policies and time-range queries.

Classes:
    MetricsPersistence: Main class handling all database operations for metrics.
"""

import sqlite3
import json
import logging
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Optional, Dict, Any
from contextlib import contextmanager

from .exceptions import (
    DatabaseConnectionError,
    MetricsPersistenceError,
    LogRotationError
)

# Configure module logger
logger = logging.getLogger(__name__)

from .performance_metrics import SystemMetrics, AIMetrics, GameplayMetrics


class MetricsPersistence:
    """Handles persistence of performance metrics to SQLite database.

    This class provides methods to store and retrieve different types of metrics
    (system, AI, gameplay) using SQLite as the backing store. It handles 
    database initialization, connection management, and data cleanup.

    Attributes:
        db_path (str): Path to the SQLite database file
    """
    def __init__(self, db_path: str = "metrics.db"):
        """Initialize metrics persistence.

        Args:
            db_path: Path to SQLite database file. Defaults to 'metrics.db'
                    in the current directory.
        """
        self.db_path = db_path
        self._init_db()

    def _init_db(self) -> None:
        """Initialize database schema if not exists."""
        with self._get_connection() as conn:
            conn.executescript("""
                PRAGMA foreign_keys = ON;
                CREATE TABLE IF NOT EXISTS system_metrics (
                    timestamp TEXT PRIMARY KEY,
                    cpu_percent REAL,
                    memory_percent REAL,
                    swap_percent REAL,
                    disk_usage_percent REAL,
                    gpu_utilization REAL,
                    gpu_memory_percent REAL
                );

                CREATE TABLE IF NOT EXISTS ai_metrics (
                    timestamp TEXT PRIMARY KEY,
                    query_latency_ms REAL,
                    memory_usage_gb REAL,
                    token_count INTEGER,
                    embedding_dimension INTEGER,
                    cache_hit_rate REAL,
                    anomaly_score REAL,
                    threshold_violations TEXT
                );

                CREATE TABLE IF NOT EXISTS gameplay_metrics (
                    timestamp TEXT PRIMARY KEY,
                    session_duration REAL,
                    actions_per_minute REAL,
                    event_counts TEXT,
                    contradiction_intensities TEXT,
                    user_choices TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp
                ON system_metrics(timestamp);
                CREATE INDEX IF NOT EXISTS idx_ai_metrics_timestamp
                ON ai_metrics(timestamp);
                CREATE INDEX IF NOT EXISTS idx_gameplay_metrics_timestamp
                ON gameplay_metrics(timestamp);
            """)

    @contextmanager
    def _get_connection(self):
        """Context manager for database connections.
        
        Provides safe database connection handling with proper error logging
        and connection cleanup.
        
        Raises:
            DatabaseConnectionError: If connection cannot be established
        """
        conn = None
        try:
            conn = sqlite3.connect(self.db_path)
            logger.debug(f"Established database connection to {self.db_path}")
            yield conn
        except sqlite3.Error as e:
            logger.error(f"Database connection error: {str(e)}")
            raise DatabaseConnectionError(f"Failed to connect to database: {str(e)}")
        finally:
            if conn:
                try:
                    conn.close()
                    logger.debug("Database connection closed")
                except sqlite3.Error as e:
                    logger.warning(f"Error closing database connection: {str(e)}")

    def save_system_metrics(self, metrics: SystemMetrics) -> None:
        """Save system metrics to database.
        
        Args:
            metrics: SystemMetrics instance to save
        """
        with self._get_connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO system_metrics
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                metrics.timestamp,
                metrics.cpu_percent,
                metrics.memory_percent,
                metrics.swap_percent,
                metrics.disk_usage_percent,
                metrics.gpu_utilization,
                metrics.gpu_memory_percent
            ))

    def save_ai_metrics(self, metrics: AIMetrics) -> None:
        """Save AI metrics to database.
        
        Args:
            metrics: AIMetrics instance to save
        """
        with self._get_connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO ai_metrics
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                datetime.now().isoformat(),
                metrics.query_latency_ms,
                metrics.memory_usage_gb,
                metrics.token_count,
                metrics.embedding_dimension,
                metrics.cache_hit_rate,
                metrics.anomaly_score,
                json.dumps(metrics.threshold_violations)
            ))

    def save_gameplay_metrics(self, metrics: GameplayMetrics) -> None:
        """Save gameplay metrics to database.
        
        Args:
            metrics: GameplayMetrics instance to save
        """
        with self._get_connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO gameplay_metrics
                VALUES (?, ?, ?, ?, ?, ?)
            """, (
                datetime.now().isoformat(),
                metrics.session_duration,
                metrics.actions_per_minute,
                json.dumps(metrics.event_counts),
                json.dumps(metrics.contradiction_intensities),
                json.dumps(metrics.user_choices)
            ))

    def get_system_metrics(
        self,
        start_time: Optional[str] = None,
        end_time: Optional[str] = None
    ) -> List[SystemMetrics]:
        """Retrieve system metrics within time range.
        
        Args:
            start_time: ISO format timestamp for range start
            end_time: ISO format timestamp for range end
            
        Returns:
            List of SystemMetrics instances
        """
        query = "SELECT * FROM system_metrics"
        params = []
        
        if start_time or end_time:
            query += " WHERE "
            if start_time:
                query += "timestamp >= ?"
                params.append(start_time)
            if end_time:
                query += " AND " if start_time else ""
                query += "timestamp <= ?"
                params.append(end_time)
                
        with self._get_connection() as conn:
            cursor = conn.execute(query, params)
            return [SystemMetrics(*row) for row in cursor.fetchall()]

    def get_ai_metrics(
        self,
        start_time: Optional[str] = None,
        end_time: Optional[str] = None
    ) -> List[AIMetrics]:
        """Retrieve AI metrics within time range."""
        query = "SELECT * FROM ai_metrics"
        params = []
        
        if start_time or end_time:
            query += " WHERE "
            if start_time:
                query += "timestamp >= ?"
                params.append(start_time)
            if end_time:
                query += " AND " if start_time else ""
                query += "timestamp <= ?"
                params.append(end_time)
                
        with self._get_connection() as conn:
            cursor = conn.execute(query, params)
            return [
                AIMetrics(
                    query_latency_ms=row[1],
                    memory_usage_gb=row[2],
                    token_count=row[3],
                    embedding_dimension=row[4],
                    cache_hit_rate=row[5],
                    anomaly_score=row[6],
                    threshold_violations=json.loads(row[7])
                )
                for row in cursor.fetchall()
            ]

    def get_gameplay_metrics(
        self,
        start_time: Optional[str] = None,
        end_time: Optional[str] = None
    ) -> List[GameplayMetrics]:
        """Retrieve gameplay metrics within time range."""
        query = "SELECT * FROM gameplay_metrics"
        params = []
        
        if start_time or end_time:
            query += " WHERE "
            if start_time:
                query += "timestamp >= ?"
                params.append(start_time)
            if end_time:
                query += " AND " if start_time else ""
                query += "timestamp <= ?"
                params.append(end_time)
                
        with self._get_connection() as conn:
            cursor = conn.execute(query, params)
            return [
                GameplayMetrics(
                    session_duration=row[1],
                    actions_per_minute=row[2],
                    event_counts=json.loads(row[3]),
                    contradiction_intensities=json.loads(row[4]),
                    user_choices=json.loads(row[5])
                )
                for row in cursor.fetchall()
            ]

    def rotate_logs(self, max_age_days: int = 30, max_size_mb: int = 10, compress: bool = False) -> None:
        """Rotate log files based on age and size.
        
        Args:
            max_age_days: Maximum age of log files in days
            max_size_mb: Maximum size of log files in MB
            compress: Whether to compress rotated logs
            
        Raises:
            LogRotationError: If log rotation fails
        """
        logger.info(f"Starting log rotation (max age: {max_age_days} days, max size: {max_size_mb}MB)")
        try:
            cutoff = datetime.now() - timedelta(days=max_age_days)
            max_bytes = max_size_mb * 1024 * 1024
            
            for log_file in Path(self.db_path).parent.glob("*.log"):
                try:
                    stats = log_file.stat()
                    
                    # Check age and size
                    if (datetime.fromtimestamp(stats.st_mtime) < cutoff or 
                        stats.st_size > max_bytes):
                        
                        logger.info(f"Rotating log file: {log_file}")
                        
                        # Rotate the file
                        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                        rotated_name = f"{log_file.stem}_{timestamp}{log_file.suffix}"
                        rotated_path = log_file.with_name(rotated_name)
                        
                        # Compress if requested
                        if compress:
                            try:
                                import gzip
                                with log_file.open('rb') as f_in:
                                    with gzip.open(f"{rotated_path}.gz", 'wb') as f_out:
                                        f_out.write(f_in.read())
                                log_file.unlink()
                                logger.debug(f"Compressed and removed original: {log_file}")
                            except (IOError, OSError) as e:
                                raise LogRotationError(f"Failed to compress log file {log_file}: {str(e)}")
                        else:
                            try:
                                log_file.rename(rotated_path)
                                logger.debug(f"Renamed log file to: {rotated_path}")
                            except OSError as e:
                                raise LogRotationError(f"Failed to rename log file {log_file}: {str(e)}")
                                
                except OSError as e:
                    logger.error(f"Error processing log file {log_file}: {str(e)}")
                    continue
                    
        except Exception as e:
            error_msg = f"Log rotation failed: {str(e)}"
            logger.error(error_msg)
            raise LogRotationError(error_msg)

    def cleanup_old_metrics(self, days_to_keep: int = 30) -> None:
        """Remove metrics older than specified days."""
        cutoff = (datetime.now() - timedelta(days=days_to_keep)).isoformat()
        
        with self._get_connection() as conn:
            for table in ['system_metrics', 'ai_metrics', 'gameplay_metrics']:
                conn.execute(f"""
                    DELETE FROM {table}
                    WHERE timestamp < ?
                """, (cutoff,))
