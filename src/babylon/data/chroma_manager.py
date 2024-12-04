from typing import Optional, Any
import chromadb
from chromadb.config import Settings
import logging
from config.base import BaseConfig as Config

logger = logging.getLogger(__name__)

class ChromaManager:
    """Manages ChromaDB client lifecycle and operations."""
    
    _instance: Optional['ChromaManager'] = None
    _client: Optional[chromadb.Client] = None
    
    def __new__(cls) -> 'ChromaManager':
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if self._client is None:
            self._initialize_client()
    
    def _initialize_client(self) -> None:
        """Initialize the ChromaDB client with proper settings."""
        try:
            self._client = chromadb.Client(Settings(
                chroma_db_impl="duckdb+parquet",
                persist_directory=Config.CHROMADB_PERSIST_DIR
            ))
        except Exception as e:
            logger.error(f"Failed to initialize ChromaDB client: {e}")
            raise
    
    @property
    def client(self) -> chromadb.Client:
        """Get the ChromaDB client instance."""
        if self._client is None:
            self._initialize_client()
        return self._client
    
    def get_or_create_collection(self, name: str) -> Any:
        """Get or create a ChromaDB collection.
        
        Args:
            name: Name of the collection
            
        Returns:
            The ChromaDB collection
        """
        return self.client.get_or_create_collection(name=name)
    
    def cleanup(self) -> None:
        """Cleanup ChromaDB resources."""
        if self._client:
            try:
                self._client.persist()
                self._client.reset()
                self._client = None
            except Exception as e:
                logger.error(f"Error during ChromaDB cleanup: {e}")
