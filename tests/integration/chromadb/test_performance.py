import numpy as np
from tests.fixtures.test_data.sample_embeddings import (
    EMBEDDING_DIMENSION, BASE_EMBEDDING, BATCH_SIZE, TOTAL_ENTITIES
)

class TestChromaDBPerformance:
    """Test ChromaDB performance characteristics."""
    
    def test_large_dataset(self, entity_registry):
        """Test performance with a larger dataset."""
        entities = []
        
        for i in range(0, TOTAL_ENTITIES, BATCH_SIZE):
            batch = self._create_entity_batch(
                entity_registry, 
                BASE_EMBEDDING, 
                BATCH_SIZE
            )
            entities.extend(batch)
            
        self._verify_similarity_search(
            entity_registry.collection, 
            BASE_EMBEDDING
        )
    
    def _create_entity_batch(self, registry, base_embedding, size):
        """Create a batch of entities with similar embeddings."""
        entities = []
        embeddings = []
        metadatas = []
        
        for _ in range(size):
            entity = registry.create_entity(
                type="TestType", 
                role="TestRole"
            )
            noise = np.random.normal(0, 0.1, EMBEDDING_DIMENSION)
            embedding = base_embedding + noise
            embedding = embedding / np.linalg.norm(embedding)
            
            entities.append(entity)
            embeddings.append(embedding.tolist())
            metadatas.append({"type": "TestType", "role": "TestRole"})
            
        registry.collection.add(
            ids=[e.id for e in entities],
            embeddings=embeddings,
            metadatas=metadatas
        )
        
        return entities

    def _verify_similarity_search(self, collection, query_embedding):
        """Verify similarity search results."""
        results = collection.query(
            query_embeddings=[query_embedding.tolist()],
            n_results=10
        )
        
        assert len(results['ids'][0]) == 10, "Expected 10 results from similarity search"
        assert all(isinstance(id, str) for id in results['ids'][0]), "All IDs should be strings"
        assert all(isinstance(d, dict) for d in results['metadatas'][0]), "All metadata should be dicts"
