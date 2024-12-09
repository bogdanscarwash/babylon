from dataclasses import dataclass
from typing import Any, Callable, Dict, Optional

@dataclass
class Trigger:
    """Represents a condition that can trigger game events.
    
    A trigger defines conditions that, when met, can cause events to occur in the game.
    It contains both the condition logic and a human-readable description.
    
    Attributes:
        condition: A callable that evaluates whether the trigger condition is met
        description: A human-readable description of what the trigger represents
        type: Optional type of trigger (economic, political, etc.)
        parameters: Optional parameters for condition evaluation
    """
    condition: Callable[[Dict[str, Any]], bool]
    description: str
    type: Optional[str] = None
    parameters: Optional[Dict[str, Any]] = None

    def evaluate(self, game_state: Dict[str, Any]) -> bool:
        """Evaluate if the trigger condition is met.
        
        Args:
            game_state: The current game state to evaluate against
            
        Returns:
            bool: True if the trigger condition is met, False otherwise
        """
        return self.condition(game_state)
