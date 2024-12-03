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
