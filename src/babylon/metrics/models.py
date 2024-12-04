from sqlalchemy import Column, Integer, String, Float, DateTime
from ..data.database import Base
from datetime import datetime

class Metric(Base):
    __tablename__ = 'metrics'

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True)
    value = Column(Float)
    timestamp = Column(DateTime, default=datetime.utcnow)
    context = Column(String)
