# Changelog
All notable changes to this project will be documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial commit adding census data files in various categories: demographics, economics, housing, and social data [COMMIT_EDITMSG].
- Implementation of a basic contradiction analysis system in `contradiction_analysis.py`.
- XML parsing for definitions of contradictions in `contradiction_parser.py`.
- Entity registry to maintain references to all game entities in `entity_registry.py`.
- `__main__.py` script provided for executing the game.
- Configuration management via environment in `__init__.py` and .env.

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
- No bug fixes applicable yet.

### Security
- No specific security improvements are noted currently.

## [0.1.0] - 2024-11-30

### Added
- Initial shell structure for project setup, including the basic directory organization, libraries in `requirements.txt`, and `pyproject.toml` for configuration.
---

To maintain this changelog, each contributor should document their changes in the "Unreleased" section with a brief description upon committing. Before a release, entries should be moved from "Unreleased" to a new version section, and the version number should be incremented according to semantic versioning rules.