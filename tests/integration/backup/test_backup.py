import pytest


class TestBackupOperations:
    """Test suite for backup operations."""
    
    def test_backup_creation(self, test_environment, populated_collection):
        """Test that backup is created successfully."""
        collection, _ = populated_collection
        
        backup_success = backup_chroma(
            collection.client,
            test_environment["backup_dir"],
            persist_directory=test_environment["persist_dir"]
        )
        
        assert backup_success, "Backup creation should succeed"
        
        backup_files = [f for f in os.listdir(test_environment["backup_dir"]) 
                       if f.endswith('.tar.gz')]
        assert len(backup_files) == 1, "Should create exactly one backup file"
    
    def test_backup_with_invalid_directory(self, populated_collection):
        """Test backup behavior with invalid directory."""
        collection, _ = populated_collection
        
        with pytest.raises(ValueError):
            backup_chroma(
                collection.client,
                "/nonexistent/directory",
                persist_directory="/another/nonexistent/dir"
            )