import uuid
from datetime import datetime
from typing import Any

from numpy.typing import NDArray


class Entity:
    """Base class for all game entities."""

    def __init__(self, type: str, role: str):
        """Initialize a new Entity.

        Args:
            type: Classification of the entity (e.g., 'Class', 'Organization')
            role: The entity's role in contradictions (e.g., 'Oppressor', 'Oppressed')
        """
        # Generate a unique ID
        self.id = str(uuid.uuid4())
        
        # Core identity attributes
        self.type = type
        self.role = role

        # Quantitative attributes that influence contradictions
        self.freedom = 1.0  # Degree of autonomy and self-determination
        self.wealth = 1.0  # Economic and material resources
        self.stability = 1.0  # Internal cohesion and resistance to change
        self.power = 1.0  # Ability to influence other entities

        # Vector embedding (initialized as None)
        self.embedding: NDArray | None = None

        # Lifecycle tracking
        self.created_at = datetime.now()
        self.last_updated = self.created_at

    def get_metadata(self) -> dict[str, Any]:
        """Get the entity's metadata for ChromaDB storage.

        Returns:
            Dict[str, Any]: A dictionary containing the entity's attributes
        """
        return {
            "type": self.type,
            "role": self.role,
            "freedom": float(self.freedom),
            "wealth": float(self.wealth),
            "stability": float(self.stability),
            "power": float(self.power),
        }
