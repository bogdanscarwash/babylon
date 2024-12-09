from typing import Any
import numpy as np
import pytest
import os
import chromadb
from chromadb.api import Collection
from babylon.config.chromadb_config import ChromaDBConfig
from babylon.data.entity_registry import EntityRegistry

# Test constants
EMBEDDING_DIMENSION: int = 384
TEST_EMBEDDING: np.ndarray = np.random.rand(EMBEDDING_DIMENSION)
TEST_EMBEDDING = TEST_EMBEDDING / np.linalg.norm(TEST_EMBEDDING)

class TestChromaDBPersistence:
    """Test ChromaDB persistence functionality."""
    
    def test_data_persistence(
        self,
        entity_registry: EntityRegistry,
        test_dir: str
    ) -> None:
        """Test that entity data persists between client restarts.
        
        Args:
            entity_registry: The entity registry fixture
            test_dir: Temporary directory for test data
            
        Raises:
            AssertionError: If data persistence verification fails
        """
        # Create test entity
        entity = entity_registry.create_entity(
            type="TestType",
            role="TestRole"
        )
        
        # Add embedding
        entity_registry.collection.add(
            ids=[entity.id],
            embeddings=[TEST_EMBEDDING.tolist()],
            metadatas=[{"type": "TestType", "role": "TestRole"}]
        )
        
        # Get the entity ID for later verification
        entity_id: str = entity.id
        
        # Get the persist_directory from the current client
        persist_directory: str = str(entity_registry.collection._client._settings.persist_directory)

        # Simulate client restart without deleting data
        settings = ChromaDBConfig.get_settings(
            persist_directory=persist_directory,
            allow_reset=True,
            anonymized_telemetry=False,
            is_persistent=True
        )
        new_client: chromadb.PersistentClient = chromadb.PersistentClient(settings=settings)

        # Create or get the collection with the new client
        collection: Collection = new_client.get_or_create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA
        )

        # Update the entity_registry's collection reference
        entity_registry.collection = collection
        
        # Verify entity data can be retrieved
        results: dict[str, Any] = entity_registry.collection.get(
            ids=[entity_id]
        )
        
        assert len(results['ids']) == 1, "Entity should be retrieved after client restart"
        assert results['ids'][0] == entity_id, "Retrieved entity should have correct ID"
        assert results['metadatas'][0]['type'] == "TestType", "Entity metadata should persist"
        assert results['metadatas'][0]['role'] == "TestRole", "Entity metadata should persist"
        
        # Cleanup
        new_client.reset()
