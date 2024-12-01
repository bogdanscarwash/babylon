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

4. **Validate XML Schemas (Optional but Recommended)**

   Ensure that all XML files conform to their schemas.

   ```shell
   # Command or script to validate XML files
   ```

## Usage Instructions

To start the game, run the main script:

```shell
python src/babylon/__main__.py
```

Currently, the game is in a development state with placeholder mechanics. The initial game world is defined by the XML files in the `data/xml/` directory.

**Note:** The game is terminal-based and interacts via text input and output.

## Game Mechanics

- **Contradiction System**: Central engine of the game, modeling societal contradictions based on Marxist theory.
- **Supply and Demand**: Dynamic resource availability affecting prices and economy.
- **Combat System**: Placeholder schemas for combat mechanics are defined but not yet implemented.
- **Political Systems**: Structures for elections, policies, and governance.

For more detailed information, refer to the [MECHANICS.md](docs/MECHANICS.md) and [IDEAS.md](docs/IDEAS.md) files.

## AI Integration

The game plans to incorporate AI models for:

- **NPC Behaviors**: More realistic and dynamic non-player character interactions.
- **Decision Making**: AI-driven events and responses based on game state.
- **Language Processing**: Understanding complex player commands (future implementation).

AI components are currently in the planning phase, as outlined in the [TODO.md](docs/TODO.md) file.

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to get involved.

**To Do:**
- Create a `CONTRIBUTING.md` file outlining the contribution process.
- Establish coding standards and pull request procedures.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

For a detailed list of changes and progress, refer to the [CHANGELOG.md](docs/CHANGELOG.md).
