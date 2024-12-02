from typing import Any, Dict, List, Optional
from dotenv import load_dotenv
from config import Config
from data.entity_registry import EntityRegistry
from data.models.event import Event
from systems.contradiction_analysis import ContradictionAnalysis
from data.models.economy import Economy
from data.models.politics import Politics

def handle_event(event: Event, game_state: Dict[str, Any]) -> None:
    """Process and apply an event's effects to the game state."""
    print(f"Event Occurred: {event.name}")
    print(event.description)
    for effect in event.effects:
        effect.apply(game_state)

def handle_event(event: Event, game_state: Dict[str, Any]) -> None:
    """Process and apply an event's effects to the game state."""
    print(f"Event Occurred: {event.name}")
    print(event.description)
    for effect in event.effects:
        effect.apply(game_state)

    # Process consequences
    if event.consequences:
        game_state['event_queue'].extend(event.consequences)
    # Access configuration variables
    secret_key: str = Config.SECRET_KEY
    database_url: str = Config.DATABASE_URL

    # Initialize systems
    entity_registry: EntityRegistry = EntityRegistry()
    contradiction_analysis: ContradictionAnalysis = ContradictionAnalysis(entity_registry)
    game_state: Dict[str, Any] = {
        "entity_registry": entity_registry,
        "economy": Economy(),
        "politics": Politics(),
        "event_queue": [],
        "is_player_responsible": False
    }

    print(f"Running with SECRET_KEY={secret_key}")
    print(f"Database URL: {database_url}")
    print(f"Debug mode: {Config.DEBUG}")

    # Game loop
    while True:
        # Update game state components
        game_state['economy'].update()
        game_state['politics'].update()

        # Update contradictions
        contradiction_analysis.update_contradictions(game_state)

        # Visualize contradictions and relationships
        contradiction_analysis.visualize_contradictions()
        contradiction_analysis.visualize_entity_relationships()

        # Process all events in the event queue
        while game_state['event_queue']:
            event = game_state['event_queue'].pop(0)
            handle_event(event, game_state)

        # Your application logic...
        break  # Replace with actual game loop condition

if __name__ == "__main__":
    main()
