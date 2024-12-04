import unittest
import os
import shutil
import tempfile
import time
import logging
import chromadb
from concurrent.futures import ThreadPoolExecutor
from src.babylon.metrics.performance_metrics import MetricsCollector
from chromadb.config import Settings
from src.babylon.entities.entity_registry import EntityRegistry
from src.babylon.utils.backup import backup_chroma, restore_chroma

class TestChromaDBIntegration(unittest.TestCase):
    def setUp(self):
        # Set up logging
        self.log_file = os.path.join(tempfile.mkdtemp(), 'test.log')
        logging.basicConfig(filename=self.log_file, level=logging.DEBUG)
        self.logger = logging.getLogger(__name__)

        # Set up metrics collection
        self.metrics = MetricsCollector()
        
        # Create temporary directories and initialize clients
        self.temp_dir = tempfile.mkdtemp()
        self.temp_persist_dir = os.path.join(self.temp_dir, 'persist')
        os.makedirs(self.temp_persist_dir)

        # Initialize ChromaDB with retry settings
        self.client = chromadb.Client(Settings(
            chroma_db_impl="duckdb+parquet",
            persist_directory=self.temp_persist_dir,
            anonymized_telemetry=False
        ))
        self.collection = self.client.get_or_create_collection(name='test_entities')

        self.entity_registry = EntityRegistry(chroma_collection=self.collection)

    def tearDown(self):
        # Clean up temporary directories
        shutil.rmtree(self.temp_dir)
        self.client.reset()

    def test_add_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type='TestType', role='TestRole')

        # Verify the entity is in the registry
        self.assertIn(entity.id, self.entity_registry._entities)

        # Verify the entity is added to ChromaDB
        results = self.collection.get(ids=[entity.id])
        self.assertEqual(len(results['ids']), 1)
        self.assertEqual(results['ids'][0], entity.id)

    def test_update_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type='TestType', role='TestRole')

        # Update the entity's attributes
        self.entity_registry.update_entity(entity.id, freedom=0.5, wealth=0.8)

        # Verify updates in the registry
        updated_entity = self.entity_registry.get_entity(entity.id)
        self.assertEqual(updated_entity.freedom, 0.5)
        self.assertEqual(updated_entity.wealth, 0.8)

        # Verify updates in ChromaDB
        results = self.collection.get(ids=[entity.id], include=['metadatas'])
        metadata = results['metadatas'][0]
        self.assertEqual(metadata['freedom'], 0.5)
        self.assertEqual(metadata['wealth'], 0.8)

    def test_delete_entity(self):
        # Create a test entity
        entity = self.entity_registry.create_entity(type='TestType', role='TestRole')

        # Delete the entity
        self.entity_registry.delete_entity(entity.id)

        # Verify the entity is removed from the registry
        self.assertNotIn(entity.id, self.entity_registry._entities)

        # Verify the entity is deleted from ChromaDB
        results = self.collection.get(ids=[entity.id])
        self.assertEqual(len(results['ids']), 0)

    def test_backup_chroma(self):
        # Perform operations to add data to ChromaDB
        backup_dir = os.path.join(self.temp_dir, 'backup')
        backup_chroma(self.client, backup_dir)

        # Verify that backup directory exists and contains data
        self.assertTrue(os.path.exists(backup_dir))
        self.assertTrue(os.listdir(backup_dir))  # Ensure it's not empty

    def test_restore_chroma(self):
        # First, create a backup as in test_backup_chroma
        backup_dir = os.path.join(self.temp_dir, 'backup')
        backup_chroma(self.client, backup_dir)

        # Clear the current persistence directory
        shutil.rmtree(self.temp_persist_dir)
        os.makedirs(self.temp_persist_dir)

        # Restore from backup
        restore_chroma(backup_dir)

        # Initialize a new client and verify data is restored
        new_client = chromadb.Client(Settings(
            chroma_db_impl="duckdb+parquet",
            persist_directory=self.temp_persist_dir
        ))
        collection = new_client.get_collection(name='test_entities')

        # Verify that entities are present
        results = collection.get()
        self.assertGreater(len(results['ids']), 0)

    def test_error_handling(self):
        """Test error handling for invalid operations"""
        # Test invalid ID
        with self.assertRaises(ValueError):
            self.entity_registry.get_entity("")
            
        # Test duplicate ID
        entity = self.entity_registry.create_entity(type='TestType', role='TestRole')
        with self.assertRaises(ValueError):
            self.collection.add(
                embeddings=[[1.0] * 384],
                ids=[entity.id]
            )

    def test_concurrent_operations(self):
        """Test concurrent entity operations"""
        def create_entity():
            return self.entity_registry.create_entity(type='TestType', role='TestRole')

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
        
        # Create 100 entities
        entities = []
        for _ in range(100):
            entity = self.entity_registry.create_entity(type='TestType', role='TestRole')
            entities.append(entity)
            
        creation_time = time.time() - start_time
        self.metrics.record_operation_time('bulk_entity_creation', creation_time)
        
        # Verify all entities exist
        for entity in entities:
            self.assertIsNotNone(self.entity_registry.get_entity(entity.id))

    def test_persistence_across_restarts(self):
        """Test data persistence across application restarts"""
        # Add entities to ChromaDB
        entity = self.entity_registry.create_entity(type='TestType', role='TestRole')
        
        # Record initial state metrics
        initial_memory = self.metrics.get_memory_usage()
        
        # Close the client to simulate app shutdown
        self.client.persist()
        self.client.reset()

        # Re-initialize client and collection
        new_client = chromadb.Client(Settings(
            chroma_db_impl="duckdb+parquet",
            persist_directory=self.temp_persist_dir
        ))
        collection = new_client.get_collection(name='test_entities')

        # Verify entities are still present
        results = collection.get(ids=[entity.id])
        self.assertEqual(len(results['ids']), 1)
        
        # Verify memory usage hasn't increased significantly
        final_memory = self.metrics.get_memory_usage()
        self.assertLess(final_memory - initial_memory, 100 * 1024 * 1024)  # 100MB threshold

if __name__ == '__main__':
    unittest.main()
