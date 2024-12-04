"""ChromaDB configuration module.

Provides centralized configuration for ChromaDB settings including:
- Client initialization parameters
- Persistence settings
- Reset behavior
- Collection defaults
"""

import os
from pathlib import Path
from chromadb.config import Settings

class ChromaDBConfig:
    """ChromaDB configuration settings.
    
    Centralizes all ChromaDB-related configuration to ensure consistent
    settings across the application. This includes client initialization,
    persistence configuration, and default collection settings.
    """
    
    # Base directory for ChromaDB persistence
    BASE_DIR = Path(os.getenv('CHROMADB_DIR', 'data/chromadb'))
    
    # Default settings for ChromaDB client
    DEFAULT_SETTINGS = Settings(
        allow_reset=True,  # Enable reset for testing
        anonymized_telemetry=False,  # Disable telemetry
        persist_directory=str(BASE_DIR),
        is_persistent=True,  # Can be overridden for testing
        chroma_db_impl="duckdb+parquet"  # Default persistence implementation
    )
    
    # Collection settings
    DEFAULT_COLLECTION_NAME = "entities"
    DEFAULT_EMBEDDING_FUNCTION = None  # Will be set by entity manager
    DEFAULT_METADATA = {"source": "babylon"}
    
    # Backup settings
    BACKUP_DIR = BASE_DIR / "backups"
    MAX_BACKUPS = 5
    
    @classmethod
    def get_settings(cls, **overrides) -> Settings:
        """Get ChromaDB settings with optional overrides.
        
        Args:
            **overrides: Override any default settings
            
        Returns:
            Settings: ChromaDB settings instance
        """
        settings_dict = cls.DEFAULT_SETTINGS.dict()
        settings_dict.update(overrides)
        return Settings(**settings_dict)
    
    @classmethod
    def ensure_directories(cls) -> None:
        """Ensure required directories exist."""
        cls.BASE_DIR.mkdir(parents=True, exist_ok=True)
        cls.BACKUP_DIR.mkdir(parents=True, exist_ok=True)
