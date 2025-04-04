# Project Structure

This page provides an overview of the Babylon project's codebase organization, key components, and file structure.

## Directory Structure

```
src/babylon/
├── __main__.py            # Main entry point and game loop
├── systems/               # Core game systems
│   └── contradiction_analysis.py  # Primary game engine
├── data/                  # Data models and storage
│   ├── models/            # Python data classes
│   ├── entity_registry.py # Entity management
│   ├── parsers/           # Data parsing utilities
│   ├── templates/         # XML templates
│   └── xml/               # XML data files
├── rag/                   # Retrieval Augmented Generation system
│   ├── lifecycle.py       # Object lifecycle management
│   ├── embeddings.py      # Embedding generation and management
│   └── exceptions.py      # RAG-specific exceptions
├── gui/                   # User interface
│   └── main_window.py     # Main GUI window
├── metrics/               # Performance tracking
│   └── collector.py       # Metrics collection
├── ai/                    # AI components
│   └── project_description.md # AI integration plans
└── utils/                 # Utility functions
    └── xml_validator.py   # XML validation tools

tests/                     # Test suite
├── test_contradiction_analysis.py
├── test_metrics_collection.py
└── test_runner.py         # Test orchestration

docs/                      # Documentation
├── TODO.md                # Development roadmap
├── CHANGELOG.md           # Version history
├── STORY.md               # Narrative design
├── AESTHETICS.md          # Visual design guidelines
└── wiki/                  # Wiki documentation
```

## Core Systems

### Contradiction Analysis System

Located in `src/babylon/systems/contradiction_analysis.py`, this is the central engine for detecting, tracking, and resolving contradictions. It includes:

- Contradiction detection algorithms
- Visualization tools for contradiction networks
- Event generation based on contradiction states

### Entity Registry

Located in `src/babylon/data/entity_registry.py`, this serves as the central repository for all game entities:

- Methods for registering, retrieving, and removing entities
- Entity access pattern tracking
- Integration with the metrics system

### RAG System

Located in `src/babylon/rag/`, this system manages object lifecycle and embeddings:

- Object lifecycle management (`lifecycle.py`)
- Embedding generation and management (`embeddings.py`)
- Custom exceptions (`exceptions.py`)

### Metrics Collection

Located in `src/babylon/metrics/collector.py`, this system monitors performance:

- Object access tracking
- Token usage monitoring
- Cache performance analysis
- Memory usage tracking

### Vector Database

Located in `src/babylon/data/chroma_manager.py`, this manages the ChromaDB integration:

- Vector storage and retrieval
- Collection management
- Persistence and backup

## Data Models

The project uses several key data models:

### Contradiction

Represents conflicts between entities with properties like:
- Intensity (Low, Medium, High)
- Antagonism (Antagonistic, Non-Antagonistic)
- State (Emerging, Developing, Resolving)

### Entity

Represents game objects like classes, factions, individuals, or organizations with:
- Attributes
- Relationships
- Historical data

### Event

Represents in-game occurrences triggered by specific conditions:
- Triggers
- Effects
- Narrative descriptions

## Configuration

The project uses several configuration mechanisms:

- Environment variables
- Configuration classes in `src/babylon/config/`
- XML schemas for data validation

## Testing

The project includes a comprehensive test suite:

- Unit tests for individual components
- Integration tests for system interactions
- Performance tests for optimization

## Documentation

Documentation is organized into several categories:

- Code documentation (docstrings)
- System documentation (in `docs/`)
- Wiki documentation (in `docs/wiki/`)
- Changelog (in `docs/CHANGELOG.md`)
