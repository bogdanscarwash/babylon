#!/usr/bin/env python3
"""
BABYLON - Revolutionary Suite
03_class_awakening.mid - "Class Awakening"

CONCEPTUAL BRIEF:
This piece represents the moment of full class consciousness - scales falling from
eyes, seeing through ideology to the material base beneath. The transformation from
confused individual to clear-eyed revolutionary subject. This is not mere anger or
discontent; it is the crystalline understanding of one's position in the relations
of production.

TECHNICAL SPECIFICATION:
- Key: D minor -> D major (Picardy third resolution - darkness to light)
- Tempo: 120 BPM (energized, clear-eyed, purposeful)
- Time Signature: 4/4
- Duration: ~90 seconds (180 beats at 120 BPM)
- Loop Points: Beat 49 (post-revelation) for looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Choir Aahs (Program 52) - "The Awakened" - Ethereal voices of clarity
- Channel 1: Brass Section (Program 61) - "Clarity" - Bold statements of truth
- Channel 2: Piano (Program 0) - "The Individual Transformed" - Now confident
- Channel 3: Strings (Program 48) - "The Collective" - Rich, full support
- Channel 4: Timpani (Program 47) - "Determination" - Driving, unstoppable

MUSICAL ARC (90 seconds = 180 beats at 120 BPM):
A. Revelation (beats 0-48): The veil lifting, D minor with chromatic tension
B. Understanding (beats 49-112): Clarity emerges, modal shift toward D major
C. Determination (beats 113-180): Full D major, triumphant Picardy resolution

COMPOSITIONAL NOTES:
- D minor represents the weight of false consciousness before awakening
- D major (Picardy third) represents the light of understanding - same root, transformed
- Choir voices represent the ethereal quality of true consciousness
- Brass statements are bold declarations of material truth
- Piano transforms from "The Spark's" hesitant questioning to confident assertion
- Strings represent the collective body now unified in understanding
- Timpani shifts from heartbeat to march - determination, not mere pulse
- The Picardy third (minor to major on same root) symbolizes transformation in place
  rather than escape: you don't leave your class position, you understand it
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 120
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 180  # ~90 seconds at 120 BPM

# Note definitions (MIDI note numbers) - D minor/major scale plus chromatics
NOTES = {
    # Bass octave
    "A1": 33,
    "Bb1": 34,
    "B1": 35,
    "C2": 36,
    "C#2": 37,
    "D2": 38,
    "Eb2": 39,
    "E2": 40,
    "F2": 41,
    "F#2": 42,
    "G2": 43,
    "G#2": 44,
    "A2": 45,
    "Bb2": 46,
    "B2": 47,
    # Low octave
    "C3": 48,
    "C#3": 49,
    "D3": 50,
    "Eb3": 51,
    "E3": 52,
    "F3": 53,
    "F#3": 54,
    "G3": 55,
    "G#3": 56,
    "A3": 57,
    "Bb3": 58,
    "B3": 59,
    # Middle octave
    "C4": 60,
    "C#4": 61,
    "D4": 62,
    "Eb4": 63,
    "E4": 64,
    "F4": 65,
    "F#4": 66,
    "G4": 67,
    "G#4": 68,
    "A4": 69,
    "Bb4": 70,
    "B4": 71,
    # High octave
    "C5": 72,
    "C#5": 73,
    "D5": 74,
    "Eb5": 75,
    "E5": 76,
    "F5": 77,
    "F#5": 78,
    "G5": 79,
    "G#5": 80,
    "A5": 81,
    "Bb5": 82,
    "B5": 83,
    # Very high
    "C6": 84,
    "D6": 86,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="Class Awakening", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="Dm", time=0))  # Start in D minor
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def add_notes_to_track(
    track: MidiTrack, notes: list[tuple[str, float, float, int]], channel: int
) -> None:
    """
    Add a list of notes to a track using delta time conversion.

    Args:
        track: The MidiTrack to add notes to
        notes: List of (note_name, start_beat, duration_beats, velocity) tuples
        channel: MIDI channel number
    """
    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    # Sort by time, with note_off before note_on at same time
    events.sort(key=lambda x: (x[0], x[1] == "note_on"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=channel, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))


def create_choir_track() -> MidiTrack:
    """
    Track 1: Choir Aahs - The Awakened (Program 52, Channel 0)
    Ethereal voices representing the clarity of true consciousness.
    Enters softly in Section A, builds to full voice in Section C.
    """
    track = MidiTrack()
    track.name = "Choir Aahs - The Awakened"
    track.append(Message("program_change", program=52, channel=0, time=0))

    notes: list[tuple[str, float, float, int]] = [
        # Section A (beats 0-48): Revelation - soft, emerging from mist
        # Sustained D minor chord, barely audible at first
        ("D4", 8, 8, 35),
        ("F4", 8, 8, 35),
        ("A4", 8, 8, 35),
        # Rising line - the veil begins to lift
        ("D4", 20, 6, 42),
        ("F4", 20, 6, 42),
        ("A4", 20, 6, 42),
        ("E4", 28, 6, 48),  # Chromatic tension
        ("G4", 28, 6, 48),
        ("Bb4", 28, 6, 48),
        # Pushing toward revelation
        ("F4", 36, 6, 55),
        ("A4", 36, 6, 55),
        ("C5", 36, 6, 55),
        ("D4", 44, 5, 60),  # Return to root, stronger
        ("F4", 44, 5, 60),
        ("A4", 44, 5, 60),
        # Section B (beats 49-112): Understanding - voices gain clarity
        # D minor but with hope - adding 9th for openness
        ("D4", 49, 8, 68),
        ("F4", 49, 8, 68),
        ("A4", 49, 8, 68),
        ("E5", 49, 8, 65),  # 9th - opens the sound
        # Movement through understanding
        ("G4", 57, 6, 70),
        ("Bb4", 57, 6, 70),
        ("D5", 57, 6, 70),
        ("A4", 65, 6, 72),
        ("C5", 65, 6, 72),
        ("E5", 65, 6, 72),
        # First hint of F# - the Picardy emerging
        ("D4", 73, 8, 75),
        ("F#4", 73, 8, 72),  # F# appears!
        ("A4", 73, 8, 75),
        # Continue building
        ("G4", 81, 6, 72),
        ("Bb4", 81, 6, 72),
        ("D5", 81, 6, 72),
        ("F4", 89, 6, 70),
        ("A4", 89, 6, 70),
        ("C5", 89, 6, 70),
        # Approach to climax
        ("E4", 97, 8, 78),
        ("G4", 97, 8, 78),
        ("A4", 97, 8, 78),
        ("D4", 105, 8, 82),
        ("F#4", 105, 8, 80),  # F# strengthening
        ("A4", 105, 8, 82),
        # Section C (beats 113-180): Determination - Full D major triumph
        # The Picardy third arrives in full force
        ("D4", 113, 12, 90),
        ("F#4", 113, 12, 88),
        ("A4", 113, 12, 90),
        ("D5", 113, 12, 88),
        # Soaring voices
        ("A4", 125, 8, 88),
        ("C#5", 125, 8, 86),
        ("E5", 125, 8, 88),
        # Return to D major root
        ("D4", 133, 10, 92),
        ("F#4", 133, 10, 90),
        ("A4", 133, 10, 92),
        ("D5", 133, 10, 90),
        # Build to final statement
        ("G4", 145, 8, 88),
        ("B4", 145, 8, 86),
        ("D5", 145, 8, 88),
        ("A4", 153, 8, 90),
        ("C#5", 153, 8, 88),
        ("E5", 153, 8, 90),
        # Final D major chord - triumphant
        ("D4", 161, 16, 95),
        ("F#4", 161, 16, 92),
        ("A4", 161, 16, 95),
        ("D5", 161, 16, 92),
        ("F#5", 161, 16, 88),
    ]

    add_notes_to_track(track, notes, channel=0)
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 2: Brass Section - Clarity (Program 61, Channel 1)
    Bold statements of truth - declarations of material reality.
    Enters in Section B, becomes triumphant in Section C.
    """
    track = MidiTrack()
    track.name = "Brass Section - Clarity"
    track.append(Message("program_change", program=61, channel=1, time=0))

    notes: list[tuple[str, float, float, int]] = [
        # Section A: Brass silent - consciousness not yet clear
        # Section B (beats 49-112): Understanding - brass begins to speak
        # First statement - tentative but true
        ("D3", 57, 4, 65),
        ("A3", 57, 4, 65),
        ("D4", 61, 2, 68),
        ("A3", 65, 4, 70),
        ("D3", 65, 4, 70),
        # Growing confidence
        ("F3", 73, 3, 72),
        ("A3", 73, 3, 72),
        ("D4", 76, 3, 75),
        ("F4", 79, 2, 72),
        # Statement strengthens
        ("G3", 85, 4, 75),
        ("Bb3", 85, 4, 75),
        ("D4", 85, 4, 75),
        ("A3", 93, 4, 78),
        ("C4", 93, 4, 78),
        ("E4", 93, 4, 78),
        # Ascending to clarity
        ("D3", 101, 6, 82),
        ("F#3", 101, 6, 80),  # F# entering brass
        ("A3", 101, 6, 82),
        ("D4", 107, 6, 85),
        ("F#4", 107, 6, 83),
        ("A4", 107, 6, 85),
        # Section C (beats 113-180): Determination - Triumphant declarations
        # Full D major fanfare
        ("D3", 113, 4, 92),
        ("F#3", 113, 4, 90),
        ("A3", 113, 4, 92),
        ("D4", 113, 4, 92),
        # Answering phrase
        ("A3", 117, 4, 88),
        ("D4", 117, 4, 88),
        ("F#4", 117, 4, 86),
        # Bold upward statement
        ("D3", 125, 3, 90),
        ("A3", 128, 3, 92),
        ("D4", 131, 3, 94),
        ("F#4", 134, 3, 92),
        ("A4", 137, 3, 90),
        # Sustained power
        ("D3", 141, 6, 92),
        ("F#3", 141, 6, 90),
        ("A3", 141, 6, 92),
        ("D4", 141, 6, 92),
        # Final call
        ("A3", 149, 4, 88),
        ("C#4", 149, 4, 86),
        ("E4", 149, 4, 88),
        ("D3", 157, 4, 90),
        ("F#3", 157, 4, 88),
        ("A3", 157, 4, 90),
        # Triumphant conclusion
        ("D3", 165, 12, 95),
        ("F#3", 165, 12, 92),
        ("A3", 165, 12, 95),
        ("D4", 165, 12, 95),
        ("F#4", 165, 12, 90),
    ]

    add_notes_to_track(track, notes, channel=1)
    return track


def create_piano_track() -> MidiTrack:
    """
    Track 3: Piano - The Individual Transformed (Program 0, Channel 2)
    Evolution from "The Spark's" hesitant questioning to confident purpose.
    Now speaks with clarity and determination.
    """
    track = MidiTrack()
    track.name = "Piano - The Individual Transformed"
    track.append(Message("program_change", program=0, channel=2, time=0))

    notes: list[tuple[str, float, float, int]] = [
        # Section A (beats 0-48): Revelation - the veil lifting
        # Opening - D minor arpeggio, still questioning but with momentum
        ("D3", 0, 1, 55),
        ("F3", 1, 1, 58),
        ("A3", 2, 1.5, 62),
        ("D4", 4, 2, 65),
        # Chromatic search - seeking truth
        ("E4", 8, 1.5, 60),
        ("F4", 10, 1, 58),
        ("G4", 12, 2, 62),
        ("A4", 15, 2, 68),
        # Rising determination
        ("Bb4", 18, 1.5, 65),
        ("A4", 20, 1, 62),
        ("G4", 22, 1.5, 60),
        ("F4", 24, 2, 65),
        # Push toward revelation
        ("D4", 28, 1.5, 68),
        ("E4", 30, 1.5, 70),
        ("F4", 32, 2, 72),
        ("G4", 35, 2, 75),
        ("A4", 38, 3, 78),
        # Arrival at first understanding
        ("D4", 42, 2, 72),
        ("F4", 42, 2, 72),
        ("A4", 42, 2, 72),
        ("D5", 45, 3, 75),
        # Section B (beats 49-112): Understanding - confident statements
        # Clear D minor chord - no longer questioning
        ("D3", 49, 2, 78),
        ("A3", 49, 2, 78),
        ("D4", 49, 2, 78),
        ("F4", 49, 2, 78),
        # Melodic line with purpose
        ("A4", 53, 2, 75),
        ("G4", 55, 1.5, 72),
        ("F4", 57, 2, 70),
        ("E4", 60, 2, 72),
        ("D4", 63, 2, 75),
        # Rising understanding
        ("F4", 67, 2, 78),
        ("G4", 69, 1.5, 75),
        ("A4", 71, 2, 78),
        ("Bb4", 74, 2, 80),
        ("C5", 77, 3, 82),
        # The F# emerges - Picardy beginning
        ("D4", 81, 2, 80),
        ("F#4", 81, 2, 78),  # F# in piano!
        ("A4", 81, 2, 80),
        # Continue building
        ("E4", 85, 1.5, 78),
        ("F#4", 87, 1.5, 80),
        ("G4", 89, 2, 82),
        ("A4", 92, 2, 85),
        ("D5", 95, 3, 88),
        # Approach climax
        ("D4", 100, 2, 85),
        ("F#4", 100, 2, 83),
        ("A4", 100, 2, 85),
        ("D5", 100, 2, 85),
        ("E5", 104, 2, 82),
        ("D5", 106, 2, 80),
        ("C#5", 108, 2, 82),  # C# approaches D major
        ("D5", 110, 3, 88),
        # Section C (beats 113-180): Determination - triumphant clarity
        # Full D major chord - transformation complete
        ("D3", 113, 3, 92),
        ("F#3", 113, 3, 90),
        ("A3", 113, 3, 92),
        ("D4", 113, 3, 92),
        ("F#4", 113, 3, 90),
        # Confident melodic statements
        ("A4", 117, 2, 88),
        ("F#4", 119, 1.5, 85),
        ("D4", 121, 2, 82),
        ("F#4", 124, 2, 85),
        ("A4", 127, 2, 88),
        ("D5", 130, 3, 92),
        # Rolling arpeggios of determination
        ("D4", 134, 1, 85),
        ("F#4", 135, 1, 87),
        ("A4", 136, 1, 89),
        ("D5", 137, 1, 91),
        ("F#5", 138, 1, 89),
        ("A5", 139, 1.5, 87),
        # Sustained power
        ("D4", 142, 4, 90),
        ("F#4", 142, 4, 88),
        ("A4", 142, 4, 90),
        ("D5", 142, 4, 90),
        # Final statements
        ("G4", 148, 2, 88),
        ("B4", 148, 2, 86),
        ("D5", 148, 2, 88),
        ("A4", 152, 2, 90),
        ("C#5", 152, 2, 88),
        ("E5", 152, 2, 90),
        # Descending to root
        ("D5", 157, 2, 92),
        ("A4", 159, 2, 90),
        ("F#4", 161, 2, 88),
        ("D4", 163, 2, 90),
        # Final triumphant chord
        ("D3", 168, 10, 95),
        ("F#3", 168, 10, 92),
        ("A3", 168, 10, 95),
        ("D4", 168, 10, 95),
        ("F#4", 168, 10, 92),
        ("A4", 168, 10, 90),
        ("D5", 168, 10, 88),
    ]

    add_notes_to_track(track, notes, channel=2)
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 4: Strings - The Collective (Program 48, Channel 3)
    Rich, full string section representing unified class consciousness.
    Present throughout, growing from tentative to triumphant.
    """
    track = MidiTrack()
    track.name = "Strings - The Collective"
    track.append(Message("program_change", program=48, channel=3, time=0))

    notes: list[tuple[str, float, float, int]] = [
        # Section A (beats 0-48): Revelation - strings set the foundation
        # D minor pedal - the weight of false consciousness
        ("D2", 0, 16, 45),
        ("A2", 0, 16, 45),
        # Subtle movement
        ("D3", 4, 12, 42),
        ("F3", 8, 8, 40),
        # Rising swell
        ("D2", 16, 16, 52),
        ("A2", 16, 16, 52),
        ("D3", 16, 16, 50),
        ("F3", 20, 12, 48),
        ("A3", 24, 8, 50),
        # Building toward revelation
        ("D2", 32, 16, 60),
        ("A2", 32, 16, 60),
        ("D3", 32, 16, 58),
        ("F3", 32, 16, 55),
        ("A3", 36, 12, 58),
        ("D4", 40, 8, 62),
        # Section B (beats 49-112): Understanding - strings gain warmth
        # D minor with more body
        ("D2", 49, 16, 68),
        ("A2", 49, 16, 68),
        ("D3", 49, 16, 65),
        ("F3", 49, 16, 62),
        ("A3", 49, 16, 65),
        # Movement through harmony
        ("G2", 65, 12, 70),
        ("D3", 65, 12, 68),
        ("G3", 65, 12, 65),
        ("Bb3", 65, 12, 68),
        # A minor color
        ("A2", 77, 12, 72),
        ("E3", 77, 12, 70),
        ("A3", 77, 12, 68),
        ("C4", 77, 12, 70),
        # F# begins to appear - approach to D major
        ("D2", 89, 12, 75),
        ("A2", 89, 12, 75),
        ("D3", 89, 12, 72),
        ("F#3", 89, 12, 70),  # F# in strings
        ("A3", 89, 12, 75),
        # Dominant preparation
        ("A2", 101, 12, 80),
        ("E3", 101, 12, 78),
        ("A3", 101, 12, 78),
        ("C#4", 101, 12, 75),  # C# leading to D
        ("E4", 101, 12, 78),
        # Section C (beats 113-180): Determination - Full triumph
        # D major arrives in full glory
        ("D2", 113, 16, 90),
        ("A2", 113, 16, 90),
        ("D3", 113, 16, 88),
        ("F#3", 113, 16, 85),
        ("A3", 113, 16, 88),
        ("D4", 113, 16, 85),
        # Sustained power
        ("D2", 129, 16, 92),
        ("A2", 129, 16, 92),
        ("D3", 129, 16, 90),
        ("F#3", 129, 16, 88),
        ("A3", 129, 16, 90),
        ("D4", 129, 16, 88),
        ("F#4", 129, 16, 85),
        # G major color (IV)
        ("G2", 145, 8, 88),
        ("D3", 145, 8, 85),
        ("G3", 145, 8, 85),
        ("B3", 145, 8, 82),
        ("D4", 145, 8, 85),
        # A major (V) leading back
        ("A2", 153, 8, 90),
        ("E3", 153, 8, 88),
        ("A3", 153, 8, 88),
        ("C#4", 153, 8, 85),
        ("E4", 153, 8, 88),
        # Final D major - triumphant resolution
        ("D2", 161, 17, 95),
        ("A2", 161, 17, 95),
        ("D3", 161, 17, 92),
        ("F#3", 161, 17, 90),
        ("A3", 161, 17, 92),
        ("D4", 161, 17, 90),
        ("F#4", 161, 17, 88),
        ("A4", 161, 17, 85),
    ]

    add_notes_to_track(track, notes, channel=3)
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 5: Timpani - Determination (Program 47, Channel 4)
    The pulse transforms from heartbeat to march - unstoppable determination.
    Driving rhythm that propels the awakening forward.
    """
    track = MidiTrack()
    track.name = "Timpani - Determination"
    track.append(Message("program_change", program=47, channel=4, time=0))

    events: list[tuple[int, str, int, int]] = []

    # Section A (beats 0-48): Revelation - building pulse
    # Sparse at first, gaining momentum
    section_a_patterns = [
        # Start very sparse - the first stirrings
        (0, "D2", 45),
        (4, "D2", 48),
        (8, "A1", 50),
        (12, "D2", 52),
        # Gaining regularity
        (16, "D2", 55),
        (18, "A1", 52),
        (20, "D2", 58),
        (22, "A1", 55),
        (24, "D2", 60),
        (26, "A1", 58),
        (28, "D2", 62),
        (30, "A1", 60),
        # Building toward revelation - quarter notes
        (32, "D2", 65),
        (33, "A1", 62),
        (34, "D2", 65),
        (35, "A1", 62),
        (36, "D2", 68),
        (37, "A1", 65),
        (38, "D2", 68),
        (39, "A1", 65),
        (40, "D2", 70),
        (41, "A1", 68),
        (42, "D2", 72),
        (43, "A1", 70),
        (44, "D2", 75),
        (45, "A1", 72),
        (46, "D2", 75),
        (47, "A1", 72),
    ]

    for beat, note_name, velocity in section_a_patterns:
        note = NOTES[note_name]
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Section B (beats 49-112): Understanding - steady march emerges
    # Regular quarter note pulse with accents
    for beat in range(49, 113):
        # Alternate D and A
        note_name = "D2" if (beat - 49) % 2 == 0 else "A1"
        note = NOTES[note_name]

        # Accent on downbeats (every 4 beats)
        base_velocity = 78 if (beat - 49) % 4 == 0 else 70

        # Gradual crescendo
        velocity_adjustment = min((beat - 49) // 8, 10)
        velocity = min(base_velocity + velocity_adjustment, 90)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.35), "note_off", note, 0))

    # Section C (beats 113-180): Determination - triumphant march
    # Stronger pattern with eighth note fills
    for beat in range(113, 177):
        # Main beat
        note_name = "D2" if (beat - 113) % 2 == 0 else "A1"
        note = NOTES[note_name]

        # Strong accents on 1 and 3
        beat_in_measure = (beat - 113) % 4
        if beat_in_measure == 0:
            velocity = 95
        elif beat_in_measure == 2:
            velocity = 90
        else:
            velocity = 82

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.3), "note_off", note, 0))

        # Add eighth note pickup before strong beats in final section
        if beat >= 145 and beat_in_measure == 3:
            events.append((beats_to_ticks(beat + 0.5), "note_on", note, 75))
            events.append((beats_to_ticks(beat + 0.8), "note_off", note, 0))

    # Final roll leading to conclusion
    for i, tick_offset in enumerate(range(0, beats_to_ticks(3), beats_to_ticks(0.25))):
        velocity = 85 + i * 2
        events.append(
            (beats_to_ticks(177) + tick_offset, "note_on", NOTES["D2"], min(velocity, 100))
        )
        events.append(
            (beats_to_ticks(177) + tick_offset + beats_to_ticks(0.15), "note_off", NOTES["D2"], 0)
        )

    # Sort events
    events.sort(key=lambda x: (x[0], x[1] == "note_on"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'Class Awakening'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_choir_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_piano_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_timpani_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = (
        "/home/user/projects/game/babylon/assets/music/revolutionary/03_class_awakening.mid"
    )

    print("Creating 'Class Awakening' - Revolutionary Suite 03")
    print("=" * 55)

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
    print("Musical arc: Revelation -> Understanding -> Determination")
    print("Key progression: D minor -> D major (Picardy third)")
    print("\nINTEGRATION GUIDANCE:")
    print("- Trigger when SocialClass.consciousness crosses threshold (e.g., 0.7)")
    print("- Loop from beat 49 for sustained awakened state")
    print("- Use Section C (beats 113+) for revolutionary event triggers")


if __name__ == "__main__":
    main()
