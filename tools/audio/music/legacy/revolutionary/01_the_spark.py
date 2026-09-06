#!/usr/bin/env python3
"""
BABYLON - Revolutionary Suite
01_the_spark.mid - "The Spark"

CONCEPTUAL BRIEF:
This piece represents the first recognition of shared class condition.
The journey from individual isolation through dawning awareness to
the first tentative connection with fellow workers. The music captures
the anxious hope of class consciousness beginning to crystallize.

TECHNICAL SPECIFICATION:
- Key: E minor -> G major resolution
- Tempo: 90 BPM (anxious but hopeful)
- Time Signature: 4/4
- Duration: ~90 seconds (135 beats at 90 BPM)
- Loop Points: Beat 33 (post-intro) for looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Cello (Program 42) - "The Masses" - Main melody, grounded
- Channel 1: Piano (Program 0) - "Individual Awakening" - Sparse, questioning
- Channel 2: Strings (Program 48) - "Rising Tide" - Sustained swells
- Channel 3: Timpani (Program 47) - "The Heartbeat" - Steady pulse

MUSICAL ARC (90 seconds = 135 beats at 90 BPM):
A. Isolation (beats 0-32): Solo piano, fragmented, uncertain
B. Recognition (beats 33-64): Cello enters, timpani pulse begins
C. First Connection (beats 65-100): Strings join, harmonic resolution to G major
D. Affirmation (beats 101-135): Full texture, hopeful cadence

COMPOSITIONAL NOTES:
- E minor represents the weight of alienation and atomization
- G major (relative major) represents solidarity and collective hope
- The cello's low register grounds the piece in material reality
- Piano's sparse notes represent individual isolation before awakening
- Timpani pulse represents the collective heartbeat of the masses
- String swells represent the rising tide of class consciousness
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 90
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 136  # ~90 seconds at 90 BPM

# Note definitions (MIDI note numbers)
NOTES = {
    "B1": 35,
    "D2": 38,
    "E2": 40,
    "F#2": 42,
    "G2": 43,
    "A2": 45,
    "B2": 47,
    "C3": 48,
    "D3": 50,
    "E3": 52,
    "F#3": 54,
    "G3": 55,
    "A3": 57,
    "B3": 59,
    "C4": 60,
    "D4": 62,
    "E4": 64,
    "F#4": 66,
    "G4": 67,
    "A4": 69,
    "B4": 71,
    "C5": 72,
    "D5": 74,
    "E5": 76,
    "F#5": 78,
    "G5": 79,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="E", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_cello_track() -> MidiTrack:
    """
    Track 1: Cello - The Masses (Program 42, Channel 0)
    Enters at beat 33 (Section B) - represents proletariat awakening.
    """
    track = MidiTrack()
    track.name = "Cello - The Masses"
    track.append(Message("program_change", program=42, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes = [
        # Section B (beats 33-64): Recognition
        ("E2", 33, 4, 60),  # First entry
        ("G2", 37, 2, 65),
        ("A2", 39, 2, 68),
        ("B2", 41, 4, 70),  # Sustained
        ("E2", 45, 2, 65),
        ("F#2", 47, 2, 68),
        ("G2", 49, 4, 72),
        ("A2", 53, 2, 70),
        ("B2", 55, 3, 75),
        ("E3", 58, 4, 78),  # Octave leap
        ("D3", 62, 2, 72),
        # Section C (beats 65-100): First Connection - G major!
        ("G2", 65, 6, 82),  # G major arrival
        ("D3", 71, 4, 80),
        ("G2", 75, 2, 78),
        ("B2", 77, 2, 80),
        ("D3", 79, 4, 82),
        ("E3", 83, 2, 78),
        ("D3", 85, 2, 75),
        ("C3", 87, 2, 72),
        ("B2", 89, 3, 75),
        ("G2", 92, 2, 78),
        ("A2", 94, 2, 80),
        ("B2", 96, 4, 85),
        # Section D (beats 101-135): Affirmation
        ("G2", 101, 4, 88),
        ("D3", 105, 4, 90),
        ("G3", 109, 4, 92),  # High point
        ("F#3", 113, 2, 88),
        ("E3", 115, 2, 85),
        ("D3", 117, 4, 82),
        ("G2", 121, 4, 85),
        ("B2", 125, 4, 82),
        ("D3", 129, 4, 78),
        ("G2", 133, 3, 72),  # Final note
    ]

    events = []
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


def create_piano_track() -> MidiTrack:
    """
    Track 2: Piano - Individual Awakening (Program 0, Channel 1)
    Starts from beat 0 - represents the isolated individual.
    """
    track = MidiTrack()
    track.name = "Piano - Individual Awakening"
    track.append(Message("program_change", program=0, channel=1, time=0))

    notes = [
        # Section A (beats 0-32): Isolation - sparse, questioning
        ("E4", 1, 1.5, 48),
        ("B3", 4, 1, 42),
        ("G4", 8, 0.75, 45),
        ("F#4", 9, 2, 50),
        ("E4", 14, 1.5, 48),
        ("D4", 17, 1, 42),
        ("C4", 20, 1.5, 45),
        ("B3", 24, 1.5, 50),
        ("E4", 27, 1.5, 55),
        ("G4", 30, 2, 58),
        # Section B (beats 33-64): Recognition - responding to cello
        ("B4", 33, 1.5, 62),
        ("G4", 36, 1.5, 58),
        ("E4", 39, 2, 55),
        ("G4", 42, 2, 62),
        ("B4", 45, 3, 65),
        ("A4", 49, 1.5, 60),
        ("G4", 52, 1.5, 58),
        ("E4", 54, 3, 55),
        ("E4", 58, 1, 58),
        ("G4", 60, 1.5, 62),
        ("B4", 62, 2, 68),
        # Section C (beats 65-100): First Connection - G major chords
        ("G4", 65, 2, 75),
        ("B4", 65, 2, 75),
        ("D5", 65, 2, 75),
        ("G4", 69, 1, 70),
        ("B4", 70, 1, 70),
        ("D5", 71, 2, 72),
        ("G5", 74, 1.5, 75),
        ("D5", 76, 1.5, 70),
        ("B4", 78, 2, 68),
        ("C5", 81, 1.5, 70),
        ("B4", 83, 1.5, 68),
        ("A4", 85, 1.5, 65),
        ("G4", 87, 3, 70),
        ("G4", 91, 1, 68),
        ("A4", 92, 1, 70),
        ("B4", 93, 1, 72),
        ("D5", 94, 4, 78),
        ("G4", 99, 2, 72),
        ("B4", 99, 2, 72),
        # Section D (beats 101-135): Affirmation - fuller chords
        ("G4", 101, 2, 82),
        ("B4", 101, 2, 82),
        ("D5", 101, 2, 82),
        ("D5", 105, 2, 78),
        ("G4", 105, 2, 78),
        ("G5", 109, 3, 85),
        ("D5", 109, 3, 85),
        ("E5", 113, 2, 80),
        ("B4", 113, 2, 80),
        ("D5", 116, 3, 78),
        ("A4", 116, 3, 78),
        ("G4", 120, 3, 82),
        ("B4", 120, 3, 82),
        ("D5", 120, 3, 82),
        ("G4", 125, 3, 85),
        ("B4", 125, 3, 85),
        ("D5", 125, 3, 85),
        ("G5", 125, 3, 85),
        ("G3", 130, 6, 75),  # Final chord
        ("B3", 130, 6, 75),
        ("D4", 130, 6, 75),
        ("G4", 130, 6, 75),
    ]

    events = []
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


def create_strings_track() -> MidiTrack:
    """
    Track 3: Strings - Rising Tide (Program 48, Channel 2)
    Enters at beat 65 (Section C) - the moment of connection.
    """
    track = MidiTrack()
    track.name = "Strings - Rising Tide"
    track.append(Message("program_change", program=48, channel=2, time=0))

    # Strings play sustained chords representing collective awakening
    notes = [
        # Section C (beats 65-100): First Connection - G major swell
        ("G3", 65, 12, 55),  # G major chord, building
        ("B3", 65, 12, 55),
        ("D4", 65, 12, 55),
        ("D3", 77, 8, 68),  # D major
        ("F#3", 77, 8, 68),
        ("A3", 77, 8, 68),
        ("C3", 85, 8, 72),  # C major
        ("E3", 85, 8, 72),
        ("G3", 85, 8, 72),
        ("G3", 93, 8, 75),  # Return to G
        ("B3", 93, 8, 75),
        ("D4", 93, 8, 75),
        # Section D (beats 101-135): Affirmation - Full power
        ("G3", 101, 12, 85),  # G major triumphant
        ("B3", 101, 12, 85),
        ("D4", 101, 12, 85),
        ("G4", 101, 12, 85),
        ("E3", 113, 6, 80),  # E minor color
        ("G3", 113, 6, 80),
        ("B3", 113, 6, 80),
        ("D3", 119, 6, 82),  # D major
        ("F#3", 119, 6, 82),
        ("A3", 119, 6, 82),
        ("G3", 125, 11, 88),  # Final G major
        ("B3", 125, 11, 88),
        ("D4", 125, 11, 88),
        ("G4", 125, 11, 88),
    ]

    events = []
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
    Track 4: Timpani - The Heartbeat (Program 47, Channel 3)
    Enters at beat 33 - the collective pulse awakens.
    """
    track = MidiTrack()
    track.name = "Timpani - The Heartbeat"
    track.append(Message("program_change", program=47, channel=3, time=0))

    events = []

    # Section B (beats 33-64): Soft pulse, awakening
    # Quarter note pulse on E and B
    for beat in range(33, 65):
        note = NOTES["E2"] if (beat - 33) % 4 < 2 else NOTES["B1"]

        # Gradual crescendo
        velocity = min(45 + (beat - 33), 70)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Section C (beats 65-100): Stronger pulse in G
    for beat in range(65, 101):
        note = NOTES["G2"] if (beat - 65) % 4 < 2 else NOTES["D2"]

        velocity = min(65 + (beat - 65) // 2, 85)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Section D (beats 101-135): Full power
    for beat in range(101, 136):
        note = NOTES["G2"] if (beat - 101) % 4 < 2 else NOTES["D2"]

        # Climax around beat 120, then slight diminuendo
        raw_velocity = 80 + (beat - 101) if beat < 120 else 95 - (beat - 120)
        velocity = max(70, min(raw_velocity, 100))

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'The Spark'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_cello_track())
    mid.tracks.append(create_piano_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_timpani_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/revolutionary/01_the_spark.mid"

    print("Creating 'The Spark' - Revolutionary Suite 01")
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
    print("Musical arc: Isolation -> Recognition -> First Connection -> Affirmation")
    print("Key progression: E minor -> G major")


if __name__ == "__main__":
    main()
