"""Babylon reference data preparation and operator tools."""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__: str = version("babylon")
except PackageNotFoundError:
    # Package not installed (running from source without pip install -e)
    __version__ = "0.0.0+unknown"

__all__ = ["__version__"]
