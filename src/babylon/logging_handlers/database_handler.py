import logging
from sqlalchemy.orm import Session
from datetime import datetime
from ..data.database import SessionLocal
from ..data.models import LogEntry

class DatabaseHandler(logging.Handler):
    def __init__(self):
        super().__init__()
        self.db: Session = SessionLocal()

    def emit(self, record):
        log_entry = LogEntry(
            level=record.levelname,
            timestamp=datetime.utcnow(),
            message=record.getMessage(),
            module=record.module,
            correlation_id=getattr(record, 'correlation_id', None)
        )
        self.db.add(log_entry)
        self.db.commit()
