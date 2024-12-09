import pytest
import os
import tempfile
import shutil
import chromadb
from babylon.config.chromadb_config import ChromaDBConfig

@pytest.fixture(scope="function")
def test_environment():
    """Create a test environment with directories."""
    temp_dir = tempfile.mkdtemp()
    backup_dir = os.path.join(temp_dir, "backups")
    persist_dir = os.path.join(temp_dir, "persist")
    os.makedirs(backup_dir, exist_ok=True)
    
    yield {
        "temp_dir": temp_dir,
        "backup_dir": backup_dir,
        "persist_dir": persist_dir
    }
    
    # Cleanup
    shutil.rmtree(temp_dir, ignore_errors=True)

@pytest.fixture(scope="function")
def chroma_client(test_environment):
    """Provide a configured ChromaDB client."""
    settings = ChromaDBConfig.get_settings(
        persist_directory=test_environment["persist_dir"],
        allow_reset=True,
        anonymized_telemetry=False,
    )
    client = chromadb.PersistentClient(settings=settings)
    yield client
    client.reset()

@pytest.fixture(scope="function")
def populated_collection(chroma_client):
    """Provide a collection with test data."""
    collection = chroma_client.create_collection(
        name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
        metadata=ChromaDBConfig.DEFAULT_METADATA,
    )
    
    test_data = {
        "ids": ["test1", "test2"],
        "embeddings": [[1.0, 0.0], [0.0, 1.0]],
        "metadatas": [{"type": "test"}, {"type": "test"}]
    }
    collection.add(**test_data)
    
    return collection, test_data