# The Fall of Babylon

*The Fall of Babylon* is a text-based role-playing game (RPG) that simulates complex social, political, and economic systems using XML data structures and AI components. The game incorporates Marxist theory and dialectical materialism to model contradictions and societal changes.

## Table of Contents

- [The Fall of Babylon](#the-fall-of-babylon)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Project Structure](#project-structure)
  - [Setup and Installation](#setup-and-installation)
    - [Prerequisites](#prerequisites)
    - [Installation Steps](#installation-steps)
  - [Usage Instructions](#usage-instructions)
  - [Game Mechanics](#game-mechanics)
  - [AI Integration](#ai-integration)
    - [ChromaDB Vector Database](#chromadb-vector-database)
    - [Metrics Collection](#metrics-collection)
    - [Current Features](#current-features)
    - [Planned Features](#planned-features)
  - [Error Handling \& Logging](#error-handling--logging)
    - [Error Management](#error-management)
    - [Logging System](#logging-system)
  - [Contributing](#contributing)
  - [License](#license)

## Introduction

*The Fall of Babylon* aims to provide an immersive experience where players navigate a dynamically changing world shaped by their decisions and underlying societal contradictions. The game leverages AI for non-player character (NPC) behaviors and incorporates real-time metrics collection and analysis to enhance gameplay dynamics.

## Project Structure

- `docs/`: Documentation including:
  - [CHANGELOG](docs/CHANGELOG.md): Version history and updates
  - [TODO](docs/TODO.md): Planned features and improvements
  - [MECHANICS](docs/MECHANICS.md): Game mechanics documentation
  - [CHROMA](docs/CHROMA.md): ChromaDB integration details
  - [ERROR_CODES](docs/ERROR_CODES.md): Error handling system
  - [LOGGING](docs/LOGGING.md): Logging system documentation
  - [CONFIGURATION](docs/CONFIGURATION.md): Configuration guide
  - [ECONOMY](docs/ECONOMY.md): Economic system documentation
- `src/babylon/`: Main source code
  - `ai/`: AI components and ChromaDB integration
  - `census/`: Census data integration and API
  - `config/`: Configuration management
  - `core/`: Core game systems (contradictions, economy, politics)
  - `data/`: Data management and persistence
    - `xml/`: Game entity and mechanics definitions
    - `models/`: Data models and schemas
    - `chromadb/`: Vector database storage
  - `metrics/`: Performance and gameplay metrics collection
  - `utils/`: Utility functions and helpers
- `tests/`: Comprehensive test suite
  - `unit/`: Unit tests
  - `integration/`: Integration tests
  - `fixtures/`: Test data and fixtures
- `website/`: Game website and documentation
- `pyproject.toml`: Project configuration
- `logging.yaml`: Logging configuration

## Setup and Installation

### Prerequisites

- Python 3.8 or higher
- PostgreSQL 13 or higher
- Virtual environment tool (recommended)
- Rust toolchain (for ChromaDB optimizations)

### Installation Steps

1. **Clone the Repository**

   ```shell
   git clone https://github.com/yourusername/fall-of-babylon.git
   cd fall-of-babylon
   ```

2. **Set Up Directory Structure**

   ```shell
   mkdir -p data/metrics
   mkdir -p logs/metrics
   mkdir -p backups
   mkdir -p chroma
   ```

3. **Create and Activate Virtual Environment**

   ```shell
   python -m venv venv
   source venv/bin/activate  # On Windows use venv\Scripts\activate
   ```

4. **Install Dependencies**

   ```shell
   pip install -r requirements.txt
   ```

5. **Configure Environment**

   Copy `.env.example` to `.env`:

   ```shell
   cp .env.example .env
   ```

   Update the values in `.env` with your configuration.

6. **Initialize Databases**

   - Set up PostgreSQL database
   - Initialize ChromaDB storage
   - Configure metrics collection

   Refer to [CONFIGURATION.md](docs/CONFIGURATION.md) for detailed setup instructions.

## Usage Instructions

Start the game:

```shell
python src/babylon/__main__.py
```

The game features:
- Terminal-based interface
- Real-time metrics collection
- Automatic state persistence
- Configurable logging levels

## Game Mechanics

- **Contradiction Analysis System**: 
  - Advanced engine modeling societal contradictions
  - Network visualization of relationships
  - Real-time intensity tracking
  - Historical data analysis
- **Event Generation System**:
  - Procedural event generation
  - Dynamic consequence chains
  - Contradiction-based escalation
- **Economic System**:
  - Dynamic resource management
  - Market simulation
  - Supply chain modeling
- **Political Systems**:
  - Electoral processes
  - Policy implementation
  - Governance structures

For details, see [MECHANICS.md](docs/MECHANICS.md).

## AI Integration

### ChromaDB Vector Database

- **Entity Storage**: Efficient vector representations
- **Similarity Search**: Fast kNN queries
- **Persistence**: DuckDB+Parquet backend
- **Performance**:
  - Query response < 100ms
  - Memory optimization
  - Cache management
  - Automatic backups

### Metrics Collection

- Real-time performance monitoring
- Gameplay pattern analysis
- System resource tracking
- Cache performance optimization

### Current Features

- Entity embeddings via SentenceTransformer
- Contradiction relationship analysis
- Dynamic event generation
- Performance metrics collection

### Planned Features

- Enhanced NPC behaviors
- Advanced decision systems
- Natural language processing
- Dynamic world generation

For implementation details, see [CHROMA.md](docs/CHROMA.md).

## Error Handling & Logging

### Error Management

- Structured error codes by subsystem
- Comprehensive error tracking
- Automatic error recovery
- Detailed error context

### Logging System

- JSON-structured logging
- Multiple log streams
- Automatic rotation
- Performance metrics
- Error context capture

For complete documentation:
- [ERROR_CODES.md](docs/ERROR_CODES.md)
- [LOGGING.md](docs/LOGGING.md)

## Contributing

Contributions welcome! Guidelines coming soon in CONTRIBUTING.md.

## License

MIT License - see [LICENSE](LICENSE).

For detailed progress and updates, see [CHANGELOG.md](docs/CHANGELOG.md).
