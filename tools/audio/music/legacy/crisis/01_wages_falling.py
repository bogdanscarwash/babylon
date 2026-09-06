#!/usr/bin/env python3
"""
BABYLON - Crisis Suite
01_wages_falling.mid - "Wages Falling"

CONCEPTUAL BRIEF:
This piece represents the first stage of economic crisis - the material conditions
worsening as wages decline. The music captures the sensation of ground shifting
beneath one's feet, the loss aversion as workers watch their purchasing power
erode. Crucially, this is BEFORE the bifurcation - agitation energy has not yet
routed to revolution or fascism. The listener should feel mounting tension
without any resolution or direction.

TECHNICAL SPECIFICATION:
- Key: F# minor (dark, unsettled, the tritone from C creates inherent instability)
- Tempo: 75 BPM (slow, ominous, the inexorable grind of material decline)
- Time Signature: 4/4
- Duration: ~120 seconds (150 beats at 75 BPM)
- Loop Point: Beat 41 (after intro) for seamless ambient looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Piano (Program 0) - "Descent" - Descending arpeggios, each phrase lower
- Channel 1: Low Strings (Program 48) - "The Base Crumbling" - Dark sustained tones
- Channel 2: Synth Pad (Program 89) - "Uncertainty" - Ambient wash, unstable harmonics
- Channel 3: Timpani (Program 47) - "Distant Thunder" - Sparse, ominous rumbles

MUSICAL ARC (120 seconds = 150 beats at 75 BPM):
A. Stability Shaken (beats 0-40): Piano begins with stable arpeggios that start descending
B. The Descent (beats 41-90): All instruments engaged, continuous downward motion
C. Uncertainty (beats 91-130): Harmonic ambiguity, no resolution, hovering tension
D. Loop Preparation (beats 131-150): Gradual fade to seamless loop point

COMPOSITIONAL NOTES:
- F# minor is inherently unstable - neither fully dark nor light
- Descending patterns represent literal wage decline (loss aversion trigger)
- No resolution to major OR minor sixths - prevents emotional routing
- Tritone relationships (F# to C) create unresolved tension
- Timpani rumbles represent distant economic thunder - crisis approaching
- Synth pad provides "fog of uncertainty" - the future is unclear
- The piece deliberately avoids:
  * Ascending patterns (hope/revolutionary direction)
  * Fascist march rhythms (authoritarian direction)
  * Clear cadential resolution (premature closure)
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
BPM = 75
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 150  # ~120 seconds at 75 BPM

# Note definitions (MIDI note numbers)
# F# minor scale: F#, G#, A, B, C#, D, E
NOTES = {
    # Octave 1
    "C#1": 25,
    "D1": 26,
    "E1": 28,
    "F#1": 30,
    "G#1": 32,
    "A1": 33,
    "B1": 35,
    # Octave 2
    "C2": 36,
    "C#2": 37,
    "D2": 38,
    "E2": 40,
    "F#2": 42,
    "G#2": 44,
    "A2": 45,
    "B2": 47,
    # Octave 3
    "C3": 48,
    "C#3": 49,
    "D3": 50,
    "E3": 52,
    "F#3": 54,
    "G#3": 56,
    "A3": 57,
    "B3": 59,
    # Octave 4
    "C4": 60,
    "C#4": 61,
    "D4": 62,
    "D#4": 63,  # Augmented tension note
    "E4": 64,
    "F#4": 66,
    "G#4": 68,
    "A4": 69,
    "B4": 71,
    # Octave 5
    "C5": 72,
    "C#5": 73,
    "D5": 74,
    "E5": 76,
    "F#5": 78,
    "G#5": 80,
    "A5": 81,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="Wages Falling - Crisis Suite 01", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    # F# minor - 3 sharps (F#, C#, G#)
    track.append(MetaMessage("key_signature", key="F#m", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_piano_track() -> MidiTrack:
    """
    Track 1: Piano - Descent (Program 0, Channel 0)
    Descending arpeggios representing wage decline.
    Each phrase starts lower than the last - inexorable descent.
    """
    track = MidiTrack()
    track.name = "Piano - Descent"
    track.append(Message("program_change", program=0, channel=0, time=0))

    notes: list[tuple[str, float, float, int]] = []

    # Section A (beats 0-40): Stability Shaken
    # Arpeggios start high and begin descending
    # First phrase (beats 0-10): F# minor arpeggio, high register
    notes.extend(
        [
            ("F#5", 0, 1.5, 55),
            ("C#5", 1.5, 1.5, 52),
            ("A4", 3, 1.5, 50),
            ("F#4", 4.5, 2, 48),
            ("C#4", 7, 1.5, 45),
            ("A3", 8.5, 1.5, 42),
        ]
    )

    # Second phrase (beats 10-20): Starting slightly lower
    notes.extend(
        [
            ("E5", 10, 1.5, 52),
            ("C#5", 11.5, 1.5, 50),
            ("A4", 13, 1.5, 48),
            ("E4", 14.5, 2, 45),
            ("C#4", 17, 1.5, 42),
            ("A3", 18.5, 1.5, 40),
        ]
    )

    # Third phrase (beats 20-30): Lower still, more unsettled
    notes.extend(
        [
            ("D5", 20, 1.5, 50),
            ("B4", 21.5, 1.5, 48),
            ("F#4", 23, 1.5, 45),
            ("D4", 24.5, 2, 42),
            ("B3", 27, 1.5, 40),
            ("F#3", 28.5, 1.5, 38),
        ]
    )

    # Fourth phrase (beats 30-40): Descending into lower register
    notes.extend(
        [
            ("C#5", 30, 1.5, 48),
            ("A4", 31.5, 1.5, 45),
            ("E4", 33, 1.5, 42),
            ("C#4", 34.5, 2, 40),
            ("A3", 37, 1.5, 38),
            ("E3", 38.5, 1.5, 35),
        ]
    )

    # Section B (beats 41-90): The Descent
    # More rapid descending patterns, increasing urgency
    # Phrase 5 (beats 41-52)
    notes.extend(
        [
            ("F#5", 41, 1, 58),
            ("E5", 42, 1, 55),
            ("C#5", 43, 1, 52),
            ("A4", 44, 1, 50),
            ("F#4", 45, 1.5, 48),
            ("E4", 46.5, 1.5, 45),
            ("C#4", 48, 1.5, 42),
            ("A3", 49.5, 2.5, 40),
        ]
    )

    # Phrase 6 (beats 52-63) - starting lower
    notes.extend(
        [
            ("E5", 52, 1, 55),
            ("D5", 53, 1, 52),
            ("B4", 54, 1, 50),
            ("G#4", 55, 1, 48),
            ("E4", 56, 1.5, 45),
            ("D4", 57.5, 1.5, 42),
            ("B3", 59, 1.5, 40),
            ("G#3", 60.5, 2.5, 38),
        ]
    )

    # Phrase 7 (beats 63-74) - tritone tension (C natural against F#)
    notes.extend(
        [
            ("D5", 63, 1, 52),
            ("C5", 64, 1, 55),  # Tritone! Creates instability
            ("A4", 65, 1, 50),
            ("F#4", 66, 1, 48),
            ("D4", 67, 1.5, 45),
            ("C4", 68.5, 1.5, 50),  # Tritone emphasis
            ("A3", 70, 1.5, 42),
            ("F#3", 71.5, 2.5, 40),
        ]
    )

    # Phrase 8 (beats 74-85) - deeper descent
    notes.extend(
        [
            ("C#5", 74, 1, 50),
            ("B4", 75, 1, 48),
            ("G#4", 76, 1, 45),
            ("E4", 77, 1, 42),
            ("C#4", 78, 1.5, 40),
            ("B3", 79.5, 1.5, 38),
            ("G#3", 81, 1.5, 35),
            ("E3", 82.5, 2.5, 32),
        ]
    )

    # Phrase 9 (beats 85-90) - bottoming out
    notes.extend(
        [
            ("A4", 85, 1, 45),
            ("F#4", 86, 1, 42),
            ("D4", 87, 1.5, 40),
            ("A3", 88.5, 1.5, 38),
        ]
    )

    # Section C (beats 91-130): Uncertainty
    # Harmonic ambiguity - fragments, no clear direction
    # Fragmentary phrases that don't resolve
    notes.extend(
        [
            ("F#4", 91, 2, 42),
            ("C4", 93, 2.5, 48),  # Tritone
            ("E4", 96, 2, 40),
            ("B3", 99, 2.5, 38),
            ("A4", 103, 2, 44),
            ("D#4", 105, 2.5, 46),  # Augmented tension
            ("F#4", 108, 2, 42),
            ("C4", 111, 2.5, 48),  # Tritone again
            ("G#4", 115, 2, 40),
            ("E4", 117, 2, 38),
            ("C#4", 119, 2.5, 36),
            ("A3", 122, 3, 34),
            ("F#4", 126, 2, 38),
            ("D4", 128, 2, 36),
        ]
    )

    # Section D (beats 131-150): Loop Preparation
    # Sparse, fading notes preparing for seamless loop
    notes.extend(
        [
            ("C#4", 131, 3, 35),
            ("A3", 134, 3, 32),
            ("F#3", 138, 4, 28),
            ("C#3", 143, 4, 25),
            ("F#3", 148, 2, 22),  # Final note, ready to loop
        ]
    )

    # Convert notes to events
    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=0, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 2: Low Strings - The Base Crumbling (Program 48, Channel 1)
    Dark sustained notes representing the structural foundation eroding.
    Slow descent through the bass register.
    """
    track = MidiTrack()
    track.name = "Low Strings - The Base Crumbling"
    track.append(Message("program_change", program=48, channel=1, time=0))

    # Sustained bass notes - the foundation slowly dropping
    notes: list[tuple[str, float, float, int]] = [
        # Section A (beats 0-40): Initial stability, then first cracks
        ("F#2", 0, 10, 45),
        ("A2", 0, 10, 42),  # F# minor root position
        ("E2", 10, 10, 48),
        ("G#2", 10, 10, 45),  # E major - deceptive stability
        ("D2", 20, 10, 50),
        ("F#2", 20, 10, 47),  # D major - first descent
        ("C#2", 30, 10, 52),
        ("E2", 30, 10, 50),  # C# minor - unsettled
        # Section B (beats 41-90): The Descent accelerates
        ("B1", 41, 12, 55),
        ("D2", 41, 12, 52),  # B minor - darker
        ("A1", 53, 12, 58),
        ("C#2", 53, 12, 55),  # A major - false hope
        ("G#1", 65, 10, 60),
        ("B1", 65, 10, 58),  # G# minor - tritone from D
        ("F#1", 75, 15, 62),
        ("A1", 75, 15, 60),  # F# minor - but lower
        # Section C (beats 91-130): Uncertainty - ambiguous harmony
        ("E1", 91, 12, 55),
        ("G#1", 91, 12, 52),
        ("B1", 91, 12, 50),  # E major 7 - suspended
        ("D1", 103, 12, 52),
        ("F#1", 103, 12, 50),
        ("C2", 103, 12, 55),  # D with C - unresolved
        ("C#1", 115, 15, 48),
        ("E1", 115, 15, 45),
        ("A1", 115, 15, 42),  # A major - but bass is low
        # Section D (beats 131-150): Fading
        ("F#1", 131, 10, 40),
        ("C#2", 131, 10, 38),  # F# open fifth - hollow
        ("F#1", 142, 8, 30),  # Fading root
    ]

    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=1, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_synth_pad_track() -> MidiTrack:
    """
    Track 3: Synth Pad - Uncertainty (Program 89, Channel 2)
    Ambient wash creating fog of uncertainty.
    Dissonant clusters that neither resolve up nor down.
    """
    track = MidiTrack()
    track.name = "Synth Pad - Uncertainty"
    track.append(Message("program_change", program=89, channel=2, time=0))

    # Ambient pads - clusters and washes
    notes: list[tuple[str, float, float, int]] = [
        # Section A (beats 0-40): Subtle unease
        ("F#3", 8, 16, 30),
        ("A3", 8, 16, 28),
        ("C#4", 8, 16, 25),
        ("E3", 26, 14, 32),
        ("G#3", 26, 14, 30),
        ("B3", 26, 14, 28),
        # Section B (beats 41-90): Growing uncertainty
        ("D3", 41, 20, 38),
        ("F#3", 41, 20, 35),
        ("A3", 41, 20, 32),
        ("C4", 45, 16, 40),  # Tritone adds tension
        ("B2", 62, 18, 42),
        ("D3", 62, 18, 40),
        ("F#3", 62, 18, 38),
        ("A3", 62, 18, 35),
        ("G#2", 80, 10, 40),
        ("B2", 80, 10, 38),
        ("E3", 80, 10, 35),
        # Section C (beats 91-130): Maximum ambiguity
        ("C#3", 91, 25, 35),
        ("E3", 91, 25, 32),
        ("G#3", 91, 25, 30),
        ("B3", 95, 21, 28),
        ("D3", 117, 13, 32),
        ("F#3", 117, 13, 30),
        ("C4", 117, 13, 35),  # Tritone cluster
        # Section D (beats 131-150): Fading wash
        ("F#3", 131, 18, 25),
        ("A3", 131, 18, 22),
        ("C#4", 135, 14, 20),
    ]

    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=2, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 4: Timpani - Distant Thunder (Program 47, Channel 3)
    Sparse, ominous rumbles representing approaching crisis.
    Not rhythmic like a march - irregular, unsettling.
    """
    track = MidiTrack()
    track.name = "Timpani - Distant Thunder"
    track.append(Message("program_change", program=47, channel=3, time=0))

    # Sparse timpani hits - irregular, like distant thunder
    notes: list[tuple[str, float, float, int]] = [
        # Section A: Very sparse
        ("F#2", 12, 2, 35),
        ("C#2", 28, 2.5, 40),
        # Section B: More frequent, still irregular
        ("F#2", 44, 2, 45),
        ("B1", 52, 3, 50),
        ("E2", 61, 2, 48),
        ("F#2", 68, 2.5, 52),
        ("C#2", 77, 3, 55),
        ("A1", 84, 2.5, 50),
        # Section C: Rumbling uncertainty
        ("F#2", 93, 3, 48),
        ("D2", 102, 3, 52),
        ("G#1", 111, 3.5, 55),
        ("C#2", 120, 3, 50),
        ("F#1", 127, 4, 45),
        # Section D: Fading
        ("F#2", 135, 3, 35),
        ("C#2", 143, 4, 28),
    ]

    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'Wages Falling'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_piano_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_synth_pad_track())
    mid.tracks.append(create_timpani_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/crisis/01_wages_falling.mid"

    print("Creating 'Wages Falling' - Crisis Suite 01")
    print("=" * 50)

    mid = create_midi_file()
    mid.save(output_path)

    print(f"Saved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Track count: {len(mid.tracks)}")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    # Use mido's built-in length calculation
    length = mid.length
    print(f"Duration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\nComposition complete.")
    print("Musical arc: Stability Shaken -> The Descent -> Uncertainty -> Loop Prep")
    print("Key: F# minor (unresolved, pre-bifurcation tension)")
    print("Loop point: Beat 41 for seamless ambient looping")


if __name__ == "__main__":
    main()
