import os
import shutil
import tempfile
import time
import unittest
import unittest
from concurrent.futures import ThreadPoolExecutor

import chromadb
import numpy as np

from babylon.config.chromadb_config import ChromaDBConfig
from babylon.data.entity_registry import EntityRegistry
from babylon.utils.backup import backup_chroma, restore_chroma
from tests.mocks import MockMetricsCollector


class TestChromaDBIntegration(unittest.TestCase):
    def setUp(self):
        """Set up test environment with temporary ChromaDB instance."""
        # Create temporary test directory with proper permissions
        self.temp_dir = tempfile.mkdtemp()
        os.chmod(self.temp_dir, 0o755)

        # Initialize ChromaDB client with new architecture
        self.client = chromadb.PersistentClient(path=self.temp_dir)

        # Create test collection
        self.collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA,
        )

        # Initialize entity registry with mock metrics collector
        self.entity_registry = EntityRegistry(self.collection)
        self.entity_registry.metrics = MockMetricsCollector()

    def tearDown(self):
        # Clean up temporary directories
        shutil.rmtree(self.temp_dir)

    def test_add_entity(self):
        try:
            # Create a test entity
            entity = self.entity_registry.create_entity(
                type="TestType", role="TestRole"
            )
            entity = self.entity_registry.create_entity(
                type="TestType", role="TestRole"
            )

            # Verify the entity is in the registry
            try:
                self.assertIn(entity.id, self.entity_registry._entities)
            except KeyError as e:
                self.fail(f"Entity not found in registry: {e!s}")
                self.fail(f"Entity not found in registry: {e!s}")

            # Verify the entity is added to ChromaDB
            try:
                results = self.collection.get(ids=[entity.id])
                self.assertEqual(len(results["ids"]), 1)
                self.assertEqual(results["ids"][0], entity.id)
                self.assertEqual(len(results["ids"]), 1)
                self.assertEqual(results["ids"][0], entity.id)
            except Exception as e:
                self.fail(f"Failed to retrieve entity from ChromaDB: {e!s}")

                self.fail(f"Failed to retrieve entity from ChromaDB: {e!s}")

        except Exception as e:
            self.fail(f"Failed to create entity: {e!s}")
            self.fail(f"Failed to create entity: {e!s}")

    def test_update_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")

        # Update the entity's attributes
        self.entity_registry.update_entity(entity.id, freedom=0.5, wealth=0.8)

        # Verify updates in the registry
        updated_entity = self.entity_registry.get_entity(entity.id)
        self.assertEqual(updated_entity.freedom, 0.5)
        self.assertEqual(updated_entity.wealth, 0.8)

        # Verify updates in ChromaDB
        results = self.collection.get(ids=[entity.id], include=["metadatas"])
        metadata = results["metadatas"][0]
        self.assertEqual(metadata["freedom"], 0.5)
        self.assertEqual(metadata["wealth"], 0.8)
        results = self.collection.get(ids=[entity.id], include=["metadatas"])
        metadata = results["metadatas"][0]
        self.assertEqual(metadata["freedom"], 0.5)
        self.assertEqual(metadata["wealth"], 0.8)

    def test_delete_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")

        # Delete the entity
        self.entity_registry.delete_entity(entity.id)

        # Verify the entity is removed from the registry
        self.assertNotIn(entity.id, self.entity_registry._entities)

        # Verify the entity is deleted from ChromaDB
        results = self.collection.get(ids=[entity.id])
        self.assertEqual(len(results["ids"]), 0)

    def test_error_handling(self):
        """Test error handling for invalid operations"""
        # Test invalid ID
        with self.assertRaises(ValueError):
            self.entity_registry.get_entity("")

        # Test duplicate ID handling (ChromaDB logs warning instead of raising error)
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")
        # Add same entity again - should not raise error but log warning
        self.collection.add(
            embeddings=[[1.0] * 384],
            ids=[entity.id],
            metadatas=[{"type": "TestType", "role": "TestRole"}]
        )
        # Verify entity still exists and is unchanged
        results = self.collection.get(ids=[entity.id])
        self.assertEqual(len(results["ids"]), 1)

    def test_concurrent_operations(self):
        """Test concurrent entity operations"""


        def create_entity():
            return self.entity_registry.create_entity(type="TestType", role="TestRole")
            return self.entity_registry.create_entity(type="TestType", role="TestRole")

        with ThreadPoolExecutor(max_workers=5) as executor:
            futures = [executor.submit(create_entity) for _ in range(10)]
            entities = [f.result() for f in futures]

        # Verify all entities were created
        self.assertEqual(len(entities), 10)
        for entity in entities:
            self.assertIsNotNone(self.entity_registry.get_entity(entity.id))

    def test_large_dataset(self):
        """Test performance with larger dataset"""
        start_time = time.time()
        batch_size = 100
        total_entities = 1000

        # Create a base embedding that we'll modify slightly for each entity
        # This ensures we have similar but not identical embeddings
        base_embedding = np.random.rand(384)
        
        # Create entities in batches with similar embeddings
        for i in range(0, total_entities, batch_size):
            entities_batch = []
            embeddings_batch = []
            metadatas_batch = []
            
            for j in range(batch_size):
                entity = self.entity_registry.create_entity(
                    type="TestType", role="TestRole"
                )
                entities_batch.append(entity)
                
                # Create a slightly modified version of the base embedding
                noise = np.random.normal(0, 0.1, 384)  # Small random variations
                embedding = base_embedding + noise
                embedding = embedding / np.linalg.norm(embedding)  # Normalize
                embeddings_batch.append(embedding.tolist())
                
                metadatas_batch.append({"type": "TestType", "role": "TestRole"})

            # Add embeddings to ChromaDB
            self.collection.add(
                ids=[e.id for e in entities_batch],
                embeddings=embeddings_batch,
                metadatas=metadatas_batch
            )

            # Batch verify
            ids = [e.id for e in entities_batch]
            results = self.collection.get(ids=ids)
            self.assertEqual(len(results["ids"]), batch_size)

        # Test similarity search with a query vector similar to base_embedding
        query_embedding = (base_embedding + np.random.normal(0, 0.1, 384))
        query_embedding = query_embedding / np.linalg.norm(query_embedding)
        similar = self.collection.query(
            query_embeddings=[query_embedding.tolist()],
            n_results=5
        )
        self.assertEqual(len(similar["ids"]), 5)

    def test_persistence_across_restarts(self):
        """Test data persistence across application restarts"""
        # Add entities to ChromaDB
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")

        # Close and reopen client to simulate restart
        collection_name = self.collection.name
        self.client = chromadb.PersistentClient(path=self.temp_dir)
        collection = self.client.get_collection(name=collection_name)

        # Verify entities are still present
        results = collection.get(ids=[entity.id])
        self.assertEqual(len(results["ids"]), 1)

if __name__ == "__main__":
    unittest.main()
