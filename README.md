# The Fall of Babylon

*The Fall of Babylon* is a text-based role-playing game (RPG) that simulates complex social, political, and economic systems using XML data structures and AI components. The game incorporates Marxist theory and dialectical materialism to model contradictions and societal changes.

## Table of Contents

- [Introduction](#introduction)
- [Project Structure](#project-structure)
- [Setup and Installation](#setup-and-installation)
- [Usage Instructions](#usage-instructions)
- [Game Mechanics](#game-mechanics)
- [AI Integration](#ai-integration)
- [Contributing](#contributing)
- [License](#license)

## Introduction

*The Fall of Babylon* aims to provide an immersive experience where players navigate a dynamically changing world shaped by their decisions and underlying societal contradictions. The game leverages AI for non-player character (NPC) behaviors and incorporates real-world data to enhance realism.

## Project Structure

- `docs/`: Contains documentation such as the [CHANGELOG](docs/CHANGELOG.md), [TODO](docs/TODO.md), [MECHANICS](docs/MECHANICS.md), and [IDEAS](docs/IDEAS.md) files.
- `src/babylon/`: The main source code for the game.
  - `data/xml/`: XML schemas and data defining game entities and mechanics.
  - `ai/`: AI components and integrations (planned).
  - `utils/`: Utility scripts and helper functions.
- `pyproject.toml`: Project configuration file.

## Setup and Installation

### Prerequisites

- Python 3.8 or higher
- PostgreSQL 13 or higher
- Virtual environment tool (optional but recommended)

### Installation Steps

1. **Clone the Repository**

   ```shell
   git clone https://github.com/yourusername/fall-of-babylon.git
   cd fall-of-babylon
   ```

2. **Create and Activate a Virtual Environment**

   ```shell
   python -m venv venv
   source venv/bin/activate  # On Windows use venv\Scripts\activate
   ```

3. **Install Dependencies**

   ```shell
   pip install -r requirements.txt
   ```

4. **Set Up the PostgreSQL Database**

   - **Install PostgreSQL** if it's not already installed.

     - **On Linux (Debian/Ubuntu):**
       ```shell
       sudo apt-get update
       sudo apt-get install postgresql postgresql-contrib
       ```

     - **On macOS using Homebrew:**
       ```shell
       brew update
       brew install postgresql
       ```

     - **On Windows:**
       Download and install from [PostgreSQL Official Site](https://www.postgresql.org/download/windows/).

   - **Create a new database for the project:**
     ```shell
     # Start the PostgreSQL service (if not already running)
     sudo service postgresql start  # On Linux
     brew services start postgresql # On macOS

     # Switch to the postgres user and access the psql shell
     sudo -u postgres psql  # On Linux
     psql postgres          # On macOS

     # In the psql shell, create the database
     CREATE DATABASE babylon_db;
     \q  # Exit the psql shell
     ```

5. **Validate XML Schemas (Optional but Recommended)**

   Ensure that all XML files conform to their schemas.

   ```shell
   # Command or script to validate XML files
   ```

5. **Initialize Vector Database**

   The game uses ChromaDB with DuckDB+Parquet backend for efficient vector storage:
   - Automatic persistence directory creation
   - Built-in backup and restore capabilities
   - Optimized for local deployment

   ```shell
   mkdir -p backups
   ```

### Environment Variables

   Create a `.env` file at the root of the project to store environment variables for local development:

   ```dotenv
   ENVIRONMENT='development'
   SECRET_KEY='your-secret-key'
   DATABASE_URL='your-database-url'
   DEBUG=True
   ```

   **Note:** The `.env` file is included in `.gitignore` and should not be committed to version control.

   In production, set these variables in your environment instead of using a `.env` file.
   
   Copy the `.env.example` file to `.env`:

   ```shell
   cp .env.example .env
   ```

   Then, update the values in `.env` with your own configuration.

## Usage Instructions

To start the game, run the main script:

```shell
python src/babylon/__main__.py
```

Currently, the game is in a development state with placeholder mechanics. The initial game world is defined by the XML files in the `data/xml/` directory.

**Note:** The game is terminal-based and interacts via text input and output.

## Backup & Recovery

The game implements a robust backup and recovery system for the vector database:

### Automatic Backups
- Scheduled backups during gameplay
- Configurable backup retention (default: 5 backups)
- Compressed storage with checksums
- Metadata tracking for version control

### Recovery Features
- Point-in-time restoration
- Backup integrity verification
- Automatic pre-restore backup
- Atomic restore operations

### Performance
- Backup compression for space efficiency
- Incremental backup support
- Memory-efficient operations
- Minimal gameplay interruption

### Usage

To restore from a backup:

```shell
# The game will prompt for backup path on startup
python src/babylon/__main__.py

# Or specify backup directory directly
BACKUP_DIR=backups/20231203_120000 python src/babylon/__main__.py
```

## Game Mechanics

- **Contradiction Analysis System**: 
  - Advanced engine modeling societal contradictions based on Marxist theory
  - Network visualization of entity relationships
  - Dialectical mapping interface for contradiction analysis
  - Real-time intensity tracking and historical data
- **Event Generation System**:
  - Procedural event generation based on contradiction states
  - Dynamic consequence chains affecting the game world
  - Escalation paths for major contradictions
- **Supply and Demand**: Dynamic resource availability affecting prices and economy (planned).
- **Combat System**: Placeholder schemas for combat mechanics are defined but not yet implemented.
- **Political Systems**: Structures for elections, policies, and governance (in development).

For more detailed information, refer to the [MECHANICS.md](docs/MECHANICS.md) and [IDEAS.md](docs/IDEAS.md) files.

## AI Integration

The game currently incorporates AI models for:

- **Vector Database**: ChromaDB integration for efficient entity storage and similarity search
- **Contradiction Analysis**: AI-powered analysis of societal contradictions and their relationships
- **Event Generation**: Smart event creation based on game state and historical patterns
- **Visualization**: Intelligent layout and organization of network graphs and dialectical maps

Key Features:
- **Entity Embeddings**: Vector representations of game entities using SentenceTransformer
- **Similarity Search**: Fast kNN search for finding related entities and patterns
- **Persistent Storage**: DuckDB+Parquet backend for efficient data management
- **Automatic Backups**: Scheduled backups and restore capabilities

Performance Metrics:
- Query response < 100ms
- Memory usage < 2GB
- Cache hit rate > 90%

Planned AI features include:
- **NPC Behaviors**: More realistic and dynamic non-player character interactions
- **Decision Making**: AI-driven events and responses based on game state
- **Language Processing**: Understanding complex player commands

For development status and upcoming features, see the [TODO.md](docs/TODO.md) file.

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to get involved.

**To Do:**
- Create a `CONTRIBUTING.md` file outlining the contribution process.
- Establish coding standards and pull request procedures.

## Error Handling & Logging

### Error Code System
The application implements a comprehensive error handling system with structured error codes:

- **Format**: `XXX_NNN` (e.g., DB_001)
- **Subsystems**:
  - `DB_XXX`: Database operations
  - `ENT_XXX`: Entity management
  - `CFG_XXX`: Configuration
  - `GAME_XXX`: Game state
  - `BACKUP_XXX`: Backup operations

Each subsystem has dedicated ranges for specific error types. For full details, see [ERROR_CODES.md](docs/ERROR_CODES.md).

### Logging System
Features a robust JSON-structured logging system with:

- **Multiple Log Streams**:
  - Main application log (`babylon_YYYYMMDD.log`)
  - Error log (`babylon_errors_YYYYMMDD.log`)
  - Metrics log (`babylon_metrics_YYYYMMDD.log`)

- **Key Features**:
  - Correlation ID tracking
  - Structured JSON format
  - Automatic rotation (10MB, 5 backups)
  - Performance metrics
  - Error context capture

For complete logging documentation, see [LOGGING.md](docs/LOGGING.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

For a detailed list of changes and progress, refer to the [CHANGELOG.md](docs/CHANGELOG.md).
