"""Base configuration module providing core settings.

This module defines the BaseConfig class which serves as the foundation for
environment-specific configurations. It loads settings from environment variables
with sensible defaults.
"""

import os
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

class BaseConfig:
    """Base configuration class providing default settings.
    
    This class defines the base configuration settings used across all environments.
    It should be subclassed by environment-specific configurations that can
    override these values as needed.
    
    Attributes:
        SECRET_KEY (str): Secret key for security features
        DATABASE_URL (str): Database connection string
        DEBUG (bool): Debug mode flag
        TESTING (bool): Testing mode flag
        CHROMADB_PERSIST_DIR (str): Directory for ChromaDB persistence
    """

    # Security settings
    SECRET_KEY = os.getenv('SECRET_KEY', 'default-secret-key')
    
    # Database settings
    DATABASE_URL = os.getenv('DATABASE_URL', 'sqlite:///default.db')
    
    # Application mode flags
    DEBUG = False
    TESTING = False
    
    # ChromaDB settings
    CHROMADB_PERSIST_DIR = os.getenv('CHROMADB_PERSIST_DIR', './chromadb')
