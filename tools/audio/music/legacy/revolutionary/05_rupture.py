#!/usr/bin/env python3
"""
BABYLON - Revolutionary Suite
05_rupture.mid - "Rupture"

CONCEPTUAL BRIEF:
This piece represents THE moment - when P(S|R) > P(S|A) and the system breaks.
The music captures the explosive catharsis of revolution succeeding: years of
accumulated tension releasing into liberation. This is not gradual transformation
but violent rupture - the dialectical leap from quantity to quality.

TECHNICAL SPECIFICATION:
- Key: C minor -> C major (the ultimate resolution from darkness to light)
- Tempo: 140 BPM building to 160 BPM (explosive, cathartic energy)
- Time Signature: 4/4
- Duration: ~90 seconds (210 beats at average 150 BPM)
- Loop Points: Beat 180 (post-liberation) for triumphant loop

INSTRUMENT ASSIGNMENTS:
- Channel 0: Full Orchestra Strings (Program 48) - "The Masses Rising"
- Channel 1: Brass (Program 61) - "The Break" - Explosive fanfares
- Channel 2: Timpani (Program 47) - "Rupture Thunder" - Thunderous rolls
- Channel 3: Choir (Program 52) - "Liberation Voices" - The freed masses
- Channel 4: Piano (Program 0) - "New Dawn" - Clear, bright new beginning

MUSICAL ARC (210 beats):
A. Tension Peak (beats 0-48): C minor, 140 BPM - grinding pressure at maximum
B. Breaking Point (beats 49-96): Accelerando to 160 BPM - the dam bursting
C. Liberation (beats 97-160): C major arrival - explosive release, catharsis
D. New Dawn (beats 161-210): C major, 160 BPM - triumphant, bright, victorious

COMPOSITIONAL NOTES:
- C minor represents the accumulated contradictions of the old system
- The accelerando represents the mathematical certainty of rupture
- C major represents liberation - not earned gradually but seized suddenly
- Timpani rolls at the break point represent the sound of the old order shattering
- Brass fanfares herald the new era with unambiguous triumph
- Choir voices represent the masses finally liberated from atomization
- Piano in the final section represents clarity, a new beginning

GAME INTEGRATION:
This track triggers when the simulation engine fires RUPTURE_EVENT
when P(S|R) exceeds P(S|A) for a critical mass of social classes.
It should play ONCE and transition to the victory/new-era music.
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
TEMPO_START = 140  # BPM for tension section
TEMPO_RUPTURE = 160  # BPM for liberation section
MICROSECONDS_PER_BEAT_START = int(60_000_000 / TEMPO_START)
MICROSECONDS_PER_BEAT_RUPTURE = int(60_000_000 / TEMPO_RUPTURE)
TOTAL_BEATS = 210

# Note definitions (MIDI note numbers)
# Includes both flat (C minor) and natural (C major) variants for the key change
NOTES = {
    # Bass register
    "C1": 24,
    "D1": 26,
    "Eb1": 27,
    "E1": 28,
    "F1": 29,
    "G1": 31,
    "Ab1": 32,
    "A1": 33,
    "Bb1": 34,
    "B1": 35,
    "C2": 36,
    "D2": 38,
    "Eb2": 39,
    "E2": 40,
    "F2": 41,
    "G2": 43,
    "Ab2": 44,
    "A2": 45,
    "Bb2": 46,
    "B2": 47,
    # Tenor register
    "C3": 48,
    "D3": 50,
    "Eb3": 51,
    "E3": 52,
    "F3": 53,
    "G3": 55,
    "Ab3": 56,
    "A3": 57,
    "Bb3": 58,
    "B3": 59,
    # Alto register
    "C4": 60,
    "D4": 62,
    "Eb4": 63,
    "E4": 64,
    "F4": 65,
    "G4": 67,
    "Ab4": 68,
    "A4": 69,
    "Bb4": 70,
    "B4": 71,
    # Soprano register
    "C5": 72,
    "D5": 74,
    "Eb5": 75,
    "E5": 76,
    "F5": 77,
    "G5": 79,
    "Ab5": 80,
    "A5": 81,
    "Bb5": 82,
    "B5": 83,
    # High register
    "C6": 84,
    "D6": 86,
    "Eb6": 87,
    "E6": 88,
    "F6": 89,
    "G6": 91,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo changes, time signature, and key."""
    track = MidiTrack()

    # Initial setup: C minor, 140 BPM
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT_START, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="Cm", time=0))  # C minor

    # Beat 49: Start accelerando (we'll use discrete tempo changes)
    # Gradual acceleration from 140 to 160 BPM over beats 49-96
    accel_steps = [
        (49, 142),
        (57, 145),
        (65, 148),
        (73, 152),
        (81, 155),
        (89, 158),
    ]

    current_time = 0
    for beat, bpm in accel_steps:
        delta_ticks = beats_to_ticks(beat) - current_time
        tempo = int(60_000_000 / bpm)
        track.append(MetaMessage("set_tempo", tempo=tempo, time=delta_ticks))
        current_time = beats_to_ticks(beat)

    # Beat 97: Liberation - full 160 BPM, key change to C MAJOR
    delta_ticks = beats_to_ticks(97) - current_time
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT_RUPTURE, time=delta_ticks))
    current_time = beats_to_ticks(97)

    # End of track
    delta_ticks = beats_to_ticks(TOTAL_BEATS) - current_time
    track.append(MetaMessage("end_of_track", time=delta_ticks))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 1: Full Orchestra Strings - The Masses Rising (Program 48, Channel 0)
    Massive, sweeping - represents the collective force of revolution.
    """
    track = MidiTrack()
    track.name = "Strings - The Masses Rising"
    track.append(Message("program_change", program=48, channel=0, time=0))

    notes = [
        # Section A (beats 0-48): Tension Peak - C minor grinding
        # Low tremolo strings building tension
        ("C2", 0, 8, 70),
        ("Eb2", 0, 8, 70),
        ("G2", 0, 8, 70),
        ("C2", 8, 8, 75),
        ("Eb2", 8, 8, 75),
        ("G2", 8, 8, 75),
        ("Ab2", 16, 8, 78),
        ("C3", 16, 8, 78),
        ("Eb3", 16, 8, 78),
        ("Bb2", 24, 8, 80),
        ("D3", 24, 8, 80),
        ("F3", 24, 8, 80),
        ("G2", 32, 8, 85),
        ("Bb2", 32, 8, 85),
        ("D3", 32, 8, 85),
        ("Ab2", 40, 8, 90),
        ("C3", 40, 8, 90),
        ("Eb3", 40, 8, 90),
        # Section B (beats 49-96): Breaking Point - intensifying
        ("G2", 49, 6, 92),
        ("Bb2", 49, 6, 92),
        ("D3", 49, 6, 92),
        ("Ab2", 55, 6, 95),
        ("C3", 55, 6, 95),
        ("Eb3", 55, 6, 95),
        ("F2", 61, 6, 98),
        ("Ab2", 61, 6, 98),
        ("C3", 61, 6, 98),
        ("G2", 67, 6, 100),
        ("Bb2", 67, 6, 100),
        ("D3", 67, 6, 100),
        ("Ab2", 73, 6, 105),
        ("C3", 73, 6, 105),
        ("Eb3", 73, 6, 105),
        ("Bb2", 79, 6, 108),
        ("D3", 79, 6, 108),
        ("F3", 79, 6, 108),
        ("C3", 85, 4, 110),
        ("Eb3", 85, 4, 110),
        ("G3", 85, 4, 110),
        ("D3", 89, 4, 112),
        ("F3", 89, 4, 112),
        ("Ab3", 89, 4, 112),
        ("Eb3", 93, 4, 115),
        ("G3", 93, 4, 115),
        ("Bb3", 93, 4, 115),
        # Section C (beats 97-160): LIBERATION - C MAJOR explosion
        # The dam bursts - massive C major chord
        ("C3", 97, 16, 127),
        ("E3", 97, 16, 127),
        ("G3", 97, 16, 127),
        ("C4", 97, 16, 127),
        ("E4", 97, 16, 127),
        ("G4", 97, 16, 127),
        # Triumphant progression
        ("G3", 113, 8, 120),
        ("B3", 113, 8, 120),
        ("D4", 113, 8, 120),
        ("F3", 121, 8, 115),
        ("A3", 121, 8, 115),
        ("C4", 121, 8, 115),
        ("E3", 129, 8, 118),
        ("G3", 129, 8, 118),
        ("B3", 129, 8, 118),
        ("C3", 137, 12, 125),
        ("E3", 137, 12, 125),
        ("G3", 137, 12, 125),
        ("C4", 137, 12, 125),
        ("E4", 137, 12, 125),
        ("D3", 149, 6, 118),
        ("F3", 149, 6, 118),
        ("A3", 149, 6, 118),
        ("G3", 155, 6, 120),
        ("B3", 155, 6, 120),
        ("D4", 155, 6, 120),
        # Section D (beats 161-210): New Dawn - bright, victorious
        ("C3", 161, 12, 115),
        ("E3", 161, 12, 115),
        ("G3", 161, 12, 115),
        ("C4", 161, 12, 115),
        ("E4", 161, 12, 115),
        ("G4", 161, 12, 115),
        ("F3", 173, 8, 110),
        ("A3", 173, 8, 110),
        ("C4", 173, 8, 110),
        ("G3", 181, 8, 112),
        ("B3", 181, 8, 112),
        ("D4", 181, 8, 112),
        ("E3", 189, 6, 108),
        ("G3", 189, 6, 108),
        ("B3", 189, 6, 108),
        # Final triumphant C major
        ("C3", 195, 15, 118),
        ("E3", 195, 15, 118),
        ("G3", 195, 15, 118),
        ("C4", 195, 15, 118),
        ("E4", 195, 15, 118),
        ("G4", 195, 15, 118),
        ("C5", 195, 15, 118),
    ]

    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        # Cap velocity at 127 (MIDI max)
        vel = min(velocity, 127)
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


def create_brass_track() -> MidiTrack:
    """
    Track 2: Brass - The Break (Program 61, Channel 1)
    Explosive fanfares heralding the new era.
    """
    track = MidiTrack()
    track.name = "Brass - The Break"
    track.append(Message("program_change", program=61, channel=1, time=0))

    notes = [
        # Section A (beats 0-48): Tension - ominous brass stabs
        ("C3", 4, 0.5, 85),
        ("G3", 4, 0.5, 85),
        ("Eb3", 12, 0.5, 88),
        ("Bb3", 12, 0.5, 88),
        ("C3", 20, 0.5, 90),
        ("G3", 20, 0.5, 90),
        ("Ab3", 28, 0.5, 92),
        ("Eb4", 28, 0.5, 92),
        ("G3", 36, 0.5, 95),
        ("D4", 36, 0.5, 95),
        ("Ab3", 44, 1, 98),
        ("Eb4", 44, 1, 98),
        # Section B (beats 49-96): Breaking Point - intensifying calls
        ("C4", 49, 1, 100),
        ("G4", 49, 1, 100),
        ("Eb4", 55, 1, 102),
        ("Bb4", 55, 1, 102),
        ("F4", 61, 1, 105),
        ("C5", 61, 1, 105),
        ("G4", 67, 1, 108),
        ("D5", 67, 1, 108),
        ("Ab4", 73, 1, 110),
        ("Eb5", 73, 1, 110),
        ("Bb4", 79, 1, 112),
        ("F5", 79, 1, 112),
        # Pre-rupture fanfare
        ("C4", 85, 0.5, 115),
        ("Eb4", 86, 0.5, 117),
        ("G4", 87, 0.5, 120),
        ("C5", 88, 0.5, 122),
        ("Eb5", 89, 0.5, 125),
        # THE RUPTURE CALL
        ("G4", 93, 4, 127),
        ("C5", 93, 4, 127),
        ("Eb5", 93, 4, 127),
        ("G5", 93, 4, 127),
        # Section C (beats 97-160): LIBERATION - Triumphant fanfares in C MAJOR
        # THE BREAK - explosive C major fanfare
        ("C4", 97, 4, 127),
        ("E4", 97, 4, 127),
        ("G4", 97, 4, 127),
        ("C5", 97, 4, 127),
        ("G4", 101, 2, 125),
        ("C5", 101, 2, 125),
        ("E5", 101, 2, 125),
        ("C4", 105, 4, 127),
        ("E4", 105, 4, 127),
        ("G4", 105, 4, 127),
        ("C5", 105, 4, 127),
        ("E5", 105, 4, 127),
        # Victory calls
        ("G4", 113, 3, 122),
        ("B4", 113, 3, 122),
        ("D5", 113, 3, 122),
        ("G4", 117, 3, 120),
        ("C5", 117, 3, 120),
        ("E5", 117, 3, 120),
        ("F4", 121, 4, 118),
        ("A4", 121, 4, 118),
        ("C5", 121, 4, 118),
        ("G4", 129, 4, 120),
        ("B4", 129, 4, 120),
        ("D5", 129, 4, 120),
        ("C4", 137, 6, 125),
        ("E4", 137, 6, 125),
        ("G4", 137, 6, 125),
        ("C5", 137, 6, 125),
        ("D4", 145, 4, 118),
        ("F4", 145, 4, 118),
        ("A4", 145, 4, 118),
        ("G4", 153, 4, 120),
        ("B4", 153, 4, 120),
        ("D5", 153, 4, 120),
        # Section D (beats 161-210): New Dawn - triumphant resolution
        ("C4", 161, 6, 115),
        ("E4", 161, 6, 115),
        ("G4", 161, 6, 115),
        ("C5", 161, 6, 115),
        ("G4", 169, 4, 112),
        ("B4", 169, 4, 112),
        ("D5", 169, 4, 112),
        ("F4", 177, 4, 110),
        ("A4", 177, 4, 110),
        ("C5", 177, 4, 110),
        ("G4", 185, 4, 115),
        ("B4", 185, 4, 115),
        ("D5", 185, 4, 115),
        # Final triumphant C major fanfare
        ("C4", 193, 8, 118),
        ("E4", 193, 8, 118),
        ("G4", 193, 8, 118),
        ("C5", 193, 8, 118),
        ("E5", 193, 8, 118),
        ("C5", 201, 9, 115),
        ("E5", 201, 9, 115),
        ("G5", 201, 9, 115),
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


def create_timpani_track() -> MidiTrack:
    """
    Track 3: Timpani - Rupture Thunder (Program 47, Channel 2)
    Thunderous rolls representing the shattering of the old order.
    """
    track = MidiTrack()
    track.name = "Timpani - Rupture Thunder"
    track.append(Message("program_change", program=47, channel=2, time=0))

    events = []

    # Section A (beats 0-48): Tension - steady ominous pulse
    for beat in range(0, 48, 2):
        note = NOTES["C2"] if beat % 8 < 4 else NOTES["G1"]
        velocity = min(70 + beat, 100)
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # Section B (beats 49-96): Breaking Point - accelerating, building to rupture
    # Faster hits as tension mounts
    beat = 49.0
    while beat < 85:
        note = NOTES["C2"] if int(beat - 49) % 8 < 4 else NOTES["G1"]
        velocity = int(min(85 + (beat - 49), 115))
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))
        beat += 1.5

    # THE RUPTURE ROLL (beats 85-96) - continuous thunder
    for beat_idx in range(85, 97):
        beat = float(beat_idx)
        for sub in [0, 0.25, 0.5, 0.75]:
            note = NOTES["C2"]
            velocity = min(100 + (beat_idx - 85) * 2, 127)
            events.append((beats_to_ticks(beat + sub), "note_on", note, velocity))
            events.append((beats_to_ticks(beat + sub + 0.2), "note_off", note, 0))

    # Section C (beats 97-160): Liberation - powerful, triumphant strikes
    # Explosive hits on strong beats
    liberation_hits = [
        (97, "C2", 127),
        (97.5, "G2", 127),
        (98, "C2", 127),
        (98.5, "G2", 127),
        (101, "C2", 120),
        (103, "G2", 118),
        (105, "C2", 125),
        (107, "G2", 122),
        (109, "C2", 118),
        (113, "G2", 115),
        (117, "C2", 118),
        (121, "G2", 115),
        (125, "C2", 120),
        (129, "G2", 118),
        (133, "C2", 122),
        (137, "G2", 120),
        (141, "C2", 118),
        (145, "G2", 115),
        (149, "C2", 118),
        (153, "G2", 115),
        (157, "C2", 120),
        (159, "G2", 118),
    ]

    for beat, note_name, velocity in liberation_hits:
        note = NOTES[note_name]
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # Section D (beats 161-210): New Dawn - steady, triumphant pulse
    for beat in range(161, 210, 4):
        note = NOTES["C2"]
        velocity = 110 if beat % 8 == 1 else 100
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))
        # Secondary hit
        events.append((beats_to_ticks(beat + 2), "note_on", NOTES["G2"], velocity - 10))
        events.append((beats_to_ticks(beat + 2.5), "note_off", NOTES["G2"], 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=2, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_choir_track() -> MidiTrack:
    """
    Track 4: Choir - Liberation Voices (Program 52, Channel 3)
    The voices of the freed masses - enters at liberation moment.
    """
    track = MidiTrack()
    track.name = "Choir - Liberation Voices"
    track.append(Message("program_change", program=52, channel=3, time=0))

    notes = [
        # Section C (beats 97-160): LIBERATION - Choir enters with freedom
        # The moment of rupture - wordless "Ahhh" of release
        ("C4", 97, 8, 115),
        ("E4", 97, 8, 115),
        ("G4", 97, 8, 115),
        ("C5", 97, 8, 115),
        ("G4", 105, 4, 112),
        ("B4", 105, 4, 112),
        ("D5", 105, 4, 112),
        ("E4", 109, 4, 110),
        ("G4", 109, 4, 110),
        ("C5", 109, 4, 110),
        ("C4", 113, 8, 118),
        ("E4", 113, 8, 118),
        ("G4", 113, 8, 118),
        ("C5", 113, 8, 118),
        ("F4", 121, 4, 110),
        ("A4", 121, 4, 110),
        ("C5", 121, 4, 110),
        ("G4", 125, 4, 112),
        ("B4", 125, 4, 112),
        ("D5", 125, 4, 112),
        ("C4", 129, 8, 120),
        ("E4", 129, 8, 120),
        ("G4", 129, 8, 120),
        ("C5", 129, 8, 120),
        # Building toward new dawn
        ("E4", 137, 6, 115),
        ("G4", 137, 6, 115),
        ("C5", 137, 6, 115),
        ("D4", 143, 6, 112),
        ("F4", 143, 6, 112),
        ("A4", 143, 6, 112),
        ("C4", 149, 6, 118),
        ("E4", 149, 6, 118),
        ("G4", 149, 6, 118),
        ("G4", 155, 6, 115),
        ("B4", 155, 6, 115),
        ("D5", 155, 6, 115),
        # Section D (beats 161-210): New Dawn - triumphant hymn
        ("C4", 161, 10, 118),
        ("E4", 161, 10, 118),
        ("G4", 161, 10, 118),
        ("C5", 161, 10, 118),
        ("G4", 171, 6, 112),
        ("B4", 171, 6, 112),
        ("D5", 171, 6, 112),
        ("F4", 177, 6, 110),
        ("A4", 177, 6, 110),
        ("C5", 177, 6, 110),
        ("E4", 183, 6, 108),
        ("G4", 183, 6, 108),
        ("B4", 183, 6, 108),
        ("D4", 189, 4, 105),
        ("F4", 189, 4, 105),
        ("A4", 189, 4, 105),
        # Final triumphant chord
        ("C4", 193, 17, 115),
        ("E4", 193, 17, 115),
        ("G4", 193, 17, 115),
        ("C5", 193, 17, 115),
        ("E5", 193, 17, 115),
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
        track.append(Message(msg_type, note=note, velocity=velocity, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_piano_track() -> MidiTrack:
    """
    Track 5: Piano - New Dawn (Program 0, Channel 4)
    Clear, bright - represents the clarity of the new beginning.
    """
    track = MidiTrack()
    track.name = "Piano - New Dawn"
    track.append(Message("program_change", program=0, channel=4, time=0))

    notes = [
        # Section A (beats 0-48): Tension - sparse, anxious figures
        ("C4", 2, 1, 55),
        ("Eb4", 6, 1, 52),
        ("G4", 10, 1.5, 58),
        ("Ab4", 16, 1, 55),
        ("G4", 22, 1, 60),
        ("Eb4", 28, 1.5, 62),
        ("D4", 34, 1, 58),
        ("C4", 40, 1.5, 65),
        ("Eb4", 44, 2, 68),
        # Section B (beats 49-96): Breaking Point - more agitated
        ("C4", 49, 0.75, 70),
        ("Eb4", 51, 0.75, 72),
        ("G4", 53, 0.75, 75),
        ("Ab4", 55, 0.75, 78),
        ("Bb4", 57, 1, 80),
        ("C5", 59, 0.75, 82),
        ("Eb5", 61, 0.75, 85),
        ("G5", 63, 0.75, 88),
        ("Ab5", 65, 0.75, 90),
        ("Bb5", 67, 1, 92),
        ("C5", 69, 0.75, 95),
        ("Eb5", 71, 0.75, 98),
        ("G4", 73, 0.5, 100),
        ("Ab4", 74, 0.5, 102),
        ("Bb4", 75, 0.5, 105),
        ("C5", 76, 0.5, 108),
        ("D5", 77, 0.5, 110),
        ("Eb5", 78, 0.5, 112),
        ("F5", 79, 0.5, 115),
        ("G5", 80, 0.5, 118),
        # Ascending run to rupture
        ("Ab5", 81, 0.5, 120),
        ("Bb5", 82, 0.5, 122),
        ("C5", 83, 0.25, 125),
        ("D5", 83.5, 0.25, 125),
        ("Eb5", 84, 0.25, 125),
        ("F5", 84.5, 0.25, 125),
        ("G5", 85, 0.25, 127),
        ("Ab5", 85.5, 0.25, 127),
        ("Bb5", 86, 0.25, 127),
        ("C6", 86.5, 0.25, 127),
        # Tremolo at breaking point
        ("C5", 88, 0.25, 127),
        ("C6", 88.5, 0.25, 127),
        ("C5", 89, 0.25, 127),
        ("C6", 89.5, 0.25, 127),
        ("C5", 90, 0.25, 127),
        ("C6", 90.5, 0.25, 127),
        ("C5", 91, 0.25, 127),
        ("C6", 91.5, 0.25, 127),
        ("C5", 92, 0.25, 127),
        ("C6", 92.5, 0.25, 127),
        ("C5", 93, 0.25, 127),
        ("C6", 93.5, 0.25, 127),
        ("C5", 94, 0.25, 127),
        ("C6", 94.5, 0.25, 127),
        ("C5", 95, 0.5, 127),
        ("G5", 95, 0.5, 127),
        ("C6", 95, 0.5, 127),
        # Section C (beats 97-160): LIBERATION - C MAJOR brightness
        # THE BREAK - triumphant C major chord
        ("C4", 97, 4, 120),
        ("E4", 97, 4, 120),
        ("G4", 97, 4, 120),
        ("C5", 97, 4, 120),
        ("E5", 97, 4, 120),
        ("G5", 97, 4, 120),
        # Joyful arpeggios
        ("C5", 101, 0.5, 110),
        ("E5", 101.5, 0.5, 110),
        ("G5", 102, 0.5, 110),
        ("C6", 102.5, 0.5, 115),
        ("G5", 103, 0.5, 110),
        ("E5", 103.5, 0.5, 108),
        ("C5", 104, 1, 105),
        ("G4", 105, 0.5, 110),
        ("B4", 105.5, 0.5, 110),
        ("D5", 106, 0.5, 112),
        ("G5", 106.5, 0.5, 115),
        ("D5", 107, 0.5, 110),
        ("B4", 107.5, 0.5, 108),
        ("G4", 108, 1, 105),
        # Celebratory figures
        ("C4", 109, 2, 115),
        ("E4", 109, 2, 115),
        ("G4", 109, 2, 115),
        ("C5", 111, 2, 118),
        ("E5", 111, 2, 118),
        ("G5", 111, 2, 118),
        ("G4", 113, 2, 112),
        ("B4", 113, 2, 112),
        ("D5", 113, 2, 112),
        ("F4", 117, 2, 108),
        ("A4", 117, 2, 108),
        ("C5", 117, 2, 108),
        ("C4", 121, 4, 115),
        ("E4", 121, 4, 115),
        ("G4", 121, 4, 115),
        ("C5", 121, 4, 115),
        ("E4", 129, 4, 110),
        ("G4", 129, 4, 110),
        ("B4", 129, 4, 110),
        ("C4", 137, 6, 118),
        ("E4", 137, 6, 118),
        ("G4", 137, 6, 118),
        ("C5", 137, 6, 118),
        ("D4", 145, 4, 108),
        ("F4", 145, 4, 108),
        ("A4", 145, 4, 108),
        ("G4", 153, 4, 112),
        ("B4", 153, 4, 112),
        ("D5", 153, 4, 112),
        # Section D (beats 161-210): New Dawn - serene, hopeful
        ("C4", 161, 4, 100),
        ("E4", 161, 4, 100),
        ("G4", 161, 4, 100),
        ("C5", 161, 4, 100),
        ("G4", 165, 2, 95),
        ("C5", 167, 2, 98),
        ("E4", 169, 2, 92),
        ("G4", 171, 2, 95),
        ("F4", 173, 4, 90),
        ("A4", 173, 4, 90),
        ("C5", 173, 4, 90),
        ("G4", 177, 2, 92),
        ("B4", 179, 2, 95),
        ("C4", 181, 4, 98),
        ("E4", 181, 4, 98),
        ("G4", 181, 4, 98),
        ("E4", 185, 2, 92),
        ("G4", 187, 2, 95),
        ("D4", 189, 4, 88),
        ("F4", 189, 4, 88),
        ("A4", 189, 4, 88),
        # Final serene C major chord - the new dawn
        ("C3", 195, 15, 85),
        ("G3", 195, 15, 85),
        ("C4", 195, 15, 90),
        ("E4", 195, 15, 90),
        ("G4", 195, 15, 90),
        ("C5", 195, 15, 95),
        ("E5", 195, 15, 95),
        ("G5", 195, 15, 95),
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
        track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'Rupture'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_choir_track())
    mid.tracks.append(create_piano_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/revolutionary/05_rupture.mid"

    print("Creating 'Rupture' - Revolutionary Suite 05")
    print("=" * 50)
    print("THE MOMENT: P(S|R) > P(S|A) - The system breaks.")
    print()

    mid = create_midi_file()
    mid.save(output_path)

    print(f"Saved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Track count: {len(mid.tracks)}")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    length = mid.length
    print(f"Duration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print()
    print("Musical arc:")
    print("  A. Tension Peak (0-48): C minor, 140 BPM - grinding pressure")
    print("  B. Breaking Point (49-96): Accelerando to 160 BPM - the dam bursts")
    print("  C. Liberation (97-160): C MAJOR - explosive catharsis")
    print("  D. New Dawn (161-210): Triumphant, bright, victorious")
    print()
    print("Key progression: C minor -> C major (darkness to light)")
    print("Tempo progression: 140 BPM -> 160 BPM (tension to release)")
    print()
    print("RUPTURE COMPLETE.")


if __name__ == "__main__":
    main()
