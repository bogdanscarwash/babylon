import os
import shutil
import tempfile
import unittest
import logging

import chromadb

from babylon.config.chromadb_config import ChromaDBConfig
from babylon.utils.backup import backup_chroma, restore_chroma

logger = logging.getLogger(__name__)

class TestChromaBackup(unittest.TestCase):
    def setUp(self):
        """Initialize test environment with ChromaDB instance."""
        # Create temporary directories
        self.temp_dir = tempfile.mkdtemp()
        self.backup_dir = os.path.join(self.temp_dir, "backups")
        os.makedirs(self.backup_dir, exist_ok=True)

        # Configure ChromaDB with settings
        self.settings = ChromaDBConfig.get_settings(
            persist_directory=os.path.join(self.temp_dir, "persist"),
            allow_reset=True,
            anonymized_telemetry=False,
        )

        # Initialize client and collection with settings
        self.client = chromadb.PersistentClient(settings=self.settings)
        self.collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA,
        )

        # Add test data
        self.collection.add(
            ids=["test1", "test2"],
            embeddings=[[1.0, 0.0], [0.0, 1.0]],
            metadatas=[{"type": "test"}, {"type": "test"}],
        )

    def tearDown(self):
        """Clean up test environment."""
        try:
            if hasattr(self, 'client'):
                self.client.reset()
            if hasattr(self, 'temp_dir') and os.path.exists(self.temp_dir):
                shutil.rmtree(self.temp_dir, ignore_errors=True)
        except Exception as e:
            print(f"Cleanup error: {e}")

    def test_backup_restore(self):
        """Test backup creation and restoration."""
        persist_dir = os.path.join(self.temp_dir, "persist")
        
        # Create backup
        backup_success = backup_chroma(
            self.client, 
            self.backup_dir,
            persist_directory=persist_dir
        )
        self.assertTrue(backup_success, "Backup creation failed")

        # Get current data for comparison
        original_results = self.collection.get()
        original_ids = set(original_results["ids"])
        self.assertEqual(len(original_ids), 2, "Expected 2 items before reset")

        # Find backup file before resetting
        backup_files = [f for f in os.listdir(self.backup_dir) if f.endswith('.tar.gz')]
        self.assertTrue(backup_files, "No backup files found")
        backup_file = os.path.join(self.backup_dir, backup_files[0])

        # Reset client and verify data is gone
        self.client.reset()
        
        # Reinitialize client and collection after reset
        self.client = chromadb.PersistentClient(settings=self.settings)
        new_collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA
        )
        results = new_collection.get()
        self.assertEqual(len(results["ids"]), 0, "Expected empty collection after reset")
        
        # Restore from backup
        restore_success = restore_chroma(
            backup_file, 
            persist_directory=persist_dir
        )
        self.assertTrue(restore_success, "Restore operation failed")

        # Reset the client to ensure clean state
        self.client.reset()

        # Reinitialize client with same settings after restore
        self.client = chromadb.PersistentClient(settings=self.settings)
        
        # Get or create collection
        try:
            restored_collection = self.client.get_collection(
                name=ChromaDBConfig.DEFAULT_COLLECTION_NAME
            )
            logger.info("Successfully retrieved restored collection")
        except ValueError as e:
            logger.error(f"Failed to get collection after restore: {e}")
            self.fail("Collection not found after restore")
        
        # Verify restored data
        restored_results = restored_collection.get()
        restored_ids = set(restored_results["ids"])
        self.assertEqual(len(restored_ids), 2, "Expected 2 items after restore")
        self.assertEqual(restored_ids, original_ids, "Restored IDs don't match original IDs")


if __name__ == "__main__":
    unittest.main()
