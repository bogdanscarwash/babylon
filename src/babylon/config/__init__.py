import os

environment = os.getenv('ENVIRONMENT', 'development').lower()

if environment == 'production':
    from config.production import ProductionConfig as Config
elif environment == 'testing':
    from config.testing import TestingConfig as Config
else:
    from config.development import DevelopmentConfig as Config