import pytest
from unittest.mock import MagicMock, patch

from babylon.rag.pre_embeddings.chunking import ChunkingStrategy, ChunkingConfig
from babylon.rag.exceptions import ChunkingError


class TestChunkingStrategy:
    """Test suite for ChunkingStrategy."""

    def test_basic_fixed_size_chunking(self):
        """Test that basic fixed-size chunking works correctly."""
        config = ChunkingConfig(chunk_size=10, overlap=0)
        chunker = ChunkingStrategy(config)
        
        content = "This is a test content for chunking"
        chunks = chunker.chunk(content)
        
        assert len(chunks) == 4
        assert chunks[0] == "This is a "
        assert chunks[1] == "test conte"
        assert chunks[2] == "nt for chu"
        assert chunks[3] == "nking"

    def test_chunking_with_overlap(self):
        """Test that chunking with overlap works correctly."""
        config = ChunkingConfig(chunk_size=10, overlap=3)
        chunker = ChunkingStrategy(config)
        
        content = "This is a test content"
        chunks = chunker.chunk(content)
        
        assert len(chunks) == 4
        assert chunks[0] == "This is a "
        assert chunks[1] == " a test co"
        assert chunks[2] == " content"
        assert chunks[3] == "t"

    def test_empty_content(self):
        """Test that empty content raises an error."""
        chunker = ChunkingStrategy()
        
        with pytest.raises(ChunkingError) as exc_info:
            chunker.chunk("")
        assert "RAG_421" in str(exc_info.value)

    def test_content_smaller_than_chunk_size(self):
        """Test that content smaller than chunk size is handled correctly."""
        config = ChunkingConfig(chunk_size=100)
        chunker = ChunkingStrategy(config)
        
        content = "Small content"
        chunks = chunker.chunk(content)
        
        assert len(chunks) == 1
        assert chunks[0] == content

    def test_semantic_chunking(self):
        """Test that semantic chunking works correctly."""
        config = ChunkingConfig(strategy="semantic", delimiter="\n\n")
        chunker = ChunkingStrategy(config)
        
        content = "Paragraph one.\n\nParagraph two.\n\nParagraph three."
        chunks = chunker.chunk(content)
        
        assert len(chunks) == 3
        assert chunks[0] == "Paragraph one."
        assert chunks[1] == "Paragraph two."
        assert chunks[2] == "Paragraph three."

    def test_metrics_collection(self):
        """Test that metrics are collected during chunking."""
        mock_metrics = MagicMock()
        
        chunker = ChunkingStrategy()
        chunker.metrics = mock_metrics
        
        chunker.chunk("Test content for chunking")
        
        mock_metrics.record_metric.assert_called()

    def test_batch_chunking(self):
        """Test that batch chunking works correctly."""
        chunker = ChunkingStrategy()
        
        contents = ["Content one", "Content two"]
        chunked_contents = chunker.chunk_batch(contents)
        
        assert len(chunked_contents) == 2
        assert isinstance(chunked_contents[0], list)
        assert isinstance(chunked_contents[1], list)
