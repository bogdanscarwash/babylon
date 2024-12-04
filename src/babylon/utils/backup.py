import os
import shutil
from datetime import datetime
import chromadb
import logging
from config.base import BaseConfig as Config

logger = logging.getLogger(__name__)

def backup_chroma(client: chromadb.Client, backup_dir: str) -> None:
    """Backup ChromaDB data to the specified backup directory.

    Args:
        client: The ChromaDB client instance
        backup_dir: The path to the backup directory
    """
    try:
        # Persist any changes to disk
        try:
            client.persist()
        except Exception as e:
            logger.error(f"Error persisting ChromaDB data: {e}")

        # Ensure backup directory exists
        os.makedirs(backup_dir, exist_ok=True)

        # Copy the persistence directory to the backup directory
        if os.path.exists(Config.CHROMADB_PERSIST_DIR):
            shutil.copytree(Config.CHROMADB_PERSIST_DIR, backup_dir, dirs_exist_ok=True)
            print(f"ChromaDB backup completed to {backup_dir}")
        else:
            print("ChromaDB persistence directory does not exist.")

    except Exception as e:
        print(f"Error during ChromaDB backup: {e}")

    def restore_chroma(backup_dir: str) -> None:
        """Restore ChromaDB data from the specified backup directory.

        Args:
            backup_dir: The path to the backup directory
        """
        try:
            # Ensure backup directory exists
            if not os.path.exists(backup_dir):
                print(f"Backup directory {backup_dir} does not exist.")
                return

            # Remove the existing persistence directory if it exists
            if os.path.exists(Config.CHROMADB_PERSIST_DIR):
                shutil.rmtree(Config.CHROMADB_PERSIST_DIR)

            # Copy the backup directory to the persistence directory
            shutil.copytree(backup_dir, Config.CHROMADB_PERSIST_DIR)
            print(f"ChromaDB restored from backup in {backup_dir}")

        except Exception as e:
            print(f"Error during ChromaDB restore: {e}")
