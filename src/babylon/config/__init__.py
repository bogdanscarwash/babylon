"""Configuration management module for the Babylon application.

This module determines which configuration to use based on the ENVIRONMENT
environment variable. It supports three environments:
- development (default)
- testing 
- production

The appropriate configuration class is imported and exposed as 'Config'.

Usage:
    from babylon.config import Config
    debug_mode = Config.DEBUG
"""

import os

# Get environment setting, defaulting to development
environment = os.getenv('ENVIRONMENT', 'development').lower()

# Import appropriate config based on environment
if environment == 'production':
    from babylon.config.production import ProductionConfig as Config
elif environment == 'testing':
    from babylon.config.testing import TestingConfig as Config
else:
    from babylon.config.development import DevelopmentConfig as Config

# Export Config as the public interface
__all__ = ['Config']
