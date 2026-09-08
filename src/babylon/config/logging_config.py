"""Logging configuration for Babylon/Babylon.

Logging is the nervous system of the simulation.
Every action must be traceable, every decision auditable.

This module provides:
- setup_logging(): Single entry point for logging initialization
- load_logging_config(): Load config from pyproject.toml
- RotatingFileHandler for main and error-only logs
- JSONFormatter for machine-parseable output
- Console handler for human-readable output

Configuration Hierarchy (highest precedence first):
1. LOG_LEVEL environment variable (global override)
2. pyproject.toml [tool.babylon.logging] section
3. logging.yaml file (if exists)
4. Default hardcoded values
"""

from __future__ import annotations

import logging
import logging.config
import os
import sys
from dataclasses import dataclass, field
from logging.handlers import RotatingFileHandler
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

import yaml

from babylon.config.base import BaseConfig
from babylon.config.log import ContextAwareFilter, JSONFormatter

if TYPE_CHECKING:
    pass

# Constants for log rotation (from logging-architecture.yaml spec)
MAIN_LOG_MAX_BYTES: int = 10_000_000  # 10MB
MAIN_LOG_BACKUP_COUNT: int = 5  # 50MB total

ERROR_LOG_MAX_BYTES: int = 5_000_000  # 5MB
ERROR_LOG_BACKUP_COUNT: int = 10  # 50MB total

# TRACE level for ultra-verbose debugging (defined in utils/log.py)
TRACE: int = 5


@dataclass
class LoggingConfig:
    """Centralized logging configuration loaded from pyproject.toml.

    Attributes:
        default_level: Global default log level
        console_level: Log level for console output
        file_level: Log level for file output
        modules: Per-module log level overrides
    """

    default_level: str = "INFO"
    console_level: str = "INFO"
    file_level: str = "DEBUG"
    modules: dict[str, str] = field(default_factory=dict)


def load_logging_config(pyproject_path: Path | None = None) -> LoggingConfig:
    """Load logging configuration from pyproject.toml.

    Args:
        pyproject_path: Path to pyproject.toml. If None, searches for it.

    Returns:
        LoggingConfig with settings from pyproject.toml or defaults.

    The configuration is loaded from [tool.babylon.logging] section.
    """
    config = LoggingConfig()

    # Find pyproject.toml
    if pyproject_path is None:
        pyproject_path = _find_pyproject_toml()

    if pyproject_path is None or not pyproject_path.exists():
        return config

    try:
        # Use tomllib for Python 3.11+ (stdlib)
        import tomllib

        with open(pyproject_path, "rb") as f:
            pyproject = tomllib.load(f)

        # Navigate to [tool.babylon.logging]
        babylon_config = pyproject.get("tool", {}).get("babylon", {}).get("logging", {})

        if babylon_config:
            config.default_level = babylon_config.get("default_level", config.default_level)
            config.console_level = babylon_config.get("console_level", config.console_level)
            config.file_level = babylon_config.get("file_level", config.file_level)
            config.modules = babylon_config.get("modules", {})

    except Exception as e:
        # If we can't load config, log a warning and use defaults
        # (can't use logging here as it's not set up yet)
        print(f"Warning: Could not load logging config from pyproject.toml: {e}", file=sys.stderr)

    return config


def _find_pyproject_toml() -> Path | None:
    """Find pyproject.toml by searching up from current directory.

    Returns:
        Path to pyproject.toml or None if not found.
    """
    # Start from the babylon package directory
    current = Path(__file__).resolve().parent

    # Search up to 10 levels (reasonable limit)
    for _ in range(10):
        candidate = current / "pyproject.toml"
        if candidate.exists():
            return candidate
        if current.parent == current:
            break
        current = current.parent

    # Also check common locations
    for location in [Path.cwd(), Path.cwd().parent]:
        candidate = location / "pyproject.toml"
        if candidate.exists():
            return candidate

    return None


def setup_logging(
    config_path: Path | None = None,
    default_level: str | None = None,
    pyproject_path: Path | None = None,
    console_stream: Literal["stdout", "stderr"] = "stdout",
) -> None:
    """Initialize the logging system.

    Args:
        config_path: Path to a YAML logging configuration file.
                    If None, uses pyproject.toml or default configuration.
        default_level: Override for the default log level. Also raises the
            console handler's level to match (mirrors the ``LOG_LEVEL``
            env var's effect below) — this is what makes a caller's
            ``--verbose``/``-v`` flag actually change console verbosity,
            not just the inert ``LoggingConfig.default_level`` field.
        pyproject_path: Path to pyproject.toml (optional, auto-detected).
        console_stream: Which stream the console handler writes to on the
            default (non-YAML) configuration path — ``"stdout"`` (default,
            unchanged behavior for the CLI and ``tools/*.py`` scripts) or
            ``"stderr"`` for callers whose stdout is a machine-readable
            contract (e.g. the headless runner prints the artifact
            directory path on stdout for command-substitution callers, so
            its console logging must never share that stream). Has no
            effect when ``config_path`` is supplied — an operator-provided
            YAML override owns its own handler stream configuration.

    Configuration is loaded in this order (highest precedence first):
    1. LOG_LEVEL environment variable
    2. default_level parameter
    3. pyproject.toml [tool.babylon.logging] section
    4. logging.yaml file (if config_path provided)
    5. Default values

    The logging system is deterministic - given the same inputs,
    it produces the same outputs. No probabilistic behavior.

    Handler hierarchy:
    - Console: Human-readable, INFO level (or configured level)
    - Main file (RotatingFileHandler): JSON Lines, DEBUG level, 10MB rotation
    - Error file (RotatingFileHandler): JSON Lines, ERROR only, 5MB rotation
    """
    # Ensure log directory exists
    BaseConfig.LOG_DIR.mkdir(parents=True, exist_ok=True)

    # Load configuration from pyproject.toml
    logging_config = load_logging_config(pyproject_path)

    # Override with environment variable if set
    env_level = os.getenv("LOG_LEVEL")
    if env_level:
        logging_config.default_level = env_level.upper()
        logging_config.console_level = env_level.upper()

    # Override with parameter if provided
    if default_level:
        logging_config.default_level = default_level.upper()
        logging_config.console_level = default_level.upper()

    # Try to load YAML config if provided
    if config_path and config_path.exists():
        with open(config_path) as f:
            config = yaml.safe_load(f)

        # Update file handler paths to use configured log directory
        if "handlers" in config:
            for _handler_name, handler_config in config["handlers"].items():
                if "filename" in handler_config:
                    # Make path relative to LOG_DIR
                    filename = Path(handler_config["filename"]).name
                    handler_config["filename"] = str(BaseConfig.LOG_DIR / filename)

        logging.config.dictConfig(config)

        # Apply per-module levels from pyproject.toml even when using yaml config
        _apply_module_levels(logging_config.modules)
    else:
        # Default configuration using pyproject.toml settings
        _setup_default_logging(logging_config, console_stream=console_stream)


def _apply_module_levels(modules: dict[str, str]) -> None:
    """Apply per-module log levels.

    Args:
        modules: Dict mapping logger names to log levels.
    """
    for module_name, level_str in modules.items():
        level = _parse_level(level_str)
        logging.getLogger(module_name).setLevel(level)


def _parse_level(level_str: str) -> int:
    """Parse a log level string to its integer value.

    Args:
        level_str: Level name (DEBUG, INFO, etc.) or TRACE.

    Returns:
        Integer log level.
    """
    level_str = level_str.upper()
    if level_str == "TRACE":
        return TRACE
    return getattr(logging, level_str, logging.INFO)


def _build_dict_config(
    config: LoggingConfig,
    log_dir: Path,
    console_stream: Literal["stdout", "stderr"] = "stdout",
) -> dict[str, Any]:
    """Build the central ``logging.config.dictConfig`` dict for the ``babylon.*`` tree.

    Args:
        config: LoggingConfig with console/file levels from pyproject.toml.
        log_dir: Directory the two rotating file handlers write into
            (the player data dir by default — see ``BaseConfig.LOG_DIR``).
        console_stream: ``"stdout"`` (default) or ``"stderr"`` — see
            :func:`setup_logging`'s ``console_stream`` parameter.

    Returns:
        A ``version: 1`` dictConfig-compatible dict — ONE declarative
        structure shared by both the default path (this function) and any
        operator-supplied YAML override (``setup_logging(config_path=...)``),
        so there is exactly one place that defines the babylon logger tree's
        handler hierarchy:

        - ``console``: human-readable, ``config.console_level``.
        - ``main_file``: rotating JSON Lines, ``config.file_level``, all
          records (spec: "Observability Spine" structured JSONL handler).
        - ``error_file``: rotating JSON Lines, ERROR-only.

        Per-subsystem level overrides (``config.modules``) are intentionally
        NOT embedded here — dictConfig's own level lookup raises on an
        unrecognized name, whereas :func:`_apply_module_levels` degrades to
        INFO (loud warning, not a hard crash) on a typo'd pyproject.toml
        entry. Callers apply them via :func:`_apply_module_levels` after
        ``dictConfig()`` returns.
    """
    return {
        "version": 1,
        "disable_existing_loggers": False,
        "formatters": {
            "console": {
                "format": "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
                "datefmt": "%Y-%m-%d %H:%M:%S",
            },
            "jsonl": {
                "class": "babylon.config.log.JSONFormatter",
            },
        },
        "filters": {
            "context": {"()": "babylon.config.log.ContextAwareFilter"},
        },
        "handlers": {
            "console": {
                "class": "logging.StreamHandler",
                "level": config.console_level,
                "formatter": "console",
                "filters": ["context"],
                "stream": f"ext://sys.{console_stream}",
            },
            "main_file": {
                "class": "logging.handlers.RotatingFileHandler",
                "level": config.file_level,
                "formatter": "jsonl",
                "filters": ["context"],
                "filename": str(log_dir / "babylon.log"),
                "maxBytes": MAIN_LOG_MAX_BYTES,
                "backupCount": MAIN_LOG_BACKUP_COUNT,
            },
            "error_file": {
                "class": "logging.handlers.RotatingFileHandler",
                "level": "ERROR",
                "formatter": "jsonl",
                "filters": ["context"],
                "filename": str(log_dir / "errors.log"),
                "maxBytes": ERROR_LOG_MAX_BYTES,
                "backupCount": ERROR_LOG_BACKUP_COUNT,
            },
        },
        "root": {
            "level": "DEBUG",  # capture everything; handlers filter by their own level
            "handlers": ["console", "main_file", "error_file"],
        },
    }


def _setup_default_logging(
    config: LoggingConfig,
    console_stream: Literal["stdout", "stderr"] = "stdout",
) -> None:
    """Set up default logging configuration via a central ``dictConfig``.

    Args:
        config: LoggingConfig with settings from pyproject.toml.
        console_stream: ``"stdout"`` (default) or ``"stderr"`` — see
            :func:`setup_logging`'s ``console_stream`` parameter.

    Creates a handler hierarchy with:
    - Console handler (human-readable)
    - Rotating main file handler (JSON Lines, ``config.file_level``)
    - Rotating error file handler (JSON Lines, ERROR only)
    """
    logging.config.dictConfig(
        _build_dict_config(config, BaseConfig.LOG_DIR, console_stream=console_stream)
    )

    # Apply per-module levels from pyproject.toml (see _build_dict_config's
    # docstring for why this stays a post-step rather than a "loggers" key).
    _apply_module_levels(config.modules)


def create_ingest_handler(
    source: str,
    run_id: str | None = None,
) -> tuple[RotatingFileHandler, str]:
    """Create a file handler for a data ingest run.

    Creates a per-ingest log file for debugging and auditing data loads.

    Args:
        source: Data source identifier (e.g., "census", "fred", "qcew").
        run_id: Optional run ID. If None, generates one from timestamp+uuid.

    Returns:
        Tuple of (handler, run_id) for the ingest log.
    """
    import uuid
    from datetime import UTC, datetime

    # Ensure ingest logs directory exists
    ingest_log_dir = BaseConfig.LOG_DIR / "ingest"
    ingest_log_dir.mkdir(parents=True, exist_ok=True)

    # Generate run_id if not provided
    if run_id is None:
        timestamp = datetime.now(UTC).strftime("%Y%m%d_%H%M%S")
        run_id = f"{timestamp}_{uuid.uuid4().hex[:8]}"

    # Build filename: {source}_ingest_{run_id}.jsonl
    filename = f"{source}_ingest_{run_id}.jsonl"

    handler = RotatingFileHandler(
        ingest_log_dir / filename,
        maxBytes=MAIN_LOG_MAX_BYTES,
        backupCount=3,
    )
    handler.setLevel(logging.DEBUG)  # Capture all levels in file
    handler.setFormatter(JSONFormatter())
    handler.addFilter(ContextAwareFilter())

    return handler, run_id


def get_current_config() -> LoggingConfig:
    """Get the current logging configuration (for debugging/inspection).

    Returns:
        LoggingConfig loaded from pyproject.toml.
    """
    return load_logging_config()
