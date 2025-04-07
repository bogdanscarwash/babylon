import pytest
import chromadb
from babylon.config.chromadb_config import ChromaDBConfig
from babylon.data.entity_registry import EntityRegistry
from tests.mocks import MockMetricsCollector

@pytest.fixture
def chroma_client(test_dir):
    """Provide a configured ChromaDB client."""
    settings = ChromaDBConfig.get_settings(
        persist_directory=test_dir,
        allow_reset=True,
        anonymized_telemetry=False,
        is_persistent=True
    )
    client = chromadb.PersistentClient(settings=settings)
    yield client
    client.reset()

@pytest.fixture
def entity_registry(chroma_client):
    """Provide an EntityRegistry instance."""
    collection = chroma_client.create_collection(
        name=ChromaDBConfig.DEFAULT_COLLECTION_NAME,
        metadata=ChromaDBConfig.DEFAULT_METADATA,
    )
    registry = EntityRegistry(collection)
    registry.metrics = MockMetricsCollector()
    return registry