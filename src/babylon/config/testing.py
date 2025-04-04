from babylon.config.base import BaseConfig


class TestingConfig(BaseConfig):
    """Testing configuration."""

    DEBUG = True
    TESTING = True
    DATABASE_URL = "sqlite:///:memory:"  # In-memory database for testing
    # Add testing-specific configurations...
