class Entity:
    """Base class for all game entities."""
    def __init__(self, id: str, type: str):
        self.id = id
        self.type = type
