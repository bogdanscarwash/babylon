"""Metric persistence module.

Provides functionality to store and retrieve performance metrics using SQLite.
Implements efficient storage and querying of time-series metric data with 
support for data retention policies and time-range queries.

Classes:
    MetricsPersistence: Main class handling all database operations for metrics.
"""

import json
import logging
import os
import sqlite3
from contextlib import contextmanager
from datetime import datetime, timedelta
from pathlib import Path
import shutil

from babylon.exceptions import (
    DatabaseConnectionError,
    LogRotationError,
    MetricsPersistenceError,
)

# Configure module logger
logger = logging.getLogger(__name__)

from .performance_metrics import AIMetrics, GameplayMetrics, SystemMetrics


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
        # Ensure parent directory exists
        db_dir = os.path.dirname(os.path.abspath(db_path))
        os.makedirs(db_dir, exist_ok=True)
        self._init_db()

    def _init_db(self) -> None:
        """Initialize database schema if not exists."""
        with self._get_connection() as conn:
            conn.executescript(
                """
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
            """
            )

    @contextmanager
    def _get_connection(self):
        """Context manager for database connections."""
        conn = None
        try:
            conn = sqlite3.connect(self.db_path, timeout=20)
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("PRAGMA busy_timeout=10000")
            logger.debug(f"Established database connection to {self.db_path}")
            yield conn
        except sqlite3.Error as e:
            logger.error(f"Database connection error: {e!s}")
            raise DatabaseConnectionError(f"Failed to connect to database: {e!s}")
        finally:
            if conn:
                try:
                    conn.close()
                    logger.debug("Database connection closed")
                except sqlite3.Error as e:
                    logger.warning(f"Error closing database connection: {e!s}")

    def rotate_logs(
        self, max_age_days: int = 30, max_size_mb: int = 10, compress: bool = False
    ) -> None:
        """Rotate database file based on age and size.
        
        Args:
            max_age_days: Maximum age of database in days before rotation
            max_size_mb: Maximum size of database in MB before rotation
            compress: Whether to compress rotated database files
        
        Raises:
            LogRotationError: If rotation fails due to permissions or other IO errors
        """
        logger.info(
            f"Starting database rotation (max size: {max_size_mb}MB)"
        )
        try:
            db_path = Path(self.db_path)
            db_size = db_path.stat().st_size
            max_bytes = max_size_mb * 1024 * 1024

            if db_size > max_bytes:
                # Create rotated filename with timestamp
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                rotated_name = f"{db_path.stem}_{timestamp}{db_path.suffix}"
                rotated_path = db_path.with_name(rotated_name)

                # Copy current database to rotated file
                shutil.copy2(db_path, rotated_path)
                logger.info(f"Created rotated database: {rotated_path}")

                if compress:
                    import gzip
                    # Compress the rotated database
                    with open(rotated_path, 'rb') as f_in:
                        with gzip.open(f"{rotated_path}.gz", 'wb') as f_out:
                            shutil.copyfileobj(f_in, f_out)
                    # Remove uncompressed rotated file
                    os.remove(rotated_path)
                    logger.debug(f"Compressed rotated database: {rotated_path}.gz")

                # Close any existing connections and vacuum the database
                with self._get_connection() as conn:
                    conn.execute("VACUUM")

                # Create new empty database
                if os.path.exists(db_path):
                    os.remove(db_path)
                self._init_db()
                logger.info("Created new empty database")

                # Keep most recent records in new database
                with self._get_connection() as new_conn:
                    # Copy recent records from rotated database
                    cutoff_date = (datetime.now() - timedelta(days=7)).isoformat()
                    for table in ["system_metrics", "ai_metrics", "gameplay_metrics"]:
                        try:
                            # Connect to rotated database
                            rotated_conn = sqlite3.connect(str(rotated_path))
                            # Copy recent records
                            recent_records = rotated_conn.execute(
                                f"SELECT * FROM {table} WHERE timestamp >= ?",
                                (cutoff_date,)
                            ).fetchall()
                            if recent_records:
                                placeholders = ",".join("?" * len(recent_records[0]))
                                new_conn.executemany(
                                    f"INSERT INTO {table} VALUES ({placeholders})",
                                    recent_records
                                )
                            rotated_conn.close()
                        except sqlite3.Error as e:
                            logger.error(f"Error copying recent records from {table}: {e}")

                # Cleanup old rotated files
                cutoff = datetime.now() - timedelta(days=max_age_days)
                for old_db in db_path.parent.glob(f"{db_path.stem}_*{db_path.suffix}*"):
                    try:
                        # Extract timestamp from filename
                        timestamp_str = old_db.stem.split("_")[-1]
                        file_date = datetime.strptime(timestamp_str, "%Y%m%d_%H%M%S")
                        if file_date < cutoff:
                            os.remove(old_db)
                            logger.debug(f"Removed old rotated database: {old_db}")
                    except (ValueError, OSError) as e:
                        logger.warning(f"Error processing old database {old_db}: {e}")

        except Exception as e:
            error_msg = f"Database rotation failed: {e}"
            logger.error(error_msg)
            raise LogRotationError(error_msg) from e

    def save_system_metrics(self, metrics: SystemMetrics) -> None:
        """Save system metrics to database."""
        with self._get_connection() as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO system_metrics
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    metrics.timestamp,
                    metrics.cpu_percent,
                    metrics.memory_percent,
                    metrics.swap_percent,
                    metrics.disk_usage_percent,
                    metrics.gpu_utilization,
                    metrics.gpu_memory_percent,
                ),
            )
            conn.commit()

    def save_ai_metrics(self, metrics: AIMetrics) -> None:
        """Save AI metrics to database."""
        with self._get_connection() as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO ai_metrics
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    datetime.now().isoformat(),
                    metrics.query_latency_ms,
                    metrics.memory_usage_gb,
                    metrics.token_count,
                    metrics.embedding_dimension,
                    metrics.cache_hit_rate,
                    metrics.anomaly_score,
                    json.dumps(metrics.threshold_violations),
                ),
            )
            conn.commit()

    def save_gameplay_metrics(self, metrics: GameplayMetrics) -> None:
        """Save gameplay metrics to database."""
        with self._get_connection() as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO gameplay_metrics
                VALUES (?, ?, ?, ?, ?, ?)
            """,
                (
                    datetime.now().isoformat(),
                    metrics.session_duration,
                    metrics.actions_per_minute,
                    json.dumps(metrics.event_counts),
                    json.dumps(metrics.contradiction_intensities),
                    json.dumps(metrics.user_choices),
                ),
            )
            conn.commit()

    def get_system_metrics(
        self, start_time: str | None = None, end_time: str | None = None
    ) -> list[SystemMetrics]:
        """Retrieve system metrics within time range.
        
        Args:
            start_time: Optional ISO format timestamp for range start
            end_time: Optional ISO format timestamp for range end
            
        Returns:
            List of SystemMetrics objects within the specified time range
            
        Raises:
            ValueError: If timestamps are invalid or end_time is before start_time
        """
        # Validate timestamp formats if provided
        if start_time:
            try:
                start_dt = datetime.fromisoformat(start_time)
            except ValueError:
                raise ValueError("start_time must be in ISO format")
                
        if end_time:
            try:
                end_dt = datetime.fromisoformat(end_time)
            except ValueError:
                raise ValueError("end_time must be in ISO format")
                
        # Validate time range if both timestamps provided
        if start_time and end_time:
            if end_dt < start_dt:
                raise ValueError("end_time cannot be before start_time")

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
        self, start_time: str | None = None, end_time: str | None = None
    ) -> list[AIMetrics]:
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
                    threshold_violations=json.loads(row[7]),
                )
                for row in cursor.fetchall()
            ]

    def get_gameplay_metrics(
        self, start_time: str | None = None, end_time: str | None = None
    ) -> list[GameplayMetrics]:
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
                    user_choices=json.loads(row[5]),
                )
                for row in cursor.fetchall()
            ]

    def cleanup_old_metrics(self, days_to_keep: int = 30) -> None:
        """Remove metrics older than specified days."""
        try:
            cutoff = (datetime.now() - timedelta(days=days_to_keep)).isoformat()
            logger.info(f"Cleaning up metrics older than {cutoff}")

            with self._get_connection() as conn:
                for table in ["system_metrics", "ai_metrics", "gameplay_metrics"]:
                    try:
                        conn.execute(
                            f"DELETE FROM {table} WHERE timestamp < ?",
                            (cutoff,),
                        )
                        conn.commit()  # Added commit after each DELETE
                        logger.debug(f"Cleaned up old records from {table}")
                    except sqlite3.Error as e:
                        error_msg = f"Failed to clean up old metrics from {table}: {e}"
                        logger.error(error_msg)
                        raise MetricsPersistenceError(error_msg)

        except Exception as e:
            error_msg = f"Metrics cleanup failed: {e}"
            logger.error(error_msg)
            raise MetricsPersistenceError(error_msg)
