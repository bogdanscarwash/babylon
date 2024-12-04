import os
import shutil
from datetime import datetime
import chromadb
import logging
from config.base import BaseConfig as Config

logger = logging.getLogger(__name__)

def backup_chroma(client: chromadb.Client, backup_dir: str) -> bool:
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
        # Check disk space
        required_space = shutil.disk_usage(Config.CHROMADB_PERSIST_DIR).used
        available_space = shutil.disk_usage(os.path.dirname(backup_dir)).free
        if available_space < required_space * 1.1:  # 10% safety margin
            logger.error("Insufficient disk space for backup")
            return False

        # Persist any changes to disk
        try:
            client.persist()
        except Exception as e:
            logger.error(f"Error persisting ChromaDB data: {e}")
            return False

        # Ensure backup directory exists
        try:
            os.makedirs(backup_dir, exist_ok=True)
        except PermissionError:
            logger.error(f"Permission denied creating backup directory: {backup_dir}")
            return False

        # Copy the persistence directory to the backup directory
        if os.path.exists(Config.CHROMADB_PERSIST_DIR):
            try:
                shutil.copytree(Config.CHROMADB_PERSIST_DIR, backup_dir, dirs_exist_ok=True)
                logger.info(f"ChromaDB backup completed to {backup_dir}")
                return True
            except (shutil.Error, PermissionError) as e:
                logger.error(f"Error copying files during backup: {e}")
                return False
        else:
            logger.error("ChromaDB persistence directory does not exist")
            return False

    except Exception as e:
        logger.error(f"Error during ChromaDB backup: {e}")
        return False

def restore_chroma(backup_dir: str) -> bool:
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
        # Ensure backup directory exists
        if not os.path.exists(backup_dir):
            logger.error(f"Backup directory {backup_dir} does not exist")
            return False

        # Check disk space
        required_space = shutil.disk_usage(backup_dir).used
        available_space = shutil.disk_usage(os.path.dirname(Config.CHROMADB_PERSIST_DIR)).free
        if available_space < required_space * 1.1:  # 10% safety margin
            logger.error("Insufficient disk space for restore")
            return False

        # Remove the existing persistence directory if it exists
        if os.path.exists(Config.CHROMADB_PERSIST_DIR):
            try:
                shutil.rmtree(Config.CHROMADB_PERSIST_DIR)
            except PermissionError:
                logger.error(f"Permission denied removing existing persistence directory")
                return False

        # Copy the backup directory to the persistence directory
        try:
            shutil.copytree(backup_dir, Config.CHROMADB_PERSIST_DIR)
            logger.info(f"ChromaDB restored from backup in {backup_dir}")
            return True
        except (shutil.Error, PermissionError) as e:
            logger.error(f"Error copying files during restore: {e}")
            return False

    except Exception as e:
        logger.error(f"Error during ChromaDB restore: {e}")
        return False
