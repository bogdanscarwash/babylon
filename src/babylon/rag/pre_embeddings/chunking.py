"""Content chunking for the RAG system.

This module provides functionality for dividing content into appropriate chunks
before embedding generation.
"""

import re
import time
from dataclasses import dataclass
from typing import List, Optional

from babylon.metrics.collector import MetricsCollector
from babylon.rag.exceptions import ChunkingError


@dataclass
class ChunkingConfig:
    """Configuration for content chunking.
    
    Attributes:
        strategy: Chunking strategy to use ("fixed" or "semantic")
        chunk_size: Size of chunks in characters (for fixed strategy)
        overlap: Number of characters to overlap between chunks (for fixed strategy)
        delimiter: Delimiter to use for semantic chunking
        min_chunk_size: Minimum allowed chunk size
        max_chunk_size: Maximum allowed chunk size
    """
    
    strategy: str = "fixed"
    chunk_size: int = 1000
    overlap: int = 0
    delimiter: str = "\n\n"
    min_chunk_size: int = 10
    max_chunk_size: int = 2000


class ChunkingStrategy:
    """Divides content into appropriate chunks for embedding.
    
    This class handles different chunking strategies including fixed-size
    chunking and semantic chunking based on content structure.
    """
    
    def __init__(self, config: Optional[ChunkingConfig] = None):
        """Initialize with configuration options.
        
        Args:
            config: Configuration for chunking behavior
        """
        self.config = config or ChunkingConfig()
        self.metrics = MetricsCollector()
    
    def chunk(self, content: str) -> List[str]:
        """Divide content into chunks based on configured strategy.
        
        Args:
            content: Content to chunk
            
        Returns:
            List of content chunks
            
        Raises:
            ChunkingError: If chunking fails or content is invalid
        """
        start_time = time.time()
        
        if not content:
            raise ChunkingError(
                "Cannot chunk empty content",
                error_code="RAG_421"
            )
        
        if self.config.strategy == "fixed":
            chunks = self._fixed_size_chunking(content)
        elif self.config.strategy == "semantic":
            chunks = self._semantic_chunking(content)
        else:
            raise ChunkingError(
                f"Unsupported chunking strategy: {self.config.strategy}",
                error_code="RAG_423"
            )
        
        chunking_time = time.time() - start_time
        self.metrics.record_metric(
            name="chunking_time",
            value=chunking_time,
            context=f"strategy={self.config.strategy},content_length={len(content)}"
        )
        
        self.metrics.record_metric(
            name="chunk_count",
            value=len(chunks),
            context=f"strategy={self.config.strategy},content_length={len(content)}"
        )
        
        if chunks:
            avg_chunk_size = sum(len(chunk) for chunk in chunks) / len(chunks)
            self.metrics.record_metric(
                name="avg_chunk_size",
                value=avg_chunk_size,
                context=f"strategy={self.config.strategy}"
            )
        
        return chunks
    
    def chunk_batch(self, contents: List[str]) -> List[List[str]]:
        """Process multiple content items efficiently.
        
        Args:
            contents: List of content items to chunk
            
        Returns:
            List of lists of chunks, one list per content item
        """
        start_time = time.time()
        
        chunked_contents = [self.chunk(content) for content in contents]
        
        batch_time = time.time() - start_time
        self.metrics.record_metric(
            name="batch_chunking_time",
            value=batch_time,
            context=f"batch_size={len(contents)}"
        )
        
        return chunked_contents
    
    def _fixed_size_chunking(self, content: str) -> List[str]:
        """Chunk content into fixed-size pieces with optional overlap.
        
        Args:
            content: Content to chunk
            
        Returns:
            List of content chunks
        """
        chunks = []
        content_length = len(content)
        
        if content_length <= self.config.chunk_size:
            return [content]
        
        step_size = self.config.chunk_size - self.config.overlap
        if step_size <= 0:
            raise ChunkingError(
                f"Invalid configuration: overlap ({self.config.overlap}) must be less than chunk_size ({self.config.chunk_size})",
                error_code="RAG_422"
            )
        
        position = 0
        while position < content_length:
            end_position = min(position + self.config.chunk_size, content_length)
            chunk = content[position:end_position]
            
            if chunk:
                chunks.append(chunk)
            
            position += step_size
            
            if position >= content_length and end_position >= content_length:
                break
        
        return chunks
    
    def _semantic_chunking(self, content: str) -> List[str]:
        """Chunk content based on semantic boundaries.
        
        Args:
            content: Content to chunk
            
        Returns:
            List of content chunks
        """
        chunks = [chunk.strip() for chunk in content.split(self.config.delimiter)]
        
        chunks = [chunk for chunk in chunks if chunk]
        
        if not chunks:
            return [content]
        
        for i, chunk in enumerate(chunks):
            if len(chunk) < self.config.min_chunk_size:
                if i < len(chunks) - 1:
                    chunks[i+1] = chunk + " " + chunks[i+1]
                    chunks[i] = ""
                elif i > 0:
                    chunks[i-1] = chunks[i-1] + " " + chunk
                    chunks[i] = ""
            elif len(chunk) > self.config.max_chunk_size:
                sub_chunks = self._fixed_size_chunking(chunk)
                chunks[i] = sub_chunks[0]
                chunks.extend(sub_chunks[1:])
        
        chunks = [chunk for chunk in chunks if chunk]
        
        return chunks
