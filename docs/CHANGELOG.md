# Changelog

All notable changes to this project will be documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Basic implementation of the contradiction analysis system in `contradiction_analysis.py`.
- XML parsing for contradiction definitions in `contradiction_parser.py`.
- Entity registry for creating and managing game entities within `entity_registry.py`.
- Entry point script `__main__.py` to execute the game.
- Environment-based configuration management in the `__init__.py`.
- Initial setup with census data files in categories like demographics, economics, housing, and social data.
- Core game libraries in `requirements.txt` to support features such as XML schema handling, environment variable management, and more.

### Changed
- Enhancements to the core game loop structure for incorporating contradiction logic within entity management.
- Placeholder sections in `combat_types.xsd` have been updated to include more detailed mechanics for future development.

### Deprecated
- No deprecated features at this time.

### Removed
- No removals at this time.

### Fixed
- Fixed test failures in metrics collection system:
  - Updated Contradiction class constructor test to include all required arguments
  - Adjusted hot object threshold in MetricsCollector from 10 to 3 accesses
  - Enhanced contradiction metrics tracking to record multiple accesses during initialization

### Security
- No specific security improvements are noted currently.

### TODO
- Complete outlined gameplay objects and mechanics as described in `TODO.md` and `MECHANICS.md`.
- Implement and enhance AI-driven components according to planned functionalities in the `IDEAS.md`.

## [0.1.0] - 2024-11-30

### Added
- Initial shell structure for project setup, defining the directory organization, libraries in `requirements.txt`, and `pyproject.toml` for package configuration.

---

Contributors are reminded to document all changes in the "Unreleased" section upon committing. Before a release, items from "Unreleased" should be transferred to a new version section, with the version number incremented according to semantic versioning rules.
