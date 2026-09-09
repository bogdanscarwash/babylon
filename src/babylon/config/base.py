"""Base configuration for the Babylon/Babylon engine.

This module loads configuration from environment variables with sensible defaults.
The configuration is materialist: it reflects the actual constraints of the runtime
environment, not abstract ideals.
"""

import os
from pathlib import Path
from typing import Final

from dotenv import load_dotenv

from babylon.config.paths import player_data_dir

# Load .env file if it exists
load_dotenv()


class BaseConfig:
    """Base configuration singleton.

    All values are class attributes for direct access without instantiation.
    This mirrors the material reality: configuration exists whether you
    acknowledge it or not.
    """

    # === Application Identity ===
    APP_NAME: Final[str] = "Babylon"
    VERSION: Final[str] = "0.2.0"

    # === Runtime Mode ===
    DEBUG: Final[bool] = os.getenv("DEBUG", "false").lower() == "true"
    TESTING: Final[bool] = os.getenv("TESTING", "false").lower() == "true"

    # === Database Configuration (The Ledger) ===
    # SQLite: reliable UPSERT support for ETL, embedded, no server required
    # Default location: data/sqlite/babylon.sqlite (relative to project root)
    DATABASE_URL: Final[str] = os.getenv(
        "DATABASE_URL", f"sqlite:///{Path.cwd() / 'data' / 'sqlite' / 'babylon.sqlite'}"
    )

    # === Logging Configuration ===
    LOG_LEVEL: Final[str] = os.getenv("LOG_LEVEL", "INFO")
    LOG_FORMAT: Final[str] = os.getenv(
        "LOG_FORMAT", "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    # Player data dir (ADR094/096 D3 provisioning canon), not a repo-relative
    # path: logs are runtime player state, same root as model weights
    # (`babylon.intelligence.provision.default_models_dir`). LOG_DIR env var
    # still overrides for dev/CI (e.g. pointing logs at a repo-local dir).
    LOG_DIR: Final[Path] = Path(os.getenv("LOG_DIR", str(player_data_dir() / "logs")))

    # === Metrics Configuration ===
    METRICS_ENABLED: Final[bool] = os.getenv("METRICS_ENABLED", "true").lower() == "true"
    METRICS_INTERVAL: Final[int] = int(os.getenv("METRICS_INTERVAL", "60"))

    # === Paths ===
    BASE_DIR: Final[Path] = Path(__file__).parent.parent.parent.parent
    DATA_DIR: Final[Path] = BASE_DIR / "data"
    EXTERNAL_DATA_DIR: Final[Path] = DATA_DIR / "external"

    @classmethod
    def get_database_url(cls) -> str:
        """Get the database URL, resolving any path variables."""
        return cls.DATABASE_URL

    @classmethod
    def ensure_directories(cls) -> None:
        """Ensure all required directories exist.

        The material preconditions for operation must be established.
        """
        cls.LOG_DIR.mkdir(parents=True, exist_ok=True)
        cls.DATA_DIR.mkdir(parents=True, exist_ok=True)
