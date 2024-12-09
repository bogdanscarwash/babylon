import logging
import shutil
from datetime import datetime
import hashlib
import json
import tarfile
import tempfile
from pathlib import Path

import chromadb
from babylon.exceptions import BackupError

logger = logging.getLogger(__name__)

def backup_chroma(
    client: chromadb.Client, 
    backup_dir: str, 
    persist_directory: str = None,
    max_backups: int = 5
) -> bool:
    """Backup ChromaDB data to the specified backup directory.

    Args:
        client: The ChromaDB client instance to backup
        backup_dir: The path where backup will be stored
        persist_directory: Optional directory where ChromaDB data is persisted
        max_backups: Maximum number of backups to keep

    Returns:
        bool: True if backup completed successfully, False otherwise
    """
    try:
        # Get persist directory from client settings if not provided
        if persist_directory is None:
            if hasattr(client, '_settings'):
                persist_directory = client._settings.persist_directory
            else:
                raise BackupError("No persistence directory specified", "BACKUP_001")

        persist_dir = Path(persist_directory)
        backup_path = Path(backup_dir)

        # Validate persistence directory exists
        if not persist_dir.exists():
            raise BackupError(
                "ChromaDB persistence directory does not exist", "BACKUP_001"
            )

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
            "size": 0,  # Will be updated after archive creation
            "checksum": None,  # Will be updated after archive creation
        }

        # Create compressed archive
        with tarfile.open(archive_path, "w:gz") as tar:
            tar.add(persist_dir, arcname="chroma_data")

        # Update metadata with actual file size
        metadata["size"] = archive_path.stat().st_size

        # Calculate checksum of archive
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

def restore_chroma(backup_path: str, persist_directory: str = None) -> bool:
    """Restore ChromaDB data from the specified backup.

    Args:
        backup_path: Path to the backup file
        persist_directory: Optional directory where ChromaDB data should be restored

    Returns:
        bool: True if restore completed successfully, False otherwise
    """
    try:
        backup_path = Path(backup_path)
        
        # Get persist directory
        if persist_directory is None:
            raise BackupError("No persistence directory specified", "BACKUP_003")
        persist_dir = Path(persist_directory)

        # Validate backup file exists and is a tar.gz
        if not backup_path.exists() or not backup_path.name.endswith(".tar.gz"):
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
