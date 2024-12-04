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
    """
    def generate_embedding(self, embedding_model: Any) -> None:
        """Generate an embedding for the entity using the given embedding model."""
        description = f"{self.type} {self.role} {self.id}"
        self.embedding = embedding_model.encode([description])[0]

    def add_to_chromadb(self, collection: Any) -> None:
        """Add the entity's embedding to the ChromaDB collection."""
        collection.add(
            documents=[self.id],
            embeddings=[self.embedding],
            ids=[self.id],
            metadatas=[{
                "type": self.type,
                "role": self.role,
                "freedom": self.freedom,
                "wealth": self.wealth,
                "stability": self.stability,
                "power": self.power
            }]
        )

    def update_in_chromadb(self, collection: Any) -> None:
        """Update the entity's embedding in the ChromaDB collection."""
        collection.update(
            ids=[self.id],
            embeddings=[self.embedding],
            metadatas=[{
                "type": self.type,
                "role": self.role,
                "freedom": self.freedom,
                "wealth": self.wealth,
                "stability": self.stability,
                "power": self.power
            }]
        )

    def delete_from_chromadb(self, collection: Any) -> None:
        """Delete the entity's embedding from the ChromaDB collection."""
        collection.delete(ids=[self.id])
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
