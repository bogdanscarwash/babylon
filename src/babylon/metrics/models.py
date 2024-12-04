from datetime import datetime
from typing import Optional
from sqlalchemy import Column, Integer, String, Float, DateTime, Index
from sqlalchemy.orm import Mapped, mapped_column
from ..data.database import Base

class Metric(Base):
    """Database model for storing performance metrics.
    
    This model stores time-series metrics data with context information
    for analysis and monitoring of system performance.
    
    Attributes:
        id: Unique identifier for the metric
        name: Name/type of the metric being recorded
        value: Numerical value of the metric
        timestamp: When the metric was recorded
        context: Additional context about the metric (e.g., "cache", "database")
    """
    __tablename__ = 'metrics'
    __table_args__ = (
        Index('idx_metrics_name_timestamp', 'name', 'timestamp'),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(100), index=True, nullable=False)
    value: Mapped[float] = mapped_column(Float, nullable=False)
    timestamp: Mapped[datetime] = mapped_column(
        DateTime, 
        default=datetime.utcnow,
        nullable=False
    )
    context: Mapped[Optional[str]] = mapped_column(String(200))
