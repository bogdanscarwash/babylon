import pytest
import os
from babylon.utils.backup import backup_chroma


class TestBackupOperations:
    """Test suite for backup operations."""
    
    def test_backup_creation(self, test_environment, populated_collection, chroma_client):
        """Test that backup is created successfully."""
        collection, _ = populated_collection
        
        backup_success = backup_chroma(
            chroma_client,
            test_environment["backup_dir"],
            persist_directory=test_environment["persist_dir"]
        )
        
        assert backup_success, "Backup creation should succeed"
        
        backup_files = [f for f in os.listdir(test_environment["backup_dir"]) 
                       if f.endswith('.tar.gz')]
        assert len(backup_files) == 1, "Should create exactly one backup file"
    
    def test_backup_with_invalid_directory(self, test_environment, populated_collection, chroma_client):
        """Test backup behavior with invalid directory."""
        collection, _ = populated_collection
        
        # Test with invalid backup directory containing illegal characters
        with pytest.raises(OSError):  # Covers both Windows and other OS path errors
            backup_chroma(
                chroma_client,
                "con:",  # 'con' is a reserved name in Windows, guaranteed to fail
                persist_directory=test_environment["persist_dir"]
            )
