"""Content preprocessing for the RAG system.

This module provides functionality for normalizing and preprocessing content
before it is chunked and embedded.
"""

import re
import time
from dataclasses import dataclass
from typing import List, Optional

from babylon.metrics.collector import MetricsCollector
from babylon.rag.exceptions import PreprocessingError


@dataclass
class PreprocessingConfig:
    """Configuration for content preprocessing.
    
    Attributes:
        normalize_whitespace: Whether to normalize whitespace in content
        normalize_case: Whether to convert content to lowercase
        remove_special_chars: Whether to remove special characters
        language_detection: Whether to detect and validate language
        min_content_length: Minimum allowed content length
        max_content_length: Maximum allowed content length
    """
    
    normalize_whitespace: bool = True
    normalize_case: bool = False
    remove_special_chars: bool = False
    language_detection: bool = False
    min_content_length: int = 1
    max_content_length: int = 100000


class ContentPreprocessor:
    """Preprocesses content before chunking and embedding.
    
    This class handles text normalization, validation, and preparation
    for the chunking and embedding processes.
    """
    
    def __init__(self, config: Optional[PreprocessingConfig] = None):
        """Initialize with configuration options.
        
        Args:
            config: Configuration for preprocessing behavior
        """
        self.config = config or PreprocessingConfig()
        self.metrics = MetricsCollector()
    
    def preprocess(self, content: str) -> str:
        """Process raw content into normalized form.
        
        Args:
            content: Raw content to preprocess
            
        Returns:
            Preprocessed content
            
        Raises:
            PreprocessingError: If content validation fails
        """
        start_time = time.time()
        original_length = len(content)
        
        if len(content) < self.config.min_content_length:
            raise PreprocessingError(
                f"Content length ({len(content)}) is below minimum ({self.config.min_content_length})",
                error_code="RAG_401"
            )
        
        if len(content) > self.config.max_content_length:
            raise PreprocessingError(
                f"Content length ({len(content)}) exceeds maximum ({self.config.max_content_length})",
                error_code="RAG_402"
            )
        
        if self.config.normalize_whitespace:
            content = re.sub(r'\s+', ' ', content)
            content = content.strip()
        
        if self.config.normalize_case:
            content = content.lower()
        
        if self.config.remove_special_chars:
            content = re.sub(r'[^\w\s]', '', content)
            content = self._remove_accents(content)
        
        if self.config.language_detection:
            pass
        
        processing_time = time.time() - start_time
        self.metrics.record_metric(
            name="preprocessing_time",
            value=processing_time,
            context=f"content_length={original_length}"
        )
        
        if len(content) != original_length:
            reduction_ratio = len(content) / original_length if original_length > 0 else 1.0
            self.metrics.record_metric(
                name="content_reduction_ratio",
                value=reduction_ratio,
                context=f"original_length={original_length},new_length={len(content)}"
            )
        
        return content
    
    def preprocess_batch(self, contents: List[str]) -> List[str]:
        """Process multiple content items efficiently.
        
        Args:
            contents: List of content items to preprocess
            
        Returns:
            List of preprocessed content items
        """
        start_time = time.time()
        
        processed_contents = [self.preprocess(content) for content in contents]
        
        batch_time = time.time() - start_time
        self.metrics.record_metric(
            name="batch_preprocessing_time",
            value=batch_time,
            context=f"batch_size={len(contents)}"
        )
        
        return processed_contents
    
    def _remove_accents(self, text: str) -> str:
        """Remove accents from text.
        
        Args:
            text: Text to remove accents from
            
        Returns:
            Text with accents removed
        """
        replacements = {
            'á': 'a', 'à': 'a', 'ä': 'a', 'â': 'a', 'ã': 'a',
            'é': 'e', 'è': 'e', 'ë': 'e', 'ê': 'e',
            'í': 'i', 'ì': 'i', 'ï': 'i', 'î': 'i',
            'ó': 'o', 'ò': 'o', 'ö': 'o', 'ô': 'o', 'õ': 'o',
            'ú': 'u', 'ù': 'u', 'ü': 'u', 'û': 'u',
            'ñ': 'n', 'ç': 'c'
        }
        
        for accent, replacement in replacements.items():
            text = text.replace(accent, replacement)
            text = text.replace(accent.upper(), replacement.upper())
        
        return text
