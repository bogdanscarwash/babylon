"""Data collection policy implementation.

This module implements ethical data collection practices:
- Anonymization of collected data
- Data minimization
- Purpose limitation
- Storage limitation
- Data security
"""

import hashlib
import logging
from datetime import datetime
from typing import Any

from .consent_manager import ConsentManager

logger = logging.getLogger(__name__)


class DataCollectionPolicy:
    """Implements ethical data collection practices."""

    def __init__(self, consent_manager: ConsentManager):
        self.consent_manager = consent_manager

    def anonymize_data(self, data: dict[str, Any], user_id: str) -> dict[str, Any]:
        """Anonymize user data before storage."""
        # Create anonymous identifier
        anon_id = hashlib.sha256(
            f"{user_id}-{datetime.now().isoformat()}".encode()
        ).hexdigest()[:16]

        # Remove any potential PII
        anonymized = {
            "anon_id": anon_id,
            "timestamp": datetime.now().isoformat(),
            "data_type": data.get("type", "unknown"),
            "metrics": {},
        }

        # Only include consented data categories
        if self.consent_manager.check_consent(user_id, "gameplay_data"):
            anonymized["metrics"]["gameplay"] = {
                "actions": data.get("actions", []),
                "choices": data.get("choices", []),
                "outcomes": data.get("outcomes", []),
            }

        if self.consent_manager.check_consent(user_id, "performance_metrics"):
            anonymized["metrics"]["performance"] = {
                "fps": data.get("fps", 0),
                "memory_usage": data.get("memory_usage", 0),
                "load_times": data.get("load_times", []),
            }

        if self.consent_manager.check_consent(user_id, "analytics"):
            anonymized["metrics"]["analytics"] = {
                "session_duration": data.get("session_duration", 0),
                "feature_usage": data.get("feature_usage", {}),
                "completion_rates": data.get("completion_rates", {}),
            }

        return anonymized

    def validate_data_collection(self, data_type: str, user_id: str) -> bool:
        """Validate if data collection is allowed."""
        # Check user consent
        if not self.consent_manager.check_consent(user_id, data_type):
            logger.info(f"Data collection not consented for {data_type}")
            return False

        # Validate data type
        valid_types = {"gameplay_data", "performance_metrics", "analytics"}
        if data_type not in valid_types:
            logger.warning(f"Invalid data type requested: {data_type}")
            return False

        return True

    def collect_data(
        self, data: dict[str, Any], user_id: str
    ) -> dict[str, Any] | None:
        """Collect data according to policy and consent."""
        data_type = data.get("type")

        if not self.validate_data_collection(data_type, user_id):
            return None

        # Anonymize and collect only necessary data
        anonymized_data = self.anonymize_data(data, user_id)

        # Log collection for audit
        logger.info(
            f"Collected {data_type} data",
            extra={
                "anon_id": anonymized_data["anon_id"],
                "timestamp": anonymized_data["timestamp"],
                "data_type": data_type,
            },
        )

        return anonymized_data
