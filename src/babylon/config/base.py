"""Base configuration module providing core settings.

This module defines the BaseConfig class which serves as the foundation for
environment-specific configurations. It loads settings from environment variables
with sensible defaults.
"""

import os
from pathlib import Path
from typing import ClassVar, Final

from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()


class BaseConfig:
    """Base configuration class providing default settings.

    This class defines the base configuration settings used across all environments.
    It should be subclassed by environment-specific configurations that can
    override these values as needed.

    Class Attributes:
        SECRET_KEY (ClassVar[str]): Secret key for security features
        DATABASE_URL (ClassVar[str]): Database connection string
        DB_POOL_SIZE (ClassVar[int]): Database connection pool size
        DB_MAX_OVERFLOW (ClassVar[int]): Maximum pool overflow
        DEBUG (ClassVar[bool]): Debug mode flag
        TESTING (ClassVar[bool]): Testing mode flag
        CHROMADB_PERSIST_DIR (ClassVar[Path]): Directory for ChromaDB persistence
        METRICS_ENABLED (ClassVar[bool]): Enable metrics collection
        METRICS_INTERVAL (ClassVar[int]): Metrics collection interval in seconds
        LOG_LEVEL (ClassVar[str]): Minimum logging level
        LOG_FORMAT (ClassVar[str]): Log message format
        LOG_DIR (ClassVar[Path]): Log file directory
    """

    # Security settings
    SECRET_KEY: ClassVar[str] = os.getenv("SECRET_KEY", "default-secret-key")

    # Database settings
    DATABASE_URL: ClassVar[str] = os.getenv(
        "DATABASE_URL", "postgresql://username:password@localhost:5432/babylon_db"
    )
    DB_POOL_SIZE: ClassVar[int] = int(os.getenv("DB_POOL_SIZE", "5"))
    DB_MAX_OVERFLOW: ClassVar[int] = int(os.getenv("DB_MAX_OVERFLOW", "10"))

    # Application mode flags
    DEBUG: ClassVar[bool] = os.getenv("DEBUG", "false").lower() == "true"
    TESTING: ClassVar[bool] = os.getenv("TESTING", "false").lower() == "true"

    # ChromaDB settings
    CHROMADB_PERSIST_DIR: ClassVar[Path] = Path(
        os.getenv("CHROMADB_PERSIST_DIR", "./chromadb")
    )

    # Metrics settings
    METRICS_ENABLED: ClassVar[bool] = (
        os.getenv("METRICS_ENABLED", "true").lower() == "true"
    )
    METRICS_INTERVAL: ClassVar[int] = int(os.getenv("METRICS_INTERVAL", "60"))

    # Logging settings
    LOG_LEVEL: Final[str] = os.getenv("LOG_LEVEL", "INFO")
    LOG_FORMAT: Final[str] = os.getenv(
        "LOG_FORMAT", "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    LOG_DIR: ClassVar[Path] = Path(os.getenv("LOG_DIR", "./logs"))
