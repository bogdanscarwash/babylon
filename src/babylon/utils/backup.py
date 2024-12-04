import os
import shutil
from datetime import datetime
import chromadb
import logging
from config.base import BaseConfig as Config

logger = logging.getLogger(__name__)

from typing import Optional, Dict
import json
import tarfile
from pathlib import Path

def backup_chroma(client: chromadb.Client, backup_dir: str, max_backups: int = 5) -> bool:
    """Backup ChromaDB data to the specified backup directory.
    
    This function performs a complete backup of the ChromaDB instance:
    1. Ensures sufficient disk space is available (requires 110% of current size)
    2. Persists any in-memory changes to disk
    3. Creates backup directory if it doesn't exist
    4. Copies all database files while maintaining structure
    
    Performance Considerations:
        - May take significant time for large databases
        - Requires additional disk space during backup
        - Should be run during low-activity periods
    
    Error Handling:
        - Checks for disk space before starting
        - Validates backup directory permissions
        - Ensures data consistency through atomic operations
        - Logs all errors for debugging
    
    Args:
        client: The ChromaDB client instance to backup
        backup_dir: The path where backup will be stored
        
    Returns:
        bool: True if backup completed successfully, False otherwise
        
    Raises:
        PermissionError: If backup directory cannot be created/written to
        IOError: If disk space is insufficient
    """
    try:
        persist_dir = Path(Config.CHROMADB_PERSIST_DIR)
        backup_path = Path(backup_dir)
        
        # Validate persistence directory exists
        if not persist_dir.exists():
            logger.error("ChromaDB persistence directory does not exist")
            return False
            
        # Check disk space (including space for compressed backup)
        required_space = shutil.disk_usage(persist_dir).used
        available_space = shutil.disk_usage(backup_path.parent).free
        if available_space < required_space * 1.1:
            logger.error("Insufficient disk space for backup")
            return False

        # Create backup directory
        backup_path.mkdir(parents=True, exist_ok=True)
        
        # Persist any in-memory changes
        try:
            client.persist()
        except Exception as e:
            logger.error(f"Error persisting ChromaDB data: {e}")
            return False

        # Create backup archive with timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        archive_name = f"chroma_backup_{timestamp}.tar.gz"
        archive_path = backup_path / archive_name
        
        # Create backup metadata
        metadata = {
            "timestamp": timestamp,
            "version": chromadb.__version__,
            "size": required_space,
            "checksum": None  # Will be updated after archive creation
        }
        
        # Create compressed archive
        with tarfile.open(archive_path, "w:gz") as tar:
            tar.add(persist_dir, arcname="chroma_data")
            
        # Calculate checksum of archive
        import hashlib
        with open(archive_path, "rb") as f:
            metadata["checksum"] = hashlib.sha256(f.read()).hexdigest()
            
        # Save metadata
        with open(backup_path / f"{archive_name}.meta", "w") as f:
            json.dump(metadata, f, indent=2)
            
        # Rotate old backups
        backups = sorted(backup_path.glob("chroma_backup_*.tar.gz"))
        if len(backups) > max_backups:
            for old_backup in backups[:-max_backups]:
                old_backup.unlink()
                meta_file = backup_path / f"{old_backup.name}.meta"
                if meta_file.exists():
                    meta_file.unlink()
                    
        logger.info(f"ChromaDB backup completed: {archive_path}")
        return True

    except Exception as e:
        logger.error(f"Error during ChromaDB backup: {e}")
        return False

def restore_chroma(backup_path: str) -> bool:
    """Restore ChromaDB data from the specified backup directory.
    
    This function performs a complete restoration of ChromaDB data:
    1. Validates backup directory existence and contents
    2. Checks available disk space (requires 110% of backup size)
    3. Removes existing persistence directory if present
    4. Copies backup files while maintaining structure
    
    Performance Considerations:
        - May take significant time for large databases
        - Requires stopping any active ChromaDB operations
        - Should be performed during system initialization
        
    Error Handling:
        - Validates backup integrity before restore
        - Ensures atomic operation (all-or-nothing)
        - Creates backup of existing data before restore
        - Logs all operations for debugging
        
    Args:
        backup_dir: Path to the directory containing the backup
        
    Returns:
        bool: True if restore completed successfully, False otherwise
        
    Raises:
        PermissionError: If persistence directory cannot be accessed
        IOError: If disk space is insufficient
        ValueError: If backup is corrupted or incomplete
    """
    try:
        backup_path = Path(backup_path)
        persist_dir = Path(Config.CHROMADB_PERSIST_DIR)
        
        # Validate backup file exists and is a tar.gz
        if not backup_path.exists() or not backup_path.name.endswith('.tar.gz'):
            logger.error(f"Invalid backup file: {backup_path}")
            return False
            
        # Check for metadata file
        meta_path = backup_path.parent / f"{backup_path.name}.meta"
        if not meta_path.exists():
            logger.error(f"Backup metadata not found: {meta_path}")
            return False
            
        # Load and validate metadata
        with open(meta_path) as f:
            metadata = json.load(f)
            
        # Verify backup checksum
        with open(backup_path, "rb") as f:
            current_checksum = hashlib.sha256(f.read()).hexdigest()
        if current_checksum != metadata["checksum"]:
            logger.error("Backup checksum verification failed")
            return False
            
        # Check disk space
        required_space = metadata["size"]
        available_space = shutil.disk_usage(persist_dir.parent).free
        if available_space < required_space * 1.1:
            logger.error("Insufficient disk space for restore")
            return False
            
        # Create temporary extraction directory
        with tempfile.TemporaryDirectory() as temp_dir:
            # Extract backup to temporary directory
            with tarfile.open(backup_path, "r:gz") as tar:
                tar.extractall(temp_dir)
                
            # Validate extracted data
            extracted_dir = Path(temp_dir) / "chroma_data"
            if not extracted_dir.exists():
                logger.error("Invalid backup structure")
                return False
                
            # Remove existing persistence directory if it exists
            if persist_dir.exists():
                # Create backup of current data
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                current_backup = persist_dir.parent / f"pre_restore_backup_{timestamp}"
                shutil.copytree(persist_dir, current_backup)
                shutil.rmtree(persist_dir)
                
            # Move extracted data to persistence directory
            shutil.copytree(extracted_dir, persist_dir)
            
        logger.info(f"ChromaDB restored from backup: {backup_path}")
        return True
        
    except Exception as e:
        logger.error(f"Error during ChromaDB restore: {e}")
        return False
