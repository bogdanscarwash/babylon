#!/usr/bin/env python3
"""
BABYLON - Crisis Suite
02_the_squeeze.mid - "The Squeeze"

CONCEPTUAL BRIEF:
This piece represents imperial rent extraction intensifying - the compression of
the working class as surplus value is extracted at accelerating rates. Life becomes
harder, breathing becomes labored. The intervals close in, harmonies constrict,
the weight presses down. The bifurcation point approaches but has not yet arrived.

TECHNICAL SPECIFICATION:
- Key: Bb minor (suffocating, heavy)
- Tempo: 80 BPM (labored, constricted)
- Time Signature: 4/4
- Duration: ~120 seconds (160 beats at 80 BPM)
- Loop Points: Beat 33 (after intro) through beat 153 for seamless loop

INSTRUMENT ASSIGNMENTS:
- Channel 0: Strings (Program 48) - "Compression" - Intervals shrinking, closing in
- Channel 1: Contrabass (Program 43) - "The Weight" - Heavy, pressing down
- Channel 2: Piano (Program 0) - "Gasping" - Short phrases, cannot breathe
- Channel 3: Synth Pad (Program 89) - "Pressure" - Building, unrelenting

MUSICAL ARC (120 seconds = 160 beats at 80 BPM):
A. Initial Pressure (beats 0-40): Tension established, weight settling
B. Compression (beats 41-100): Intervals narrow, harmony constricts
C. Near Breaking Point (beats 101-160): Maximum pressure, but NOT rupture

COMPOSITIONAL NOTES:
- Bb minor is the key of weight and suffocation (Mahler's "Tragic" key)
- Descending chromatic lines represent wages falling
- Shrinking intervals represent shrinking life possibilities
- Sustained bass pedal points represent the inescapable weight of capital
- Synth pad builds inexorably - the pressure never relents
- Piano gasps in short phrases - the worker cannot catch breath
- Ends on unresolved tension - the bifurcation has not yet occurred
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 80
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 160  # ~120 seconds at 80 BPM

# Note definitions (MIDI note numbers) - Bb minor scale and chromatic
NOTES = {
    # Octave 1
    "Bb1": 34,
    "B1": 35,
    "C2": 36,
    # Octave 2
    "Db2": 37,
    "D2": 38,
    "Eb2": 39,
    "E2": 40,
    "F2": 41,
    "Gb2": 42,
    "G2": 43,
    "Ab2": 44,
    "A2": 45,
    "Bb2": 46,
    "B2": 47,
    # Octave 3
    "C3": 48,
    "Db3": 49,
    "D3": 50,
    "Eb3": 51,
    "E3": 52,
    "F3": 53,
    "Gb3": 54,
    "G3": 55,
    "Ab3": 56,
    "A3": 57,
    "Bb3": 58,
    "B3": 59,
    # Octave 4
    "C4": 60,
    "Db4": 61,
    "D4": 62,
    "Eb4": 63,
    "E4": 64,
    "F4": 65,
    "Gb4": 66,
    "G4": 67,
    "Ab4": 68,
    "A4": 69,
    "Bb4": 70,
    "B4": 71,
    # Octave 5
    "C5": 72,
    "Db5": 73,
    "D5": 74,
    "Eb5": 75,
    "E5": 76,
    "F5": 77,
    "Gb5": 78,
    "G5": 79,
    "Ab5": 80,
    "A5": 81,
    "Bb5": 82,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="The Squeeze - Conductor", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    # Bb minor key signature (5 flats)
    track.append(MetaMessage("key_signature", key="Bbm", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 1: Strings - Compression (Program 48, Channel 0)
    Represents the narrowing of possibilities, intervals shrinking.
    The walls closing in on the working class.
    """
    track = MidiTrack()
    track.name = "Strings - Compression"
    track.append(Message("program_change", program=48, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes = [
        # Section A (beats 0-40): Initial Pressure - establishing the weight
        # Open fifth - wide interval, still some space
        ("Bb3", 0, 8, 45),
        ("F4", 0, 8, 45),
        # Narrowing to fourth
        ("Bb3", 8, 8, 50),
        ("Eb4", 8, 8, 50),
        # Narrowing to third
        ("Bb3", 16, 8, 55),
        ("Db4", 16, 8, 55),
        # Minor second - claustrophobic
        ("Bb3", 24, 8, 60),
        ("B3", 24, 8, 58),  # Chromatic dissonance - the squeeze begins
        # Unison - no space at all
        ("Bb3", 32, 8, 65),
        ("Bb3", 32, 8, 62),  # Doubled for weight
        # Section B (beats 41-100): Compression - intervals systematically shrinking
        # Chromatic descent in tight intervals
        ("Bb4", 41, 6, 68),
        ("A4", 41, 6, 65),  # Minor second
        ("A4", 47, 6, 70),
        ("Ab4", 47, 6, 67),  # Minor second
        ("Ab4", 53, 6, 72),
        ("G4", 53, 6, 69),  # Minor second
        ("G4", 59, 6, 74),
        ("Gb4", 59, 6, 71),  # Minor second
        # Brief expansion then collapse
        ("F4", 65, 4, 68),
        ("Bb4", 65, 4, 68),  # Fourth - momentary relief
        ("F4", 69, 4, 72),
        ("Gb4", 69, 4, 70),  # Minor second - crushed again
        # Descending chromatic clusters
        ("Eb4", 73, 5, 75),
        ("E4", 73, 5, 73),
        ("F4", 73, 5, 71),  # Cluster
        ("D4", 78, 5, 77),
        ("Eb4", 78, 5, 75),
        ("E4", 78, 5, 73),  # Cluster descending
        ("Db4", 83, 5, 79),
        ("D4", 83, 5, 77),
        ("Eb4", 83, 5, 75),  # Cluster continues down
        ("C4", 88, 6, 80),
        ("Db4", 88, 6, 78),
        ("D4", 88, 6, 76),  # Nearly at bottom
        # Held dissonance
        ("Bb3", 94, 6, 82),
        ("B3", 94, 6, 80),
        ("C4", 94, 6, 78),  # Tritone cluster
        # Section C (beats 101-160): Near Breaking Point
        # Maximum compression - sustained crushing weight
        ("Bb3", 101, 12, 85),
        ("B3", 101, 12, 83),  # Minor second sustained
        ("Bb3", 113, 10, 88),
        ("A3", 113, 10, 86),  # Minor second shifting down
        # Building toward (but not reaching) rupture
        ("Bb3", 123, 8, 90),
        ("B3", 123, 8, 88),
        ("C4", 123, 8, 86),  # Tritone cluster
        ("Bb3", 131, 8, 92),
        ("Cb4", 131, 8, 90),  # Using Cb for enharmonic tension
        ("Db4", 131, 8, 88),
        # Final sustained tension - unresolved
        ("Bb3", 139, 10, 85),
        ("B3", 139, 10, 83),
        ("Db4", 139, 10, 81),  # Tritone + minor second
        # Fade but do not resolve
        ("Bb3", 149, 11, 75),
        ("B3", 149, 11, 73),  # Minor second - still squeezed
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES.get(note_name)
        if note is None:
            # Handle enharmonic Cb4 = B3
            if note_name == "Cb4":
                note = NOTES["B3"]
            else:
                raise ValueError(f"Unknown note: {note_name}")
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


def create_bass_track() -> MidiTrack:
    """
    Track 2: Contrabass - The Weight (Program 43, Channel 1)
    Heavy, pressing down - the inescapable weight of imperial rent extraction.
    Pedal points that never release, chromatic descents as wages fall.
    """
    track = MidiTrack()
    track.name = "Contrabass - The Weight"
    track.append(Message("program_change", program=43, channel=1, time=0))

    notes = [
        # Section A (beats 0-40): Establishing the weight
        # Long pedal on Bb - the foundation of oppression
        ("Bb1", 0, 16, 70),
        ("Bb1", 16, 16, 75),
        ("Bb1", 32, 8, 80),
        # Section B (beats 41-100): Chromatic descent - wages falling
        ("Bb1", 41, 8, 82),
        ("A1", 49, 8, 84),  # Wages drop
        ("Ab2", 57, 8, 86),  # Continue falling (shifted octave for variety)
        ("G2", 65, 8, 88),
        ("Gb2", 73, 8, 90),  # Wages keep falling
        ("F2", 81, 6, 92),
        ("E2", 87, 7, 94),  # Near the bottom
        ("Eb2", 94, 6, 92),
        # Section C (beats 101-160): Maximum pressure
        # Heavy repeated Bb pedal - grinding, relentless
        ("Bb1", 101, 6, 95),
        ("Bb1", 107, 6, 95),
        ("Bb1", 113, 5, 96),
        ("Bb1", 118, 5, 96),
        ("Bb1", 123, 4, 97),
        ("Bb1", 127, 4, 97),
        ("Bb1", 131, 4, 98),
        ("Bb1", 135, 4, 98),
        # Grinding chromatic at the end
        ("Bb1", 139, 3, 95),
        ("A1", 142, 3, 93),
        ("Bb1", 145, 3, 91),
        ("B1", 148, 3, 89),  # Upward tension - about to break?
        ("Bb1", 151, 9, 85),  # No. Return to the weight.
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES.get(note_name)
        if note is None:
            # Handle A1 = MIDI 33
            if note_name == "A1":
                note = 33
            else:
                raise ValueError(f"Unknown note: {note_name}")
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


def create_piano_track() -> MidiTrack:
    """
    Track 3: Piano - Gasping (Program 0, Channel 2)
    Short, fragmented phrases - the worker cannot catch their breath.
    Each phrase shorter than the last. Desperate attempts at melody
    that are cut off before completion.
    """
    track = MidiTrack()
    track.name = "Piano - Gasping"
    track.append(Message("program_change", program=0, channel=2, time=0))

    notes = [
        # Section A (beats 0-40): Attempts at breath
        # First gasp - relatively long
        ("Bb4", 4, 2, 55),
        ("Db5", 6, 1.5, 52),
        ("F5", 7.5, 1, 48),  # Ascending, trying to rise
        # Cut off, silence, second gasp - shorter
        ("Eb4", 14, 1.5, 58),
        ("F4", 15.5, 1, 55),
        # Third gasp - even shorter
        ("Bb4", 22, 1, 60),
        ("Ab4", 23, 0.75, 57),
        # Fragmented attempts
        ("F4", 30, 0.5, 55),
        ("Gb4", 31, 0.5, 52),
        ("F4", 32, 0.5, 50),
        ("Eb4", 34, 0.75, 55),
        ("Db4", 36, 1, 58),
        # Section B (beats 41-100): Compression - gasps becoming shorter
        ("Bb4", 42, 1.5, 62),
        ("A4", 44, 1, 58),  # Chromatic fall
        ("Ab4", 48, 1, 65),
        ("G4", 49.5, 0.75, 60),  # Shorter
        ("Gb4", 54, 0.75, 68),
        ("F4", 55, 0.5, 63),  # Even shorter
        # Repeated short notes - hyperventilating
        ("Eb4", 60, 0.5, 70),
        ("Eb4", 61, 0.5, 68),
        ("Eb4", 62, 0.5, 66),
        ("Db4", 64, 0.5, 72),
        ("Db4", 65, 0.5, 70),
        ("Db4", 66, 0.5, 68),
        # Descending chromatic fragments
        ("Bb4", 70, 0.5, 75),
        ("A4", 71, 0.5, 72),
        ("Ab4", 72, 0.5, 70),
        ("G4", 73, 0.5, 68),
        ("Gb4", 76, 0.5, 75),
        ("F4", 77, 0.5, 72),
        ("E4", 78, 0.5, 70),
        ("Eb4", 79, 0.5, 68),
        # Desperate repeated notes
        ("Db4", 84, 0.5, 78),
        ("Db4", 84.75, 0.5, 76),
        ("Db4", 85.5, 0.5, 74),
        ("Db4", 86.25, 0.5, 72),
        ("C4", 90, 0.5, 80),
        ("C4", 90.75, 0.5, 78),
        ("C4", 91.5, 0.5, 76),
        ("B3", 95, 0.75, 75),
        ("Bb3", 96, 1, 70),  # Momentary rest
        # Section C (beats 101-160): Near Breaking Point
        # Gasps become even more fragmented and desperate
        ("Bb4", 102, 0.5, 82),
        ("B4", 103, 0.5, 80),  # Chromatic tension
        ("Bb4", 106, 0.5, 84),
        ("A4", 107, 0.5, 82),
        ("Bb4", 108, 0.5, 85),
        # Staccato panic
        ("Db5", 112, 0.25, 88),
        ("C5", 112.5, 0.25, 86),
        ("Bb4", 113, 0.25, 84),
        ("A4", 113.5, 0.25, 82),
        ("Ab4", 114, 0.25, 80),
        # Brief silence, then more panic
        ("F4", 120, 0.5, 85),
        ("Gb4", 121, 0.5, 83),
        ("F4", 122, 0.5, 81),
        ("E4", 123, 0.5, 79),
        ("Eb4", 124, 0.5, 77),
        # Repeated single note - stuck, cannot escape
        ("Bb3", 130, 0.5, 88),
        ("Bb3", 131, 0.5, 86),
        ("Bb3", 132, 0.5, 84),
        ("Bb3", 133, 0.5, 82),
        ("Bb3", 134, 0.5, 80),
        ("Bb3", 135, 0.5, 78),
        # Final desperate chromatic crawl
        ("Bb3", 140, 0.5, 75),
        ("B3", 141, 0.5, 73),
        ("C4", 142, 0.5, 71),
        ("Db4", 143, 0.5, 69),
        ("D4", 144, 0.5, 67),  # Rising but...
        ("Db4", 148, 0.75, 65),  # Falls back
        ("C4", 150, 0.75, 63),
        ("B3", 152, 0.75, 60),
        ("Bb3", 154, 3, 55),  # Exhausted, held note - still trapped
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


def create_synth_pad_track() -> MidiTrack:
    """
    Track 4: Synth Pad - Pressure (Program 89, Channel 3)
    Warm Pad (Program 89) - Building, unrelenting pressure.
    Starts quiet, builds inexorably, never releases.
    Represents the systemic pressure of imperial rent extraction.
    """
    track = MidiTrack()
    track.name = "Synth Pad - Pressure"
    track.append(Message("program_change", program=89, channel=3, time=0))

    notes = [
        # Section A (beats 0-40): Pressure begins
        # Low sustained chord - barely perceptible at first
        ("Bb2", 0, 20, 30),
        ("F3", 0, 20, 28),
        # Building
        ("Bb2", 20, 20, 40),
        ("Db3", 20, 20, 38),
        ("F3", 20, 20, 36),
        # Section B (beats 41-100): Pressure building
        ("Bb2", 41, 15, 50),
        ("Db3", 41, 15, 48),
        ("E3", 41, 15, 46),  # Tritone - tension
        ("Bb2", 56, 15, 58),
        ("C3", 56, 15, 56),
        ("E3", 56, 15, 54),  # More tritone
        ("Bb2", 71, 14, 65),
        ("Db3", 71, 14, 63),
        ("Fb3", 71, 14, 61),  # E natural written as Fb for tension
        ("Bb2", 85, 15, 72),
        ("B2", 85, 15, 70),  # Minor second added
        ("Db3", 85, 15, 68),
        ("E3", 85, 15, 66),
        # Section C (beats 101-160): Maximum sustained pressure
        ("Bb2", 101, 20, 78),
        ("B2", 101, 20, 76),  # Grinding minor second
        ("Db3", 101, 20, 74),
        ("E3", 101, 20, 72),  # Tritone
        # Intensity peak
        ("Bb2", 121, 20, 85),
        ("B2", 121, 20, 83),
        ("C3", 121, 20, 81),  # Cluster
        ("Db3", 121, 20, 79),
        ("E3", 121, 20, 77),
        # Sustain to the end - no release
        ("Bb2", 141, 19, 80),
        ("B2", 141, 19, 78),
        ("Db3", 141, 19, 76),
        ("E3", 141, 19, 74),  # Held tritone cluster
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES.get(note_name)
        if note is None:
            # Handle Fb3 = E3
            if note_name == "Fb3":
                note = NOTES["E3"]
            else:
                raise ValueError(f"Unknown note: {note_name}")
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
    """Create the complete MIDI file for 'The Squeeze'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_bass_track())
    mid.tracks.append(create_piano_track())
    mid.tracks.append(create_synth_pad_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/crisis/02_the_squeeze.mid"

    print("Creating 'The Squeeze' - Crisis Suite 02")
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
    print("Musical arc: Initial Pressure -> Compression -> Near Breaking Point")
    print("Key: Bb minor (suffocating)")
    print("Theme: Imperial rent extraction intensifying - the squeeze before bifurcation")


if __name__ == "__main__":
    main()
