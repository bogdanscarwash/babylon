from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, declarative_base
from config.base import BaseConfig as Config

engine = create_engine(Config.DATABASE_URL)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()
