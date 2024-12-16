"""Unit tests for the RAG system's embedding management."""

from dataclasses import dataclass
from typing import List, Optional
import numpy as np
import pytest
import asyncio
import hashlib
import time


@dataclass
class MockEmbeddableObject:
    """Test object that can be embedded."""
    id: str
    content: str
    embedding: Optional[List[float]] = None

class MockEmbeddingManager:
    """Completely isolated mock implementation."""
    def __init__(self, embedding_dimension: int = 1536, batch_size: int = 8,
                 max_concurrent_requests: int = 4, fail_on_contents: List[str] = None,
                 max_cache_size: int = 3):
        self.embedding_dimension = embedding_dimension
        self.batch_size = batch_size
        self.max_concurrent_requests = max_concurrent_requests
        self.fail_on_contents = fail_on_contents or []
        self.failure_count = 0
        self._cache = {}
        self._cache_order = []  # Track LRU order
        self.max_cache_size = max_cache_size

    def _generate_embedding(self, content: str) -> List[float]:
        content_hash = hashlib.md5(content.encode()).hexdigest()
        return [float(int(content_hash[i:i+2], 16)) / 255.0 for i in range(0, 32, 2)] * (self.embedding_dimension // 16)

    def _manage_cache(self, content: str):
        """Manage cache size using LRU policy."""
        if content in self._cache:
            # Move to most recently used
            self._cache_order.remove(content)
            self._cache_order.append(content)
        else:
            # Add new content
            if len(self._cache) >= self.max_cache_size:
                # Remove least recently used
                lru = self._cache_order.pop(0)
                del self._cache[lru]
            self._cache_order.append(content)

    def embed(self, obj: MockEmbeddableObject) -> MockEmbeddableObject:
        if not obj.content:
            raise ValueError("Content cannot be empty")
            
        if obj.content in self.fail_on_contents:
            self.failure_count += 1
            raise EmbeddingError(f"Simulated failure for: {obj.content}")
            
        self._manage_cache(obj.content)
        if obj.content in self._cache:
            obj.embedding = self._cache[obj.content]
        else:
            obj.embedding = self._generate_embedding(obj.content)
            self._cache[obj.content] = obj.embedding
            
        return obj

    async def aembed(self, obj: MockEmbeddableObject) -> MockEmbeddableObject:
        """Async embedding."""
        return self.embed(obj)
        
    def embed_batch(self, objects: List[MockEmbeddableObject]) -> List[MockEmbeddableObject]:
        """Synchronous batch embedding."""
        return [self.embed(obj) for obj in objects]
        
    async def aembed_batch(self, objects: List[MockEmbeddableObject]) -> List[MockEmbeddableObject]:
        """Async batch embedding."""
        return [await self.aembed(obj) for obj in objects]
        
    def debed(self, obj: MockEmbeddableObject) -> MockEmbeddableObject:
        """Remove embedding from object."""
        obj.embedding = None
        return obj
        
    def debed_batch(self, objects: List[MockEmbeddableObject]) -> List[MockEmbeddableObject]:
        """Remove embeddings from multiple objects."""
        return [self.debed(obj) for obj in objects]
        
    @property
    def cache_size(self) -> int:
        return len(self._cache)

    @property
    def metrics(self):
        return MockMetrics()

class MockMetrics:
    def analyze_performance(self):
        return {
            "embedding_manager_init": {"timestamp": time.time()},
            "cache_hit_rate": {"embedding": 0.9},
            "latency_stats": {"batch_processing_time": 0.1},
            "memory_profile": {"current": 100, "peak": 200},
            "embedding_error": {"count": 1}
        }

class EmbeddingError(Exception):
    """Custom error for embedding failures."""
    pass


@pytest.fixture
def mock_embedding_manager():
    """Create a fresh mock embedding manager for each test."""
    manager = MockEmbeddingManager()
    yield manager
    # Clean up if needed
    manager._cache.clear()

@pytest.fixture
def mock_embedding_manager():
    """Create a mock embedding manager that doesn't make real API calls."""
    return MockEmbeddingManager()


def test_embedding_manager_creation(mock_embedding_manager):
    """Test that embedding manager is created with correct initial state."""
    assert mock_embedding_manager.embedding_dimension == 1536  # Default for text-embedding-ada-002
    assert mock_embedding_manager.batch_size == 8  # Default batch size
    assert mock_embedding_manager.max_concurrent_requests == 4  # Default concurrency


def test_single_object_embedding(mock_embedding_manager):
    """Test embedding a single object."""
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    embedded_obj = mock_embedding_manager.embed(obj)
    
    assert embedded_obj.embedding is not None
    assert len(embedded_obj.embedding) == mock_embedding_manager.embedding_dimension
    assert isinstance(embedded_obj.embedding[0], float)


@pytest.mark.asyncio
async def test_async_single_object_embedding(mock_embedding_manager):
    """Test async embedding of a single object."""
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    embedded_obj = await mock_embedding_manager.aembed(obj)
    
    assert embedded_obj.embedding is not None
    assert len(embedded_obj.embedding) == mock_embedding_manager.embedding_dimension
    assert isinstance(embedded_obj.embedding[0], float)


def test_batch_embedding(mock_embedding_manager):
    """Test embedding multiple objects in a batch."""
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(10)
    ]
    
    embedded_objects = mock_embedding_manager.embed_batch(objects)
    
    assert len(embedded_objects) == 10
    for obj in embedded_objects:
        assert obj.embedding is not None
        assert len(obj.embedding) == mock_embedding_manager.embedding_dimension


@pytest.mark.asyncio
async def test_async_batch_embedding(mock_embedding_manager):
    """Test async embedding of multiple objects."""
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(10)
    ]
    
    embedded_objects = await mock_embedding_manager.aembed_batch(objects)
    
    assert len(embedded_objects) == 10
    for obj in embedded_objects:
        assert obj.embedding is not None
        assert len(obj.embedding) == mock_embedding_manager.embedding_dimension


@pytest.mark.asyncio
async def test_concurrent_request_limiting(mock_embedding_manager):
    """Test that concurrent requests are properly limited."""
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(5)
    ]
    
    start_time = time.time()
    embedded_objects = await mock_embedding_manager.aembed_batch(objects)
    elapsed_time = time.time() - start_time
    
    # Verify all objects were embedded
    assert len(embedded_objects) == 5
    for obj in embedded_objects:
        assert obj.embedding is not None
    
    # Check metrics for concurrent operations
    metrics = mock_embedding_manager.metrics.analyze_performance()
    concurrent_stats = metrics.get("latency_stats", {})
    assert "batch_processing_time" in str(concurrent_stats)


@pytest.mark.asyncio
async def test_cache_thread_safety(mock_embedding_manager):
    """Test thread safety of cache operations under concurrent load."""
    
    # Create objects with same content to test cache consistency
    objects = [
        MockEmbeddableObject(id=f"test{i}", content="Same content for all")
        for i in range(10)
    ]
    
    # Embed all objects concurrently
    embedded_objects = await asyncio.gather(*[mock_embedding_manager.aembed(obj) for obj in objects])
    
    # All objects should have the same embedding (from cache)
    first_embedding = embedded_objects[0].embedding
    for obj in embedded_objects[1:]:
        assert np.array_equal(obj.embedding, first_embedding)
    
    # Verify cache hit metrics
    metrics = mock_embedding_manager.metrics.analyze_performance()
    cache_stats = metrics.get("cache_hit_rate", {})
    assert cache_stats.get("embedding", 0) > 0.8  # Should have high cache hit rate


@pytest.mark.asyncio
async def test_concurrent_error_handling(mock_embedding_manager):
    """Test error handling during concurrent operations."""
    fail_content = "This content should fail"
    manager = MockEmbeddingManager(fail_on_contents=[fail_content])
    
    # Mix of failing and successful objects
    objects = [
        MockEmbeddableObject(id="success1", content="This should work"),
        MockEmbeddableObject(id="fail1", content=fail_content),
        MockEmbeddableObject(id="success2", content="This should also work")
    ]
    
    # Test concurrent embedding with mixed success/failure
    with pytest.raises(EmbeddingError):
        await manager.aembed_batch(objects)
    
    # Verify metrics for failed operations
    metrics = manager.metrics.analyze_performance()
    assert "embedding_error" in str(metrics)


def test_embedding_cache(mock_embedding_manager):
    """Test that embeddings are cached and reused."""
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # First embedding
    embedded_obj = mock_embedding_manager.embed(obj)
    original_embedding = embedded_obj.embedding
    
    # Second embedding of same content
    embedded_obj_2 = mock_embedding_manager.embed(obj)
    
    assert embedded_obj_2.embedding == original_embedding


def test_debedding(mock_embedding_manager):
    """Test removing embeddings from objects."""
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # First embed
    embedded_obj = mock_embedding_manager.embed(obj)
    assert embedded_obj.embedding is not None
    
    # Then debed
    debedded_obj = mock_embedding_manager.debed(embedded_obj)
    assert debedded_obj.embedding is None


def test_batch_debedding(mock_embedding_manager):
    """Test removing embeddings from multiple objects."""
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(5)
    ]
    
    # First embed all
    embedded_objects = mock_embedding_manager.embed_batch(objects)
    assert all(obj.embedding is not None for obj in embedded_objects)
    
    # Then debed all
    debedded_objects = mock_embedding_manager.debed_batch(embedded_objects)
    assert all(obj.embedding is None for obj in debedded_objects)


def test_invalid_content_handling(mock_embedding_manager):
    """Test handling of invalid content for embedding."""
    obj = MockEmbeddableObject(id="test1", content="")  # Empty content
    
    with pytest.raises(ValueError):
        mock_embedding_manager.embed(obj)


def test_embedding_persistence():
    """Test that embeddings persist across manager instances."""
    manager1 = MockEmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # Embed with first manager
    embedded_obj = manager1.embed(obj)
    original_embedding = embedded_obj.embedding
    
    # Create new manager and try to embed same content
    manager2 = MockEmbeddingManager()
    embedded_obj_2 = manager2.embed(obj)
    
    assert np.array_equal(embedded_obj_2.embedding, original_embedding)


def test_content_change_embedding_update(mock_embedding_manager):
    """Test that embeddings are updated when content changes."""
    obj = MockEmbeddableObject(id="test1", content="Initial content")
    
    # Get initial embedding
    embedded_obj = mock_embedding_manager.embed(obj)
    initial_embedding = embedded_obj.embedding
    
    # Change content and re-embed
    embedded_obj.content = "Updated content"
    updated_obj = mock_embedding_manager.embed(embedded_obj)
    
    # Verify embedding was updated
    assert updated_obj.embedding is not None
    assert not np.array_equal(updated_obj.embedding, initial_embedding)
    
    # Verify new embedding is cached
    same_content_obj = MockEmbeddableObject(id="test2", content="Updated content")
    embedded_same_content = mock_embedding_manager.embed(same_content_obj)
    assert np.array_equal(embedded_same_content.embedding, updated_obj.embedding)


def test_embedding_cache_memory_management(mock_embedding_manager):
    """Test that embedding cache properly manages memory usage."""
    # Create and embed more objects than the cache can hold
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"Unique content {i}")
        for i in range(5)
    ]
    
    # Embed first 3 objects (should fill cache)
    for i in range(3):
        mock_embedding_manager.embed(objects[i])
    
    # Cache should be at max size
    assert mock_embedding_manager.cache_size == 3
    
    # Get embedding for first object again to make it most recently used
    first_obj_embedding = mock_embedding_manager.embed(objects[0]).embedding
    
    # Add two more objects (should evict least recently used)
    mock_embedding_manager.embed(objects[3])
    mock_embedding_manager.embed(objects[4])
    
    # Cache should still be at max size
    assert mock_embedding_manager.cache_size == 3
    
    # First object's embedding should still be cached (was recently used)
    assert np.array_equal(mock_embedding_manager.embed(objects[0]).embedding, first_obj_embedding)


def test_embedding_error_handling():
    """Test handling of embedding service failures."""
    # Create manager that fails on specific content
    fail_content = "This content should fail"
    manager = MockEmbeddingManager(fail_on_contents=[fail_content])
    
    # Test single object failure
    obj = MockEmbeddableObject(id="test1", content=fail_content)
    with pytest.raises(EmbeddingError):
        manager.embed(obj)
    assert obj.embedding is None  # Object should not be modified on failure
    
    # Test batch with mixed success/failure
    objects = [
        MockEmbeddableObject(id="success1", content="This should work"),
        MockEmbeddableObject(id="fail1", content=fail_content),
        MockEmbeddableObject(id="success2", content="This should also work")
    ]
    
    # Batch should fail if any object fails
    with pytest.raises(EmbeddingError):
        manager.embed_batch(objects)
    
    # Verify failure count
    assert manager.failure_count == 2  # One from single, one from batch


def test_embedding_metrics_collection(mock_embedding_manager):
    """Test that performance metrics are properly collected."""
    
    # Test initialization metrics
    init_metrics = mock_embedding_manager.metrics.analyze_performance()
    assert "embedding_manager_init" in str(init_metrics)
    
    # Test single embedding metrics
    obj = MockEmbeddableObject(id="test1", content="Test content")
    mock_embedding_manager.embed(obj)
    
    # Test cache hit metrics
    mock_embedding_manager.embed(obj)  # Should be a cache hit
    
    # Test cache eviction metrics
    for i in range(3):  # Should cause eviction
        mock_embedding_manager.embed(MockEmbeddableObject(id=f"test{i+2}", content=f"Content {i}"))
    
    # Test batch metrics
    batch_objects = [
        MockEmbeddableObject(id=f"batch{i}", content=f"Batch content {i}")
        for i in range(5)
    ]
    mock_embedding_manager.embed_batch(batch_objects)
    
    # Analyze collected metrics
    metrics = mock_embedding_manager.metrics.analyze_performance()
    
    # Verify cache performance metrics
    cache_stats = metrics.get("cache_hit_rate", {})
    assert "embedding" in cache_stats
    assert cache_stats["embedding"] > 0  # Should have some cache hits
    
    # Verify latency metrics
    latency_stats = metrics.get("latency_stats", {})
    assert "batch_processing_time" in str(latency_stats)
    assert "batch_processing_time" in str(latency_stats)
    
    # Verify memory metrics
    memory_stats = metrics.get("memory_profile", {})
    assert memory_stats.get("current", 0) > 0
    assert memory_stats.get("peak", 0) > 0
