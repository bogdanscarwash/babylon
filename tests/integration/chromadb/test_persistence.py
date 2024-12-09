import numpy as np
import pytest
import os

# Test constants
import chromadb

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
        
        # Simulate client restart to test data persistence
        collection_name = entity_registry.collection.name

        # Close the current client
        entity_registry.collection._client.reset()

        # Re-initialize the client with the same settings
        new_client = chromadb.Client(
            chromadb.config.Settings(
                persist_directory=entity_registry.collection._client._settings.persist_directory
            )
        )

        # Update the entity_registry's collection reference
        entity_registry.collection = new_client.get_collection(
            name=collection_name
        )
        
        # Verify entity data can be retrieved
        results = entity_registry.collection.get(
            ids=[entity_id]
        )
        
        assert len(results['ids']) == 1, "Entity should be retrieved after client restart"
        assert results['ids'][0] == entity_id, "Retrieved entity should have correct ID"
        assert results['metadatas'][0]['type'] == "TestType", "Entity metadata should persist"
        assert results['metadatas'][0]['role'] == "TestRole", "Entity metadata should persist"
        
        # Cleanup
        entity_registry.collection._client.reset()
