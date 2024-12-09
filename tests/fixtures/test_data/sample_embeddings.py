import numpy as np

EMBEDDING_DIMENSION = 384
BASE_EMBEDDING = np.random.rand(EMBEDDING_DIMENSION)
BATCH_SIZE = 10
TOTAL_ENTITIES = 50

# tests/integration/chromadb/test_crud.py
import pytest

class TestChromaDBCrud:
    """Test basic CRUD operations for ChromaDB."""
    
    def test_create_entity(self, entity_registry):
        """Test entity creation and verification."""
        entity = entity_registry.create_entity(
            type="TestType", role="TestRole"
        )
        
        assert entity.id in entity_registry._entities, (
            "Entity should exist in registry"
        )
        
        results = entity_registry.collection.get(ids=[entity.id])
        assert len(results["ids"]) == 1, "Entity should exist in ChromaDB"