from typing import Any, Dict, Optional
from numpy.typing import NDArray
import numpy as np
from datetime import datetime
from utils.retry import retry_on_exception
import logging
from babylon.exceptions import EntityError, EntityValidationError

logger = logging.getLogger(__name__)

class Entity:
    """Base class for all game entities.
    
    Represents any actor or object in the game world that can participate in
    dialectical contradictions. This includes social classes, organizations,
    individuals, and other forces of historical materialism.

    Attributes:
        id (str): Unique identifier for the entity
        type (str): Classification of the entity (e.g., 'Class', 'Organization')
        role (str): The entity's role in contradictions (e.g., 'Oppressor', 'Oppressed')
        freedom (float): Measure of the entity's autonomy and self-determination (0.0-1.0)
        wealth (float): Economic resources and material conditions (0.0-1.0)
        stability (float): Resistance to change and internal cohesion (0.0-1.0)
        power (float): Ability to influence other entities and events (0.0-1.0)
        embedding (Optional[NDArray]): Vector embedding representation of the entity
    """
    def __init__(self, id: str, type: str, role: str):
        # Core identity attributes
        self.id = id  # Unique identifier (e.g., "proletariat", "bourgeoisie")
        self.type = type  # Entity classification (e.g., "Class", "Organization")
        self.role = role  # Dialectical role (e.g., "Oppressor", "Oppressed")
        
        # Quantitative attributes that influence contradictions
        self.freedom = 1.0   # Degree of autonomy and self-determination
        self.wealth = 1.0    # Economic and material resources
        self.stability = 1.0 # Internal cohesion and resistance to change
        self.power = 1.0     # Ability to influence other entities
        
        # Vector embedding
        self.embedding: Optional[NDArray] = None
        
        # Lifecycle tracking
        self.created_at = datetime.now()
        self.last_updated = self.created_at

    def generate_embedding(self, embedding_model: Any) -> NDArray:
        """Generate a vector embedding for the entity using the given embedding model.
        
        Args:
            embedding_model: A model capable of generating embeddings (e.g., SentenceTransformer)
            
        Returns:
            NDArray: The generated embedding vector
            
        Note:
            The embedding combines the entity's type, role, and attributes to create
            a rich representation for similarity comparisons.
        """
        # Create a rich description incorporating all relevant attributes
        description = (
            f"{self.type} {self.role} with "
            f"freedom: {self.freedom:.2f}, "
            f"wealth: {self.wealth:.2f}, "
            f"stability: {self.stability:.2f}, "
            f"power: {self.power:.2f}"
        )
        
        # Generate and store the embedding
        self.embedding = embedding_model.encode([description])[0]
        return self.embedding

    def get_metadata(self) -> Dict[str, Any]:
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
            "power": float(self.power)
        }

    @retry_on_exception(max_retries=3, delay=2, exceptions=(Exception,))
    def add_to_chromadb(self, collection: Any) -> None:
        """Add the entity's embedding and metadata to the ChromaDB collection.
        
        Args:
            collection: A ChromaDB collection instance
            
        Raises:
            ValueError: If embedding hasn't been generated yet
        """
        if self.embedding is None:
            raise EntityValidationError("Embedding must be generated before adding to ChromaDB", "ENTITY_001")
            
        try:
            collection.add(
                documents=[self.id],
                embeddings=[self.embedding],
                ids=[self.id],
                metadatas=[self.get_metadata()]
            )
        except Exception as e:
            logger.error(f"Error adding entity '{self.id}' to ChromaDB: {e}")

    def update_in_chromadb(self, collection: Any) -> None:
        """Update the entity's embedding and metadata in the ChromaDB collection.
        
        Args:
            collection: A ChromaDB collection instance
            
        Raises:
            ValueError: If embedding hasn't been generated yet
        """
        if self.embedding is None:
            raise ValueError("Embedding must be generated before updating in ChromaDB")
            
        collection.update(
            ids=[self.id],
            embeddings=[self.embedding],
            metadatas=[self.get_metadata()]
        )

    def delete_from_chromadb(self, collection: Any) -> None:
        """Delete the entity's embedding and metadata from the ChromaDB collection.
        
        Args:
            collection: A ChromaDB collection instance
        """
        collection.delete(ids=[self.id])

    def find_similar_entities(self, collection: Any, n_results: int = 5) -> Dict[str, Any]:
        """Find similar entities in the ChromaDB collection.
        
        Args:
            collection: A ChromaDB collection instance
            n_results: Number of similar entities to return (default: 5)
            
        Returns:
            Dict[str, Any]: Query results containing similar entities
            
        Raises:
            ValueError: If embedding hasn't been generated yet
        """
        if self.embedding is None:
            raise ValueError("Embedding must be generated before querying similar entities")
            
        return collection.query(
            query_embeddings=[self.embedding],
            n_results=n_results,
            include=['metadatas', 'distances', 'documents']
        )
    def __init__(self, id: str, type: str, role: str):
        # Core identity attributes
        self.id = id  # Unique identifier (e.g., "proletariat", "bourgeoisie")
        self.type = type  # Entity classification (e.g., "Class", "Organization")
        self.role = role  # Dialectical role (e.g., "Oppressor", "Oppressed")
        
        # Quantitative attributes that influence contradictions
        # All attributes initialized at 1.0 (maximum) for testing purposes
        self.freedom = 1.0   # Degree of autonomy and self-determination
        self.wealth = 1.0    # Economic and material resources
        self.stability = 1.0 # Internal cohesion and resistance to change
        self.power = 1.0     # Ability to influence other entities
