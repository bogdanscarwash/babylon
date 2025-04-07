import numpy as np
import pytest
from .sample_embeddings import EMBEDDING_DIMENSION, BASE_EMBEDDING, BATCH_SIZE, TOTAL_ENTITIES

def test_embedding_dimension():
    """Test if embedding dimension matches expected value"""
    assert EMBEDDING_DIMENSION == 384
    assert BASE_EMBEDDING.shape == (EMBEDDING_DIMENSION,)

def test_base_embedding_properties():
    """Test properties of base embedding"""
    assert isinstance(BASE_EMBEDDING, np.ndarray)
    assert np.all((BASE_EMBEDDING >= 0) & (BASE_EMBEDDING <= 1))

def test_batch_size():
    """Test if batch size is positive"""
    assert BATCH_SIZE > 0
    assert isinstance(BATCH_SIZE, int)

def test_total_entities():
    """Test if total entities is positive and greater than batch size"""
    assert TOTAL_ENTITIES > 0
    assert TOTAL_ENTITIES >= BATCH_SIZE
    assert isinstance(TOTAL_ENTITIES, int)
