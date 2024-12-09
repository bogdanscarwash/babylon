import pytest
import os

class TestRestoreOperations:
    """Test suite for restore operations."""
    
    def test_successful_restore(self, test_environment, populated_collection, 
                              chroma_client):
        """Test complete backup and restore cycle."""
        collection, test_data = populated_collection
        
        # Create and verify backup
        backup_file = self._create_backup(
            collection.client,
            test_environment["backup_dir"],
            test_environment["persist_dir"]
        )
        
        # Reset and restore
        collection.client.reset()
        restore_success = restore_chroma(
            backup_file,
            persist_directory=test_environment["persist_dir"]
        )
        assert restore_success, "Restore operation should succeed"
        
        # Verify restored data
        restored_collection = chroma_client.get_collection(
            name=ChromaDBConfig.DEFAULT_COLLECTION_NAME
        )
        self._verify_collection_data(restored_collection, test_data)
    
    def test_restore_with_invalid_backup(self, test_environment, chroma_client):
        """Test restore behavior with invalid backup file."""
        with pytest.raises(Exception):
            restore_chroma(
                "nonexistent_backup.tar.gz",
                persist_directory=test_environment["persist_dir"]
            )
    
    def _create_backup(self, client, backup_dir, persist_dir):
        """Helper method to create a backup file."""
        backup_chroma(client, backup_dir, persist_directory=persist_dir)
        backup_files = [f for f in os.listdir(backup_dir) 
                       if f.endswith('.tar.gz')]
        return os.path.join(backup_dir, backup_files[0])
    
    def _verify_collection_data(self, collection, expected_data):
        """Helper method to verify collection data."""
        results = collection.get()
        assert set(results["ids"]) == set(expected_data["ids"]), (
            "Restored IDs should match original"
        )
        assert len(results["embeddings"]) == len(expected_data["embeddings"]), (
            "Should restore all embeddings"
        )
        assert all(meta["type"] == "test" for meta in results["metadatas"]), (
            "Should restore all metadata"
        )