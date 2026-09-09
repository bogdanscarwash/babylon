#!/usr/bin/env python3
"""
Fascist Faction Soundtrack - Generate All Tracks

Generates the complete ~63 minute soundtrack for the National Revival Movement.

12 tracks exploring two faces of fascism:
- PUBLIC (Menacing): The Apparatus, Viktor's March, The Nationalist Guard,
                     Repression Protocol, National Revival
- PRIVATE (Anxious): Economic Crisis, The Corporate State, Propaganda Broadcast,
                     The Juggling Act, The Mirror, The Void Beneath, Desperate Return

Usage:
    cd babylon
    uv run python -m tools.fascist_soundtrack.generate_all

Or:
    uv run python tools/fascist_soundtrack/generate_all.py
"""

from collections.abc import Callable

# Import all track generators as modules
from . import (
    generate_01_apparatus,
    generate_02_viktors_march,
    generate_03_nationalist_guard,
    generate_04_repression_protocol,
    generate_05_national_revival,
    generate_06_economic_crisis,
    generate_07_corporate_state,
    generate_08_propaganda_broadcast,
    generate_09_juggling_act,
    generate_10_the_mirror,
    generate_11_the_void_beneath,
    generate_12_desperate_return,
)


def print_header() -> None:
    """Print soundtrack generation header."""
    print("=" * 60)
    print("FASCIST FACTION SOUNDTRACK")
    print("National Revival Movement - 12 Tracks")
    print("=" * 60)
    print()
    print("Generating ~63 minutes of MIDI music...")
    print()


def print_section(name: str) -> None:
    """Print section header."""
    print("-" * 40)
    print(f"  {name}")
    print("-" * 40)


def print_footer() -> None:
    """Print completion summary."""
    print()
    print("=" * 60)
    print("GENERATION COMPLETE")
    print("=" * 60)
    print()
    print("Track List:")
    print()
    print("  MENACING (Public Face)")
    print("  01. The Apparatus       - 5:00  - Cold, efficient surveillance")
    print("  02. Viktor's March      - 4:30  - The strongman's theme")
    print("  03. The Nationalist Guard - 6:00 - Paramilitary violence")
    print("  04. Repression Protocol - 6:00  - State violence crushing dissent")
    print("  05. National Revival    - 6:00  - The hollow anthem")
    print()
    print("  ANXIOUS (Private Reality)")
    print("  06. Economic Crisis     - 5:30  - The chaos that birthed fascism")
    print("  07. The Corporate State - 4:00  - Cold bureaucratic evil")
    print("  08. Propaganda Broadcast - 7:00 - The beautiful lie")
    print("  09. The Juggling Act    - 5:30  - Spinning plates")
    print("  10. The Mirror          - 4:30  - Self-awareness/dread")
    print("  11. The Void Beneath    - 5:00  - The abyss under everything")
    print("  12. Desperate Return    - 4:00  - The machine must continue")
    print()
    print("  TOTAL: ~63:00")
    print()
    print("Output: babylon/assets/music/fascist/")
    print()
    print("They know what they are. They cannot stop.")
    print("=" * 60)


# Track generator mapping for cleaner iteration
MENACING_TRACKS: list[tuple[str, Callable[[], None]]] = [
    ("Track 01: The Apparatus", generate_01_apparatus.main),
    ("Track 02: Viktor's March", generate_02_viktors_march.main),
    ("Track 03: The Nationalist Guard", generate_03_nationalist_guard.main),
    ("Track 04: Repression Protocol", generate_04_repression_protocol.main),
    ("Track 05: National Revival", generate_05_national_revival.main),
]

ANXIOUS_TRACKS: list[tuple[str, Callable[[], None]]] = [
    ("Track 06: Economic Crisis", generate_06_economic_crisis.main),
    ("Track 07: The Corporate State", generate_07_corporate_state.main),
    ("Track 08: Propaganda Broadcast", generate_08_propaganda_broadcast.main),
    ("Track 09: The Juggling Act", generate_09_juggling_act.main),
    ("Track 10: The Mirror", generate_10_the_mirror.main),
    ("Track 11: The Void Beneath", generate_11_the_void_beneath.main),
    ("Track 12: Desperate Return", generate_12_desperate_return.main),
]


def generate_all() -> None:
    """Generate all 12 tracks of the fascist soundtrack."""
    print_header()

    # Menacing tracks (Public Face)
    print_section("MENACING TRACKS (Public Face)")
    print()

    for name, generator in MENACING_TRACKS:
        print(name)
        generator()
        print()

    # Anxious tracks (Private Reality)
    print_section("ANXIOUS TRACKS (Private Reality)")
    print()

    for name, generator in ANXIOUS_TRACKS:
        print(name)
        generator()
        print()

    print_footer()


if __name__ == "__main__":
    generate_all()
