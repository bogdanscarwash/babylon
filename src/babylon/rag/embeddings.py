"""Embedding management for the RAG system."""

from typing import List, Dict, Any, Optional, Protocol
import numpy as np
import logging
import time
import asyncio
import aiohttp
from concurrent.futures import ThreadPoolExecutor
import threading
from dataclasses import dataclass
import hashlib
from collections import OrderedDict
import backoff
from ratelimit import limits, sleep_and_retry
from babylon.metrics.collector import MetricsCollector
from babylon.config.openai_config import OpenAIConfig

logger = logging.getLogger(__name__)


class EmbeddingError(Exception):
    """Exception raised for embedding service failures."""

    pass


class OpenAIError(EmbeddingError):
    """Exception raised for OpenAI API-specific errors."""

    pass


class Embeddable(Protocol):
    """Protocol for objects that can be embedded."""

    id: str
    content: str
    embedding: Optional[List[float]]


class EmbeddingManager:
    """Manages embeddings for RAG objects.

    The EmbeddingManager handles:
    - Generating embeddings via OpenAI API
    - Caching embeddings for reuse with LRU eviction
    - Rate limiting and retry logic
    - Batch operations for efficiency
    - Error handling and recovery
    - Performance metrics collection
    - Concurrent operations
    """

    def __init__(
        self,
        embedding_dimension: Optional[int] = None,
        batch_size: Optional[int] = None,
        max_cache_size: int = 1000,
        max_concurrent_requests: int = 4,
    ):
        """Initialize the embedding manager.

        Args:
            embedding_dimension: Size of embedding vectors (default: from OpenAIConfig)
            batch_size: Number of objects to embed in each batch (default: from OpenAIConfig)
            max_cache_size: Maximum number of embeddings to keep in cache (default: 1000)
            max_concurrent_requests: Maximum number of concurrent embedding requests (default: 4)

        Raises:
            ValueError: If a custom embedding dimension is provided (not supported with OpenAI API)
        """
        # Validate OpenAI configuration
        OpenAIConfig.validate()

        # Custom dimensions are not supported with OpenAI API
        if (
            embedding_dimension is not None
            and embedding_dimension != OpenAIConfig.get_model_dimensions()
        ):
            raise ValueError(
                f"Custom embedding dimensions are not supported. The OpenAI API returns fixed-size embeddings "
                f"({OpenAIConfig.get_model_dimensions()} dimensions for {OpenAIConfig.EMBEDDING_MODEL})"
            )

        self.embedding_dimension = OpenAIConfig.get_model_dimensions()
        self.batch_size = batch_size or OpenAIConfig.BATCH_SIZE
        self.max_cache_size = max_cache_size
        self.max_concurrent_requests = max_concurrent_requests

        # Use OrderedDict for LRU cache implementation
        self._cache: OrderedDict[str, List[float]] = OrderedDict()
        self._cache_lock = threading.Lock()

        # Thread pool for concurrent operations
        self._thread_pool = ThreadPoolExecutor(max_workers=max_concurrent_requests)

        # Semaphore for limiting concurrent requests
        self._request_semaphore = asyncio.Semaphore(max_concurrent_requests)

        # HTTP session for API requests
        self._session: Optional[aiohttp.ClientSession] = None

        # Initialize metrics collector
        self.metrics = MetricsCollector()

        # Record initialization metrics
        self.metrics.record_metric(
            name="embedding_manager_init",
            value=1.0,
            context=f"dim={self.embedding_dimension},batch={self.batch_size},cache={max_cache_size},concurrent={max_concurrent_requests}",
        )

    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create HTTP session for API requests."""
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                headers=OpenAIConfig.get_headers(),
                timeout=aiohttp.ClientTimeout(total=OpenAIConfig.REQUEST_TIMEOUT),
            )
        return self._session

    @property
    def cache_size(self) -> int:
        """Get current number of embeddings in cache."""
        with self._cache_lock:
            return len(self._cache)

    @backoff.on_exception(
        backoff.expo,
        (aiohttp.ClientError, asyncio.TimeoutError),
        max_tries=OpenAIConfig.MAX_RETRIES,
        max_time=30,
    )
    @sleep_and_retry
    @limits(calls=OpenAIConfig.RATE_LIMIT_RPM, period=60)
    async def _generate_embedding_api(self, content: str) -> List[float]:
        """Generate embedding using OpenAI API with retry and rate limiting.

        Args:
            content: Text content to embed

        Returns:
            List[float]: Embedding vector

        Raises:
            OpenAIError: If the API request fails
        """
        session = await self._get_session()

        try:
            async with session.post(
                "https://api.openai.com/v1/embeddings",
                json={"input": content, "model": OpenAIConfig.EMBEDDING_MODEL},
            ) as response:
                if response.status != 200:
                    error_data = await response.json()
                    raise OpenAIError(
                        f"OpenAI API error: {error_data.get('error', {}).get('message', 'Unknown error')}"
                    )

                data = await response.json()
                return data["data"][0]["embedding"]

        except aiohttp.ClientError as e:
            raise OpenAIError(f"API request failed: {str(e)}") from e
        except asyncio.TimeoutError as e:
            raise OpenAIError("API request timed out") from e
        except Exception as e:
            raise OpenAIError(f"Unexpected error: {str(e)}") from e

    async def aembed(self, obj: Embeddable) -> Embeddable:
        """Asynchronously generate and attach embedding for a single object.

        Args:
            obj: Object to embed

        Returns:
            Object with embedding attached

        Raises:
            ValueError: If object content is invalid
            EmbeddingError: If embedding generation fails
        """
        if not obj.content:
            raise ValueError("Cannot embed empty content")

        start_time = time.time()
        cache_key = self._get_cache_key(obj.content)

        # Check cache first
        with self._cache_lock:
            if cache_key in self._cache:
                # Move to end to mark as most recently used
                embedding = self._cache.pop(cache_key)
                self._cache[cache_key] = embedding
                obj.embedding = embedding

                # Record cache hit
                self.metrics.record_cache_event("embedding", hit=True)
                self.metrics.record_metric(
                    name="embedding_cache_lookup_time",
                    value=time.time() - start_time,
                    context="cache_hit",
                    object_id=obj.id,
                )
                return obj

        # Record cache miss
        self.metrics.record_cache_event("embedding", hit=False)

        try:
            # Acquire semaphore to limit concurrent requests
            async with self._request_semaphore:
                # Generate new embedding
                embedding_start = time.time()
                embedding = await self._generate_embedding_api(obj.content)

                # Record embedding generation time
                self.metrics.record_metric(
                    name="embedding_generation_time",
                    value=time.time() - embedding_start,
                    context="generation",
                    object_id=obj.id,
                )

                # Add to cache with LRU eviction if needed
                with self._cache_lock:
                    if self.cache_size >= self.max_cache_size:
                        # Remove least recently used (first item)
                        self._cache.popitem(last=False)
                        self.metrics.record_metric(
                            name="cache_eviction", value=1.0, context="lru_eviction"
                        )

                    self._cache[cache_key] = embedding
                    obj.embedding = embedding

                # Record memory usage
                self.metrics.record_memory_usage(
                    len(self._cache)
                    * self.embedding_dimension
                    * 8  # Approximate bytes used
                )

                # Record total operation time
                self.metrics.record_metric(
                    name="embedding_total_time",
                    value=time.time() - start_time,
                    context="total",
                    object_id=obj.id,
                )

                return obj

        except Exception as e:
            # Record error
            self.metrics.record_metric(
                name="embedding_error", value=1.0, context=str(e), object_id=obj.id
            )
            logger.error(f"Failed to generate embedding for object {obj.id}: {str(e)}")
            raise

    def embed(self, obj: Embeddable) -> Embeddable:
        """Synchronously generate and attach embedding for a single object.

        This is a convenience wrapper around aembed for synchronous code.
        For better performance in async contexts, use aembed directly.

        Args:
            obj: Object to embed

        Returns:
            Object with embedding attached

        Raises:
            ValueError: If object content is invalid
            EmbeddingError: If embedding generation fails
        """
        return asyncio.run(self.aembed(obj))

    async def aembed_batch(self, objects: List[Embeddable]) -> List[Embeddable]:
        """Asynchronously generate embeddings for multiple objects efficiently.

        Args:
            objects: List of objects to embed

        Returns:
            List of objects with embeddings attached

        Raises:
            EmbeddingError: If any object's embedding generation fails
        """
        start_time = time.time()
        results = []
        batch = []

        try:
            for obj in objects:
                if len(batch) >= self.batch_size:
                    batch_start = time.time()
                    # Process batch concurrently
                    batch_results = await asyncio.gather(
                        *[self.aembed(obj) for obj in batch]
                    )
                    results.extend(batch_results)
                    # Record batch processing time
                    self.metrics.record_metric(
                        name="batch_processing_time",
                        value=time.time() - batch_start,
                        context=f"batch_size_{len(batch)}",
                    )
                    batch = []
                batch.append(obj)

            if batch:
                batch_start = time.time()
                # Process remaining batch concurrently
                batch_results = await asyncio.gather(
                    *[self.aembed(obj) for obj in batch]
                )
                results.extend(batch_results)
                self.metrics.record_metric(
                    name="batch_processing_time",
                    value=time.time() - batch_start,
                    context=f"batch_size_{len(batch)}",
                )

            # Record batch metrics
            total_time = time.time() - start_time
            self.metrics.record_metric(
                name="batch_embedding_time",
                value=total_time,
                context=f"batch_size_{len(objects)}",
            )
            self.metrics.record_metric(
                name="batch_throughput",
                value=len(objects) / total_time,
                context="objects_per_second",
            )

            return results

        except Exception as e:
            # Record error
            self.metrics.record_metric(
                name="batch_embedding_error",
                value=1.0,
                context=f"failed_at_{len(results)}_of_{len(objects)}",
            )
            # Log the error and number of successful embeddings
            logger.error(
                f"Batch embedding failed after {len(results)} successful embeddings: {str(e)}"
            )
            raise EmbeddingError(
                f"Batch embedding failed: {str(e)}. {len(results)} objects were successfully embedded."
            ) from e

    def embed_batch(self, objects: List[Embeddable]) -> List[Embeddable]:
        """Synchronously generate embeddings for multiple objects efficiently.

        This is a convenience wrapper around aembed_batch for synchronous code.
        For better performance in async contexts, use aembed_batch directly.

        Args:
            objects: List of objects to embed

        Returns:
            List of objects with embeddings attached

        Raises:
            EmbeddingError: If any object's embedding generation fails
        """
        return asyncio.run(self.aembed_batch(objects))

    def debed(self, obj: Embeddable) -> Embeddable:
        """Remove embedding from an object.

        Args:
            obj: Object to remove embedding from

        Returns:
            Object with embedding removed
        """
        obj.embedding = None
        return obj

    def debed_batch(self, objects: List[Embeddable]) -> List[Embeddable]:
        """Remove embeddings from multiple objects.

        Args:
            objects: List of objects to remove embeddings from

        Returns:
            List of objects with embeddings removed
        """
        return [self.debed(obj) for obj in objects]

    def _get_cache_key(self, content: str) -> str:
        """Generate cache key for content."""
        return hashlib.sha256(content.encode()).hexdigest()

    async def close(self) -> None:
        """Close resources used by the embedding manager."""
        if self._session is not None and not self._session.closed:
            await self._session.close()
        self._thread_pool.shutdown(wait=True)
