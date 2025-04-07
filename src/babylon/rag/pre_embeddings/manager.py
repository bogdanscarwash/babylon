"""Pre-embeddings management for the RAG system.

This module provides the PreEmbeddingsManager which integrates preprocessing,
chunking, and caching components to prepare content for embedding.
"""

import time
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Union

from babylon.metrics.collector import MetricsCollector
from babylon.rag.exceptions import PreEmbeddingError
from babylon.rag.pre_embeddings.preprocessor import ContentPreprocessor, PreprocessingConfig
from babylon.rag.pre_embeddings.chunking import ChunkingStrategy, ChunkingConfig
from babylon.rag.pre_embeddings.cache_manager import EmbeddingCacheManager, CacheConfig


@dataclass
class PreEmbeddingsConfig:
    """Configuration for the pre-embeddings system.
    
    Attributes:
        preprocessing_config: Configuration for content preprocessing
        chunking_config: Configuration for content chunking
        cache_config: Configuration for embedding cache management
    """
    
    preprocessing_config: Optional[PreprocessingConfig] = None
    chunking_config: Optional[ChunkingConfig] = None
    cache_config: Optional[CacheConfig] = None


class PreEmbeddingsManager:
    """Manages the pre-embeddings pipeline for the RAG system.
    
    This class integrates preprocessing, chunking, and caching components
    to prepare content for embedding generation.
    """
    
    def __init__(
        self,
        config: Optional[PreEmbeddingsConfig] = None,
        preprocessor: Optional[ContentPreprocessor] = None,
        chunker: Optional[ChunkingStrategy] = None,
        cache_manager: Optional[EmbeddingCacheManager] = None,
        lifecycle_manager: Optional[Any] = None,
    ):
        """Initialize with configuration and optional component instances.
        
        Args:
            config: Configuration for the pre-embeddings system
            preprocessor: Custom preprocessor instance
            chunker: Custom chunker instance
            cache_manager: Custom cache manager instance
            lifecycle_manager: Lifecycle manager for object state tracking
        """
        self.config = config or PreEmbeddingsConfig()
        
        self.preprocessor = preprocessor or ContentPreprocessor(
            self.config.preprocessing_config
        )
        self.chunker = chunker or ChunkingStrategy(
            self.config.chunking_config
        )
        self.cache_manager = cache_manager or EmbeddingCacheManager(
            self.config.cache_config
        )
        
        self.lifecycle_manager = lifecycle_manager
        
        self.metrics = MetricsCollector()
    
    def process_content(self, content: str) -> List[Dict[str, Any]]:
        """Process a single content item through the pre-embeddings pipeline.
        
        Args:
            content: Raw content to process
            
        Returns:
            List of processed chunks with metadata
            
        Raises:
            PreEmbeddingError: If processing fails at any stage
        """
        start_time = time.time()
        
        try:
            preprocessed_content = self.preprocessor.preprocess(content)
            
            chunks = self.chunker.chunk(preprocessed_content)
            
            processed_chunks = []
            for chunk in chunks:
                chunk_hash = self.cache_manager.hash_content(chunk)
                
                cached_embedding = self.cache_manager.get_from_cache(chunk_hash)
                
                chunk_data = {
                    "content": chunk,
                    "content_hash": chunk_hash,
                    "from_cache": cached_embedding is not None,
                }
                
                if cached_embedding:
                    chunk_data["embedding"] = cached_embedding
                
                processed_chunks.append(chunk_data)
            
            processing_time = time.time() - start_time
            self.metrics.record_metric(
                name="pre_embeddings_processing_time",
                value=processing_time,
                context=f"content_length={len(content)},chunks={len(chunks)}"
            )
            
            return processed_chunks
            
        except Exception as e:
            self.metrics.record_metric(
                name="pre_embeddings_error",
                value=1,
                context=f"error={str(e)[:100]}"
            )
            
            raise PreEmbeddingError(
                f"Pre-embeddings processing failed: {str(e)}",
                error_code="RAG_450"
            ) from e
    
    def process_batch(self, contents: List[str]) -> List[List[Dict[str, Any]]]:
        """Process multiple content items efficiently.
        
        Args:
            contents: List of raw content items to process
            
        Returns:
            List of lists of processed chunks with metadata
            
        Raises:
            PreEmbeddingError: If batch processing fails
        """
        start_time = time.time()
        
        try:
            preprocessed_contents = self.preprocessor.preprocess_batch(contents)
            
            chunked_contents = self.chunker.chunk_batch(preprocessed_contents)
            
            results = []
            for content_chunks in chunked_contents:
                processed_chunks = []
                for chunk in content_chunks:
                    chunk_hash = self.cache_manager.hash_content(chunk)
                    
                    cached_embedding = self.cache_manager.get_from_cache(chunk_hash)
                    
                    chunk_data = {
                        "content": chunk,
                        "content_hash": chunk_hash,
                        "from_cache": cached_embedding is not None,
                    }
                    
                    if cached_embedding:
                        chunk_data["embedding"] = cached_embedding
                    
                    processed_chunks.append(chunk_data)
                
                results.append(processed_chunks)
            
            batch_time = time.time() - start_time
            total_chunks = sum(len(chunks) for chunks in chunked_contents)
            self.metrics.record_metric(
                name="pre_embeddings_batch_time",
                value=batch_time,
                context=f"batch_size={len(contents)},total_chunks={total_chunks}"
            )
            
            return results
            
        except Exception as e:
            self.metrics.record_metric(
                name="pre_embeddings_batch_error",
                value=1,
                context=f"error={str(e)[:100]}"
            )
            
            raise PreEmbeddingError(
                f"Pre-embeddings batch processing failed: {str(e)}",
                error_code="RAG_451"
            ) from e
    
    def prepare_for_embedding(self, obj: Any) -> Dict[str, Any]:
        """Prepare an object for embedding by processing its content.
        
        This method is designed to work with objects that follow the
        Embeddable protocol from the embedding system.
        
        Args:
            obj: Object with content to prepare for embedding
            
        Returns:
            Dictionary with processed content and metadata
            
        Raises:
            PreEmbeddingError: If preparation fails
        """
        if not hasattr(obj, "content") or not obj.content:
            raise PreEmbeddingError(
                "Object must have non-empty content attribute",
                error_code="RAG_452"
            )
        
        if self.lifecycle_manager and hasattr(obj, "id"):
            try:
                obj = self.lifecycle_manager.get_object(obj.id)
            except Exception as e:
                self.metrics.record_metric(
                    name="lifecycle_manager_error",
                    value=1,
                    context=f"error={str(e)[:100]}"
                )
        
        processed_chunks = self.process_content(obj.content)
        
        return {
            "object_id": getattr(obj, "id", None),
            "chunks": processed_chunks,
            "chunk_count": len(processed_chunks),
        }
    
    def prepare_batch_for_embedding(self, objects: List[Any]) -> List[Dict[str, Any]]:
        """Prepare multiple objects for embedding by processing their content.
        
        Args:
            objects: List of objects with content to prepare
            
        Returns:
            List of dictionaries with processed content and metadata
            
        Raises:
            PreEmbeddingError: If batch preparation fails
        """
        contents = []
        for obj in objects:
            if not hasattr(obj, "content") or not obj.content:
                raise PreEmbeddingError(
                    "All objects must have non-empty content attribute",
                    error_code="RAG_452"
                )
            contents.append(obj.content)
        
        processed_batches = self.process_batch(contents)
        
        results = []
        for i, obj in enumerate(objects):
            results.append({
                "object_id": getattr(obj, "id", None),
                "chunks": processed_batches[i],
                "chunk_count": len(processed_batches[i]),
            })
        
        return results
    
    def get_stats(self) -> Dict[str, Any]:
        """Get statistics about the pre-embeddings system.
        
        Returns:
            Dictionary of statistics
        """
        return {
            "cache_stats": self.cache_manager.get_cache_stats(),
            "component_versions": {
                "preprocessor": getattr(self.preprocessor, "__version__", "1.0.0"),
                "chunker": getattr(self.chunker, "__version__", "1.0.0"),
                "cache_manager": getattr(self.cache_manager, "__version__", "1.0.0"),
            }
        }
