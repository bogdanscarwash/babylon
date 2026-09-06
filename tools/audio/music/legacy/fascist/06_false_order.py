#!/usr/bin/env python3
"""
BABYLON - Fascist Suite
06_false_order.mid - "False Order"

CONCEPTUAL BRIEF:
This piece represents stability through oppression - the grinding mechanism of
authoritarian control that maintains "order" not through solidarity but through
force. The machine never stops, never resolves, never releases. It is the sound
of a system that has crushed all resistance and now operates in perpetual
mechanical routine. There is no hope, no change, only the eternal grind.

TECHNICAL SPECIFICATION:
- Key: E minor (NO RESOLUTION - stays in E minor throughout)
- Tempo: 100 BPM (grinding, relentless routine)
- Time Signature: 4/4
- Duration: ~100 seconds (166 beats at 100 BPM, designed for seamless looping)
- Loop Points: The ending loops back to the beginning - no resolution

INSTRUMENT ASSIGNMENTS (5 tracks):
- Channel 0: Harpsichord (Program 6) - "The Machine" - Cold, mechanical patterns
- Channel 1: String Ensemble (Program 48) - "Suppression" - Heavy, pressing down
- Channel 2: Timpani (Program 47) - "The Clock" - Relentless ticking
- Channel 3: Trombone (Program 57) - "Order" - Authority, weight
- Channel 4: Church Organ (Program 19) - "False Peace" - Hollow grandeur

MUSICAL ARC (3 cycles, no resolution):
A. Order Established (beats 0-55): Machine begins, cold and efficient
B. Cracks Appear (beats 56-110): Slight dissonances, suppressed immediately
C. Order Reasserts (beats 111-166): Return to mechanical routine (loops back)

EXPRESSION AUTOMATION (CRITICAL):
- CC11 (Expression): 60 -> 80 -> 60 (cycles, never resolves)
- CC93 (Chorus): ZERO throughout (NO SOLIDARITY)
- CC94 (Detune): 30 constant (controlled fragmentation)
- CC1 (Modulation): 40 -> 60 -> 40 (suppressed anxiety cycles)
- CC71 (Resonance): 70 constant (harsh but stable)

COMPOSITIONAL PHILOSOPHY:
This is NOT a piece about fascism triumphant - it is about fascism as
maintenance, as grinding routine, as the machinery of oppression operating
at steady state. The horror is not in climax but in continuation.
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
BPM = 100
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)  # 600,000 microseconds per beat
TOTAL_BEATS = 166  # ~100 seconds at 100 BPM, designed for seamless loop

# Note definitions (MIDI note numbers) - E minor scale
NOTES = {
    # Bass register
    "E1": 28,
    "F#1": 30,
    "G1": 31,
    "A1": 33,
    "B1": 35,
    # Low register
    "C2": 36,
    "D2": 38,
    "E2": 40,
    "F#2": 42,
    "G2": 43,
    "A2": 45,
    "B2": 47,
    # Mid-low register
    "C3": 48,
    "D3": 50,
    "E3": 52,
    "F3": 53,  # For dissonance
    "F#3": 54,
    "G3": 55,
    "A3": 57,
    "B3": 59,
    # Mid register
    "C4": 60,
    "D4": 62,
    "E4": 64,
    "F#4": 66,
    "G4": 67,
    "A4": 69,
    "B4": 71,
    # Upper register
    "C5": 72,
    "D5": 74,
    "E5": 76,
    "F#5": 78,
    "G5": 79,
}

# CC Constants
CC_MODULATION = 1
CC_EXPRESSION = 11
CC_RESONANCE = 71
CC_CHORUS = 93
CC_DETUNE = 94


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(
        MetaMessage("track_name", name="False Order - Stability Through Oppression", time=0)
    )
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="Em", time=0))
    # Marker for loop point
    track.append(MetaMessage("marker", text="LOOP_START", time=0))
    track.append(MetaMessage("marker", text="LOOP_END", time=beats_to_ticks(TOTAL_BEATS)))
    track.append(MetaMessage("end_of_track", time=0))
    return track


def add_expression_automation(events: list, channel: int) -> None:
    """
    Add expression CC automation for cycling intensity.
    CC11 (Expression): 60 -> 80 -> 60 (cycles, never resolves)
    Pattern: Each 55-beat section follows this curve
    """
    # Section 1: beats 0-55 (Order Established)
    for beat in range(0, 56, 4):
        # Curve: 60 -> 80 over 55 beats, then back
        progress = beat / 55.0
        if progress < 0.5:
            value = int(60 + (20 * (progress * 2)))  # 60 -> 80
        else:
            value = int(80 - (20 * ((progress - 0.5) * 2)))  # 80 -> 60
        events.append((beats_to_ticks(beat), "cc", CC_EXPRESSION, value, channel))

    # Section 2: beats 56-110 (Cracks Appear) - slightly higher tension
    for beat in range(56, 111, 4):
        progress = (beat - 56) / 54.0
        if progress < 0.5:
            value = int(60 + (25 * (progress * 2)))  # 60 -> 85 (higher peak)
        else:
            value = int(85 - (25 * ((progress - 0.5) * 2)))  # 85 -> 60
        events.append((beats_to_ticks(beat), "cc", CC_EXPRESSION, value, channel))

    # Section 3: beats 111-166 (Order Reasserts) - back to normal cycle
    for beat in range(111, 167, 4):
        progress = (beat - 111) / 55.0
        if progress < 0.5:
            value = int(60 + (20 * (progress * 2)))  # 60 -> 80
        else:
            value = int(80 - (20 * ((progress - 0.5) * 2)))  # 80 -> 60
        events.append((beats_to_ticks(beat), "cc", CC_EXPRESSION, value, channel))


def add_modulation_automation(events: list, channel: int) -> None:
    """
    Add modulation CC automation for suppressed anxiety.
    CC1 (Modulation): 40 -> 60 -> 40 (cycles)
    """
    # Section 1: stable, low anxiety
    for beat in range(0, 56, 8):
        progress = beat / 55.0
        value = int(40 + (15 * abs(2 * progress - 1)))  # Triangle wave
        events.append((beats_to_ticks(beat), "cc", CC_MODULATION, value, channel))

    # Section 2: higher anxiety (cracks)
    for beat in range(56, 111, 6):
        progress = (beat - 56) / 54.0
        value = int(50 + (20 * abs(2 * progress - 1)))  # Higher baseline
        events.append((beats_to_ticks(beat), "cc", CC_MODULATION, value, channel))

    # Section 3: return to stable
    for beat in range(111, 167, 8):
        progress = (beat - 111) / 55.0
        value = int(40 + (15 * abs(2 * progress - 1)))
        events.append((beats_to_ticks(beat), "cc", CC_MODULATION, value, channel))


def add_static_ccs(events: list, channel: int) -> None:
    """
    Add static CC values that don't change.
    CC93 (Chorus): ZERO (NO SOLIDARITY)
    CC94 (Detune): 30 constant (controlled fragmentation)
    CC71 (Resonance): 70 constant (harsh but stable)
    """
    events.append((0, "cc", CC_CHORUS, 0, channel))  # NO SOLIDARITY
    events.append((0, "cc", CC_DETUNE, 30, channel))  # Controlled fragmentation
    events.append((0, "cc", CC_RESONANCE, 70, channel))  # Harsh but stable


def create_harpsichord_track() -> MidiTrack:
    """
    Track 1: Harpsichord - The Machine (Program 6, Channel 0)
    Cold, mechanical patterns. Relentless, inhuman precision.
    The harpsichord represents the bureaucratic machine.
    """
    track = MidiTrack()
    track.name = "Harpsichord - The Machine"
    track.append(Message("program_change", program=6, channel=0, time=0))

    events: list = []

    # Add CC automation
    add_static_ccs(events, 0)
    add_expression_automation(events, 0)
    add_modulation_automation(events, 0)

    # The Machine pattern: relentless sixteenth notes
    # E minor arpeggios, mechanical and cold

    # Section A (beats 0-55): Order Established - steady mechanical pulse
    # Pattern: E-B-E-G repeating, like gears turning
    machine_pattern = [
        (NOTES["E3"], 60),
        (NOTES["B3"], 55),
        (NOTES["E4"], 58),
        (NOTES["G3"], 52),
    ]

    for beat in range(0, 56):
        for i, (note, velocity) in enumerate(machine_pattern):
            offset = i * 0.25  # Sixteenth notes
            # Slight velocity variation for mechanical feel
            vel = velocity + (beat % 4) - 2
            events.append((beats_to_ticks(beat + offset), "note_on", note, vel))
            events.append((beats_to_ticks(beat + offset + 0.2), "note_off", note, 0))

    # Section B (beats 56-110): Cracks Appear - dissonances intrude then suppress
    for beat in range(56, 111):
        pattern = machine_pattern.copy()
        # Every 8 beats, introduce a crack (wrong note) then correct
        if beat % 8 == 4:
            # Crack: F natural instead of F#, or A# instead of A
            pattern = [
                (NOTES["E3"], 65),
                (NOTES["B3"], 60),
                (NOTES["F#4"] - 1, 62),  # F natural - dissonance!
                (NOTES["G3"], 58),
            ]
        elif beat % 8 == 5:
            # Suppression: harder, correcting
            pattern = [
                (NOTES["E3"], 75),  # Louder, asserting
                (NOTES["B3"], 70),
                (NOTES["E4"], 72),
                (NOTES["G3"], 68),
            ]

        for i, (note, velocity) in enumerate(pattern):
            offset = i * 0.25
            events.append((beats_to_ticks(beat + offset), "note_on", note, velocity))
            events.append((beats_to_ticks(beat + offset + 0.2), "note_off", note, 0))

    # Section C (beats 111-166): Order Reasserts - return to mechanical routine
    # Same as section A but ending leads back to beginning
    for beat in range(111, 166):
        for i, (note, velocity) in enumerate(machine_pattern):
            offset = i * 0.25
            # Slight fade at very end to enable seamless loop
            fade = 1.0 if beat < 162 else (166 - beat) / 4.0
            vel = int((velocity + (beat % 4) - 2) * fade)
            if vel > 0:
                events.append((beats_to_ticks(beat + offset), "note_on", note, vel))
                events.append((beats_to_ticks(beat + offset + 0.2), "note_off", note, 0))

    # Sort and convert to delta time
    events.sort(key=lambda x: (x[0], 0 if x[1] == "cc" else 1, x[1] == "note_off"))

    last_time = 0
    for event in events:
        event_time = event[0]
        delta = event_time - last_time
        if event[1] == "cc":
            _, _, cc_num, cc_val, ch = event
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=ch, time=delta)
            )
        else:
            _, msg_type, note, velocity = event[:4]
            track.append(Message(msg_type, note=note, velocity=velocity, channel=0, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 2: String Ensemble - Suppression (Program 48, Channel 1)
    Heavy, pressing down. The weight of the state on the people.
    Long sustained notes that press down relentlessly.
    """
    track = MidiTrack()
    track.name = "Strings - Suppression"
    track.append(Message("program_change", program=48, channel=1, time=0))

    events: list = []

    add_static_ccs(events, 1)
    add_expression_automation(events, 1)

    # Sustained chords that press downward - heavy, oppressive
    notes = [
        # Section A (Order Established): E minor drone, steady pressure
        ("E2", 0, 16, 55),
        ("B2", 0, 16, 52),
        ("E3", 0, 16, 48),
        ("E2", 16, 16, 58),
        ("B2", 16, 16, 55),
        ("G3", 16, 16, 50),
        ("E2", 32, 24, 60),
        ("B2", 32, 24, 57),
        ("E3", 32, 24, 52),
        ("G3", 40, 16, 48),
        # Section B (Cracks Appear): slight dissonances, then suppression
        ("E2", 56, 16, 62),
        ("B2", 56, 16, 58),
        ("E3", 56, 16, 54),
        # Crack: add F natural briefly
        ("F3", 72, 4, 45),  # Dissonance creeping in
        ("E2", 72, 20, 65),
        ("B2", 72, 20, 60),
        ("E3", 76, 16, 58),  # Correction - back to E
        # More pressure to suppress
        ("E2", 92, 19, 68),
        ("B2", 92, 19, 64),
        ("G3", 92, 19, 58),
        ("E3", 92, 19, 55),
        # Section C (Order Reasserts): return to initial pressure
        ("E2", 111, 20, 60),
        ("B2", 111, 20, 56),
        ("E3", 111, 20, 52),
        ("E2", 131, 20, 58),
        ("B2", 131, 20, 54),
        ("G3", 131, 20, 50),
        ("E2", 151, 15, 55),  # Fade for loop
        ("B2", 151, 15, 52),
        ("E3", 151, 15, 48),
    ]

    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], 0 if x[1] == "cc" else 1, x[1] == "note_off"))

    last_time = 0
    for event in events:
        event_time = event[0]
        delta = event_time - last_time
        if event[1] == "cc":
            _, _, cc_num, cc_val, ch = event
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=ch, time=delta)
            )
        else:
            _, msg_type, note, velocity = event[:4]
            track.append(Message(msg_type, note=note, velocity=velocity, channel=1, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 3: Timpani - The Clock (Program 47, Channel 2)
    Relentless ticking. Time passing under oppression.
    Not dramatic thunder, but the mechanical tick of state time.
    """
    track = MidiTrack()
    track.name = "Timpani - The Clock"
    track.append(Message("program_change", program=47, channel=2, time=0))

    events: list = []

    add_static_ccs(events, 2)
    add_expression_automation(events, 2)

    # Clock pattern: steady quarter notes, relentless
    # E2 on beats 1 and 3, B1 on beats 2 and 4

    # Section A: steady clock
    for beat in range(0, 56):
        if beat % 4 == 0:
            note = NOTES["E2"]
            velocity = 58
        elif beat % 4 == 2:
            note = NOTES["B1"]
            velocity = 52
        elif beat % 4 == 1:
            note = NOTES["E2"]
            velocity = 45
        else:  # beat % 4 == 3
            note = NOTES["B1"]
            velocity = 42

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Section B: clock becomes slightly erratic during cracks, then steadies
    for beat in range(56, 111):
        # Occasional irregular beat during "cracks"
        if beat % 8 == 4:
            # Irregular: double hit (crack showing)
            events.append((beats_to_ticks(beat), "note_on", NOTES["E2"], 65))
            events.append((beats_to_ticks(beat + 0.2), "note_off", NOTES["E2"], 0))
            events.append((beats_to_ticks(beat + 0.5), "note_on", NOTES["E2"], 70))
            events.append((beats_to_ticks(beat + 0.7), "note_off", NOTES["E2"], 0))
        elif beat % 8 == 5:
            # Suppression: harder hit
            events.append((beats_to_ticks(beat), "note_on", NOTES["B1"], 75))
            events.append((beats_to_ticks(beat + 0.5), "note_off", NOTES["B1"], 0))
        else:
            # Normal pattern
            if beat % 4 == 0:
                note = NOTES["E2"]
                velocity = 60
            elif beat % 4 == 2:
                note = NOTES["B1"]
                velocity = 55
            elif beat % 4 == 1:
                note = NOTES["E2"]
                velocity = 48
            else:
                note = NOTES["B1"]
                velocity = 45

            events.append((beats_to_ticks(beat), "note_on", note, velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    # Section C: back to steady, ending fades to loop
    for beat in range(111, 166):
        if beat % 4 == 0:
            note = NOTES["E2"]
            velocity = 58
        elif beat % 4 == 2:
            note = NOTES["B1"]
            velocity = 52
        elif beat % 4 == 1:
            note = NOTES["E2"]
            velocity = 45
        else:
            note = NOTES["B1"]
            velocity = 42

        # Slight fade at end for loop
        if beat >= 162:
            velocity = int(velocity * (166 - beat) / 4.0)

        if velocity > 0:
            events.append((beats_to_ticks(beat), "note_on", note, velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], 0 if x[1] == "cc" else 1, x[1] == "note_off"))

    last_time = 0
    for event in events:
        event_time = event[0]
        delta = event_time - last_time
        if event[1] == "cc":
            _, _, cc_num, cc_val, ch = event
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=ch, time=delta)
            )
        else:
            _, msg_type, note, velocity = event[:4]
            track.append(Message(msg_type, note=note, velocity=velocity, channel=2, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 4: Trombone - Order (Program 57, Channel 3)
    Authority, weight. The voice of state power.
    Long, heavy phrases that assert order.
    """
    track = MidiTrack()
    track.name = "Low Brass - Order"
    track.append(Message("program_change", program=57, channel=3, time=0))

    events: list = []

    add_static_ccs(events, 3)
    add_expression_automation(events, 3)

    # Heavy, authoritative statements of ORDER
    notes = [
        # Section A: Declarations of order
        ("E2", 0, 8, 65),
        ("E3", 0, 8, 60),
        ("B2", 8, 8, 62),
        ("E3", 8, 8, 58),
        ("E2", 16, 12, 68),
        ("B2", 20, 8, 64),
        ("E3", 16, 12, 60),
        ("E2", 32, 8, 65),
        ("G2", 40, 8, 62),
        ("E3", 32, 16, 58),
        ("B2", 48, 8, 68),
        ("E3", 48, 8, 64),
        # Section B: Suppressing cracks with force
        ("E2", 56, 8, 70),
        ("E3", 56, 8, 66),
        # Asserting after crack
        ("E2", 72, 12, 78),  # Louder after crack
        ("B2", 72, 12, 74),
        ("E3", 72, 12, 70),
        ("E2", 88, 8, 72),
        ("G2", 88, 8, 68),
        ("E3", 92, 8, 65),
        ("E2", 100, 11, 70),
        ("B2", 104, 7, 68),
        # Section C: Return to steady authority
        ("E2", 111, 10, 65),
        ("E3", 111, 10, 60),
        ("B2", 121, 10, 62),
        ("E3", 125, 6, 58),
        ("E2", 131, 12, 65),
        ("G2", 139, 8, 60),
        ("E2", 147, 10, 62),
        ("B2", 151, 8, 58),
        # Fade for loop
        ("E2", 159, 6, 55),
        ("E3", 159, 6, 50),
    ]

    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], 0 if x[1] == "cc" else 1, x[1] == "note_off"))

    last_time = 0
    for event in events:
        event_time = event[0]
        delta = event_time - last_time
        if event[1] == "cc":
            _, _, cc_num, cc_val, ch = event
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=ch, time=delta)
            )
        else:
            _, msg_type, note, velocity = event[:4]
            track.append(Message(msg_type, note=note, velocity=velocity, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_organ_track() -> MidiTrack:
    """
    Track 5: Church Organ - False Peace (Program 19, Channel 4)
    Hollow grandeur. The false legitimacy of the regime.
    Sustained chords that sound "official" but are spiritually empty.
    """
    track = MidiTrack()
    track.name = "Organ - False Peace"
    track.append(Message("program_change", program=19, channel=4, time=0))

    events: list = []

    add_static_ccs(events, 4)
    add_expression_automation(events, 4)

    # Sustained organ chords - hollow grandeur
    notes = [
        # Section A: Establishing false legitimacy
        ("E2", 0, 28, 45),
        ("B2", 0, 28, 42),
        ("E3", 4, 24, 40),
        ("G3", 12, 16, 38),
        ("E2", 28, 28, 48),
        ("B2", 28, 28, 45),
        ("E3", 32, 24, 42),
        # Section B: Slightly more intense during cracks
        ("E2", 56, 28, 52),
        ("B2", 56, 28, 48),
        ("E3", 60, 24, 45),
        ("G3", 68, 16, 42),
        # After crack: reasserting
        ("E2", 84, 27, 55),
        ("B2", 84, 27, 52),
        ("E3", 88, 23, 48),
        ("G3", 96, 15, 45),
        # Section C: Return to hollow peace
        ("E2", 111, 28, 48),
        ("B2", 111, 28, 45),
        ("E3", 115, 24, 42),
        ("G3", 123, 16, 40),
        ("E2", 139, 26, 45),
        ("B2", 139, 26, 42),
        ("E3", 143, 20, 40),
        # Fade for loop
    ]

    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], 0 if x[1] == "cc" else 1, x[1] == "note_off"))

    last_time = 0
    for event in events:
        event_time = event[0]
        delta = event_time - last_time
        if event[1] == "cc":
            _, _, cc_num, cc_val, ch = event
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=ch, time=delta)
            )
        else:
            _, msg_type, note, velocity = event[:4]
            track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'False Order'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_harpsichord_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_organ_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/fascist/06_false_order.mid"

    print("Creating 'False Order' - Fascist Suite 06")
    print("=" * 60)
    print("Concept: Stability through oppression - the grinding continues")
    print("The machine that cannot stop")
    print()

    mid = create_midi_file()
    mid.save(output_path)

    print(f"Saved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Tempo: {BPM} BPM")
    print(f"Total beats: {TOTAL_BEATS}")
    print(f"Track count: {len(mid.tracks)}")
    print()
    print("Track listing:")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    length = mid.length
    print(f"\nDuration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\nExpression Automation:")
    print("  CC11 (Expression): 60 -> 80 -> 60 (cycling, no resolution)")
    print("  CC93 (Chorus): 0 (ZERO - NO SOLIDARITY)")
    print("  CC94 (Detune): 30 (controlled fragmentation)")
    print("  CC1 (Modulation): 40 -> 60 -> 40 (suppressed anxiety)")
    print("  CC71 (Resonance): 70 (harsh but stable)")

    print("\nMusical Arc:")
    print("  A. Order Established (0-55): Machine begins, cold efficiency")
    print("  B. Cracks Appear (56-110): Dissonances suppressed immediately")
    print("  C. Order Reasserts (111-166): Return to routine (loops)")

    print("\nIntegration Guidance:")
    print("  - Use for fascist faction steady-state gameplay")
    print("  - Loop seamlessly - ending returns to beginning")
    print("  - NO RESOLUTION - the machine never stops")
    print("  - Pair with repression events and surveillance mechanics")


if __name__ == "__main__":
    main()
