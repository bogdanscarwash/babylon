#!/usr/bin/env python3
"""
BABYLON - Fascist Suite
02_scapegoat.mid - "The Scapegoat"

CONCEPTUAL BRIEF:
This piece represents the psychological mechanism of scapegoating - the atomized
worker, lacking class analysis and solidarity networks, redirects systemic anger
toward "the other." Without understanding that their suffering stems from capital's
extraction of surplus value, the isolated worker seeks a visible, vulnerable target.
The music captures the hunting, paranoid quality of this misdirected rage.

TECHNICAL SPECIFICATION:
- Key: G minor (aggressive, hunting, dark)
- Tempo: 95 BPM (agitated, searching)
- Time Signature: 4/4
- Duration: ~90 seconds (142 beats at 95 BPM)
- Loop Points: Beat 48 for looping (after search phase establishes)

INSTRUMENT ASSIGNMENTS:
- Channel 0: Strings (Program 48) - "Hunting" - Sharp, stalking figures
- Channel 1: Low Brass/Tuba (Program 58) - "Accusation" - Pointing, blaming gestures
- Channel 2: Snare Drum (Program 115) - "Mob Heartbeat" - Increasingly insistent
- Channel 3: Piano (Program 0) - "Paranoia" - Darting, suspicious phrases

MUSICAL ARC (90 seconds = ~142 beats at 95 BPM):
A. Searching (beats 0-47): Restless strings, paranoid piano, uneasy atmosphere
B. Finding a Target (beats 48-95): Brass enters with accusatory gestures, snare intensifies
C. Accusation (beats 96-142): Full texture pointing, blaming, mob crescendo

COMPOSITIONAL NOTES:
- G minor provides the aggressive, hunting quality needed for scapegoating
- Stalking string figures represent the predatory searching for a target
- Brass "pointing" motifs are short, stabbing, accusatory
- Snare drum represents the gathering mob mentality, increasingly insistent
- Piano's darting phrases represent the paranoid, suspicious mindset
- The arc moves from diffuse anxiety to focused aggression
- Unlike revolutionary music, there is no resolution - only escalating blame
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 95
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 143  # ~90 seconds at 95 BPM

# Note definitions (MIDI note numbers) - G minor scale and chromatic neighbors
NOTES = {
    # Low register for bass/brass
    "D1": 26,
    "G1": 31,
    "A1": 33,
    "Bb1": 34,
    "C2": 36,
    "D2": 38,
    "Eb2": 39,
    "F2": 41,
    "F#2": 42,
    "G2": 43,
    "A2": 45,
    "Bb2": 46,
    "C3": 48,
    "D3": 50,
    "Eb3": 51,
    "F3": 53,
    "F#3": 54,
    "G3": 55,
    "A3": 57,
    "Bb3": 58,
    "C4": 60,
    "D4": 62,
    "Eb4": 63,
    "F4": 65,
    "F#4": 66,
    "G4": 67,
    "A4": 69,
    "Bb4": 70,
    "C5": 72,
    "D5": 74,
    "Eb5": 75,
    "F5": 77,
    "G5": 79,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="The Scapegoat", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    # G minor - using Bb major signature (2 flats), minor mode
    track.append(MetaMessage("key_signature", key="Bb", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 1: Strings - Hunting (Program 48, Channel 0)
    Sharp, stalking figures that search restlessly for a target.
    Begins immediately - the hunt is already underway.
    """
    track = MidiTrack()
    track.name = "Strings - Hunting"
    track.append(Message("program_change", program=48, channel=0, time=0))

    notes = []

    # Section A (beats 0-47): Searching - stalking figures
    # Motif 1: Rising chromatic tension (searching)
    search_patterns = [
        # Beat, notes in quick succession
        (0, [("G3", 0.5, 55), ("A3", 0.5, 58), ("Bb3", 1, 52)]),
        (3, [("F3", 0.5, 52), ("G3", 0.5, 55), ("A3", 0.75, 50)]),
        (6, [("D4", 0.75, 60), ("Eb4", 0.5, 58), ("D4", 0.5, 55)]),
        (9, [("G3", 1.5, 48)]),
        (12, [("A3", 0.5, 58), ("Bb3", 0.5, 62), ("C4", 0.75, 58), ("Bb3", 0.5, 55)]),
        (15, [("G3", 0.5, 52), ("F#3", 0.5, 55), ("G3", 1, 50)]),
        (18, [("D4", 0.5, 62), ("C4", 0.5, 58), ("Bb3", 0.5, 55), ("A3", 0.75, 52)]),
        (21, [("G3", 1.5, 48)]),
        # More agitated searching
        (24, [("Bb3", 0.5, 65), ("C4", 0.5, 68), ("D4", 0.75, 65)]),
        (27, [("Eb4", 0.5, 62), ("D4", 0.5, 65), ("C4", 0.5, 60), ("Bb3", 0.75, 58)]),
        (30, [("A3", 0.5, 65), ("Bb3", 0.5, 68), ("C4", 0.75, 72), ("D4", 0.5, 70)]),
        (33, [("Eb4", 1, 68), ("D4", 0.5, 65)]),
        (36, [("G4", 0.5, 72), ("F4", 0.5, 68), ("Eb4", 0.5, 65), ("D4", 0.75, 62)]),
        (39, [("C4", 0.5, 60), ("Bb3", 0.5, 58), ("A3", 1, 55)]),
        (42, [("D4", 0.5, 68), ("Eb4", 0.5, 72), ("F4", 0.75, 75), ("Eb4", 0.5, 70)]),
        (45, [("D4", 1.5, 65)]),
    ]

    for base_beat, pattern in search_patterns:
        offset = 0.0
        for note_name, duration, velocity in pattern:
            notes.append((note_name, base_beat + offset, duration, velocity))
            offset += duration

    # Section B (beats 48-95): Finding a Target - more focused, aggressive
    target_patterns = [
        (48, [("G4", 0.5, 78), ("D4", 0.5, 75), ("G4", 1, 80)]),
        (51, [("F4", 0.5, 72), ("Eb4", 0.5, 70), ("D4", 1, 68)]),
        (54, [("G4", 0.5, 82), ("A4", 0.5, 85), ("Bb4", 0.75, 82)]),
        (57, [("A4", 0.5, 78), ("G4", 1.5, 75)]),
        (60, [("D5", 0.5, 88), ("C5", 0.5, 85), ("Bb4", 0.5, 82), ("A4", 0.75, 80)]),
        (63, [("G4", 0.5, 78), ("F4", 0.5, 75), ("Eb4", 1, 72)]),
        (66, [("D4", 0.5, 80), ("Eb4", 0.5, 82), ("F4", 0.5, 85), ("G4", 0.75, 88)]),
        (69, [("A4", 0.5, 85), ("Bb4", 1.5, 82)]),
        # Circling the target
        (72, [("G4", 0.5, 85), ("G4", 0.5, 88), ("G4", 0.5, 90)]),
        (75, [("D4", 0.5, 82), ("Eb4", 0.5, 85), ("F4", 1, 88)]),
        (78, [("G4", 0.5, 90), ("A4", 0.5, 92), ("Bb4", 0.75, 90)]),
        (81, [("C5", 0.5, 88), ("Bb4", 0.5, 85), ("A4", 1, 82)]),
        (84, [("G4", 0.5, 88), ("F4", 0.5, 85), ("Eb4", 0.5, 82), ("D4", 0.75, 80)]),
        (87, [("G4", 2, 85)]),
        (90, [("D5", 0.5, 92), ("C5", 0.5, 90), ("Bb4", 0.5, 88), ("A4", 0.75, 85)]),
        (93, [("G4", 2, 82)]),
    ]

    for base_beat, pattern in target_patterns:
        offset = 0.0
        for note_name, duration, velocity in pattern:
            notes.append((note_name, base_beat + offset, duration, velocity))
            offset += duration

    # Section C (beats 96-142): Accusation - sustained aggressive pointing
    accusation_patterns = [
        (96, [("G4", 2, 95), ("D5", 2, 95)]),  # Sustained accusation
        (100, [("Eb5", 1.5, 92), ("D5", 1.5, 90)]),
        (104, [("G4", 2, 95), ("Bb4", 2, 95), ("D5", 2, 95)]),  # Chord stab
        (108, [("F4", 1, 88), ("Eb4", 1, 85), ("D4", 2, 82)]),
        (112, [("G4", 2, 98), ("Bb4", 2, 98), ("D5", 2, 98)]),  # Louder
        (116, [("C5", 1.5, 95), ("Bb4", 1, 92), ("A4", 1.5, 90)]),
        (120, [("G4", 3, 100), ("D5", 3, 100)]),  # Peak accusation
        (124, [("Eb5", 1.5, 95), ("D5", 1, 92), ("C5", 1.5, 90)]),
        (128, [("G4", 2, 98), ("Bb4", 2, 98)]),
        (132, [("D5", 2, 100), ("G5", 2, 100)]),  # High pointing
        (136, [("Eb5", 1.5, 95), ("D5", 1, 92)]),
        (139, [("G4", 4, 100), ("Bb4", 4, 100), ("D5", 4, 100)]),  # Final chord
    ]

    for base_beat, pattern in accusation_patterns:
        for note_name, duration, velocity in pattern:
            notes.append((note_name, base_beat, duration, velocity))

    # Convert to events
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


def create_brass_track() -> MidiTrack:
    """
    Track 2: Low Brass/Tuba - Accusation (Program 58, Channel 1)
    Enters at beat 48 - the moment a target is found.
    Short, stabbing, pointing gestures that grow more insistent.
    """
    track = MidiTrack()
    track.name = "Low Brass - Accusation"
    # Program 58 = Tuba (more menacing than trombone)
    track.append(Message("program_change", program=58, channel=1, time=0))

    notes = []

    # Section B (beats 48-95): Finding a Target - accusatory stabs
    # Short, pointed phrases - "pointing the finger"
    brass_motifs = [
        # (beat, note, duration, velocity)
        (48, "G2", 0.75, 72),
        (50, "D2", 0.5, 70),
        (52, "G2", 1, 75),
        (56, "Bb2", 0.75, 72),
        (58, "A2", 0.5, 70),
        (60, "G2", 1.5, 78),
        # More insistent
        (64, "G2", 0.5, 80),
        (65, "G2", 0.5, 82),
        (66, "D3", 1, 78),
        (68, "C3", 0.75, 75),
        (70, "Bb2", 0.5, 72),
        (72, "G2", 1.5, 80),
        # Building accusation
        (76, "G2", 0.5, 82),
        (77, "A2", 0.5, 85),
        (78, "Bb2", 0.5, 88),
        (80, "D3", 1.5, 85),
        (84, "G2", 0.5, 85),
        (85, "Bb2", 0.5, 88),
        (86, "D3", 1, 90),
        (88, "C3", 0.75, 85),
        (90, "Bb2", 0.5, 82),
        (92, "G2", 2, 88),
    ]

    for beat, note_name, duration, velocity in brass_motifs:
        notes.append((note_name, beat, duration, velocity))

    # Section C (beats 96-142): Accusation - full pointing power
    accusation_motifs = [
        (96, "G2", 2, 95),
        (100, "D3", 1.5, 92),
        (102, "G2", 1.5, 90),
        (104, "G2", 0.5, 95),
        (105, "Bb2", 0.5, 98),
        (106, "D3", 2, 95),
        (110, "C3", 1, 90),
        (112, "Bb2", 1, 88),
        (114, "G2", 2, 95),
        # Climactic pointing
        (118, "G2", 0.5, 98),
        (119, "G2", 0.5, 100),
        (120, "D3", 3, 100),
        (124, "G2", 0.5, 95),
        (125, "Bb2", 0.5, 98),
        (126, "D3", 2, 100),
        (130, "G2", 1, 95),
        (132, "D2", 1, 92),
        (134, "G2", 2, 98),
        # Final accusatory blast
        (138, "G2", 0.5, 100),
        (139, "D3", 4, 100),
    ]

    for beat, note_name, duration, velocity in accusation_motifs:
        notes.append((note_name, beat, duration, velocity))

    # Convert to events
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


def create_snare_track() -> MidiTrack:
    """
    Track 3: Snare Drum - Mob Heartbeat (Channel 9, percussion)
    Program 115 maps to General MIDI percussion - using channel 9.
    Increasingly insistent rhythm representing mob mentality gathering.
    """
    track = MidiTrack()
    track.name = "Snare - Mob Heartbeat"
    # Channel 9 is percussion - note numbers map to instruments
    # Snare drum = note 38 (Acoustic Snare) or 40 (Electric Snare)
    SNARE = 38
    RIM = 37  # Side stick for quieter accents

    events = []

    # Section A (beats 0-47): Searching - sparse, uneasy pulse
    # Occasional rim clicks, building tension
    for beat in range(0, 48, 4):
        # Sparse rim clicks
        velocity = 35 + (beat // 4) * 2  # Gradual increase
        events.append((beats_to_ticks(beat), "note_on", RIM, velocity))
        events.append((beats_to_ticks(beat + 0.2), "note_off", RIM, 0))
        # Sometimes add an offbeat
        if beat % 8 == 4:
            events.append((beats_to_ticks(beat + 2), "note_on", RIM, velocity - 10))
            events.append((beats_to_ticks(beat + 2.2), "note_off", RIM, 0))

    # Section B (beats 48-95): Finding a Target - snare enters, more insistent
    for beat in range(48, 96):
        beat_in_measure = beat % 4

        # Basic pattern: snare on 2 and 4, building
        if beat_in_measure == 1 or beat_in_measure == 3:
            velocity = min(60 + (beat - 48), 85)
            events.append((beats_to_ticks(beat), "note_on", SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.3), "note_off", SNARE, 0))
        # Add 16th note pickups occasionally
        elif beat % 8 == 7:
            velocity = min(55 + (beat - 48), 75)
            events.append((beats_to_ticks(beat + 0.5), "note_on", SNARE, velocity - 10))
            events.append((beats_to_ticks(beat + 0.65), "note_off", SNARE, 0))
            events.append((beats_to_ticks(beat + 0.75), "note_on", SNARE, velocity - 5))
            events.append((beats_to_ticks(beat + 0.9), "note_off", SNARE, 0))

    # Section C (beats 96-142): Accusation - driving, relentless
    for beat in range(96, 143):
        beat_in_measure = beat % 4

        # Every beat now has snare
        velocity = min(85 + (beat - 96) // 4, 100)

        if beat_in_measure == 0 or beat_in_measure == 2:
            # Downbeats - accent
            events.append((beats_to_ticks(beat), "note_on", SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.25), "note_off", SNARE, 0))
        else:
            # Backbeats - even stronger accent (mob emphasis)
            events.append((beats_to_ticks(beat), "note_on", SNARE, min(velocity + 5, 110)))
            events.append((beats_to_ticks(beat + 0.3), "note_off", SNARE, 0))

        # Add 8th note offbeats in final section
        if beat >= 120:
            events.append((beats_to_ticks(beat + 0.5), "note_on", SNARE, velocity - 15))
            events.append((beats_to_ticks(beat + 0.65), "note_off", SNARE, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=9, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_piano_track() -> MidiTrack:
    """
    Track 4: Piano - Paranoia (Program 0, Channel 3)
    Darting, suspicious phrases that search nervously.
    Represents the paranoid mindset seeking enemies everywhere.
    """
    track = MidiTrack()
    track.name = "Piano - Paranoia"
    track.append(Message("program_change", program=0, channel=3, time=0))

    notes = []

    # Section A (beats 0-47): Searching - paranoid, darting
    paranoia_phrases = [
        # Quick, nervous phrases with wide leaps
        (2, [("D4", 0.25, 48), ("G4", 0.25, 52), ("Bb4", 0.5, 50)]),
        (5, [("F4", 0.25, 45), ("A4", 0.5, 48)]),
        (8, [("G4", 0.25, 52), ("D5", 0.25, 55), ("C5", 0.25, 50), ("Bb4", 0.5, 48)]),
        (11, [("A4", 0.25, 45), ("G4", 0.5, 42)]),
        (14, [("Eb4", 0.25, 55), ("G4", 0.25, 58), ("Bb4", 0.25, 55), ("D5", 0.5, 52)]),
        (17, [("C5", 0.25, 50), ("A4", 0.5, 48)]),
        (20, [("G4", 0.25, 55), ("Bb4", 0.25, 58), ("D5", 0.25, 60), ("Eb5", 0.5, 55)]),
        (23, [("D5", 0.25, 52), ("Bb4", 0.5, 48)]),
        # More agitated
        (26, [("G4", 0.2, 62), ("A4", 0.2, 65), ("Bb4", 0.2, 68), ("C5", 0.4, 65)]),
        (29, [("D5", 0.25, 68), ("Eb5", 0.25, 70), ("D5", 0.5, 65)]),
        (32, [("G4", 0.2, 70), ("Bb4", 0.2, 72), ("D5", 0.2, 75), ("F5", 0.4, 72)]),
        (35, [("Eb5", 0.25, 68), ("D5", 0.25, 65), ("C5", 0.5, 62)]),
        (38, [("Bb4", 0.2, 70), ("C5", 0.2, 72), ("D5", 0.2, 75), ("Eb5", 0.4, 72)]),
        (41, [("D5", 0.25, 68), ("Bb4", 0.5, 65)]),
        (44, [("G4", 0.2, 72), ("A4", 0.2, 75), ("Bb4", 0.2, 78), ("D5", 0.5, 75)]),
    ]

    for base_beat, pattern in paranoia_phrases:
        offset = 0.0
        for note_name, duration, velocity in pattern:
            notes.append((note_name, base_beat + offset, duration, velocity))
            offset += duration

    # Section B (beats 48-95): Finding a Target - more pointed, focused
    target_phrases = [
        (49, [("G4", 0.25, 75), ("D5", 0.5, 78)]),
        (52, [("Bb4", 0.25, 72), ("D5", 0.25, 75), ("G5", 0.5, 78)]),
        (55, [("F5", 0.25, 75), ("Eb5", 0.25, 72), ("D5", 0.5, 70)]),
        (58, [("G4", 0.2, 78), ("Bb4", 0.2, 80), ("D5", 0.5, 82)]),
        (61, [("C5", 0.25, 78), ("Bb4", 0.25, 75), ("A4", 0.5, 72)]),
        (64, [("G4", 0.2, 82), ("D5", 0.2, 85), ("G5", 0.5, 88)]),
        (67, [("F5", 0.25, 82), ("Eb5", 0.25, 80), ("D5", 0.5, 78)]),
        # Circling
        (70, [("G4", 0.2, 85), ("G4", 0.2, 85), ("G4", 0.2, 85), ("D5", 0.5, 88)]),
        (73, [("G5", 0.5, 90), ("D5", 0.5, 85)]),
        (76, [("Bb4", 0.2, 88), ("D5", 0.2, 90), ("G5", 0.5, 92)]),
        (79, [("F5", 0.25, 88), ("Eb5", 0.25, 85), ("D5", 0.5, 82)]),
        (82, [("G4", 0.2, 88), ("Bb4", 0.2, 90), ("D5", 0.2, 92), ("G5", 0.5, 95)]),
        (85, [("F5", 0.25, 90), ("D5", 0.5, 88)]),
        (88, [("G4", 0.2, 90), ("D5", 0.2, 92), ("G5", 0.5, 95)]),
        (91, [("Eb5", 0.25, 90), ("D5", 0.5, 88)]),
    ]

    for base_beat, pattern in target_phrases:
        offset = 0.0
        for note_name, duration, velocity in pattern:
            notes.append((note_name, base_beat + offset, duration, velocity))
            offset += duration

    # Section C (beats 96-142): Accusation - pounding, accusatory chords
    accusation_chords = [
        # (beat, [(note, duration, velocity), ...])
        (96, [("G3", 1.5, 92), ("Bb3", 1.5, 92), ("D4", 1.5, 92)]),
        (99, [("G4", 1, 88), ("Bb4", 1, 88), ("D5", 1, 88)]),
        (102, [("F4", 1.5, 85), ("A4", 1.5, 85), ("C5", 1.5, 85)]),
        (105, [("G3", 2, 95), ("Bb3", 2, 95), ("D4", 2, 95), ("G4", 2, 95)]),
        (109, [("Eb4", 1.5, 90), ("G4", 1.5, 90), ("Bb4", 1.5, 90)]),
        (112, [("D4", 1, 88), ("F4", 1, 88), ("A4", 1, 88)]),
        (115, [("G3", 2, 98), ("Bb3", 2, 98), ("D4", 2, 98), ("G4", 2, 98)]),
        (119, [("F4", 1.5, 92), ("A4", 1.5, 92), ("C5", 1.5, 92)]),
        # Climax
        (122, [("G3", 3, 100), ("Bb3", 3, 100), ("D4", 3, 100), ("G4", 3, 100)]),
        (127, [("F4", 1.5, 95), ("Bb4", 1.5, 95), ("D5", 1.5, 95)]),
        (130, [("Eb4", 1, 92), ("G4", 1, 92), ("Bb4", 1, 92)]),
        (133, [("D4", 2, 95), ("G4", 2, 95), ("Bb4", 2, 95)]),
        (137, [("G3", 6, 100), ("Bb3", 6, 100), ("D4", 6, 100), ("G4", 6, 100)]),  # Final
    ]

    for base_beat, chord in accusation_chords:
        for note_name, duration, velocity in chord:
            notes.append((note_name, base_beat, duration, velocity))

    # Convert to events
    events = []
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
    """Create the complete MIDI file for 'The Scapegoat'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_snare_track())
    mid.tracks.append(create_piano_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/fascist/02_scapegoat.mid"

    print("Creating 'The Scapegoat' - Fascist Suite 02")
    print("=" * 50)
    print("Concept: Anger seeking a target - without class analysis,")
    print("the atomized worker blames 'the other'")
    print()

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

    print()
    print("Musical arc: Searching -> Finding a Target -> Accusation")
    print("Key: G minor (aggressive, hunting)")
    print("Tempo: 95 BPM (agitated, searching)")
    print()
    print("INTEGRATION GUIDANCE:")
    print("- Trigger when: Player faction chooses scapegoating rhetoric")
    print("- Trigger when: Immigration/ethnic tension events occur without solidarity")
    print("- Trigger when: Economic crisis + low class consciousness")
    print("- Loop point: Beat 48 (after search phase establishes)")
    print()
    print("This music should feel like a hunt - the anger from crisis")
    print("needs SOMEWHERE to go, and without solidarity, it finds a scapegoat.")


if __name__ == "__main__":
    main()
