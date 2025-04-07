"""Token counting utilities for Context Window Management."""

from typing import Union, List, Dict, Any


def count_tokens(content: Union[str, List, Dict, Any]) -> int:
    """Count the number of tokens in content of various types.
    
    This is a simple implementation that estimates token counts. For production,
    this should be replaced with a proper tokenizer for the target model.
    
    Args:
        content: Content to count tokens for. Can be string, list, dict, or other type.
        
    Returns:
        Estimated token count
    """
    if isinstance(content, str):
        words = content.split()
        return max(1, int(len(words) * 1.3))
    
    elif isinstance(content, list):
        return sum(count_tokens(item) for item in content)
    
    elif isinstance(content, dict):
        key_tokens = sum(count_tokens(str(k)) for k in content.keys())
        value_tokens = sum(count_tokens(v) for v in content.values())
        return key_tokens + value_tokens
    
    elif hasattr(content, '__dict__'):
        return count_tokens(content.__dict__)
    
    else:
        return 1
