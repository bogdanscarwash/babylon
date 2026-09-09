"""Custom logging utilities for Babylon.

Logging is the nervous system of the simulation.
Every action must be traceable, every decision auditable.

This module provides:
- JSONFormatter: Dependency-free JSON Lines formatter
- TRACE level: Ultra-verbose debugging (level=5)
- LogContext: Context propagation for correlation IDs and tick numbers
"""

from __future__ import annotations

import json
import logging
from contextlib import contextmanager
from contextvars import ContextVar
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Generator

# =============================================================================
# TRACE Level (value=5)
# =============================================================================

TRACE: int = 5
"""Ultra-verbose tracing level for deep debugging.

NEVER enable in production. For materialist microscopy only.
"""

logging.addLevelName(TRACE, "TRACE")


def _trace(self: logging.Logger, message: str, *args: object, **kwargs: Any) -> None:
    """Log a TRACE-level message.

    Args:
        message: The message format string.
        *args: Arguments for message formatting.
        **kwargs: Keyword arguments passed to _log().
    """
    if self.isEnabledFor(TRACE):
        self._log(TRACE, message, args, **kwargs)


# Monkey-patch Logger class with trace method
logging.Logger.trace = _trace  # type: ignore[attr-defined]


# =============================================================================
# JSON Formatter (no external dependencies)
# =============================================================================


class JSONFormatter(logging.Formatter):
    """JSON Lines formatter - zero external dependencies.

    Produces machine-parseable JSONL output suitable for log aggregation.
    Each log record becomes a single JSON object on one line.

    Output format:
        {"ts":"2025-01-09T14:23:45.123Z","level":"ERROR","logger":"babylon.intelligence.rag",...}

    Extra fields (tick, correlation_id, exception, etc.) are automatically
    included from the LogRecord's __dict__.
    """

    # Standard LogRecord attributes to exclude from extra fields
    STANDARD_FIELDS: frozenset[str] = frozenset(
        {
            "name",
            "msg",
            "args",
            "created",
            "filename",
            "funcName",
            "levelname",
            "levelno",
            "lineno",
            "module",
            "msecs",
            "pathname",
            "process",
            "processName",
            "relativeCreated",
            "stack_info",
            "thread",
            "threadName",
            "exc_info",
            "exc_text",
            "message",
            "taskName",
        }
    )

    def format(self, record: logging.LogRecord) -> str:
        """Format log record as JSON.

        Args:
            record: The LogRecord to format.

        Returns:
            JSON string representing the log entry.
        """
        # Build base log dictionary
        log_dict: dict[str, Any] = {
            "ts": datetime.now(UTC).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "msg": record.getMessage(),
            "func": record.funcName,
            "line": record.lineno,
        }

        # Add extra fields (tick, correlation_id, exception, etc.)
        for key, value in record.__dict__.items():
            if key not in self.STANDARD_FIELDS:
                log_dict[key] = value

        # Handle exception info
        if record.exc_info:
            log_dict["exc_info"] = self.formatException(record.exc_info)

        return json.dumps(log_dict, default=str)


# =============================================================================
# Log Context Propagation
# =============================================================================

# Note: Using None as default to avoid mutable default warning (B039)
# The get_log_context() function handles the None case by returning an empty dict
_log_context: ContextVar[dict[str, Any] | None] = ContextVar("log_context", default=None)


def get_log_context() -> dict[str, Any]:
    """Get the current log context.

    Returns:
        Dictionary containing context fields (tick, simulation_id, etc.).
    """
    ctx = _log_context.get()
    if ctx is None:
        return {}
    return ctx.copy()


def set_log_context(**kwargs: Any) -> None:
    """Set log context fields.

    These fields will be automatically included in log entries
    when using the ContextAwareFilter.

    Args:
        **kwargs: Context fields to set (tick, simulation_id, correlation_id, etc.).
    """
    current = _log_context.get()
    current = {} if current is None else current.copy()
    current.update(kwargs)
    _log_context.set(current)


def clear_log_context() -> None:
    """Clear all log context fields."""
    _log_context.set(None)


@contextmanager
def log_context_scope(**kwargs: Any) -> Generator[None, None, None]:
    """Context manager for scoped log context.

    Context fields are automatically restored when exiting the scope.

    Usage:
        with log_context_scope(tick=42, simulation_id="abc"):
            logger.info("Tick complete")  # Includes tick and simulation_id

    Args:
        **kwargs: Context fields to set within the scope.

    Yields:
        None
    """
    old_context = _log_context.get()
    if old_context is None:
        old_context_copy: dict[str, Any] = {}
    else:
        old_context_copy = old_context.copy()
    try:
        new_context = old_context_copy.copy()
        new_context.update(kwargs)
        _log_context.set(new_context)
        yield
    finally:
        _log_context.set(old_context)


class ContextAwareFilter(logging.Filter):
    """Logging filter that injects context fields into LogRecords.

    Automatically adds fields from the current log context
    (tick, simulation_id, correlation_id, etc.) to every log record.

    Usage:
        handler.addFilter(ContextAwareFilter())
    """

    def filter(self, record: logging.LogRecord) -> bool:
        """Add context fields to the log record.

        Args:
            record: The LogRecord to modify.

        Returns:
            True (always passes the record through).
        """
        context = get_log_context()
        for key, value in context.items():
            if not hasattr(record, key):
                setattr(record, key, value)
        return True


# =============================================================================
# URL and Parameter Redaction
# =============================================================================

# Keys that should have their values redacted in logs
_SENSITIVE_KEYS: frozenset[str] = frozenset(
    {
        "key",
        "api_key",
        "apikey",
        "api-key",
        "secret",
        "password",
        "token",
        "auth",
        "authorization",
        "access_token",
        "refresh_token",
        "client_secret",
    }
)


def redact_url(url: str) -> str:
    """Redact sensitive query parameters from a URL.

    Args:
        url: URL string that may contain API keys in query params.

    Returns:
        URL with sensitive parameter values replaced by '***'.

    Example:
        >>> redact_url("https://api.census.gov/data?key=SECRET&get=NAME")
        'https://api.census.gov/data?key=***&get=NAME'
    """
    from urllib.parse import parse_qs, urlencode, urlparse, urlunparse

    parsed = urlparse(url)
    if not parsed.query:
        return url

    params = parse_qs(parsed.query, keep_blank_values=True)
    redacted: dict[str, list[str]] = {}

    for key, values in params.items():
        if key.lower() in _SENSITIVE_KEYS:
            redacted[key] = ["***"] * len(values)
        else:
            redacted[key] = values

    # Reconstruct query string (doseq=True handles lists, safe='*' preserves asterisks)
    new_query = urlencode(redacted, doseq=True, safe="*")
    return urlunparse(parsed._replace(query=new_query))


def redact_params(params: dict[str, Any]) -> dict[str, Any]:
    """Redact sensitive values from a parameters dictionary.

    Args:
        params: Dictionary of parameters that may contain API keys.

    Returns:
        Dictionary with sensitive values replaced by '***'.

    Example:
        >>> redact_params({"key": "SECRET", "state": "06"})
        {'key': '***', 'state': '06'}
    """
    redacted: dict[str, Any] = {}
    for key, value in params.items():
        if key.lower() in _SENSITIVE_KEYS:
            redacted[key] = "***"
        elif isinstance(value, dict):
            redacted[key] = redact_params(value)
        else:
            redacted[key] = value
    return redacted


# =============================================================================
# Exports
# =============================================================================

__all__ = [
    "TRACE",
    "JSONFormatter",
    "ContextAwareFilter",
    "get_log_context",
    "set_log_context",
    "clear_log_context",
    "log_context_scope",
    "redact_url",
    "redact_params",
]
