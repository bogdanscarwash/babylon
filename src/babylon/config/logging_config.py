"""Logging configuration for the Babylon application.

Provides centralized logging configuration with:
- Structured logging format
- Correlation ID tracking
- Consistent log levels
- ChromaDB specific handlers
"""

import logging
import logging.handlers
import os
from typing import Optional
from datetime import datetime
import uuid

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

def setup_logging() -> None:
    """Configure logging for the application."""
    
    # Create logs directory if needed
    log_dir = "logs"
    os.makedirs(log_dir, exist_ok=True)
    
    # Create formatters
    console_formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - [%(correlation_id)s] %(message)s'
    )
    file_formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - [%(correlation_id)s] - %(pathname)s:%(lineno)d - %(message)s'
    )
    
    # Configure console handler
    console_handler = logging.StreamHandler()
    console_handler.setFormatter(console_formatter)
    console_handler.setLevel(logging.INFO)
    
    # Configure file handler
    log_file = os.path.join(log_dir, f"babylon_{datetime.now():%Y%m%d}.log")
    file_handler = logging.handlers.RotatingFileHandler(
        log_file,
        maxBytes=10485760,  # 10MB
        backupCount=5
    )
    file_handler.setFormatter(file_formatter)
    file_handler.setLevel(logging.DEBUG)
    
    # Configure root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)
    
    # Add correlation ID filter
    correlation_filter = CorrelationIDFilter()
    root_logger.addFilter(correlation_filter)
    
    # Add handlers
    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)
    
    # Configure ChromaDB specific logger
    chroma_logger = logging.getLogger('chromadb')
    chroma_logger.setLevel(logging.INFO)
    
    # Configure specific module loggers
    for module in ['babylon.data', 'babylon.entities', 'babylon.systems']:
        module_logger = logging.getLogger(module)
        module_logger.setLevel(logging.DEBUG)
