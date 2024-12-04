import os
import shutil
from datetime import datetime
import chromadb
from config.base import BaseConfig as Config

def backup_chroma(client: chromadb.Client, backup_dir: str) -> None:
    """Backup ChromaDB data to the specified backup directory.

    Args:
        client: The ChromaDB client instance
        backup_dir: The path to the backup directory
    """
    try:
        # Persist any changes to disk
        client.persist()

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
