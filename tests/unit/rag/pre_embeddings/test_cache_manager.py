import pytest
from unittest.mock import MagicMock, patch
import time

from babylon.rag.pre_embeddings.cache_manager import EmbeddingCacheManager, CacheConfig
from babylon.rag.exceptions import CacheError


class TestEmbeddingCacheManager:
    """Test suite for EmbeddingCacheManager."""

    def test_cache_hit(self):
        """Test that cache hits return the cached embedding."""
        cache_manager = EmbeddingCacheManager()
        
        content_hash = "test_hash"
        embedding = [0.1, 0.2, 0.3]
        cache_manager.add_to_cache(content_hash, embedding)
        
        cached_embedding = cache_manager.get_from_cache(content_hash)
        
        assert cached_embedding == embedding

    def test_cache_miss(self):
        """Test that cache misses return None."""
        cache_manager = EmbeddingCacheManager()
        
        cached_embedding = cache_manager.get_from_cache("nonexistent_hash")
        
        assert cached_embedding is None

    def test_cache_eviction(self):
        """Test that items are evicted from cache when it reaches capacity."""
        config = CacheConfig(max_cache_size=2)
        cache_manager = EmbeddingCacheManager(config)
        
        cache_manager.add_to_cache("hash1", [0.1, 0.2])
        cache_manager.add_to_cache("hash2", [0.3, 0.4])
        
        assert cache_manager.get_from_cache("hash1") is not None
        assert cache_manager.get_from_cache("hash2") is not None
        
        cache_manager.add_to_cache("hash3", [0.5, 0.6])
        
        assert cache_manager.get_from_cache("hash1") is None
        assert cache_manager.get_from_cache("hash2") is not None
        assert cache_manager.get_from_cache("hash3") is not None

    def test_cache_update(self):
        """Test that updating an existing cache entry works correctly."""
        cache_manager = EmbeddingCacheManager()
        
        content_hash = "test_hash"
        embedding = [0.1, 0.2, 0.3]
        cache_manager.add_to_cache(content_hash, embedding)
        
        new_embedding = [0.4, 0.5, 0.6]
        cache_manager.add_to_cache(content_hash, new_embedding)
        
        cached_embedding = cache_manager.get_from_cache(content_hash)
        
        assert cached_embedding == new_embedding

    def test_cache_clear(self):
        """Test that clearing the cache removes all items."""
        cache_manager = EmbeddingCacheManager()
        
        cache_manager.add_to_cache("hash1", [0.1, 0.2])
        cache_manager.add_to_cache("hash2", [0.3, 0.4])
        
        cache_manager.clear_cache()
        
        assert cache_manager.get_from_cache("hash1") is None
        assert cache_manager.get_from_cache("hash2") is None

    def test_content_hashing(self):
        """Test that content hashing works correctly."""
        cache_manager = EmbeddingCacheManager()
        
        content1 = "This is a test content"
        content2 = "This is a test content"
        content3 = "This is different content"
        
        hash1 = cache_manager.hash_content(content1)
        hash2 = cache_manager.hash_content(content2)
        hash3 = cache_manager.hash_content(content3)
        
        assert hash1 == hash2
        assert hash1 != hash3

    def test_metrics_collection(self):
        """Test that metrics are collected during cache operations."""
        mock_metrics = MagicMock()
        
        cache_manager = EmbeddingCacheManager()
        cache_manager.metrics = mock_metrics
        
        cache_manager.add_to_cache("test_hash", [0.1, 0.2])
        
        cache_manager.get_from_cache("test_hash")
        
        cache_manager.get_from_cache("nonexistent_hash")
        
        mock_metrics.record_metric.assert_called()

    def test_cache_persistence(self):
        """Test that cache persistence works correctly."""
        config = CacheConfig(persist_cache=True, cache_file_path="/tmp/test_cache.pkl")
        cache_manager = EmbeddingCacheManager(config)
        
        cache_manager.add_to_cache("hash1", [0.1, 0.2])
        cache_manager.add_to_cache("hash2", [0.3, 0.4])
        
        cache_manager.save_cache()
        
        new_cache_manager = EmbeddingCacheManager(config)
        new_cache_manager.load_cache()
        
        assert new_cache_manager.get_from_cache("hash1") == [0.1, 0.2]
        assert new_cache_manager.get_from_cache("hash2") == [0.3, 0.4]
