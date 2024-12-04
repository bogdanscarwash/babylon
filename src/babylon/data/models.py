from sqlalchemy import Column, Integer, String, DateTime
from .database import Base
from datetime import datetime

class LogEntry(Base):
    __tablename__ = 'logs'

    id = Column(Integer, primary_key=True, index=True)
    level = Column(String)
    timestamp = Column(DateTime, default=datetime.utcnow)
    message = Column(String)
    module = Column(String)
    correlation_id = Column(String, nullable=True)
