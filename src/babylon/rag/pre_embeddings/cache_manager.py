"""Embedding cache management for the RAG system.

This module provides functionality for caching embeddings to reduce
duplicate operations and API costs.
"""

import hashlib
import pickle
import time
from collections import OrderedDict
from dataclasses import dataclass
from typing import Dict, List, Optional, Any

from babylon.metrics.collector import MetricsCollector
from babylon.rag.exceptions import CacheError


@dataclass
class CacheConfig:
    """Configuration for embedding cache management.
    
    Attributes:
        max_cache_size: Maximum number of embeddings to keep in cache
        persist_cache: Whether to persist cache to disk
        cache_file_path: Path to cache file for persistence
        hash_algorithm: Algorithm to use for content hashing
    """
    
    max_cache_size: int = 10000
    persist_cache: bool = False
    cache_file_path: str = "/tmp/embedding_cache.pkl"
    hash_algorithm: str = "sha256"


class EmbeddingCacheManager:
    """Manages caching of embeddings to reduce duplicate operations.
    
    This class handles caching of embeddings based on content hashes,
    with support for LRU eviction, persistence, and metrics collection.
    """
    
    def __init__(self, config: Optional[CacheConfig] = None):
        """Initialize with configuration options.
        
        Args:
            config: Configuration for cache behavior
        """
        self.config = config or CacheConfig()
        self.metrics = MetricsCollector()
        self.cache: Dict[str, List[float]] = OrderedDict()
        
        if self.config.persist_cache:
            self.load_cache()
    
    def hash_content(self, content: str) -> str:
        """Generate a hash for the given content.
        
        Args:
            content: Content to hash
            
        Returns:
            Hash string for the content
        """
        start_time = time.time()
        
        if self.config.hash_algorithm == "sha256":
            hash_obj = hashlib.sha256(content.encode())
        elif self.config.hash_algorithm == "md5":
            hash_obj = hashlib.md5(content.encode())
        else:
            hash_obj = hashlib.sha256(content.encode())
        
        content_hash = hash_obj.hexdigest()
        
        hashing_time = time.time() - start_time
        self.metrics.record_metric(
            name="content_hashing_time",
            value=hashing_time,
            context=f"algorithm={self.config.hash_algorithm},content_length={len(content)}"
        )
        
        return content_hash
    
    def get_from_cache(self, content_hash: str) -> Optional[List[float]]:
        """Retrieve an embedding from cache by content hash.
        
        Args:
            content_hash: Hash of the content to retrieve embedding for
            
        Returns:
            Cached embedding or None if not found
        """
        start_time = time.time()
        
        if content_hash in self.cache:
            embedding = self.cache.pop(content_hash)
            self.cache[content_hash] = embedding
            
            self.metrics.record_metric(
                name="cache_hit",
                value=1,
                context=f"hash={content_hash[:8]}"
            )
            
            lookup_time = time.time() - start_time
            self.metrics.record_metric(
                name="cache_lookup_time",
                value=lookup_time,
                context="result=hit"
            )
            
            return embedding
        else:
            self.metrics.record_metric(
                name="cache_miss",
                value=1,
                context=f"hash={content_hash[:8]}"
            )
            
            lookup_time = time.time() - start_time
            self.metrics.record_metric(
                name="cache_lookup_time",
                value=lookup_time,
                context="result=miss"
            )
            
            return None
    
    def add_to_cache(self, content_hash: str, embedding: List[float]) -> None:
        """Add or update an embedding in the cache.
        
        Args:
            content_hash: Hash of the content
            embedding: Embedding vector to cache
        """
        start_time = time.time()
        
        is_update = content_hash in self.cache
        
        if is_update:
            self.cache.pop(content_hash)
        
        self.cache[content_hash] = embedding
        
        if len(self.cache) > self.config.max_cache_size:
            oldest_key = next(iter(self.cache))
            self.cache.pop(oldest_key)
            
            self.metrics.record_metric(
                name="cache_eviction",
                value=1,
                context=f"max_size={self.config.max_cache_size}"
            )
        
        operation = "update" if is_update else "add"
        self.metrics.record_metric(
            name="cache_operation",
            value=1,
            context=f"operation={operation},hash={content_hash[:8]}"
        )
        
        operation_time = time.time() - start_time
        self.metrics.record_metric(
            name="cache_operation_time",
            value=operation_time,
            context=f"operation={operation}"
        )
        
        if self.config.persist_cache:
            self.save_cache()
    
    def clear_cache(self) -> None:
        """Clear all entries from the cache."""
        start_time = time.time()
        
        cache_size = len(self.cache)
        self.cache.clear()
        
        clear_time = time.time() - start_time
        self.metrics.record_metric(
            name="cache_clear_time",
            value=clear_time,
            context=f"items_cleared={cache_size}"
        )
        
        if self.config.persist_cache:
            self.save_cache()
    
    def save_cache(self) -> None:
        """Save the cache to disk if persistence is enabled."""
        if not self.config.persist_cache:
            return
        
        start_time = time.time()
        
        try:
            with open(self.config.cache_file_path, "wb") as f:
                pickle.dump(dict(self.cache), f)
            
            save_time = time.time() - start_time
            self.metrics.record_metric(
                name="cache_save_time",
                value=save_time,
                context=f"items={len(self.cache)},path={self.config.cache_file_path}"
            )
        except Exception as e:
            error_msg = f"Failed to save cache to {self.config.cache_file_path}: {str(e)}"
            self.metrics.record_metric(
                name="cache_save_error",
                value=1,
                context=error_msg[:100]
            )
            raise CacheError(error_msg, error_code="RAG_442", cache_key=None)
    
    def load_cache(self) -> None:
        """Load the cache from disk if persistence is enabled."""
        if not self.config.persist_cache:
            return
        
        start_time = time.time()
        
        try:
            try:
                with open(self.config.cache_file_path, "rb") as f:
                    loaded_cache = pickle.load(f)
                
                self.cache = OrderedDict(loaded_cache)
                
                load_time = time.time() - start_time
                self.metrics.record_metric(
                    name="cache_load_time",
                    value=load_time,
                    context=f"items={len(self.cache)},path={self.config.cache_file_path}"
                )
            except FileNotFoundError:
                self.cache = OrderedDict()
                self.metrics.record_metric(
                    name="cache_load_info",
                    value=1,
                    context="no_cache_file_found"
                )
        except Exception as e:
            error_msg = f"Failed to load cache from {self.config.cache_file_path}: {str(e)}"
            self.metrics.record_metric(
                name="cache_load_error",
                value=1,
                context=error_msg[:100]
            )
            self.cache = OrderedDict()
            raise CacheError(error_msg, error_code="RAG_443", cache_key=None)
    
    def get_cache_stats(self) -> Dict[str, Any]:
        """Get statistics about the current cache state.
        
        Returns:
            Dictionary of cache statistics
        """
        return {
            "size": len(self.cache),
            "max_size": self.config.max_cache_size,
            "utilization": len(self.cache) / self.config.max_cache_size if self.config.max_cache_size > 0 else 0,
            "persistence_enabled": self.config.persist_cache,
            "cache_file_path": self.config.cache_file_path if self.config.persist_cache else None,
        }
