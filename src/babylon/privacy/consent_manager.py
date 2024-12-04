"""Consent management system for user privacy controls.

This module implements GDPR and CCPA compliant consent management:
- Explicit opt-in for data collection
- Granular consent controls
- Data access and deletion rights
- Consent withdrawal
"""

import logging
from typing import Dict, List, Optional
from datetime import datetime
import json
from pathlib import Path
import uuid

logger = logging.getLogger(__name__)

class ConsentManager:
    """Manages user consent for data collection and privacy preferences."""
    
    def __init__(self, data_dir: str = "user_data"):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)
        self.consent_file = self.data_dir / "consent_records.json"
        self._load_consent_records()

    def _load_consent_records(self) -> None:
        """Load existing consent records."""
        if self.consent_file.exists():
            try:
                with open(self.consent_file, 'r') as f:
                    self.consent_records = json.load(f)
            except Exception as e:
                logger.error(f"Error loading consent records: {e}")
                self.consent_records = {}
        else:
            self.consent_records = {}

    def _save_consent_records(self) -> None:
        """Save consent records to file."""
        try:
            with open(self.consent_file, 'w') as f:
                json.dump(self.consent_records, f, indent=2)
        except Exception as e:
            logger.error(f"Error saving consent records: {e}")

    def request_consent(self, user_id: str) -> Dict[str, bool]:
        """Request user consent for data collection categories.
        
        Returns:
            Dict[str, bool]: Consent status for each category
        """
        consent_categories = {
            "gameplay_data": False,
            "performance_metrics": False,
            "analytics": False
        }
        
        # Here you would implement your UI for consent collection
        # This is a placeholder for the actual UI implementation
        
        if self._record_consent(user_id, consent_categories):
            return consent_categories
        return {}

    def _record_consent(self, user_id: str, 
                       consent_categories: Dict[str, bool]) -> bool:
        """Record user consent choices with timestamp."""
        try:
            self.consent_records[user_id] = {
                "timestamp": datetime.now().isoformat(),
                "categories": consent_categories,
                "consent_id": str(uuid.uuid4())
            }
            self._save_consent_records()
            return True
        except Exception as e:
            logger.error(f"Error recording consent: {e}")
            return False

    def check_consent(self, user_id: str, 
                     category: str) -> bool:
        """Check if user has consented to specific data collection."""
        if user_id in self.consent_records:
            return self.consent_records[user_id]["categories"].get(category, False)
        return False

    def withdraw_consent(self, user_id: str, 
                        categories: Optional[List[str]] = None) -> bool:
        """Allow user to withdraw consent for all or specific categories."""
        if user_id not in self.consent_records:
            return False

        if categories is None:
            # Withdraw all consent
            self.consent_records[user_id]["categories"] = {
                k: False for k in self.consent_records[user_id]["categories"]
            }
        else:
            # Withdraw specific categories
            for category in categories:
                if category in self.consent_records[user_id]["categories"]:
                    self.consent_records[user_id]["categories"][category] = False

        self._save_consent_records()
        return True

    def export_user_data(self, user_id: str) -> Optional[Dict]:
        """Export all data associated with a user."""
        if user_id in self.consent_records:
            return self.consent_records[user_id]
        return None

    def delete_user_data(self, user_id: str) -> bool:
        """Delete all data associated with a user."""
        if user_id in self.consent_records:
            del self.consent_records[user_id]
            self._save_consent_records()
            return True
        return False
