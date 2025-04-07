import pytest
from unittest.mock import MagicMock, patch

from babylon.rag.pre_embeddings.preprocessor import ContentPreprocessor, PreprocessingConfig
from babylon.rag.exceptions import PreprocessingError


class TestContentPreprocessor:
    """Test suite for ContentPreprocessor."""

    def test_basic_whitespace_normalization(self):
        """Test that whitespace is properly normalized."""
        preprocessor = ContentPreprocessor()
        normalized = preprocessor.preprocess("  Multiple   spaces\nand\tTabs  ")
        assert normalized == "Multiple spaces and Tabs"

    def test_empty_content(self):
        """Test that empty content raises an error."""
        preprocessor = ContentPreprocessor()
        with pytest.raises(PreprocessingError) as exc_info:
            preprocessor.preprocess("")
        assert "RAG_401" in str(exc_info.value)

    def test_content_length_validation(self):
        """Test that content length is validated."""
        config = PreprocessingConfig(min_content_length=10, max_content_length=20)
        preprocessor = ContentPreprocessor(config)
        
        with pytest.raises(PreprocessingError) as exc_info:
            preprocessor.preprocess("Short")
        assert "RAG_401" in str(exc_info.value)
        
        with pytest.raises(PreprocessingError) as exc_info:
            preprocessor.preprocess("This content is way too long for our configuration")
        assert "RAG_402" in str(exc_info.value)

    def test_case_normalization(self):
        """Test that case normalization works when configured."""
        config = PreprocessingConfig(normalize_case=True)
        preprocessor = ContentPreprocessor(config)
        normalized = preprocessor.preprocess("Mixed CASE text")
        assert normalized == "mixed case text"
        
        preprocessor = ContentPreprocessor()
        normalized = preprocessor.preprocess("Mixed CASE text")
        assert normalized == "Mixed CASE text"

    def test_special_character_handling(self):
        """Test that special characters are handled correctly."""
        config = PreprocessingConfig(remove_special_chars=True)
        preprocessor = ContentPreprocessor(config)
        processed = preprocessor.preprocess("Text with special chars: é, ñ, ü")
        assert "é" not in processed
        assert "ñ" not in processed
        assert "ü" not in processed

    def test_metrics_collection(self):
        """Test that metrics are collected during preprocessing."""
        mock_metrics = MagicMock()
        
        preprocessor = ContentPreprocessor()
        preprocessor.metrics = mock_metrics
        
        preprocessor.preprocess("Test content")
        
        mock_metrics.record_metric.assert_called()

    def test_batch_processing(self):
        """Test that batch processing works correctly."""
        preprocessor = ContentPreprocessor()
        contents = ["  Text 1  ", "  Text 2  "]
        processed = preprocessor.preprocess_batch(contents)
        
        assert len(processed) == 2
        assert processed[0] == "Text 1"
        assert processed[1] == "Text 2"
