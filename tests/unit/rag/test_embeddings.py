"""Unit tests for the RAG system's embedding management."""

import pytest
import numpy as np
import asyncio
from typing import List, Optional
from dataclasses import dataclass
from babylon.rag.embeddings import EmbeddingManager, EmbeddingError
import time

@dataclass
class MockEmbeddableObject:
    """Test object that can be embedded."""
    id: str
    content: str
    embedding: Optional[List[float]] = None


class MockFailingEmbeddingManager(EmbeddingManager):
    """Mock embedding manager that simulates embedding service failures."""
    
    def __init__(self, *args, fail_on_contents: List[str] = None, **kwargs):
        """Initialize with list of content strings that should trigger failures."""
        super().__init__(*args, **kwargs)
        self.fail_on_contents = fail_on_contents or []
        self.failure_count = 0
    
    def _generate_embedding(self, content: str) -> List[float]:
        """Simulate embedding service failures for specific content."""
        if content in self.fail_on_contents:
            self.failure_count += 1
            raise RuntimeError(f"Simulated embedding service failure for: {content}")
        return super()._generate_embedding(content)


def test_embedding_manager_creation():
    """Test that embedding manager is created with correct initial state."""
    manager = EmbeddingManager()
    assert manager.embedding_dimension == 1536  # Default for text-embedding-ada-002
    assert manager.batch_size == 8  # Default batch size
    assert manager.max_concurrent_requests == 4  # Default concurrency


def test_single_object_embedding():
    """Test embedding a single object."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    embedded_obj = manager.embed(obj)
    
    assert embedded_obj.embedding is not None
    assert len(embedded_obj.embedding) == manager.embedding_dimension
    assert isinstance(embedded_obj.embedding[0], float)


@pytest.mark.asyncio
async def test_async_single_object_embedding():
    """Test async embedding of a single object."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    embedded_obj = await manager.aembed(obj)
    
    assert embedded_obj.embedding is not None
    assert len(embedded_obj.embedding) == manager.embedding_dimension
    assert isinstance(embedded_obj.embedding[0], float)


def test_batch_embedding():
    """Test embedding multiple objects in a batch."""
    manager = EmbeddingManager()
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(10)
    ]
    
    embedded_objects = manager.embed_batch(objects)
    
    assert len(embedded_objects) == 10
    for obj in embedded_objects:
        assert obj.embedding is not None
        assert len(obj.embedding) == manager.embedding_dimension


@pytest.mark.asyncio
async def test_async_batch_embedding():
    """Test async embedding of multiple objects."""
    manager = EmbeddingManager()
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(10)
    ]
    
    embedded_objects = await manager.aembed_batch(objects)
    
    assert len(embedded_objects) == 10
    for obj in embedded_objects:
        assert obj.embedding is not None
        assert len(obj.embedding) == manager.embedding_dimension


@pytest.mark.asyncio
async def test_concurrent_request_limiting():
    """Test that concurrent requests are properly limited."""
    manager = EmbeddingManager(max_concurrent_requests=2)
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(5)
    ]
    
    start_time = time.time()
    embedded_objects = await manager.aembed_batch(objects)
    elapsed_time = time.time() - start_time
    
    # Verify all objects were embedded
    assert len(embedded_objects) == 5
    for obj in embedded_objects:
        assert obj.embedding is not None
    
    # Check metrics for concurrent operations
    metrics = manager.metrics.analyze_performance()
    concurrent_stats = metrics.get("latency_stats", {})
    assert "batch_processing_time" in str(concurrent_stats)


@pytest.mark.asyncio
async def test_cache_thread_safety():
    """Test thread safety of cache operations under concurrent load."""
    manager = EmbeddingManager(max_concurrent_requests=4)
    
    # Create objects with same content to test cache consistency
    objects = [
        MockEmbeddableObject(id=f"test{i}", content="Same content for all")
        for i in range(10)
    ]
    
    # Embed all objects concurrently
    embedded_objects = await asyncio.gather(*[manager.aembed(obj) for obj in objects])
    
    # All objects should have the same embedding (from cache)
    first_embedding = embedded_objects[0].embedding
    for obj in embedded_objects[1:]:
        assert np.array_equal(obj.embedding, first_embedding)
    
    # Verify cache hit metrics
    metrics = manager.metrics.analyze_performance()
    cache_stats = metrics.get("cache_hit_rate", {})
    assert cache_stats.get("embedding", 0) > 0.8  # Should have high cache hit rate


@pytest.mark.asyncio
async def test_concurrent_error_handling():
    """Test error handling during concurrent operations."""
    fail_content = "This content should fail"
    manager = MockFailingEmbeddingManager(
        fail_on_contents=[fail_content],
        max_concurrent_requests=4
    )
    
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


def test_embedding_cache():
    """Test that embeddings are cached and reused."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # First embedding
    embedded_obj = manager.embed(obj)
    original_embedding = embedded_obj.embedding
    
    # Second embedding of same content
    embedded_obj_2 = manager.embed(obj)
    
    assert embedded_obj_2.embedding == original_embedding


def test_debedding():
    """Test removing embeddings from objects."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # First embed
    embedded_obj = manager.embed(obj)
    assert embedded_obj.embedding is not None
    
    # Then debed
    debedded_obj = manager.debed(embedded_obj)
    assert debedded_obj.embedding is None


def test_batch_debedding():
    """Test removing embeddings from multiple objects."""
    manager = EmbeddingManager()
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"This is test document {i}")
        for i in range(5)
    ]
    
    # First embed all
    embedded_objects = manager.embed_batch(objects)
    assert all(obj.embedding is not None for obj in embedded_objects)
    
    # Then debed all
    debedded_objects = manager.debed_batch(embedded_objects)
    assert all(obj.embedding is None for obj in debedded_objects)


def test_custom_embedding_dimension_validation():
    """Test validation of custom embedding dimensions."""
    # Should raise ValueError for custom dimension
    with pytest.raises(ValueError) as exc_info:
        EmbeddingManager(embedding_dimension=512)
    assert "Custom embedding dimensions are not supported" in str(exc_info.value)


def test_invalid_content_handling():
    """Test handling of invalid content for embedding."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="")  # Empty content
    
    with pytest.raises(ValueError):
        manager.embed(obj)


def test_embedding_persistence():
    """Test that embeddings persist across manager instances."""
    manager1 = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="This is a test document")
    
    # Embed with first manager
    embedded_obj = manager1.embed(obj)
    original_embedding = embedded_obj.embedding
    
    # Create new manager and try to embed same content
    manager2 = EmbeddingManager()
    embedded_obj_2 = manager2.embed(obj)
    
    assert np.array_equal(embedded_obj_2.embedding, original_embedding)


def test_content_change_embedding_update():
    """Test that embeddings are updated when content changes."""
    manager = EmbeddingManager()
    obj = MockEmbeddableObject(id="test1", content="Initial content")
    
    # Get initial embedding
    embedded_obj = manager.embed(obj)
    initial_embedding = embedded_obj.embedding
    
    # Change content and re-embed
    embedded_obj.content = "Updated content"
    updated_obj = manager.embed(embedded_obj)
    
    # Verify embedding was updated
    assert updated_obj.embedding is not None
    assert not np.array_equal(updated_obj.embedding, initial_embedding)
    
    # Verify new embedding is cached
    same_content_obj = MockEmbeddableObject(id="test2", content="Updated content")
    embedded_same_content = manager.embed(same_content_obj)
    assert np.array_equal(embedded_same_content.embedding, updated_obj.embedding)


def test_embedding_cache_memory_management():
    """Test that embedding cache properly manages memory usage."""
    # Create manager with small cache size for testing
    manager = EmbeddingManager(max_cache_size=3)
    
    # Create and embed more objects than the cache can hold
    objects = [
        MockEmbeddableObject(id=f"test{i}", content=f"Unique content {i}")
        for i in range(5)
    ]
    
    # Embed first 3 objects (should fill cache)
    for i in range(3):
        manager.embed(objects[i])
    
    # Cache should be at max size
    assert manager.cache_size == 3
    
    # Get embedding for first object again to make it most recently used
    first_obj_embedding = manager.embed(objects[0]).embedding
    
    # Add two more objects (should evict least recently used)
    manager.embed(objects[3])
    manager.embed(objects[4])
    
    # Cache should still be at max size
    assert manager.cache_size == 3
    
    # First object's embedding should still be cached (was recently used)
    assert np.array_equal(manager.embed(objects[0]).embedding, first_obj_embedding)
    
    # Second object's embedding should be regenerated (was evicted)
    second_embedding = manager.embed(objects[1]).embedding
    assert not np.array_equal(second_embedding, objects[1].embedding)


def test_embedding_error_handling():
    """Test handling of embedding service failures."""
    # Create manager that fails on specific content
    fail_content = "This content should fail"
    manager = MockFailingEmbeddingManager(fail_on_contents=[fail_content])
    
    # Test single object failure
    obj = MockEmbeddableObject(id="test1", content=fail_content)
    with pytest.raises(EmbeddingError) as exc_info:
        manager.embed(obj)
    assert "Embedding generation failed" in str(exc_info.value)
    assert obj.embedding is None  # Object should not be modified on failure
    
    # Test batch with mixed success/failure
    objects = [
        MockEmbeddableObject(id="success1", content="This should work"),
        MockEmbeddableObject(id="fail1", content=fail_content),
        MockEmbeddableObject(id="success2", content="This should also work")
    ]
    
    # Batch should fail if any object fails
    with pytest.raises(EmbeddingError) as exc_info:
        manager.embed_batch(objects)
    
    # Verify failure count
    assert manager.failure_count == 2  # One from single, one from batch
    
    # Cache should still work for successful content
    success_obj = MockEmbeddableObject(id="success3", content="This should work")
    embedded_obj = manager.embed(success_obj)
    assert embedded_obj.embedding is not None


def test_embedding_metrics_collection():
    """Test that performance metrics are properly collected."""
    manager = EmbeddingManager(max_cache_size=2)
    
    # Test initialization metrics
    init_metrics = manager.metrics.analyze_performance()
    assert "embedding_manager_init" in str(init_metrics)
    
    # Test single embedding metrics
    obj = MockEmbeddableObject(id="test1", content="Test content")
    manager.embed(obj)
    
    # Test cache hit metrics
    manager.embed(obj)  # Should be a cache hit
    
    # Test cache eviction metrics
    for i in range(3):  # Should cause eviction
        manager.embed(MockEmbeddableObject(id=f"test{i+2}", content=f"Content {i}"))
    
    # Test batch metrics
    batch_objects = [
        MockEmbeddableObject(id=f"batch{i}", content=f"Batch content {i}")
        for i in range(5)
    ]
    manager.embed_batch(batch_objects)
    
    # Analyze collected metrics
    metrics = manager.metrics.analyze_performance()
    
    # Verify cache performance metrics
    cache_stats = metrics.get("cache_hit_rate", {})
    assert "embedding" in cache_stats
    assert cache_stats["embedding"] > 0  # Should have some cache hits
    
    # Verify latency metrics
    latency_stats = metrics.get("latency_stats", {})
    assert "embedding_generation_time" in str(latency_stats)
    assert "batch_processing_time" in str(latency_stats)
    
    # Verify memory metrics
    memory_stats = metrics.get("memory_profile", {})
    assert memory_stats.get("current", 0) > 0
    assert memory_stats.get("peak", 0) > 0


# TODO: Add methods for:
# - Connecting to embedding service (OpenAI)
