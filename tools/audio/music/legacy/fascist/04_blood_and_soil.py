#!/usr/bin/env python3
"""
BABYLON - Fascist Suite
04_blood_and_soil.mid - "Blood and Soil"

CONCEPTUAL BRIEF:
This piece represents national identity triumphant - the mystical connection to land
and race replacing class analysis. It sounds TRIUMPHANT but is fundamentally hollow.
The key insight: NO CHORUS (CC93=0) throughout - there is NO solidarity in fascism,
only enforced conformity. The piece stays in D minor - never resolving to major -
because this "triumph" leads nowhere but darkness.

TECHNICAL SPECIFICATION:
- Key: D minor (stays minor - NO resolution to major)
- Tempo: 120 BPM (triumphant but dark)
- Time Signature: 4/4
- Duration: ~90 seconds (180 beats at 120 BPM)
- Loop Points: Beat 32 (after Invocation) for ritual looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Brass Section (Program 61) - "False Glory" - Fanfares but hollow
- Channel 1: Timpani (Program 47) - "The Earth" - Deep, earthbound, tribal
- Channel 2: String Ensemble (Program 48) - "Blood Ties" - UNISON (conformity)
- Channel 3: Church Organ (Program 19) - "Sacred Soil" - False religious sanctity
- Channel 4: Choir Aahs (Program 52) - "The Volk" - Mass voice, monotone
- Channel 15: Expression/Automation - CC values for emotional shaping

MUSICAL ARC (90 seconds = 180 beats at 120 BPM):
A. Invocation (beats 0-31): The call to blood - timpani summons, brass heralds
B. Ritual (beats 32-95): The sacred ceremony - strings in unison, choir drones
C. False Transcendence (beats 96-179): Full power but stays D minor - hollow victory

CRITICAL EXPRESSION REQUIREMENTS:
- CC11 (Expression): 70 -> 120 (building false glory)
- CC93 (Chorus): ZERO throughout (NO SOLIDARITY - this is fascism)
- CC94 (Detune): 40 -> 20 (conformity increasing)
- CC1 (Modulation): 30 constant (suppressed anxiety beneath the surface)
- CC71 (Resonance): 70 -> 100 (harsh, aggressive timbre)

COMPOSITIONAL NOTES:
- D minor maintains darkness despite triumphant energy
- Unlike revolutionary harmony, this uses parallel motion (conformity)
- The organ provides false religious sanctity to blood mythology
- Timpani represents "soil" - deep, earthbound, primitive
- Strings play in UNISON - no individual voice, only the Volk
- Choir is MONOTONE - mass voice with no variation
- The piece never resolves to major - the "triumph" is an illusion
- CC93 (Chorus) at ZERO = NO SOLIDARITY - the defining absence
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
BPM = 120
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 180  # ~90 seconds at 120 BPM

# Note definitions (MIDI note numbers) - D minor scale
NOTES = {
    # Low register (timpani, bass)
    "D1": 26,
    "A1": 33,
    "D2": 38,
    "E2": 40,
    "F2": 41,
    "G2": 43,
    "A2": 45,
    "Bb2": 46,
    "C3": 48,
    # Mid register
    "D3": 50,
    "E3": 52,
    "F3": 53,
    "G3": 55,
    "A3": 57,
    "Bb3": 58,
    "C4": 60,
    # Upper register
    "D4": 62,
    "E4": 64,
    "F4": 65,
    "G4": 67,
    "A4": 69,
    "Bb4": 70,
    "C5": 72,
    # High register
    "D5": 74,
    "E5": 76,
    "F5": 77,
    "G5": 79,
    "A5": 81,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="Blood and Soil - False Triumph", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="Dm", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_expression_track() -> MidiTrack:
    """
    Track 6: Expression Automation (Channel 15)

    CRITICAL CC VALUES:
    - CC11 (Expression): 70 -> 120 (building false glory)
    - CC93 (Chorus): ZERO throughout (NO SOLIDARITY)
    - CC94 (Detune): 40 -> 20 (conformity increasing)
    - CC1 (Modulation): 30 constant (suppressed anxiety)
    - CC71 (Resonance): 70 -> 100 (harsh, aggressive timbre)
    """
    track = MidiTrack()
    track.name = "Expression Automation"

    events = []

    # Initialize all CC values at beat 0 for all channels (0-4)
    for channel in range(5):
        events.append((0, channel, 11, 70))  # Expression starts at 70
        events.append((0, channel, 93, 0))  # Chorus ZERO - NO SOLIDARITY
        events.append((0, channel, 94, 40))  # Detune starts at 40
        events.append((0, channel, 1, 30))  # Modulation constant at 30
        events.append((0, channel, 71, 70))  # Resonance starts at 70

    # Automation curves over the piece (180 beats)
    # Update every 4 beats for smooth automation
    automation_points = 45  # 180 / 4 = 45 points

    for i in range(automation_points):
        beat = i * 4
        progress = i / (automation_points - 1)  # 0.0 to 1.0

        # CC11 (Expression): 70 -> 120
        expression = int(70 + (50 * progress))

        # CC93 (Chorus): ZERO throughout - THE KEY ABSENCE
        chorus = 0

        # CC94 (Detune): 40 -> 20 (conformity increasing = less variation)
        detune = int(40 - (20 * progress))

        # CC1 (Modulation): constant 30
        modulation = 30

        # CC71 (Resonance): 70 -> 100
        resonance = int(70 + (30 * progress))

        for channel in range(5):
            events.append((beats_to_ticks(beat), channel, 11, expression))
            events.append((beats_to_ticks(beat), channel, 93, chorus))
            events.append((beats_to_ticks(beat), channel, 94, detune))
            events.append((beats_to_ticks(beat), channel, 1, modulation))
            events.append((beats_to_ticks(beat), channel, 71, resonance))

    # Sort by time
    events.sort(key=lambda x: x[0])

    last_time = 0
    for event_time, channel, cc, value in events:
        delta = event_time - last_time
        track.append(
            Message("control_change", control=cc, value=value, channel=channel, time=delta)
        )
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 1: Brass Section - False Glory (Program 61, Channel 0)
    Fanfares that herald the blood myth - triumphant but hollow.
    Octave doubling creates imposing but empty sound.
    """
    track = MidiTrack()
    track.name = "Brass - False Glory"
    track.append(Message("program_change", program=61, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes = [
        # Section A (beats 0-31): Invocation - heralding fanfares
        ("D3", 0, 2, 75),
        ("D4", 0, 2, 75),  # Octave doubling - imposing but hollow
        ("A3", 4, 1.5, 70),
        ("A4", 4, 1.5, 70),
        ("D3", 8, 3, 78),
        ("D4", 8, 3, 78),
        ("F3", 12, 2, 72),
        ("F4", 12, 2, 72),
        ("E3", 16, 2, 74),
        ("E4", 16, 2, 74),
        ("D3", 20, 4, 80),
        ("D4", 20, 4, 80),
        ("A3", 24, 2, 76),
        ("A4", 24, 2, 76),
        ("D3", 28, 4, 82),
        ("D4", 28, 4, 82),
        # Section B (beats 32-95): Ritual - punctuating the ceremony
        ("D3", 32, 1, 85),
        ("D4", 32, 1, 85),
        ("D3", 36, 1, 85),
        ("D4", 36, 1, 85),
        ("A3", 40, 2, 82),
        ("A4", 40, 2, 82),
        ("D3", 48, 1, 88),
        ("D4", 48, 1, 88),
        ("D3", 52, 1, 88),
        ("D4", 52, 1, 88),
        ("F3", 56, 2, 85),
        ("F4", 56, 2, 85),
        ("D3", 64, 1, 90),
        ("D4", 64, 1, 90),
        ("E3", 68, 1, 88),
        ("E4", 68, 1, 88),
        ("F3", 72, 2, 90),
        ("F4", 72, 2, 90),
        ("D3", 80, 1, 92),
        ("D4", 80, 1, 92),
        ("A3", 84, 2, 90),
        ("A4", 84, 2, 90),
        ("D3", 88, 4, 92),
        ("D4", 88, 4, 92),
        # Section C (beats 96-179): False Transcendence - full power, still D minor
        ("D3", 96, 2, 95),
        ("D4", 96, 2, 95),
        ("A3", 100, 2, 92),
        ("A4", 100, 2, 92),
        ("D3", 104, 2, 95),
        ("D4", 104, 2, 95),
        ("F3", 108, 2, 92),
        ("F4", 108, 2, 92),
        ("D3", 112, 2, 98),
        ("D4", 112, 2, 98),
        ("E3", 116, 2, 95),
        ("E4", 116, 2, 95),
        ("D3", 120, 4, 98),
        ("D4", 120, 4, 98),
        ("D3", 128, 2, 100),
        ("D4", 128, 2, 100),
        ("A3", 132, 2, 98),
        ("A4", 132, 2, 98),
        ("D3", 136, 2, 100),
        ("D4", 136, 2, 100),
        ("Bb3", 140, 2, 98),  # Bb - staying minor!
        ("Bb4", 140, 2, 98),
        ("D3", 144, 4, 102),
        ("D4", 144, 4, 102),
        ("E3", 150, 2, 100),
        ("E4", 150, 2, 100),
        ("D3", 154, 6, 102),
        ("D4", 154, 6, 102),
        # Final climax (beats 160-179)
        ("D3", 160, 4, 108),
        ("D4", 160, 4, 108),
        ("D5", 160, 4, 105),  # Triple octave
        ("A3", 166, 2, 105),
        ("A4", 166, 2, 105),
        ("D3", 170, 2, 108),
        ("D4", 170, 2, 108),
        ("D5", 170, 2, 105),
        ("F3", 174, 2, 105),  # F natural - minor!
        ("F4", 174, 2, 105),
        ("D3", 178, 2, 115),  # Final blast - D minor chord
        ("D4", 178, 2, 115),
        ("D5", 178, 2, 112),
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


def create_timpani_track() -> MidiTrack:
    """
    Track 2: Timpani - The Earth (Program 47, Channel 1)
    Deep, earthbound, tribal. The "soil" of blood and soil.
    Primitive rhythms suggesting pre-rational connection to land.
    """
    track = MidiTrack()
    track.name = "Timpani - The Earth"
    track.append(Message("program_change", program=47, channel=1, time=0))

    events = []

    # Section A (beats 0-31): Invocation - deep summoning thuds
    for beat in range(0, 32, 2):
        note = NOTES["D2"] if beat % 4 == 0 else NOTES["A1"]
        velocity = 55 + beat
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.8), "note_off", note, 0))

    # Section B (beats 32-95): Ritual - steady tribal pulse
    for beat in range(32, 96):
        if beat % 4 == 0:
            note = NOTES["D2"]
            velocity = 80
        elif beat % 4 == 2:
            note = NOTES["A1"]
            velocity = 75
        else:
            continue

        velocity = velocity + ((beat - 32) // 8)
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.6), "note_off", note, 0))

    # Section C (beats 96-179): False Transcendence - driving, relentless
    for beat in range(96, 160):
        if beat % 2 == 0:
            note = NOTES["D2"]
            velocity = 90
        else:
            note = NOTES["A1"]
            velocity = 85

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # Final section (beats 160-179): Thunderous climax
    for beat in range(160, 180):
        note = NOTES["D2"] if beat % 2 == 0 else NOTES["A1"]
        velocity = min(95 + (beat - 160), 120)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

        # Add sub-beat for intensity
        if beat >= 170:
            sub_note = NOTES["D1"]
            events.append((beats_to_ticks(beat + 0.5), "note_on", sub_note, velocity - 10))
            events.append((beats_to_ticks(beat + 0.7), "note_off", sub_note, 0))

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
    Track 3: Strings - Blood Ties (Program 48, Channel 2)
    All in UNISON - no harmony, only conformity.
    The "blood" connection - all voices merged into one racial identity.
    """
    track = MidiTrack()
    track.name = "Strings - Blood Ties (Unison)"
    track.append(Message("program_change", program=48, channel=2, time=0))

    # All strings play in UNISON - multiple octaves but same pitch class
    # This represents conformity, the erasure of individual voice
    notes = [
        # Section B (beats 32-95): Ritual melody - UNISON across octaves
        # First phrase - D minor scale, no major resolution
        ("D3", 32, 8, 70),
        ("D4", 32, 8, 70),
        ("D5", 32, 8, 60),  # Higher octave, softer
        ("E3", 40, 4, 72),
        ("E4", 40, 4, 72),
        ("E5", 40, 4, 62),
        ("F3", 44, 4, 75),  # F natural - minor scale
        ("F4", 44, 4, 75),
        ("F5", 44, 4, 65),
        ("D3", 48, 8, 78),
        ("D4", 48, 8, 78),
        ("D5", 48, 8, 68),
        # Second phrase
        ("D3", 56, 4, 75),
        ("D4", 56, 4, 75),
        ("A3", 60, 4, 78),
        ("A4", 60, 4, 78),
        ("G3", 64, 4, 72),
        ("G4", 64, 4, 72),
        ("F3", 68, 4, 75),
        ("F4", 68, 4, 75),
        ("E3", 72, 4, 78),
        ("E4", 72, 4, 78),
        ("D3", 76, 8, 80),
        ("D4", 76, 8, 80),
        # Building
        ("D3", 84, 4, 82),
        ("D4", 84, 4, 82),
        ("D5", 84, 4, 72),
        ("F3", 88, 4, 85),
        ("F4", 88, 4, 85),
        ("F5", 88, 4, 75),
        ("A3", 92, 4, 88),
        ("A4", 92, 4, 88),
        # Section C (beats 96-179): False Transcendence - full unison power
        ("D3", 96, 8, 90),
        ("D4", 96, 8, 90),
        ("D5", 96, 8, 80),
        ("D3", 104, 4, 88),
        ("D4", 104, 4, 88),
        ("E3", 108, 4, 90),
        ("E4", 108, 4, 90),
        ("F3", 112, 8, 92),
        ("F4", 112, 8, 92),
        ("F5", 112, 8, 82),
        ("E3", 120, 4, 90),
        ("E4", 120, 4, 90),
        ("D3", 124, 4, 95),
        ("D4", 124, 4, 95),
        ("D5", 124, 4, 85),
        # March continues
        ("D3", 128, 8, 95),
        ("D4", 128, 8, 95),
        ("D5", 128, 8, 85),
        ("A3", 136, 4, 92),
        ("A4", 136, 4, 92),
        ("G3", 140, 4, 90),
        ("G4", 140, 4, 90),
        ("F3", 144, 4, 92),
        ("F4", 144, 4, 92),
        ("E3", 148, 4, 95),
        ("E4", 148, 4, 95),
        ("D3", 152, 8, 98),
        ("D4", 152, 8, 98),
        ("D5", 152, 8, 88),
        # Final climax (beats 160-179)
        ("D3", 160, 8, 102),
        ("D4", 160, 8, 102),
        ("D5", 160, 8, 92),
        ("F3", 168, 4, 105),
        ("F4", 168, 4, 105),
        ("F5", 168, 4, 95),
        ("A3", 172, 4, 108),
        ("A4", 172, 4, 108),
        ("D3", 176, 4, 115),  # Final sustained - D minor!
        ("D4", 176, 4, 115),
        ("D5", 176, 4, 105),
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


def create_organ_track() -> MidiTrack:
    """
    Track 4: Church Organ - Sacred Soil (Program 19, Channel 3)
    False religious sanctity applied to blood mythology.
    Sustained drones that provide hollow spiritual weight.
    """
    track = MidiTrack()
    track.name = "Organ - Sacred Soil"
    track.append(Message("program_change", program=19, channel=3, time=0))

    # Long sustained drones - false sanctity to blood mythology
    notes = [
        # Section A (beats 0-31): Invocation - ominous drone builds
        ("D2", 0, 16, 45),
        ("A2", 0, 16, 42),
        ("D2", 16, 16, 55),
        ("A2", 16, 16, 52),
        ("D3", 24, 8, 50),
        # Section B (beats 32-95): Ritual - sustained bed
        ("D2", 32, 32, 60),
        ("A2", 32, 32, 58),
        ("D3", 32, 32, 55),
        ("D2", 64, 32, 65),
        ("A2", 64, 32, 62),
        ("D3", 64, 32, 60),
        ("F3", 80, 16, 57),  # F natural - minor
        # Section C (beats 96-179): False Transcendence - full organ power
        ("D2", 96, 32, 75),
        ("A2", 96, 32, 72),
        ("D3", 96, 32, 70),
        ("F3", 96, 32, 65),
        ("D2", 128, 32, 80),
        ("A2", 128, 32, 78),
        ("D3", 128, 32, 75),
        ("F3", 128, 32, 70),
        ("A3", 144, 16, 72),
        # Final section (beats 160-180)
        ("D2", 160, 20, 90),
        ("A2", 160, 20, 88),
        ("D3", 160, 20, 85),
        ("F3", 160, 20, 80),  # F natural - stays minor!
        ("A3", 160, 20, 78),
        ("D4", 170, 10, 82),  # High note for climax
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


def create_choir_track() -> MidiTrack:
    """
    Track 5: Choir Aahs - The Volk (Program 52, Channel 4)
    Mass voice, but MONOTONE - no individual expression.
    The "blood" - all voices merged into one racial identity.
    """
    track = MidiTrack()
    track.name = "Choir - The Volk (Monotone)"
    track.append(Message("program_change", program=52, channel=4, time=0))

    # Choir is MONOTONE - mass voice with minimal variation
    # This represents the erasure of individual identity into the Volk
    notes = [
        # Section B (beats 32-95): Ritual - monotone drones
        ("D3", 32, 16, 55),
        ("D4", 32, 16, 55),  # Unison - same note
        ("D3", 48, 16, 60),
        ("D4", 48, 16, 60),
        ("D3", 64, 16, 65),
        ("D4", 64, 16, 65),
        ("D3", 80, 16, 70),
        ("D4", 80, 16, 70),
        # Section C (beats 96-179): False Transcendence - louder but still monotone
        ("D3", 96, 16, 78),
        ("D4", 96, 16, 78),
        ("A3", 96, 16, 72),  # Fifth, not harmony - primitive
        ("D3", 112, 16, 82),
        ("D4", 112, 16, 82),
        ("A3", 112, 16, 76),
        ("D3", 128, 16, 88),
        ("D4", 128, 16, 88),
        ("A3", 128, 16, 82),
        ("D3", 144, 16, 92),
        ("D4", 144, 16, 92),
        ("A3", 144, 16, 86),
        # Final climax (beats 160-179)
        ("D3", 160, 20, 100),
        ("D4", 160, 20, 100),
        ("A3", 160, 20, 95),
        ("D5", 168, 12, 95),  # High D for climax
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
    """Create the complete MIDI file for 'Blood and Soil'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_organ_track())
    mid.tracks.append(create_choir_track())
    mid.tracks.append(create_expression_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/fascist/04_blood_and_soil.mid"

    print("Creating 'Blood and Soil' - Fascist Suite 04")
    print("=" * 60)
    print("Concept: National identity triumphant - blood mythology replaces class")
    print("Key insight: Triumphant but hollow - stays D minor, NO chorus (CC93=0)")

    mid = create_midi_file()
    mid.save(output_path)

    print(f"\nSaved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Tempo: {BPM} BPM")
    print(f"Track count: {len(mid.tracks)}")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    length = mid.length
    print(f"\nDuration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\nExpression Automation (CRITICAL):")
    print("  - CC11 (Expression): 70 -> 120 (building false glory)")
    print("  - CC93 (Chorus): ZERO throughout (NO SOLIDARITY - this is fascism)")
    print("  - CC94 (Detune): 40 -> 20 (conformity increasing)")
    print("  - CC1 (Modulation): 30 constant (suppressed anxiety)")
    print("  - CC71 (Resonance): 70 -> 100 (harsh, aggressive timbre)")

    print("\nComposition complete.")
    print("Musical arc: Invocation -> Ritual -> False Transcendence (stays D minor!)")
    print("Key: D minor (NO resolution to major)")
    print("\nThematic notes:")
    print("  - Brass: Hollow octave doubling = false triumph")
    print("  - Timpani: Deep, earthbound = the 'soil' of blood and soil")
    print("  - Strings: All UNISON = conformity, no individual voice")
    print("  - Organ: False religious sanctity to blood mythology")
    print("  - Choir: MONOTONE = mass voice with no variation")
    print("  - CC93 = 0: The defining absence - NO solidarity in fascism")


if __name__ == "__main__":
    main()
