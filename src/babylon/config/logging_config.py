"""Logging configuration for the Babylon application.

Provides centralized logging configuration with:
- JSON structured logging
- Correlation ID tracking
- Performance metrics
- Error tracking
- User analytics
"""

import logging
import logging.handlers
import os
from typing import Optional, Dict, Any
from datetime import datetime
import uuid
from pythonjsonlogger import jsonlogger
import traceback

class CorrelationIDFilter(logging.Filter):
    """Adds correlation ID to all log records."""
    
    def __init__(self):
        super().__init__()
        self._correlation_id: Optional[str] = None
        
    @property
    def correlation_id(self) -> str:
        """Get current correlation ID, creating if needed."""
        if self._correlation_id is None:
            self._correlation_id = str(uuid.uuid4())
        return self._correlation_id
        
    @correlation_id.setter
    def correlation_id(self, value: str) -> None:
        """Set correlation ID."""
        self._correlation_id = value
        
    def filter(self, record: logging.LogRecord) -> bool:
        """Add correlation ID to log record."""
        record.correlation_id = self.correlation_id
        return True

class ErrorContextFilter(logging.Filter):
    """Adds error context information to log records."""

    def filter(self, record: logging.LogRecord) -> bool:
        if record.exc_info:
            record.error_details = {
                "type": record.exc_info[0].__name__,
                "message": str(record.exc_info[1]),
                "traceback": traceback.format_exception(*record.exc_info)
            }
        return True

def setup_logging() -> None:
    """Configure logging for the application."""
    
    # Create logs directory if needed
    log_dir = "logs"
    os.makedirs(log_dir, exist_ok=True)
    
    # Create JSON formatters
    class CustomJsonFormatter(jsonlogger.JsonFormatter):
        def add_fields(self, log_record: Dict[str, Any], record: logging.LogRecord, message_dict: Dict[str, Any]) -> None:
            super().add_fields(log_record, record, message_dict)
            log_record['timestamp'] = datetime.utcnow().isoformat()
            log_record['level'] = record.levelname
            log_record['correlation_id'] = getattr(record, 'correlation_id', None)
            
            # Add error context if present
            if hasattr(record, 'error_details'):
                log_record['error'] = record.error_details
                
            # Add metrics if present
            if hasattr(record, 'metrics'):
                log_record['metrics'] = record.metrics

    # Configure formatters
    json_formatter = CustomJsonFormatter(
        '%(timestamp)s %(level)s %(name)s %(correlation_id)s %(message)s'
    )
    
    # Configure handlers
    console_handler = logging.StreamHandler()
    console_handler.setFormatter(json_formatter)
    console_handler.setLevel(logging.INFO)
    
    # Main log file
    main_log = os.path.join(log_dir, f"babylon_{datetime.now():%Y%m%d}.log")
    file_handler = logging.handlers.RotatingFileHandler(
        main_log,
        maxBytes=10485760,  # 10MB
        backupCount=5
    )
    file_handler.setFormatter(json_formatter)
    file_handler.setLevel(logging.DEBUG)
    
    # Error log file
    error_log = os.path.join(log_dir, f"babylon_errors_{datetime.now():%Y%m%d}.log")
    error_handler = logging.handlers.RotatingFileHandler(
        error_log,
        maxBytes=10485760,
        backupCount=5
    )
    error_handler.setFormatter(json_formatter)
    error_handler.setLevel(logging.ERROR)
    
    # Metrics log file
    metrics_log = os.path.join(log_dir, f"babylon_metrics_{datetime.now():%Y%m%d}.log")
    metrics_handler = logging.handlers.RotatingFileHandler(
        metrics_log,
        maxBytes=10485760,
        backupCount=5
    )
    metrics_handler.setFormatter(json_formatter)
    metrics_handler.setLevel(logging.INFO)
    
    # Configure root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)
    
    # Add filters
    correlation_filter = CorrelationIDFilter()
    error_filter = ErrorContextFilter()
    root_logger.addFilter(correlation_filter)
    root_logger.addFilter(error_filter)
    
    # Add handlers
    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)
    root_logger.addHandler(error_handler)
    root_logger.addHandler(metrics_handler)
    
    # Configure specific loggers
    loggers = {
        'chromadb': logging.INFO,
        'babylon.data': logging.DEBUG,
        'babylon.entities': logging.DEBUG,
        'babylon.systems': logging.DEBUG,
        'babylon.metrics': logging.INFO,
        'babylon.ai': logging.DEBUG
    }
    
    for logger_name, level in loggers.items():
        logger = logging.getLogger(logger_name)
        logger.setLevel(level)
