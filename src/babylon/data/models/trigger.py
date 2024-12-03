from typing import Callable, Dict, Any

class Trigger:
    def __init__(self, condition: Callable[[Dict[str, Any]], bool], description: str):
        self.condition = condition
        self.description = description

    def evaluate(self, game_state: Dict[str, Any]) -> bool:
        return self.condition(game_state)
