from typing import List, Optional, Union
from .trigger import Trigger
from .contradiction import Effect

class Event:
    def __init__(
        self,
        id: str,
        name: str,
        description: str,
        effects,
        triggers: List[Trigger],
        escalation_level: str,
        consequences: Optional[List[Union['Event', Effect]]] = None,
        escalation_paths: Optional[List['Event']] = None
    ):
        self.id = id
        self.name = name
        self.description = description
        self.effects = effects
        self.triggers = triggers  # List of Trigger objects
        self.escalation_level = escalation_level
        self.consequences = consequences if consequences is not None else []
        self.escalation_paths = escalation_paths if escalation_paths is not None else []
