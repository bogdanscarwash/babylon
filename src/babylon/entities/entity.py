class Entity:
    """Base class for all game entities."""
    def __init__(self, id: str, type: str, role: str):
        self.id = id
        self.type = type
        self.role = role
        # Add attributes that are being accessed in tests
        self.freedom = 1.0
        self.wealth = 1.0
        self.stability = 1.0
        self.power = 1.0
