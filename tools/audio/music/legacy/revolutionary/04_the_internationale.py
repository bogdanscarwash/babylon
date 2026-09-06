#!/usr/bin/env python3
"""
BABYLON - Revolutionary Suite
04_the_internationale.mid - "The Internationale"

CONCEPTUAL BRIEF:
This is the climax of the Revolutionary Suite - the moment when workers of the
world unite in coordinated international action. After the spark of consciousness,
the flames of struggle, and the gathering storm, we arrive at VICTORY. The music
must feel EARNED - triumphant not because victory is easy, but because it was
forged through collective struggle. This is the anthem of the organized masses.

TECHNICAL SPECIFICATION:
- Key: F major (bright, victorious, the people's key)
- Tempo: 130 BPM (triumphant marching tempo)
- Time Signature: 4/4 (strong, unified march)
- Duration: ~100 seconds (216 beats at 130 BPM)
- Loop Points: Beat 49 (after intro fanfare) for gameplay looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Brass Section (Program 61) - Victory fanfares
- Channel 1: Choir Aahs (Program 52) - United voices of the masses
- Channel 2: String Ensemble (Program 48) - Driving rhythmic foundation
- Channel 3: French Horn (Program 60) - Noble solidarity theme
- Channel 4: Timpani (Program 47) - The people's unstoppable march
- Channel 9: Snare Drum (Percussion) - Organizational precision

MUSICAL ARC (100 seconds = ~216 beats at 130 BPM):
A. Assembly (beats 0-48): Fanfare call to action, forces gathering
B. The March (beats 49-128): Full texture, unstoppable forward momentum
C. Victory (beats 129-216): Triumphant climax, the world transformed

COMPOSITIONAL NOTES:
- F major is historically associated with pastoral joy and triumph
- The brass fanfares echo revolutionary anthems (Shostakovich's 5th finale)
- The snare provides military precision - but this is the people's army
- The choir represents the unified voice of international solidarity
- French horn carries the noble theme of "Solidarity Forever"
- The 130 BPM tempo creates urgency without losing dignity
- Harmonic language: I-IV-V-I progressions for clarity and power
- The piece should feel like an unstoppable historical force
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 130
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 217  # ~100 seconds at 130 BPM

# Note definitions (MIDI note numbers) - extended range for full orchestration
NOTES = {
    # Bass register
    "C2": 36,
    "D2": 38,
    "E2": 40,
    "F2": 41,
    "G2": 43,
    "A2": 45,
    "Bb2": 46,
    "B2": 47,
    # Low register
    "C3": 48,
    "D3": 50,
    "E3": 52,
    "F3": 53,
    "G3": 55,
    "A3": 57,
    "Bb3": 58,
    "B3": 59,
    # Middle register
    "C4": 60,
    "D4": 62,
    "E4": 64,
    "F4": 65,
    "G4": 67,
    "A4": 69,
    "Bb4": 70,
    "B4": 71,
    # Upper register
    "C5": 72,
    "D5": 74,
    "E5": 76,
    "F5": 77,
    "G5": 79,
    "A5": 81,
    "Bb5": 82,
    "B5": 83,
    # High register
    "C6": 84,
    "D6": 86,
    "E6": 88,
    "F6": 89,
    "G6": 91,
}

# Percussion note numbers (GM Standard - Channel 9)
PERC = {
    "BASS_DRUM": 35,
    "SNARE": 38,
    "CLOSED_HH": 42,
    "OPEN_HH": 46,
    "CRASH": 49,
    "RIDE": 51,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo, time signature, and markers."""
    track = MidiTrack()
    track.name = "Conductor"
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="F", time=0))
    track.append(MetaMessage("track_name", name="The Internationale", time=0))

    # Section markers for game integration
    track.append(MetaMessage("marker", text="Assembly", time=0))
    track.append(MetaMessage("marker", text="The March", time=beats_to_ticks(49)))
    track.append(MetaMessage("marker", text="Victory", time=beats_to_ticks(80)))  # 129-49=80 delta

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS - 129)))
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 1: Brass Section - Victory (Program 61, Channel 0)
    Bold, triumphant fanfares that announce the arrival of the organized masses.
    The brass represents the clarion call of revolution - clear, powerful, undeniable.
    """
    track = MidiTrack()
    track.name = "Brass Section - Victory"
    track.append(Message("program_change", program=61, channel=0, time=0))

    notes = [
        # ===== SECTION A: ASSEMBLY (beats 0-48) =====
        # Opening fanfare - the call goes out
        ("F4", 0, 2, 95),
        ("A4", 0, 2, 95),
        ("C5", 0, 2, 95),
        # Ascending motif - rising up
        ("F4", 3, 1, 90),
        ("G4", 4, 1, 92),
        ("A4", 5, 1, 94),
        ("Bb4", 6, 2, 96),
        # Second fanfare - the response
        ("F4", 9, 2, 98),
        ("A4", 9, 2, 98),
        ("C5", 9, 2, 98),
        ("F5", 9, 2, 98),
        # Descending answer
        ("C5", 12, 1, 92),
        ("Bb4", 13, 1, 90),
        ("A4", 14, 2, 88),
        # Building phrase
        ("F4", 17, 3, 95),
        ("A4", 17, 3, 95),
        ("C5", 20, 2, 97),
        ("D5", 22, 2, 98),
        # Climax of assembly call
        ("F4", 25, 4, 100),
        ("A4", 25, 4, 100),
        ("C5", 25, 4, 100),
        ("F5", 25, 4, 100),
        # Sustained power chord
        ("F4", 30, 6, 95),
        ("C5", 30, 6, 95),
        ("F5", 30, 6, 95),
        # Transitional fanfare
        ("Bb4", 37, 2, 92),
        ("D5", 37, 2, 92),
        ("F5", 37, 2, 92),
        ("C5", 40, 2, 94),
        ("E5", 40, 2, 94),
        ("G5", 40, 2, 94),
        # Lead into march
        ("F4", 44, 4, 100),
        ("A4", 44, 4, 100),
        ("C5", 44, 4, 100),
        # ===== SECTION B: THE MARCH (beats 49-128) =====
        # Strong rhythmic brass punches
        ("F3", 49, 2, 100),
        ("C4", 49, 2, 100),
        ("F4", 49, 2, 100),
        ("F3", 53, 2, 95),
        ("C4", 53, 2, 95),
        ("F4", 53, 2, 95),
        # Bb chord
        ("Bb3", 57, 2, 100),
        ("D4", 57, 2, 100),
        ("F4", 57, 2, 100),
        ("Bb3", 61, 2, 95),
        ("D4", 61, 2, 95),
        ("F4", 61, 2, 95),
        # C chord (dominant)
        ("C4", 65, 2, 100),
        ("E4", 65, 2, 100),
        ("G4", 65, 2, 100),
        ("C4", 69, 2, 95),
        ("E4", 69, 2, 95),
        ("G4", 69, 2, 95),
        # Return to F
        ("F3", 73, 4, 100),
        ("A3", 73, 4, 100),
        ("C4", 73, 4, 100),
        ("F4", 73, 4, 100),
        # Second march phrase
        ("F4", 81, 2, 100),
        ("A4", 81, 2, 100),
        ("C5", 81, 2, 100),
        ("Bb4", 85, 2, 98),
        ("D5", 85, 2, 98),
        ("F5", 85, 2, 98),
        ("C5", 89, 2, 100),
        ("E5", 89, 2, 100),
        ("G5", 89, 2, 100),
        ("F4", 93, 4, 100),
        ("A4", 93, 4, 100),
        ("C5", 93, 4, 100),
        ("F5", 93, 4, 100),
        # Building intensity
        ("F4", 101, 2, 102),
        ("A4", 101, 2, 102),
        ("C5", 101, 2, 102),
        ("G4", 105, 2, 100),
        ("Bb4", 105, 2, 100),
        ("D5", 105, 2, 100),
        ("A4", 109, 2, 104),
        ("C5", 109, 2, 104),
        ("E5", 109, 2, 104),
        ("Bb4", 113, 2, 106),
        ("D5", 113, 2, 106),
        ("F5", 113, 2, 106),
        # Pre-victory buildup
        ("C5", 117, 4, 108),
        ("E5", 117, 4, 108),
        ("G5", 117, 4, 108),
        ("F4", 121, 8, 110),
        ("A4", 121, 8, 110),
        ("C5", 121, 8, 110),
        ("F5", 121, 8, 110),
        # ===== SECTION C: VICTORY (beats 129-216) =====
        # Triumphant fanfare - the world is won!
        ("F4", 129, 4, 115),
        ("A4", 129, 4, 115),
        ("C5", 129, 4, 115),
        ("F5", 129, 4, 115),
        # Ascending victory theme
        ("F5", 133, 2, 112),
        ("G5", 135, 2, 114),
        ("A5", 137, 4, 116),
        # Answer phrase
        ("F5", 141, 2, 114),
        ("E5", 143, 2, 112),
        ("D5", 145, 2, 110),
        ("C5", 147, 4, 115),
        # Second victory phrase
        ("F4", 153, 4, 115),
        ("A4", 153, 4, 115),
        ("C5", 153, 4, 115),
        ("F5", 153, 4, 115),
        ("Bb4", 157, 4, 112),
        ("D5", 157, 4, 112),
        ("F5", 157, 4, 112),
        ("C5", 161, 4, 110),
        ("E5", 161, 4, 110),
        ("G5", 161, 4, 110),
        ("F4", 165, 8, 118),
        ("A4", 165, 8, 118),
        ("C5", 165, 8, 118),
        ("F5", 165, 8, 118),
        # Final triumphant section
        ("F4", 177, 4, 120),
        ("A4", 177, 4, 120),
        ("C5", 177, 4, 120),
        ("F5", 177, 4, 120),
        ("Bb4", 181, 4, 118),
        ("D5", 181, 4, 118),
        ("F5", 181, 4, 118),
        ("Bb5", 181, 4, 118),
        ("C5", 185, 4, 116),
        ("E5", 185, 4, 116),
        ("G5", 185, 4, 116),
        # Build to final cadence
        ("F4", 189, 4, 120),
        ("A4", 189, 4, 120),
        ("C5", 189, 4, 120),
        ("F5", 189, 4, 120),
        ("G4", 193, 4, 118),
        ("Bb4", 193, 4, 118),
        ("D5", 193, 4, 118),
        ("A4", 197, 4, 120),
        ("C5", 197, 4, 120),
        ("F5", 197, 4, 120),
        ("Bb4", 201, 4, 118),
        ("D5", 201, 4, 118),
        ("F5", 201, 4, 118),
        # FINAL CHORD - The world is ours!
        ("F3", 205, 12, 127),
        ("C4", 205, 12, 127),
        ("F4", 205, 12, 127),
        ("A4", 205, 12, 127),
        ("C5", 205, 12, 127),
        ("F5", 205, 12, 127),
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        vel = min(velocity, 127)  # Clamp velocity
        events.append((beats_to_ticks(start), "note_on", note, vel))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=0, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_choir_track() -> MidiTrack:
    """
    Track 2: Choir Aahs - United Voices (Program 52, Channel 1)
    The unified voice of the international working class - millions singing as one.
    Sustained harmonies that represent the collective will of the organized masses.
    """
    track = MidiTrack()
    track.name = "Choir Aahs - United Voices"
    track.append(Message("program_change", program=52, channel=1, time=0))

    notes = [
        # ===== SECTION A: ASSEMBLY (beats 0-48) =====
        # Choir enters at beat 16 - the masses gathering
        ("F4", 16, 8, 70),
        ("A4", 16, 8, 70),
        ("C5", 16, 8, 70),
        # Building swell
        ("F4", 25, 12, 80),
        ("A4", 25, 12, 80),
        ("C5", 25, 12, 80),
        # Transitional phrase
        ("Bb4", 38, 6, 78),
        ("D5", 38, 6, 78),
        ("F5", 38, 6, 78),
        ("C5", 44, 5, 82),
        ("E5", 44, 5, 82),
        # ===== SECTION B: THE MARCH (beats 49-128) =====
        # Full choral sound - the masses united
        ("F4", 49, 8, 88),
        ("A4", 49, 8, 88),
        ("C5", 49, 8, 88),
        ("Bb4", 57, 8, 86),
        ("D5", 57, 8, 86),
        ("F5", 57, 8, 86),
        ("C4", 65, 8, 90),
        ("E4", 65, 8, 90),
        ("G4", 65, 8, 90),
        ("C5", 65, 8, 90),
        ("F4", 73, 8, 92),
        ("A4", 73, 8, 92),
        ("C5", 73, 8, 92),
        ("F5", 73, 8, 92),
        # Second march phrase
        ("F4", 81, 12, 94),
        ("A4", 81, 12, 94),
        ("C5", 81, 12, 94),
        ("F5", 81, 12, 94),
        ("Bb4", 93, 8, 92),
        ("D5", 93, 8, 92),
        ("F5", 93, 8, 92),
        ("C5", 101, 8, 96),
        ("E5", 101, 8, 96),
        ("G5", 101, 8, 96),
        ("F4", 109, 12, 98),
        ("A4", 109, 12, 98),
        ("C5", 109, 12, 98),
        ("F5", 109, 12, 98),
        # Building to victory
        ("C5", 121, 8, 100),
        ("E5", 121, 8, 100),
        ("G5", 121, 8, 100),
        # ===== SECTION C: VICTORY (beats 129-216) =====
        # Full power - the people have won!
        ("F4", 129, 12, 105),
        ("A4", 129, 12, 105),
        ("C5", 129, 12, 105),
        ("F5", 129, 12, 105),
        ("Bb4", 141, 12, 102),
        ("D5", 141, 12, 102),
        ("F5", 141, 12, 102),
        ("C5", 153, 12, 108),
        ("E5", 153, 12, 108),
        ("G5", 153, 12, 108),
        ("F4", 165, 12, 110),
        ("A4", 165, 12, 110),
        ("C5", 165, 12, 110),
        ("F5", 165, 12, 110),
        # Triumphant final phrase
        ("F4", 177, 8, 112),
        ("A4", 177, 8, 112),
        ("C5", 177, 8, 112),
        ("F5", 177, 8, 112),
        ("Bb4", 185, 8, 110),
        ("D5", 185, 8, 110),
        ("F5", 185, 8, 110),
        ("C5", 193, 12, 115),
        ("E5", 193, 12, 115),
        ("G5", 193, 12, 115),
        # FINAL CHORD
        ("F4", 205, 12, 120),
        ("A4", 205, 12, 120),
        ("C5", 205, 12, 120),
        ("F5", 205, 12, 120),
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        vel = min(velocity, 127)
        events.append((beats_to_ticks(start), "note_on", note, vel))
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
    Track 3: Strings - The March (Program 48, Channel 2)
    Driving rhythmic strings that propel the march forward.
    Relentless, unstoppable momentum - history in motion.
    """
    track = MidiTrack()
    track.name = "Strings - The March"
    track.append(Message("program_change", program=48, channel=2, time=0))

    notes = []

    # ===== SECTION A: ASSEMBLY (beats 0-48) =====
    # Strings enter at beat 8 with sustained foundation
    assembly_chords = [
        (8, 8, ["F3", "C4", "F4"], 72),
        (16, 8, ["F3", "A3", "C4", "F4"], 78),
        (25, 12, ["F3", "A3", "C4", "F4"], 85),
        (37, 6, ["Bb3", "D4", "F4"], 80),
        (44, 5, ["C4", "E4", "G4"], 82),
    ]
    for start, dur, chord, vel in assembly_chords:
        for note_name in chord:
            notes.append((note_name, start, dur, vel))

    # ===== SECTION B: THE MARCH (beats 49-128) =====
    # Driving eighth-note rhythm - the unstoppable march
    march_pattern_limit = 40  # Number of iterations for march section
    for i in range(march_pattern_limit):
        beat = 49 + i * 2
        if beat >= 129:
            break
        # Alternating bass pattern
        if i % 4 == 0:
            chord = ["F2", "C3", "F3"]
            vel = 90
        elif i % 4 == 1:
            chord = ["F2", "A2", "C3"]
            vel = 85
        elif i % 4 == 2:
            chord = ["Bb2", "D3", "F3"]
            vel = 88
        else:
            chord = ["C3", "E3", "G3"]
            vel = 87

        # Crescendo through the march
        vel = min(vel + i // 4, 110)

        for note_name in chord:
            notes.append((note_name, beat, 1.5, vel))

    # ===== SECTION C: VICTORY (beats 129-216) =====
    # Full string power - sustained triumph
    victory_chords = [
        (129, 12, ["F2", "C3", "F3", "A3", "C4", "F4"], 105),
        (141, 12, ["Bb2", "F3", "Bb3", "D4", "F4"], 102),
        (153, 12, ["C3", "G3", "C4", "E4", "G4"], 108),
        (165, 12, ["F2", "C3", "F3", "A3", "C4", "F4"], 112),
        (177, 8, ["F2", "C3", "F3", "A3", "C4"], 115),
        (185, 8, ["Bb2", "D3", "F3", "Bb3", "D4"], 112),
        (193, 12, ["C3", "E3", "G3", "C4", "E4"], 118),
        (205, 12, ["F2", "C3", "F3", "A3", "C4", "F4"], 125),
    ]
    for start, dur, chord, vel in victory_chords:
        for note_name in chord:
            notes.append((note_name, start, dur, min(vel, 127)))

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


def create_french_horn_track() -> MidiTrack:
    """
    Track 4: French Horn - Solidarity Forever (Program 60, Channel 3)
    Noble, sustaining melody that carries the theme of eternal solidarity.
    The horn represents dignity, honor, and the unbreakable bonds between workers.
    """
    track = MidiTrack()
    track.name = "French Horn - Solidarity Forever"
    track.append(Message("program_change", program=60, channel=3, time=0))

    notes = [
        # ===== SECTION A: ASSEMBLY (beats 0-48) =====
        # Horn enters with noble call at beat 4
        ("F4", 4, 3, 75),
        ("A4", 8, 2, 78),
        ("C5", 11, 3, 80),
        ("F4", 16, 4, 82),
        ("G4", 20, 2, 80),
        ("A4", 22, 4, 85),
        # Building phrase
        ("Bb4", 28, 4, 88),
        ("A4", 32, 2, 85),
        ("G4", 34, 2, 82),
        ("F4", 36, 4, 80),
        # Transition to march
        ("C5", 42, 3, 85),
        ("Bb4", 45, 3, 82),
        # ===== SECTION B: THE MARCH (beats 49-128) =====
        # Solidarity theme - noble and unstoppable
        ("F4", 49, 4, 90),
        ("A4", 53, 2, 88),
        ("C5", 55, 2, 90),
        ("D5", 57, 4, 92),
        ("C5", 61, 2, 90),
        ("Bb4", 63, 2, 88),
        ("A4", 65, 4, 90),
        ("G4", 69, 2, 88),
        ("F4", 71, 2, 90),
        ("F4", 73, 4, 95),
        # Second statement
        ("C5", 81, 4, 95),
        ("D5", 85, 2, 92),
        ("E5", 87, 2, 94),
        ("F5", 89, 4, 98),
        ("E5", 93, 2, 95),
        ("D5", 95, 2, 92),
        ("C5", 97, 4, 95),
        # Building intensity
        ("F4", 105, 4, 98),
        ("G4", 109, 2, 96),
        ("A4", 111, 2, 98),
        ("Bb4", 113, 4, 100),
        ("C5", 117, 4, 102),
        ("D5", 121, 4, 105),
        ("C5", 125, 4, 108),
        # ===== SECTION C: VICTORY (beats 129-216) =====
        # Triumphant solidarity theme
        ("F5", 129, 4, 110),
        ("E5", 133, 2, 108),
        ("D5", 135, 2, 106),
        ("C5", 137, 4, 110),
        ("D5", 141, 4, 108),
        ("E5", 145, 4, 112),
        ("F5", 149, 4, 115),
        # Second victorious phrase
        ("F5", 157, 4, 115),
        ("G5", 161, 4, 118),
        ("A5", 165, 8, 120),
        # Descending glory
        ("G5", 173, 4, 118),
        ("F5", 177, 4, 115),
        ("E5", 181, 4, 112),
        ("D5", 185, 4, 110),
        ("C5", 189, 4, 115),
        # Final approach
        ("D5", 193, 4, 118),
        ("E5", 197, 4, 120),
        ("F5", 201, 4, 122),
        # FINAL NOTE
        ("F5", 205, 12, 127),
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        vel = min(velocity, 127)
        events.append((beats_to_ticks(start), "note_on", note, vel))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 5: Timpani - The People's March (Program 47, Channel 4)
    Strong, regular, unstoppable - the drums of history.
    The timpani represents the inexorable march of the organized masses.
    """
    track = MidiTrack()
    track.name = "Timpani - The People's March"
    track.append(Message("program_change", program=47, channel=4, time=0))

    events = []

    # ===== SECTION A: ASSEMBLY (beats 0-48) =====
    # Timpani enters at beat 25 with the gathering
    assembly_limit = 24
    for i in range(assembly_limit):
        beat = 25 + i
        if beat >= 49:
            break
        # Downbeat emphasis
        if i % 4 == 0:
            note = NOTES["F2"]
            velocity = 85
        elif i % 4 == 2:
            note = NOTES["C2"]
            velocity = 75
        else:
            continue  # Only on 1 and 3

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # ===== SECTION B: THE MARCH (beats 49-128) =====
    # Full march pattern - unstoppable momentum
    march_limit = 80
    for i in range(march_limit):
        beat = 49 + i
        if beat >= 129:
            break
        # Strong pattern: 1-2-3-4 with emphasis on 1 and 3
        beat_in_bar = i % 4

        if beat_in_bar == 0:
            note = NOTES["F2"]
            velocity = min(100 + i // 8, 115)
        elif beat_in_bar == 2:
            note = NOTES["C2"]
            velocity = min(85 + i // 8, 100)
        else:
            continue

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # ===== SECTION C: VICTORY (beats 129-216) =====
    # Triumphant rolls and strong beats
    victory_limit = 88
    for i in range(victory_limit):
        beat = 129 + i
        if beat >= 217:
            break
        beat_in_bar = i % 4

        if beat_in_bar == 0:
            note = NOTES["F2"]
            velocity = 115
        elif beat_in_bar == 2:
            note = NOTES["C2"]
            velocity = 105
        elif beat_in_bar == 1 or beat_in_bar == 3:
            # Add off-beat hits in victory section
            note = NOTES["F2"] if beat_in_bar == 1 else NOTES["C2"]
            velocity = 70
        else:
            continue

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Final timpani roll on last chord
    roll_limit = 24
    for i in range(roll_limit):
        beat = 205 + i * 0.25
        if beat >= 217:
            break
        note = NOTES["F2"]
        velocity = min(100 + i * 2, 127)
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.2), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_snare_track() -> MidiTrack:
    """
    Track 6: Snare Drum - Organization (Channel 9 - Percussion)
    Military precision, but for the people's army.
    The snare represents organizational discipline - coordinated action.
    """
    track = MidiTrack()
    track.name = "Snare - Organization"
    # Channel 9 is percussion - no program change needed for GM

    events = []
    snare = PERC["SNARE"]
    bass_drum = PERC["BASS_DRUM"]
    crash = PERC["CRASH"]

    # ===== SECTION A: ASSEMBLY (beats 0-48) =====
    # Snare enters at beat 33 with military cadence
    assembly_limit = 16
    for i in range(assembly_limit):
        beat = 33 + i
        if beat >= 49:
            break
        beat_in_bar = i % 4

        if beat_in_bar == 0:
            events.append((beats_to_ticks(beat), "note_on", bass_drum, 85))
            events.append((beats_to_ticks(beat + 0.3), "note_off", bass_drum, 0))
        elif beat_in_bar == 2:
            events.append((beats_to_ticks(beat), "note_on", snare, 80))
            events.append((beats_to_ticks(beat + 0.2), "note_off", snare, 0))

    # Crash on downbeat of march
    events.append((beats_to_ticks(49), "note_on", crash, 100))
    events.append((beats_to_ticks(49.5), "note_off", crash, 0))

    # ===== SECTION B: THE MARCH (beats 49-128) =====
    # Full military cadence - organized precision
    march_limit = 80
    for i in range(march_limit):
        beat = 49 + i
        if beat >= 129:
            break
        beat_in_bar = i % 4
        sixteenth = (i * 4) % 16

        # Pattern: BD on 1, snare on 2 and 4
        if beat_in_bar == 0:
            events.append((beats_to_ticks(beat), "note_on", bass_drum, 95))
            events.append((beats_to_ticks(beat + 0.3), "note_off", bass_drum, 0))
        elif beat_in_bar == 1 or beat_in_bar == 3:
            events.append((beats_to_ticks(beat), "note_on", snare, 85))
            events.append((beats_to_ticks(beat + 0.2), "note_off", snare, 0))

        # Crash every 16 beats
        if i > 0 and i % 16 == 0:
            events.append((beats_to_ticks(beat), "note_on", crash, 95))
            events.append((beats_to_ticks(beat + 0.5), "note_off", crash, 0))

    # Crash on victory
    events.append((beats_to_ticks(129), "note_on", crash, 110))
    events.append((beats_to_ticks(129.5), "note_off", crash, 0))

    # ===== SECTION C: VICTORY (beats 129-216) =====
    # Triumphant pattern - full power
    victory_limit = 88
    for i in range(victory_limit):
        beat = 129 + i
        if beat >= 217:
            break
        beat_in_bar = i % 4

        # Fuller pattern with more energy
        if beat_in_bar == 0:
            events.append((beats_to_ticks(beat), "note_on", bass_drum, 105))
            events.append((beats_to_ticks(beat + 0.3), "note_off", bass_drum, 0))
        elif beat_in_bar == 2:
            events.append((beats_to_ticks(beat), "note_on", bass_drum, 90))
            events.append((beats_to_ticks(beat + 0.3), "note_off", bass_drum, 0))

        if beat_in_bar == 1 or beat_in_bar == 3:
            events.append((beats_to_ticks(beat), "note_on", snare, 95))
            events.append((beats_to_ticks(beat + 0.2), "note_off", snare, 0))

        # Crash every 12 beats in victory
        if i > 0 and i % 12 == 0:
            events.append((beats_to_ticks(beat), "note_on", crash, 105))
            events.append((beats_to_ticks(beat + 0.5), "note_off", crash, 0))

    # Final crash
    events.append((beats_to_ticks(205), "note_on", crash, 127))
    events.append((beats_to_ticks(206), "note_off", crash, 0))

    # Snare roll to end
    roll_limit = 24
    for i in range(roll_limit):
        beat = 205 + i * 0.25
        if beat >= 217:
            break
        velocity = min(90 + i * 2, 120)
        events.append((beats_to_ticks(beat), "note_on", snare, velocity))
        events.append((beats_to_ticks(beat + 0.1), "note_off", snare, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=9, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'The Internationale'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_choir_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_french_horn_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_snare_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = (
        "/home/user/projects/game/babylon/assets/music/revolutionary/04_the_internationale.mid"
    )

    print("Creating 'The Internationale' - Revolutionary Suite 04")
    print("=" * 60)
    print("The climax of the Revolutionary Suite")
    print("Workers of the world, unite!")
    print("=" * 60)

    mid = create_midi_file()
    mid.save(output_path)

    print(f"\nSaved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Tempo: {BPM} BPM")
    print("Key: F major")
    print(f"Track count: {len(mid.tracks)}")

    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    # Calculate duration
    length = mid.length
    print(f"\nDuration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\n" + "=" * 60)
    print("COMPOSITIONAL NOTES:")
    print("-" * 60)
    print("Musical Arc: Assembly -> The March -> Victory")
    print("")
    print("Section A (Assembly, beats 0-48):")
    print("  - Brass fanfares call the workers to gather")
    print("  - Choir enters at beat 16 as masses assemble")
    print("  - Building intensity toward the march")
    print("")
    print("Section B (The March, beats 49-128):")
    print("  - Full orchestration - the unstoppable forward movement")
    print("  - Driving rhythmic strings propel the march")
    print("  - Snare provides military precision for the people's army")
    print("")
    print("Section C (Victory, beats 129-216):")
    print("  - Triumphant climax - the world is won!")
    print("  - Full choral and brass power")
    print("  - Final chord sustains with timpani and snare roll")
    print("")
    print("=" * 60)
    print("INTEGRATION GUIDANCE:")
    print("-" * 60)
    print("- Play during: Successful revolution, max organization events")
    print("- Loop point: Beat 49 (after assembly fanfare)")
    print("- Crossfade from: 03_the_gathering_storm.mid")
    print("- Trigger conditions:")
    print("    * Organization >= 0.9")
    print("    * Revolution probability >= 0.8")
    print("    * Successful rupture event")
    print("=" * 60)


if __name__ == "__main__":
    main()
