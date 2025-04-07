"""OpenAI configuration module providing API settings.

This module defines the OpenAIConfig class which manages OpenAI API settings
and provides utilities for rate limiting and retry behavior.
"""

import os
from dataclasses import dataclass
from typing import ClassVar, Optional
from .base import BaseConfig


@dataclass
class OpenAIConfig(BaseConfig):
    """Configuration for OpenAI API integration.

    This class manages OpenAI API settings including authentication,
    model selection, rate limits, and retry behavior.

    Class Attributes:
        API_KEY (ClassVar[str]): OpenAI API key
        ORGANIZATION_ID (ClassVar[Optional[str]]): Optional OpenAI organization ID
        EMBEDDING_MODEL (ClassVar[str]): Model to use for embeddings
        MAX_RETRIES (ClassVar[int]): Maximum number of retry attempts
        RETRY_DELAY (ClassVar[float]): Base delay between retries in seconds
        RATE_LIMIT_RPM (ClassVar[int]): Rate limit in requests per minute
        BATCH_SIZE (ClassVar[int]): Maximum batch size for embedding requests
        REQUEST_TIMEOUT (ClassVar[float]): Request timeout in seconds
    """

    # API Authentication
    API_KEY: ClassVar[str] = os.getenv("OPENAI_API_KEY", "")
    ORGANIZATION_ID: ClassVar[Optional[str]] = os.getenv("OPENAI_ORGANIZATION_ID")

    # Model Settings
    EMBEDDING_MODEL: ClassVar[str] = os.getenv(
        "OPENAI_EMBEDDING_MODEL", "text-embedding-ada-002"
    )

    # Retry Settings
    MAX_RETRIES: ClassVar[int] = int(os.getenv("OPENAI_MAX_RETRIES", "3"))
    RETRY_DELAY: ClassVar[float] = float(os.getenv("OPENAI_RETRY_DELAY", "1.0"))

    # Rate Limiting
    RATE_LIMIT_RPM: ClassVar[int] = int(os.getenv("OPENAI_RATE_LIMIT_RPM", "60"))

    # Request Settings
    BATCH_SIZE: ClassVar[int] = int(os.getenv("OPENAI_BATCH_SIZE", "8"))
    REQUEST_TIMEOUT: ClassVar[float] = float(
        os.getenv("OPENAI_REQUEST_TIMEOUT", "10.0")
    )

    @classmethod
    def validate(cls) -> None:
        """Validate the configuration.

        Raises:
            ValueError: If required settings are missing or invalid
        """
        if not cls.API_KEY:
            raise ValueError("OpenAI API key is required")

        if cls.MAX_RETRIES < 0:
            raise ValueError("MAX_RETRIES must be non-negative")

        if cls.RETRY_DELAY <= 0:
            raise ValueError("RETRY_DELAY must be positive")

        if cls.RATE_LIMIT_RPM <= 0:
            raise ValueError("RATE_LIMIT_RPM must be positive")

        if cls.BATCH_SIZE <= 0:
            raise ValueError("BATCH_SIZE must be positive")

        if cls.REQUEST_TIMEOUT <= 0:
            raise ValueError("REQUEST_TIMEOUT must be positive")

    @classmethod
    def get_headers(cls) -> dict:
        """Get HTTP headers for OpenAI API requests.

        Returns:
            dict: Headers including authentication and organization ID if set
        """
        headers = {
            "Authorization": f"Bearer {cls.API_KEY}",
            "Content-Type": "application/json",
        }
        if cls.ORGANIZATION_ID:
            headers["OpenAI-Organization"] = cls.ORGANIZATION_ID
        return headers

    @classmethod
    def get_model_dimensions(cls) -> int:
        """Get embedding dimensions for the configured model.

        Returns:
            int: Number of dimensions in the embedding vector
        """
        # Model-specific dimensions
        dimensions = {
            "text-embedding-ada-002": 1536,
            # Add other models as they become available
        }
        return dimensions.get(cls.EMBEDDING_MODEL, 1536)
