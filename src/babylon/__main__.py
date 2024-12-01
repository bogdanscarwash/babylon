from dotenv import load_dotenv
from config import Config
from data.entity_registry import EntityRegistry
from systems.contradiction_analysis import ContradictionAnalysis
from data.models.economy import Economy
from data.models.politics import Politics

def handle_event(event, game_state):
    """Process and apply an event's effects to the game state."""
    print(f"Event Occurred: {event.name}")
    print(event.description)
    for effect in event.effects:
        effect.apply(game_state)

def main():
    # Access configuration variables
    secret_key = Config.SECRET_KEY
    database_url = Config.DATABASE_URL

    # Initialize systems
    entity_registry = EntityRegistry()
    contradiction_analysis = ContradictionAnalysis(entity_registry)
    game_state = {
        "entity_registry": entity_registry,
        "economy": Economy(),
        "politics": Politics()
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

        # Generate and handle events
        events = contradiction_analysis.generate_events(game_state)
        for event in events:
            handle_event(event, game_state)

        # Your application logic...
        break  # Replace with actual game loop condition

if __name__ == "__main__":
    main()
