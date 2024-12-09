"""ChromaDB configuration module.

Provides centralized configuration for ChromaDB settings including:
- Client initialization parameters
- Persistence settings
- Reset behavior
- Collection defaults
"""

import os
from pathlib import Path

import chromadb


class ChromaDBConfig:
    """ChromaDB configuration settings."""

    # Base directory for ChromaDB persistence
    BASE_DIR = Path(os.getenv("CHROMADB_DIR", "data/chromadb"))

    # Collection settings
    DEFAULT_COLLECTION_NAME = "entities"
    DEFAULT_METADATA = {"source": "babylon"}
    CHROMADB_PERSIST_DIR = ""


    # Backup settings
    BACKUP_DIR = BASE_DIR / "backups"
    MAX_BACKUPS = 5


    @classmethod
    def get_settings(cls, **overrides) -> chromadb.Settings:
        """Get ChromaDB settings with optional overrides.


        Args:
            **overrides: Override any default settings


        Returns:
            chromadb.Settings: ChromaDB settings instance
        """
        settings_dict = {
            "allow_reset": True,
            "anonymized_telemetry": False,
            "is_persistent": True,
        }
        settings_dict.update(overrides)
        return chromadb.Settings(**settings_dict)

    @classmethod
    def ensure_directories(cls) -> None:
        """Ensure required directories exist."""
        cls.BASE_DIR.mkdir(parents=True, exist_ok=True)
        cls.BACKUP_DIR.mkdir(parents=True, exist_ok=True)
