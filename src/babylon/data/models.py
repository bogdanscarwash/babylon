from datetime import datetime

from sqlalchemy import Column, DateTime, Integer, String

from .database import Base


class LogEntry(Base):
    __tablename__ = "logs"

    id = Column(Integer, primary_key=True, index=True)
    level = Column(String)
    timestamp = Column(DateTime, default=datetime.utcnow)
    message = Column(String)
    module = Column(String)
    correlation_id = Column(String, nullable=True)
