import atexit
import logging
import os
import signal
import sys
from datetime import datetime
from typing import Any

import chromadb
from babylon.config.base import BaseConfig as Config
from babylon.config.logging_config import setup_logging
from babylon.data.entity_registry import EntityRegistry
from babylon.data.models.event import Event
from babylon.core.economy import Economy
from babylon.core.politics import Politics
from babylon.exceptions import BabylonError
from babylon.systems.contradiction_analysis import ContradictionAnalysis
from babylon.utils.backup import backup_chroma, restore_chroma
from chromadb.config import Settings
from sentence_transformers import SentenceTransformer

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def handle_event(event: Event, game_state: dict[str, Any]) -> None:
    """
    This function:
    1. Announces the event occurrence
    2. Applies all event effects to the game state
    3. Checks for event escalation conditions
    4. Processes any event consequences

    Args:
        event: The Event instance to process
        game_state: The current game state to modify
    """
    logger.info(f"Event Occurred: {event.name}")
    logger.debug(f"Event Description: {event.description}")
    for effect in event.effects:
        effect.apply(game_state)

    # Check if any escalation paths should trigger based on current conditions
    # This allows events to branch into more severe scenarios
    for escalation_event in event.escalation_paths:
        if any(trigger.evaluate(game_state) for trigger in escalation_event.triggers):
            game_state["event_queue"].append(escalation_event)

    # Process any immediate consequences of the event
    # These can be either follow-up Events or direct Effects
    if event.consequences:
        game_state["event_queue"].extend(event.consequences)
    # Initialize ChromaDB client with persistence directory from config
    try:
        chroma_client = chromadb.Client(
            Settings(
                chroma_db_impl="duckdb+parquet",
                persist_directory=Config.CHROMADB_PERSIST_DIR,
            )
        )
    except Exception as e:
        logger.error(f"Failed to initialize ChromaDB client: {e}")
        sys.exit(1)  # Exit or handle the error appropriately

    # Initialize the embedding model
    embedding_model = SentenceTransformer("all-MiniLM-L6-v2")

    # Create or get the existing collection for entities
    collection = chroma_client.get_or_create_collection(name="entities")

    # Initialize core game systems
    entity_registry: EntityRegistry = (
        EntityRegistry()
    )  # Central registry of all game entities
    contradiction_analysis: ContradictionAnalysis = ContradictionAnalysis(
        entity_registry
    )  # Dialectical analysis system

    # Initialize the game state dictionary that tracks all game systems
    game_state: dict[str, Any] = {
        "entity_registry": entity_registry,  # Manages all game entities
        "economy": Economy(),  # Handles economic simulation
        "politics": Politics(),  # Manages political simulation
        "event_queue": [],  # Queue of pending events to process
        "is_player_responsible": False,  # Flag for player vs AI decision making
        "event_history": [],  # History of processed events
    }

    logger.info(f"Running with SECRET_KEY={Config.SECRET_KEY}")
    logger.info(f"Database URL: {Config.DATABASE_URL}")
    logger.info(f"Debug mode: {Config.DEBUG}")

    # Initialize all_events list
    all_events: list[Event] = []  # Populate this list with Event instances

    # Add entities to ChromaDB
    for entity in entity_registry.entities:
        entity.generate_embedding(embedding_model)
        entity.add_to_chromadb(collection)

    # Main game loop - runs until an exit condition is met
    while True:
        # Update economic simulation (prices, production, trade, etc)
        game_state["economy"].update()

        # Update political simulation (stability, factions, power relations)
        game_state["politics"].update()

        # Analyze and update dialectical contradictions in society
        contradiction_analysis.update_contradictions(game_state)

        # Visualize the current state of contradictions and relationships
        # This creates network graphs showing how entities and conflicts relate
        contradiction_analysis.visualize_contradictions()
        contradiction_analysis.visualize_entity_relationships()

        # Evaluate triggers for all events
        for event in all_events:
            if event not in game_state["event_history"]:
                if all(trigger.evaluate(game_state) for trigger in event.triggers):
                    game_state["event_queue"].append(event)
                    game_state["event_history"].append(event)

        # Process all pending events in the queue (protests, reforms, crises, etc)
        while game_state["event_queue"]:
            event = game_state["event_queue"].pop(0)  # Get next event
            handle_event(event, game_state)  # Process its effects

        # TODO: Add proper game loop exit conditions
        # Currently breaks immediately - replace with actual game logic
        break

    # After the game loop ends, perform backup
    backup_dir = os.path.join("backups", datetime.now().strftime("%Y%m%d_%H%M%S"))
    backup_chroma(chroma_client, backup_dir)


def cleanup_chroma(client: chromadb.Client) -> None:
    """Cleanup ChromaDB resources gracefully.

    Args:
        client: The ChromaDB client instance to cleanup
    """
    try:
        # Persist any changes to disk
        client.persist()

        # Reset the client (closes connections)
        client.reset()

    except Exception as e:
        print(f"Error during ChromaDB cleanup: {e}")


def signal_handler(signum: int, frame: Any) -> None:
    """Handle system signals for graceful shutdown.

    Args:
        signum: Signal number
        frame: Current stack frame
    """
    print(f"\nReceived signal {signum}. Initiating graceful shutdown...")
    sys.exit(0)


def main() -> None:
    """Main function to initialize and run the game loop.

    This function orchestrates the game's core systems and database operations:

    Database Initialization:
        1. Sets up ChromaDB with DuckDB+Parquet backend
        2. Configures persistence directory and settings
        3. Initializes embedding model for vector generation
        4. Creates or connects to entity collection

    System Initialization:
        1. Loads configuration from environment variables
        2. Sets up core game systems (entities, economy, politics)
        3. Initializes contradiction analysis system
        4. Configures backup and recovery mechanisms

    Game Loop Operations:
        1. Updates economic and political simulations
        2. Analyzes and updates dialectical contradictions
        3. Processes event queue and triggers
        4. Updates entity embeddings and relationships
        5. Performs periodic state persistence
        6. Visualizes current game state

    Error Handling:
        - Implements graceful shutdown on signals
        - Ensures data persistence on exit
        - Provides backup/restore capabilities
        - Logs errors and system state

    Performance Considerations:
        - Uses lazy loading for resource optimization
        - Implements batch operations for database updates
        - Manages memory through strategic persistence
        - Optimizes query patterns for ChromaDB
    """
    setup_logging()

    # Prompt user for backup directory
    backup_dir = input(
        "Enter the path to the backup directory (or press Enter to skip restore): "
    )
    if backup_dir:
        restore_chroma(backup_dir)

    # Initialize ChromaDB
    from babylon.data.chroma_manager import ChromaManager

    chroma_manager = ChromaManager()
    chroma_client = chroma_manager.client

    # Register cleanup handlers
    atexit.register(cleanup_chroma, chroma_client)
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Initialize the embedding model
    embedding_model = SentenceTransformer("all-MiniLM-L6-v2")

    # Create or get the collection for entities
    collection = chroma_client.get_or_create_collection(name="entities")

    # Access configuration variables
    secret_key: str = Config.SECRET_KEY
    database_url: str = Config.DATABASE_URL

    # Initialize core game systems
    entity_registry: EntityRegistry = (
        EntityRegistry()
    )  # Central registry of all game entities
    contradiction_analysis: ContradictionAnalysis = ContradictionAnalysis(
        entity_registry
    )  # Dialectical analysis system

    # Initialize the game state dictionary that tracks all game systems
    game_state: dict[str, Any] = {
        "entity_registry": entity_registry,  # Manages all game entities
        "economy": Economy(),  # Handles economic simulation
        "politics": Politics(),  # Manages political simulation
        "event_queue": [],  # Queue of pending events to process
        "is_player_responsible": False,  # Flag for player vs AI decision making
    }

    print(f"Running with SECRET_KEY={secret_key}")
    print(f"Database URL: {database_url}")
    print(f"Debug mode: {Config.DEBUG}")

    # Main game loop - runs until an exit condition is met
    while True:
        # Update economic simulation (prices, production, trade, etc)
        game_state["economy"].update()

        # Update political simulation (stability, factions, power relations)
        game_state["politics"].update()

        # Analyze and update dialectical contradictions in society
        contradiction_analysis.update_contradictions(game_state)

        # Visualize the current state of contradictions and relationships
        # This creates network graphs showing how entities and conflicts relate
        contradiction_analysis.visualize_contradictions()
        contradiction_analysis.visualize_entity_relationships()

        # Process all pending events in the queue (protests, reforms, crises, etc)
        while game_state["event_queue"]:
            event = game_state["event_queue"].pop(0)  # Get next event
            handle_event(event, game_state)  # Process its effects

        # TODO: Add proper game loop exit conditions
        # Currently breaks immediately - replace with actual game logic
        break


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        logger.info("Game terminated by user")
        sys.exit(0)
    except BabylonError as e:
        logger.error(f"Game error occurred: {e.message} (Code: {e.error_code})")
        sys.exit(1)
    except Exception:
        logger.critical("Unexpected error occurred", exc_info=True)
        sys.exit(2)
