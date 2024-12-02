from typing import List, Optional, Union

class Event:
    def __init__(self, id: str, name: str, description: str, effects, triggers, escalation_level: float, consequences: Optional[List[Union[Event, Effect]]] = None):
        self.id = id
        self.name = name
        self.description = description
        self.effects = effects      # List of Effect objects
        self.triggers = triggers    # Conditions or functions
        self.escalation_level = escalation_level
        self.consequences = consequences if consequences is not None else []
