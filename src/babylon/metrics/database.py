"""Database configuration for metrics collection.

This module provides database session management for metrics persistence,
reusing the main application's database configuration.
"""

from ..data.database import SessionLocal

# Re-export SessionLocal for use in metrics modules
__all__ = ["SessionLocal"]
