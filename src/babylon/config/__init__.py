import os

environment = os.getenv('ENVIRONMENT', 'development').lower()

if environment == 'production':
    from babylon.config.production import ProductionConfig as Config
elif environment == 'testing':
    from babylon.config.testing import TestingConfig as Config
else:
    from babylon.config.development import DevelopmentConfig as Config
