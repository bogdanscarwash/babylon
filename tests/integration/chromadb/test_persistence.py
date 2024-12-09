import numpy as np
import pytest
import os

# Test constants
EMBEDDING_DIMENSION = 384
TEST_EMBEDDING = np.random.rand(EMBEDDING_DIMENSION)
TEST_EMBEDDING = TEST_EMBEDDING / np.linalg.norm(TEST_EMBEDDING)

class TestChromaDBPersistence:
    """Test ChromaDB persistence functionality."""
    
    def test_data_persistence(self, entity_registry, test_dir):
        """Test that entity data persists between client restarts."""
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
        entity_id = entity.id
        
        # Force client recreation to test persistence
        entity_registry.collection._client._cleanup()
        
        # Create new registry with same persistence directory
        from chromadb import PersistentClient
        from babylon.config.chromadb_config import ChromaDBConfig
        
        settings = ChromaDBConfig.get_settings(
            persist_directory=test_dir,
            allow_reset=True,
            anonymized_telemetry=False,
            is_persistent=True
        )
        new_client = PersistentClient(settings=settings)
        new_collection = new_client.get_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME
        )
        
        # Verify entity data persisted
        results = new_collection.get(
            ids=[entity_id]
        )
        
        assert len(results['ids']) == 1, "Entity should be retrieved after client restart"
        assert results['ids'][0] == entity_id, "Retrieved entity should have correct ID"
        assert results['metadatas'][0]['type'] == "TestType", "Entity metadata should persist"
        assert results['metadatas'][0]['role'] == "TestRole", "Entity metadata should persist"
        
        # Cleanup
        new_client.reset()
