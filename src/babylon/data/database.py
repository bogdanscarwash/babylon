"""Database configuration and session management.

Provides SQLAlchemy engine and session configuration for the application.
See CONFIGURATION.md for detailed documentation of database settings.
"""

from typing import Any

from sqlalchemy import Engine, create_engine
from sqlalchemy.orm import Session, declarative_base, sessionmaker

from babylon.config.base import BaseConfig as Config

# Initialize database engine with connection pooling
engine_args = {}
if 'sqlite' not in Config.DATABASE_URL:
    # Only add these parameters for non-SQLite databases
    engine_args.update({
        'pool_size': Config.DB_POOL_SIZE,
        'max_overflow': Config.DB_MAX_OVERFLOW,
    })

engine: Engine = create_engine(Config.DATABASE_URL, **engine_args)

# Configure session factory with explicit typing
SessionLocal: Any = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Base class for declarative models
Base = declarative_base()


def get_db() -> Session:
    """Get a database session from the pool.

    Returns:
        Session: SQLAlchemy database session
    """
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
