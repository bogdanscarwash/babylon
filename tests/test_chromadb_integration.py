import os
import shutil
import tempfile
import time
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

        # Initialize ChromaDB client with settings that allow reset
        self.settings = ChromaDBConfig.get_settings(
            persist_directory=self.temp_dir,
            allow_reset=True,
            anonymized_telemetry=False,
            is_persistent=True  # Ensure persistence is enabled
        )
        self.client = chromadb.PersistentClient(settings=self.settings)

        # Create test collection
        self.collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA,
        )

        # Initialize entity registry with mock metrics collector
        self.entity_registry = EntityRegistry(self.collection)
        self.entity_registry.metrics = MockMetricsCollector()

    def tearDown(self):
        """Clean up temporary directories."""
        try:
            if hasattr(self, 'client'):
                self.client.reset()
            time.sleep(0.1)  # Give OS time to release file handles
            if hasattr(self, 'temp_dir') and os.path.exists(self.temp_dir):
                shutil.rmtree(self.temp_dir, ignore_errors=True)
        except Exception as e:
            print(f"Cleanup error: {e}")

    def test_add_entity(self):
        try:
            # Create a test entity
            entity = self.entity_registry.create_entity(
                type="TestType", role="TestRole"
            )

            # Verify the entity is in the registry
            try:
                self.assertIn(entity.id, self.entity_registry._entities)
            except KeyError as e:
                self.fail(f"Entity not found in registry: {e!s}")

            # Verify the entity is added to ChromaDB
            try:
                results = self.collection.get(ids=[entity.id])
                self.assertEqual(len(results["ids"]), 1)
                self.assertEqual(results["ids"][0], entity.id)
            except Exception as e:
                self.fail(f"Failed to retrieve entity from ChromaDB: {e!s}")

        except Exception as e:
            self.fail(f"Failed to create entity: {e!s}")

    def test_update_entity(self):
        # Create a test entity
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

    def test_delete_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")

        # Delete the entity
        self.entity_registry.delete_entity(entity.id)

        # Verify entity is removed from registry
        self.assertNotIn(entity.id, self.entity_registry._entities)

        # Verify entity is removed from ChromaDB
        results = self.collection.get(ids=[entity.id])
        self.assertEqual(len(results["ids"]), 0)

    def test_large_dataset(self):
        """Test performance with larger dataset"""
        batch_size = 10  # Reduced for testing
        total_entities = 50  # Reduced for testing

        # Create a base embedding that we'll modify slightly for each entity
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
                noise = np.random.normal(0, 0.1, 384)
                embedding = base_embedding + noise
                embedding = embedding / np.linalg.norm(embedding)
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
            n_results=5,
            include=["metadatas", "distances"]
        )
        
        # Verify we get exactly 5 results
        self.assertEqual(len(similar["ids"][0]), 5)
        # Verify distances are sorted in ascending order
        distances = similar["distances"][0]
        self.assertEqual(distances, sorted(distances))

    def test_persistence_across_restarts(self):
        """Test data persistence across application restarts"""
        # Add entities to ChromaDB
        entity = self.entity_registry.create_entity(type="TestType", role="TestRole")
        
        # Get the entity ID and collection name for later verification
        entity_id = entity.id
        collection_name = self.collection.name

        # Store current data for comparison
        original_results = self.collection.get(ids=[entity_id])
        self.assertEqual(len(original_results["ids"]), 1, "Entity should exist before reset")

        # Close and reopen client to simulate restart
        self.client.reset()
        time.sleep(0.1)  # Give OS time to release file handles
        
        # Reinitialize with same settings
        self.client = chromadb.PersistentClient(settings=self.settings)
        
        # Recreate collection
        try:
            self.collection = self.client.create_collection(
                name=collection_name,
                metadata=ChromaDBConfig.DEFAULT_METADATA
            )
            
            # Verify entities are still present
            results = self.collection.get(ids=[entity_id])
            self.assertEqual(len(results["ids"]), 1, "Entity should exist after reset")
            self.assertEqual(results["ids"][0], entity_id, "Entity ID should match")
        except Exception as e:
            self.fail(f"Failed to recreate collection or retrieve entity after restart: {e}")

    def test_concurrent_operations(self):
        """Test concurrent entity operations"""
        def create_entity():
            return self.entity_registry.create_entity(type="TestType", role="TestRole")

        with ThreadPoolExecutor(max_workers=5) as executor:
            futures = [executor.submit(create_entity) for _ in range(10)]
            entities = [f.result() for f in futures]

        # Verify all entities were created
        self.assertEqual(len(entities), 10)
        for entity in entities:
            self.assertIsNotNone(self.entity_registry.get_entity(entity.id))


if __name__ == "__main__":
    unittest.main()
