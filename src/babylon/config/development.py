"""Development configuration module.

This module defines development-specific settings, enabling features
useful during development like debug mode and detailed logging.
"""

from babylon.config.base import BaseConfig

class DevelopmentConfig(BaseConfig):
    """Development configuration class.
    
    This class extends BaseConfig to provide development-specific settings.
    It enables features that are helpful during development but should
    not be enabled in production.
    
    Attributes:
        DEBUG (bool): Enable debug mode for detailed error messages
        
    Note:
        This configuration automatically enables DEBUG mode and may include
        other development-friendly settings that would be inappropriate
        for production use.
    """

    DEBUG = True
