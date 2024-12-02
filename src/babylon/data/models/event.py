from typing import List

class Event:
    def __init__(self, id: str, name: str, description: str, effects, triggers, escalation_level: float):
        self.id = id
        self.name = name
        self.description = description
        self.effects = effects      # List of Effect objects
        self.triggers = triggers    # Conditions or functions
        self.escalation_level = escalation_level
