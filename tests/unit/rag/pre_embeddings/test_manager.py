import pytest
from unittest.mock import MagicMock, patch

from babylon.rag.pre_embeddings.manager import PreEmbeddingsManager, PreEmbeddingsConfig
from babylon.rag.pre_embeddings.preprocessor import ContentPreprocessor
from babylon.rag.pre_embeddings.chunking import ChunkingStrategy
from babylon.rag.pre_embeddings.cache_manager import EmbeddingCacheManager
from babylon.rag.exceptions import PreEmbeddingError


class TestPreEmbeddingsManager:
    """Test suite for PreEmbeddingsManager."""

    def test_initialization(self):
        """Test that the manager initializes with default components."""
        manager = PreEmbeddingsManager()
        
        assert isinstance(manager.preprocessor, ContentPreprocessor)
        assert isinstance(manager.chunker, ChunkingStrategy)
        assert isinstance(manager.cache_manager, EmbeddingCacheManager)

    def test_initialization_with_custom_components(self):
        """Test that the manager initializes with custom components."""
        preprocessor = MagicMock(spec=ContentPreprocessor)
        chunker = MagicMock(spec=ChunkingStrategy)
        cache_manager = MagicMock(spec=EmbeddingCacheManager)
        
        manager = PreEmbeddingsManager(
            preprocessor=preprocessor,
            chunker=chunker,
            cache_manager=cache_manager
        )
        
        assert manager.preprocessor is preprocessor
        assert manager.chunker is chunker
        assert manager.cache_manager is cache_manager

    def test_process_content(self):
        """Test that content is processed through the pipeline."""
        preprocessor = MagicMock(spec=ContentPreprocessor)
        preprocessor.preprocess.return_value = "preprocessed content"
        
        chunker = MagicMock(spec=ChunkingStrategy)
        chunker.chunk.return_value = ["chunk1", "chunk2"]
        
        cache_manager = MagicMock(spec=EmbeddingCacheManager)
        cache_manager.hash_content.return_value = "content_hash"
        cache_manager.get_from_cache.return_value = None
        
        manager = PreEmbeddingsManager(
            preprocessor=preprocessor,
            chunker=chunker,
            cache_manager=cache_manager
        )
        
        result = manager.process_content("raw content")
        
        preprocessor.preprocess.assert_called_once_with("raw content")
        chunker.chunk.assert_called_once_with("preprocessed content")
        cache_manager.hash_content.assert_called()
        
        assert len(result) == 2
        assert result[0]["content"] == "chunk1"
        assert result[1]["content"] == "chunk2"
        assert "content_hash" in result[0]
        assert "content_hash" in result[1]

    def test_process_content_with_cache_hit(self):
        """Test that cached embeddings are used when available."""
        preprocessor = MagicMock(spec=ContentPreprocessor)
        preprocessor.preprocess.return_value = "preprocessed content"
        
        chunker = MagicMock(spec=ChunkingStrategy)
        chunker.chunk.return_value = ["chunk1"]
        
        cache_manager = MagicMock(spec=EmbeddingCacheManager)
        cache_manager.hash_content.return_value = "content_hash"
        cache_manager.get_from_cache.return_value = [0.1, 0.2, 0.3]
        
        manager = PreEmbeddingsManager(
            preprocessor=preprocessor,
            chunker=chunker,
            cache_manager=cache_manager
        )
        
        result = manager.process_content("raw content")
        
        cache_manager.get_from_cache.assert_called_with("content_hash")
        
        assert result[0]["embedding"] == [0.1, 0.2, 0.3]
        assert result[0]["from_cache"] is True

    def test_process_batch(self):
        """Test that batch processing works correctly."""
        preprocessor = MagicMock(spec=ContentPreprocessor)
        preprocessor.preprocess_batch.return_value = ["preprocessed1", "preprocessed2"]
        
        chunker = MagicMock(spec=ChunkingStrategy)
        chunker.chunk_batch.return_value = [["chunk1"], ["chunk2", "chunk3"]]
        
        cache_manager = MagicMock(spec=EmbeddingCacheManager)
        cache_manager.hash_content.side_effect = lambda x: f"hash_{x}"
        cache_manager.get_from_cache.return_value = None
        
        manager = PreEmbeddingsManager(
            preprocessor=preprocessor,
            chunker=chunker,
            cache_manager=cache_manager
        )
        
        results = manager.process_batch(["content1", "content2"])
        
        preprocessor.preprocess_batch.assert_called_once_with(["content1", "content2"])
        chunker.chunk_batch.assert_called_once_with(["preprocessed1", "preprocessed2"])
        
        assert len(results) == 2
        assert len(results[0]) == 1  # First content has 1 chunk
        assert len(results[1]) == 2  # Second content has 2 chunks

    def test_error_handling(self):
        """Test that errors are properly handled and propagated."""
        preprocessor = MagicMock(spec=ContentPreprocessor)
        preprocessor.preprocess.side_effect = Exception("Preprocessing failed")
        
        manager = PreEmbeddingsManager(preprocessor=preprocessor)
        
        with pytest.raises(PreEmbeddingError) as exc_info:
            manager.process_content("raw content")
        
        assert "RAG_450" in str(exc_info.value)
        assert "Preprocessing failed" in str(exc_info.value)

    def test_metrics_collection(self):
        """Test that metrics are collected during processing."""
        manager = PreEmbeddingsManager()
        manager.metrics = MagicMock()
        
        manager.process_content("test content")
        
        manager.metrics.record_metric.assert_called()

    def test_integration_with_lifecycle_manager(self):
        """Test integration with the lifecycle manager."""
        lifecycle_manager = MagicMock()
        
        mock_obj = MagicMock()
        mock_obj.content = "test content"
        mock_obj.id = "test_id"
        lifecycle_manager.get_object.return_value = mock_obj
        
        manager = PreEmbeddingsManager(lifecycle_manager=lifecycle_manager)
        
        embeddable = MagicMock()
        embeddable.content = "test content"
        embeddable.id = "test_id"
        
        manager.prepare_for_embedding(embeddable)
        
        lifecycle_manager.get_object.assert_called_with("test_id")
