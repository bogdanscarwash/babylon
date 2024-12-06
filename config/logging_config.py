import logging
import logging.config
import os

import yaml


def setup_logging(default_path="logging.yaml", default_level=logging.INFO):
    """Set up logging configuration from a YAML file."""
    path = default_path
    if os.path.exists(path):
        with open(path) as f:
            config = yaml.safe_load(f.read())
        logging.config.dictConfig(config)
    else:
        logging.basicConfig(level=default_level)
