## v0.4.0 (2026-09-06)

### Native preview

- Package the Bevy viewer and Rust runtime for Ubuntu 24.04 x86_64, with a local
  PostgreSQL launcher, editable simulation parameters, and release checksums.
- Advance the simulation by one four-week period per tick; inspect production,
  staffing, held history, and saved-campaign comparisons.
- Keep current-format campaign save/reopen and retire obsolete development
  session upgrades and duplicate Python mechanics.
- Integrate reviewed reference data and reduce Python tests and development CI
  while retaining full qualification before a public main release.
- Validate Conventional Commits locally and in CI; identify each development
  build by release distance and Git SHA.

This is an administrative simulation viewer. It has no player action loop.
See `tools/release/DOWNLOAD.md` for runtime prerequisites and launch instructions.

## Historical development before the native cutover

### Feat

- **packaging**: babylon CLI entry point (play/doctor/login/telemetry/self-update/uninstall),
  server extra split + uuid relic deletion, and the uv single-lock migration (ADR095). Follow-up
  debt: docs/ still references poetry in 37 files (tracked separately).
- **ideology**: add wealth extraction as crisis trigger
- **physics**: implement Epoch 0 quantization and weekly ticks (GREEN phase)
- **ui**: wire 4-node imperial circuit to dashboard
- **graph-bridge**: implement Slice 1.7 Graph Bridge (GREEN phase)
- **endgame**: implement Slice 1.6 Endgame Detection (GREEN phase)
- **ui**: implement Slice 1.5 gauge and trend panel components
- **ui**: integrate WirePanel into main.py for Gramscian Wire display
- **ui**: implement WirePanel component for Gramscian Wire (GREEN phase)
- **ai**: implement dual narrative system for Gramscian Wire (GREEN phase)
- **engine**: integrate StateFinance with WorldState and add cost-checking
- **models**: implement finance models for Political Economy of Liquidity
- **metabolic**: wire MetabolismSystem into simulation engine (TDD Green Phase)
- **metabolic**: implement MetabolismSystem (TDD Green Phase)
- **metabolic**: add Slice 1.4A material basis for Metabolic Rift
- Epoch 1 specification completion and P0-P3 readiness review (#67)
- **ci**: add coverage gates and incremental mutation testing
- **epoch2**: implement Balkanization System with Colonial Stance mechanics (v1.2.0)
- **architecture**: implement Dynamic Sovereignty refactor (ADR029-A1)
- **architecture**: add Epoch 2 hybrid graph architecture (ADR029)
- **phase4**: Developer Dashboard MVP with MetricsCollector (#65)
- **ai-docs**: implement P1 priorities - balance targets and State AI algorithm
- **ai-docs**: implement P0 Epoch 2 sub-epoch staging (SCOPE-001)
- **tooling**: add mutation testing runner with multiprocessing fix
- **engine**: implement Event Interceptor pattern for Epoch 1→2 Bridge
- **mise**: add trace commands with optional tick parameter
- **tools**: add JSON export for causal DAG structure preservation
- **metrics**: GREEN phase - pass 40 failing tests for Phase 4.1B
- **metrics**: implement MetricsCollector observer (TDD GREEN phase)
- **engine**: add AsyncSimulationRunner for non-blocking UI
- **mise**: add UI launch tasks for Synopticon dashboard
- **ui**: implement 4-panel Synopticon dashboard layout (TDD)
- **ui**: add StateInspector component with JSON viewer (TDD)
- **ui**: add TrendPlotter component with EChart line graphs (TDD)
- **ui**: add SystemLog component with instant display (TDD)
- **ui**: add NarrativeTerminal with typewriter animation (TDD)
- **ui**: add ControlDeck component with TDD methodology
- **engine**: add mechanical event template system
- **ai**: add NarrativeCommissar LLM-as-judge for narrative evaluation
- **ai**: tighten Persephone persona with length and grounding constraints
- **ai**: rewrite Persephone persona for visceral game narrative
- **ai**: add logging and markdown transcript to interview_persona
- **ai**: add interview_persona.py tuning fork for Persephone testing
- **ai**: implement Persona system for narrative voice customization (Sprint 4.2)
- **ai**: upgrade NarrativeDirector to consume typed SimulationEvent objects (Sprint 4.1)
- **docs**: royal purple cover + cream pages for eye comfort
- **topology**: add solid phase to TopologyMonitor (4-phase model)
- **topology**: implement phase transition event emission (Sprint 3.3)
- **events**: expand event type hierarchy for all 10 EventTypes
- **engine**: create TickContext in step() function
- **context**: add TickContext model with dict compatibility
- **events**: add structured Pydantic events to WorldState (Sprint 3.1)

### Fix

- **config**: remove duplicate per-file-ignores in pyproject.toml
- **tests**: remove brittle exact event count assertions
- **testing**: add deterministic random seed for test isolation
- **ci**: add pydantic mypy plugin and format main.py
- **tests**: isolate random state between tests to prevent flakiness
- **tests**: update persona tests after content refresh
- **deps**: update filelock to 3.20.1 for CVE-2025-68146
- **ui**: add w-full classes to fix Grid/Flexbox width propagation
- **ui**: use viewport-relative heights for dashboard panel sizing
- **ui**: add min-height to dashboard panels to prevent collapse
- **ui**: populate dashboard with initial data at tick 0
- **ui**: improve Synopticon panel sizing and readability
- **ui**: properly await async callbacks in ControlDeck
- **ui**: resolve NiceGUI mode conflict RuntimeError
- **ai**: improve U-Curve analysis with state-aware prompting
- **docs**: professional LaTeX/PDF with ToC bookmarks and book styling

### Refactor

- **scenarios**: eliminate superwage_multiplier duplication (Mikado Method)
- **physics**: remove red_phase markers (REFACTOR phase)
- **graph-bridge**: cleanup and update exports (REFACTOR phase)
- **ui**: extract design system and wire endgame thresholds (REFACTOR phase)
- **topology**: wire phase thresholds to GameDefines
- **economic**: wire negligible thresholds to GameDefines
- **defines**: add MetabolismDefines and TopologyDefines to GameDefines
- **complexity**: add pre-commit complexity check and refactor high-CC functions
- **tools**: integrate MetricsCollector into parameter_analysis.py
- **metrics**: polish Phase 4.1B implementation
- **ui**: integrate MetricsCollector for unified dashboard metrics
- **ui**: integrate AsyncSimulationRunner for non-blocking UI
- **protocol**: update System.step to accept ContextType
- **ideology**: support TickContext for persistent_data

## v0.3.0 (2025-12-29)

### BREAKING CHANGE

- ideology field is now IdeologicalProfile instead of scalar float

### Feat

- **version**: expose babylon.__version__ for runtime access
- **schemas**: add NarrativeFrame JSON Schema validation
- **observers**: add CausalChainObserver for Shock Doctrine pattern detection
- **observers**: add EconomyMonitor for crisis detection
- **metrics**: add MetricsCollectorProtocol interface
- **tests**: add BabylonAssert fluent assertion library
- **docs**: apply Bunker Constructivism theme to PDF output
- **docs**: add PDF generation, GFM extensions, and new how-to guides
- **docs**: replace ASCII diagrams with Mermaid
- **observer**: add TopologyMonitor for condensation detection (Sprint 3.1)
- **analysis**: track full ideology model in trace tool
- **engine**: add dynamic displacement priority modes (Sprint 3.7.1)
- **engine**: add Carceral Geography to TerritorySystem (Sprint 3.7)
- **engine**: add StruggleSystem for Agency Layer
- **mise**: add analyze-sweep task for rich parameter sweeps
- **tools**: add sweep command to parameter_analysis.py
- **mise**: add analyze-trace task and results directory
- **tools**: add parameter analysis tool with trace command
- **tools**: add parameter tuning tool with mise integration
- **tools**: add parameter tuning tool for sensitivity analysis
- **config**: add centralized logging configuration in pyproject.toml
- **tools**: refactor vertical_slice for Imperial Circuit topology
- **ai**: add semantic bridge to translate events to RAG queries
- **engine**: add 4-node Imperial Circuit scenario with TDD
- **music**: complete Bifurcation Suite soundtrack system
- **music**: add Revolutionary Suite track "The Spark"
- **scenarios**: add WAGES edge to two-node scenario for PPP model
- **tools**: update verify_math_divergence.py for PPP model checks
- **economic**: implement PPP model for superwage distribution
- **config**: add superwage_multiplier and superwage_ppp_impact to SimulationConfig
- **engine**: add GameDefines to ServiceContainer
- **config**: export GameDefines from config package
- **config**: add GameDefines model for centralized coefficients
- **engine**: implement Multiverse Protocol scenario injection
- **chronicle**: add three archetypal events for THE_CHRONICLE MVP
- **logging**: implement comprehensive logging architecture with TDD
- **engine**: implement Sprint 3.4.4 - Dynamic Balance (Gas Tank & Driver)
- **tools**: enhance vertical_slice.py with verbose logging and diagnostics
- **ideology**: implement George Jackson Refactor - multi-dimensional consciousness model (Sprint 3.4.3)
- **territory**: implement Layer 0 - The Territorial Substrate (Sprint 3.5)
- **engine**: implement Fascist Bifurcation mechanic (Sprint 3.4.2b)
- **engine**: implement Sprint 3.4.2 Proletarian Internationalism
- **engine**: implement Sprint 3.4.1 Imperial Circuit 4-node model
- **tools**: add vertical slice CLI demo for Phase 3
- **ai**: wire NarrativeDirector to generate LLM commentary from events
- **engine**: instrument ImperialRentSystem to emit SURPLUS_EXTRACTION events
- **rag**: add MVP corpus ingestion with HTML-to-Markdown conversion
- **music**: add mise tasks and fluidsynth conversion scripts
- **music**: add scalable soundtrack infrastructure for multi-faction support
- **music**: add 12-track fascist faction soundtrack (~63 min)
- **rag**: add corpus ingestion tool for RAG Memory pipeline
- **ai**: add LLM Provider strategy pattern for narrative generation
- **music**: add procedural MIDI theme generation
- **ai**: implement RAG integration for NarrativeDirector (Sprint 3.2)
- **engine**: implement Observer Pattern for AI narrative integration (Sprint 3.1)
- **engine**: add Simulation facade and entity factories for integration proof
- **engine**: implement dependency injection system (Sprint 3)
- **models**: implement Component system for Material Ontology (Sprint 1)
- **data**: set up SQLite schema and reorganize data directory
- **engine**: implement History Stack system for state persistence
- **engine**: implement Phase 2 simulation engine (Sprints 3-6)
- **babylon**: implement Phase 1 type system, formulas, and entity models
- Implement PreEmbeddingsManager component to integrate preprocessing, chunking, and caching
- Implement EmbeddingCacheManager component for Pre-embeddings system
- Implement ChunkingStrategy component for Pre-embeddings system
- Implement ContentPreprocessor component for Pre-embeddings system
- Improve lifecycle management with enhanced tier transitions and recovery logic
- Fix object tier transitions by preserving access counts
- Enhance lifecycle management with state consistency and validation
- Add missing lifecycle management methods to LifecycleManager
- Fix ChromaDB persistence test by using correct collection interface
- Add MockMetricsCollector import to tests.mocks package
- Update pyproject.toml to improve package build configuration
- Add pre-commit hook script for Python code quality checks
- Create GitHub Kanban board structure for project tasks
- Add os import and improve database directory handling
- Add error handling and logging to metrics cleanup method
- Enhance metrics persistence with robust error handling and logging
- Add type hints and improve type safety for test_metrics_collector.py
- Add type hints and improve type safety in exception classes
- Add type hints to DevelopmentConfig for improved type safety
- Enhance Metric model with type hints, docstring, and indexing
- Enhance ChromaDB, metrics, and logging test infrastructure
- Add timedelta import to metrics persistence module
- Implement metric persistence with SQLite storage and retrieval
- Enhance performance metrics module with improved validation and documentation

### Fix

- **topology**: rename test_resilience to check_resilience
- **docs**: switch to xelatex for Unicode support in PDF builds
- **ci**: make Sphinx warnings fatal with -W flag
- **docs**: resolve Sphinx documentation warnings
- **ci**: resolve yamllint failures
- **ci**: resolve formatting and timing test issues
- **ci**: resolve test failures and enhance CI pipeline
- **__main__**: update unpacking for create_two_node_scenario
- **rag**: include embeddings in vector store query results
- **engine**: calculate wages from tribute flow, not accumulated wealth
- **audio**: increase MIDI conversion volume with loudnorm
- **types**: resolve 14 mypy strict type errors
- **security**: update h11 to 0.16.0 for CVE-2025-43859
- **deps**: update certifi, psutil, pytest-asyncio to latest versions
- **deps**: remove unused FastAPI/uvicorn direct dependencies
- **deps**: address 16 Dependabot security vulnerabilities
- **scenarios**: superwage_multiplier now sets config field, not extraction
- **engine**: resolve solidarity disconnect and economic inversion bugs
- **rag**: handle None results from ChromaDB query and use consistent embeddings
- **rag**: resolve asyncio event loop deadlock in EmbeddingManager
- **config**: support Ollama embeddings for local RAG operation
- **rag**: resolve 90 mypy type errors in babylon.rag module
- Improve object promotion logic in lifecycle management
- Add missing _find_promotion_candidate method to LifecycleManager
- Improve object tier management based on access patterns
- Remove commented-out time-based demotion code in `_rebalance_all_tiers()`
- Improve object demotion logic using access count tracking
- Adjust object demotion logic in lifecycle manager to pass test_object_access_patterns
- Improve lifecycle management and object tier promotion logic
- Improve lifecycle manager state consistency and memory pressure handling
- Improve lifecycle management for memory pressure and access patterns
- Improve lifecycle manager state transitions and memory pressure handling
- Update test to check for correct latency metric in embeddings test
- Handle database initialization and rollback in metrics collector
- Handle missing metrics table in MetricsCollector
- Prevent data loss in ChromaDB persistence test by avoiding client reset
- Update ChromaDB persistence test and backup extraction with safe methods
- Update ChromaDB persistence test to correctly handle collection recreation
- Modify ChromaDB collection retrieval in persistence test
- Modify ChromaDB collection deletion to handle empty delete request
- Use existing client for reset in persistence test
- Modify test_persistence to delete and recreate collection for testing
- Correct ChromaDB client attribute access in persistence test
- Add missing _verify_similarity_search method to ChromaDB performance test
- Handle OSError separately in backup_chroma to support test validation
- Modify backup test to use reserved Windows device name for OSError
- Update test to use OSError instead of WindowsError for cross-platform compatibility
- Update test to catch WindowsError for invalid backup directory
- Improve backup test with invalid directory path
- Ensure persist directory exists and handle backup directory validation
- Update backup tests to use chroma_client fixture
- Import backup_chroma function in backup and restore tests
- Add version constraints for chromadb and jellyfish dependencies
- Resolve pre-commit hook syntax error and improve script structure
- Restrict Python version to 3.11 in project configuration
- Add package configuration for Hatch build system
- Update Python version and add DuckDB version constraint for compatibility
- Remove invalid ChromaDB settings and use correct persistence configuration
- Resolve ChromaDB test SQLite write permissions by using in-memory database
- Improve SQLite database permissions and concurrency handling
- Update ChromaDB client initialization to use new PersistentClient
- Add missing Path import to metrics persistence module
- Resolve unterminated docstring in BackupError class
- Remove invalid syntax and extra docstrings in exceptions module
- Close unterminated docstring in BackupError class

### Refactor

- **tests**: simplify MockMetricsCollector to pure spy pattern
- **tests**: use BabylonAssert in test_simulation_engine.py
- **tests**: use DomainFactory in test_simulation_engine.py
- **tests**: add DomainFactory for centralized test entity creation
- **docs**: simplify PDF to 2 books (docs + commentary)
- **ci**: simplify CI to essential correctness checks
- **models**: replace IdeologicalComponent with George Jackson Model
- **config**: complete Paradox Refactor - externalize game coefficients to GameDefines
- **build**: consolidate mise task runner configuration
- **docs**: replace Hugo with Sphinx for documentation
- **tools**: update verify_math_divergence for SW terminology
- **economic**: use GameDefines for default coefficients
- **solidarity**: use GameDefines for negligible threshold
- **survival**: use GameDefines for default coefficients
- **scenario**: rename rent_level to superwage_multiplier
- **exceptions**: unify exception hierarchy (33 → 10 classes)
- **ideology**: complete IdeologicalProfile migration - remove legacy float branches
- remove legacy data system and enforce Embedded Trinity
- **db**: migrate ChromaDB API from deprecated Client to PersistentClient
- **engine**: wire ServiceContainer into simulation systems (Sprint 4a)
- **models**: implement Component Model architecture for entities (Phase 2.2)
- **engine**: modularize simulation engine with System protocol (Phase 2.1)
- **core**: delete Economy/Politics classes per ADR011
- technical debt cleanup before Phase 2
- remove legacy code and update source files
- Update mark_inactive to handle objects in BACKGROUND state
- Adjust LifecycleManager access count and state transition logic
- Improve lifecycle management with access count decay and priority-based demotion
- Fix indentation of _find_promotion_candidate method in lifecycle.py
- Enhance lifecycle management with performance metrics and memory pressure handling
- Rewrite test suite with isolated mock embedding manager
- Improve performance analysis unit tests with fixtures and error handling
- Add test_environment fixture to backup invalid directory test
- Restructure project directory to follow Hatch and Python best practices
- Update project configuration and dependencies for Fall of Babylon RPG
- Optimize pre-commit script to handle modified Python files more efficiently
- Improve pre-commit script file checking logic
- Update Python target version from py312 to py311 in pyproject.toml
- Update ChromaDB test imports and client configuration
- Update ChromaDB integration test setup with ChromaDBConfig
- Update metrics persistence import to use root exceptions module
- Merge metrics exceptions into main exceptions module
- Add type hints and expand configuration options in BaseConfig
- Enhance BabylonError with type hints, logging, and context details
- Improve type safety and consistency in metrics collector
- Add performance thresholds and memory tracking to metrics collection

## v0.2.0 (2025-12-29)

### Feat

- Bump version to 0.2.0 with major database and logging improvements
- Add optional object access tracking to record_metric method
- Integrate PostgreSQL database for metrics and logging
- Add error handling and logging documentation to README.md
- Add import for BabylonError in __main__.py to resolve undefined name error
- Implement comprehensive exception handling system with custom exceptions
- Add privacy management system with consent and data collection policies
- Add comprehensive metrics and logging system with performance tracking
- Enhance ChromaDB logging and exception handling with structured logging, correlation tracking, and retry mechanisms
- Enhance ChromaDB backup and restore with compression, metadata, and improved error handling
- Enhance ChromaDB manager documentation and code comments
- Improve ChromaDB backup and client management with error handling and logging
- Add comprehensive ChromaDB integration test suite
- Implement comprehensive logging solution with structured logging and retry mechanism
- Add logging configuration and error handling for ChromaDB operations
- Add ChromaDB restore functionality with user input
- Add backup mechanism after game loop ends
- Implement ChromaDB backup system with timestamped directory
- Initialize ChromaDB with embedding model and persistent storage
- Add find_similar_entities method for semantic entity search in ChromaDB
- Implement EntityRegistry for centralized entity lifecycle management
- Enhance Entity class with robust ChromaDB embedding methods
- Initialize all_events list to resolve undefined name error
- Integrate ChromaDB for vector database with entity embeddings
- Add chromadb and sentence-transformers to requirements
- Integrate ChromaDB for entity embedding and vector storage
- Add GitHub Action to sync TODO.md with Kanban board
- Publish blog page by setting draft to false
- Initialize Hugo site with Ananke theme and homepage content

### Fix

- Remove duplicate docstring and replace ASCII tree with plain text hierarchy
- Add retry utility and update import path in entity module
- Update import paths and add utils package __init__.py
- Update import paths to use full package namespace
- Add missing uuid import in chroma_manager.py
- Correct import path in development config and update .env file
- Update config import paths and add .env configuration
- Update import path for config module in backup utility
- Add missing hashlib and tempfile imports to backup module
- Add missing Any type import in chroma_manager.py
- Correct docstring syntax in `handle_event` function

### Refactor

- Replace hyphenated error code ranges with text ranges
- Update import paths to use babylon namespace consistently
- Simplify roadmap structure and formatting in TODO.md

## v0.1.0 (2025-12-29)

### Feat

- Update docstrings with more detailed, constructivist-inspired descriptions
- Enhance test docstrings with detailed verification descriptions
- Add metrics tracking and datetime imports to contradiction analysis system
- Add metrics collection to entity registry and contradiction analysis
- Update OBJECT_TRACKING.md with detailed performance tracking and metrics guidelines
- Add comprehensive character creation system with design documentation
- Add aesthetic, narrative, and content documentation for game development
- Implement constructivist-inspired design for Babylon GUI
- Add initial GUI implementation with basic Tkinter layout
- Add GUI development roadmap and library recommendations to TODO.md
- Implement event triggers and escalation paths in game system
- Add main function to initialize game loop and system components
- Add type checking and fix undefined name errors in event and contradiction analysis modules
- Implement consequence chains for event handling system
- Add comprehensive type hints to contradiction analysis tests
- Add type-annotated Contradiction, Entity, Effect, and Attribute classes
- Add contradiction.py module to project
- Add EntityRegistry import and fix type hints in contradiction_analysis.py
- Add type hints to __main__.py for improved type safety
- Implement procedural event generation for contradiction system
- Implement resolution options for contradictions with suppression, reform, and revolution methods
- Implement modular and dynamic contradiction intensity metrics
- Add missing imports for Entity, Economy, and Politics classes
- Enhance contradiction detection with economic and political metrics
- Add network visualization for entity relationships in contradictions
- Add dialectical mapping visualization for contradictions
- Implement dynamic event generation for contradiction escalation
- implement environment configuration system
- Implement secure environment variable management with .env template
- Create comprehensive legacy data validation testing module with schema, reference, and naming checks
- Implement Contradiction Analysis System core modules
- Add Persephone Raskova character legacy data and update .gitignore
- add schema definitions for game systems and socially determined objects

### Fix

- Convert datetime to ISO format for JSON serialization in metrics saving
- Ensure contradiction metrics capture multiple object accesses
- Update test and metrics collection to handle Contradiction initialization and hot object thresholds
- Import Effect directly in event.py to resolve runtime NameError
- Update TODO.md with comprehensive vector database and RAG implementation plan
- Update lambda signature in contradiction test to correctly pass game_state
- Resolve entity ID and lambda function signature issues in contradiction analysis
- Add entity_id alias and update_intensity method for tests
- Update Contradiction class to match test usage
- Update Entity and Contradiction classes to resolve test failures
- Update import path for EntityRegistry in contradiction_analysis.py
- Update Location.legacy schema to allow more flexible location IDs and add ControlledBy element

### Refactor

- Streamline project description document, remove redundancy and improve consistency
- Reorganize TODO.md to align with milestone-based development structure
- Remove unused main() function and fix Event type hints
- Add type hints to Event class constructor
- Remove lambda functions and implement proper contradiction intensity update method
- Fix method binding for contradiction intensity update
- Improve contradiction intensity update with class method
- Enhance Contradiction model with type hints and optional methods
- Fix entity imports and create base Entity class
- Add type hints and clean up imports in entity registry and contradiction analysis
- Add type hints to contradiction analysis system
