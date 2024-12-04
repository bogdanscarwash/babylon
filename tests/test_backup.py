import unittest
import os
import tempfile
import shutil
import chromadb
from src.babylon.config.chromadb_config import ChromaDBConfig
from src.babylon.utils.backup import backup_chroma, restore_chroma

class TestChromaBackup(unittest.TestCase):
    def setUp(self):
        """Initialize test environment with ChromaDB instance."""
        # Create temporary directories
        self.temp_dir = tempfile.mkdtemp()
        self.backup_dir = os.path.join(self.temp_dir, 'backups')
        
        # Configure ChromaDB
        self.settings = ChromaDBConfig.get_settings(
            persist_directory=self.temp_dir,
            allow_reset=True,
            anonymized_telemetry=False
        )
        
        # Initialize client and collection
        self.client = chromadb.Client(self.settings)
        self.collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
            metadata=ChromaDBConfig.DEFAULT_METADATA
        )
        
        # Add test data
        self.collection.add(
            ids=["test1", "test2"],
            embeddings=[[1.0, 0.0], [0.0, 1.0]],
            metadatas=[{"type": "test"}, {"type": "test"}]
        )

    def tearDown(self):
        """Clean up test environment."""
        try:
            self.client.reset()
            shutil.rmtree(self.temp_dir)
        except Exception as e:
            print(f"Cleanup error: {e}")

    def test_backup_restore(self):
        """Test backup creation and restoration."""
        # Create backup
        self.assertTrue(backup_chroma(self.client, self.backup_dir))
        
        # Reset client
        self.client.reset()
        
        # Verify data is gone
        new_collection = self.client.create_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME
        )
        results = new_collection.get()
        self.assertEqual(len(results['ids']), 0)
        
        # Restore from backup
        backup_file = os.path.join(self.backup_dir, os.listdir(self.backup_dir)[0])
        self.assertTrue(restore_chroma(backup_file))
        
        # Verify data is restored
        restored_collection = self.client.get_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME
        )
        results = restored_collection.get()
        self.assertEqual(len(results['ids']), 2)
        self.assertEqual(set(results['ids']), {"test1", "test2"})

if __name__ == '__main__':
    unittest.main()
