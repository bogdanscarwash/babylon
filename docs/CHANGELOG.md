# Changelog

All notable changes to this project will be documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-12-03

### Breaking Changes
- Introduced new database architecture requiring PostgreSQL
- Added mandatory configuration for ChromaDB and vector storage

### Added
- Integrated PostgreSQL database system with SQLAlchemy ORM
- Added ChromaDB vector database integration with DuckDB+Parquet backend
- Implemented comprehensive metrics collection and analysis system:
  - Added SystemMetrics for CPU, memory, disk, and GPU monitoring
  - Added AIMetrics for query latency, cache performance, and anomaly detection
  - Added GameplayMetrics for session tracking and user behavior analysis
  - Implemented MetricsPersistence with SQLite backend for historical data
  - Added automatic cleanup of old metrics data
  - Added performance threshold monitoring and violation tracking
- Added structured JSON logging with correlation IDs and multiple streams
- Introduced custom exception hierarchy for error handling
- Added backup and recovery system for ChromaDB
- Implemented entity embedding using SentenceTransformer
- Enhanced contradiction analysis system with visualization capabilities in `contradiction_analysis.py`:
  - Added network graph visualization for entity relationships
  - Added dialectical map visualization for contradictions
  - Implemented intensity tracking and history
- Expanded XML parsing for contradictions in `contradiction_parser.py`:
  - Added support for parsing effects and attributes
  - Enhanced entity relationship parsing
- Improved entity registry in `entity_registry.py`:
  - Added entity removal functionality
  - Enhanced type safety with Optional types
- Enhanced game loop in `__main__.py`:
  - Added event queue processing
  - Implemented contradiction detection and resolution
  - Added game state management
- Configuration management via environment in `__init__.py` and .env.
- Implemented comprehensive logging system:
  - Configured JSON-structured logging with correlation IDs
  - Added multiple log streams for application, errors, and metrics
  - Integrated logging with exception handling
- Introduced custom exception hierarchy in `src/babylon/exceptions.py`:
  - Created `BabylonError` as base class for all custom exceptions
  - Defined specific exceptions for database, entity, configuration, game state, and backup errors
- Developed metrics collection system in `src/babylon/metrics/collector.py`:
  - Collects performance metrics, including object access frequency, token usage, cache performance, latency, and memory usage
  - Provides analysis and optimization suggestions based on metrics
- Integrated ChromaDB for vector storage and similarity search:
  - Set up ChromaDB with DuckDB+Parquet backend
  - Implemented entity embedding using SentenceTransformer
  - Added persistence and backup/restore capabilities
- Extended configuration management:
  - Added base configuration class in `src/babylon/config/base.py`
  - Set up comprehensive logging configuration in `src/babylon/config/logging_config.py`
- Added backup and recovery system for ChromaDB:
  - Implemented automatic backup creation and restoration features
- Updated `README.md` with database system integration and setup instructions
- Updated `TODO.md` with database implementation roadmap and action items

### Changed
- Placeholder sections for game mechanics such as skills and guerrilla warfare in `combat_types.xsd`.
- Updates to the core game loop and data handling structures to support contradiction logic and entity management.

### TODO
- Complete game objects and mechanics listed in `TODO.md`.
- Implement remaining functionalities in game systems as outlined in `IDEAS.md`.

### Deprecated
- No deprecated features at this time.

### Removed
- No removals at this time.

### Fixed
- Fixed test failures in metrics collection system:
  - Updated Contradiction class constructor test to include all required arguments
  - Adjusted hot object threshold in MetricsCollector from 10 to 3 accesses
  - Enhanced contradiction metrics tracking to record multiple accesses during initialization

### Fixed
- Fixed test failures in metrics collection system:
  - Updated `Contradiction` class constructor test to include all required arguments
  - Adjusted hot object threshold in `MetricsCollector` from 10 to 3 accesses
  - Enhanced contradiction metrics tracking to record multiple accesses during initialization

### Security
- Implemented secure database connections and protected sensitive information in configuration files

## [0.1.0] - 2024-11-30

### Added
- Initial shell structure for project setup, including the basic directory organization, libraries in `requirements.txt`, and `pyproject.toml` for configuration.
---

To maintain this changelog, each contributor should document their changes in the "Unreleased" section with a brief description upon committing. Before a release, entries should be moved from "Unreleased" to a new version section, and the version number should be incremented according to semantic versioning rules.
