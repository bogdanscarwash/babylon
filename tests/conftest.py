import pytest
import tempfile
import os
import shutil
import time
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from babylon.data.database import Base
from babylon.config.testing import TestingConfig
from babylon.metrics.collector import MetricsCollector
# Import all models to ensure they're registered with SQLAlchemy
from babylon.metrics.models import Metric
from babylon.data.models.contradiction import *
from babylon.data.models.event import *
from babylon.data.models.trigger import *

@pytest.fixture(scope="session")
def test_dir():
    """Create a temporary directory for all tests."""
    temp_dir = tempfile.mkdtemp()
    os.chmod(temp_dir, 0o755)
    yield temp_dir
    time.sleep(0.1)  # Allow OS to release handles
    shutil.rmtree(temp_dir, ignore_errors=True)

@pytest.fixture(scope="session")
def test_db():
    """Create a test database."""
    # Create test engine with SQLite in-memory database
    engine = create_engine(TestingConfig.DATABASE_URL)
    
    # Create all tables
    Base.metadata.create_all(bind=engine)
    
    # Create session factory
    TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    
    # Override the SessionLocal in the database module
    import babylon.data.database
    babylon.data.database.SessionLocal = TestingSessionLocal
    
    return engine

@pytest.fixture(scope="function")
def metrics_collector(test_db):
    """Create a fresh metrics collector for each test."""
    return MetricsCollector()
