import os
from dotenv import load_dotenv

load_dotenv()  # Load variables from .env file

class BaseConfig:
   """Base configuration."""

   SECRET_KEY = os.getenv('SECRET_KEY', 'default-secret-key')
   DATABASE_URL = os.getenv('DATABASE_URL', 'sqlite:///default.db')
   DEBUG = False
   TESTING = False