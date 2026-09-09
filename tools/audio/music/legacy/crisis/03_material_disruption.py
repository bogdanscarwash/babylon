#!/usr/bin/env python3
"""
BABYLON - Crisis Suite
03_material_disruption.mid - "Material Disruption"

CONCEPTUAL BRIEF:
This piece represents the moment when the material base begins crumbling but
bifurcation has not yet occurred. The ground shifts beneath everyone's feet.
There is instability, disruption, uncertainty - but crucially, the routing to
either revolution or fascism remains undetermined. CC93 (Chorus) is held at
30 - the NEUTRAL position where solidarity is uncertain, could go either way.
The listener should feel that everything is in flux, but with no indication
of which direction history will turn.

TECHNICAL SPECIFICATION:
- Key: Ambiguous (oscillating between F minor and Db major - never settling)
- Tempo: 85 BPM (lurching, unstable - neither the confidence of march nor flow)
- Time Signature: 4/4 (but with irregular phrase lengths creating destabilization)
- Duration: ~120 seconds (170 beats at 85 BPM)
- Loop Points: Beat 25 through beat 161 for seamless looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Piano (Program 0) - "Fragments" - Broken phrases, searching for stability
- Channel 1: Tremolo Strings (Program 44) - "Trembling Ground" - Unstable foundation
- Channel 2: Contrabass (Program 43) - "Shifting Base" - Irregular, unpredictable
- Channel 3: Warm Pad (Program 89) - "Uncertainty" - Ambient unease, questioning
- Channel 4: Timpani (Program 47) - "Distant Rumble" - Occasional ominous thuds

EXPRESSION AUTOMATION (per specification):
- CC11 (Expression): 40 -> 70 -> 50 (builds then recedes, NEVER resolves)
- CC93 (Chorus): 30 constant (NEUTRAL - solidarity uncertain, could go either way)
- CC94 (Detune): 0 constant (not atomized yet - that comes later in the sequence)
- CC1 (Modulation): 30 -> 60 (mounting unease)
- CC71 (Resonance): 50 -> 80 (grinding discomfort)
- Pitch Bend: Irregular wavering (ground literally shifting)

MUSICAL ARC (120 seconds = 170 beats at 85 BPM):
A. Initial Instability (beats 0-42): Ground begins to shake, certainties dissolve
B. Disruption (beats 43-100): Full destabilization, all instruments unsettled
C. Uncertainty (beats 101-170): Hovering tension, NO RESOLUTION (bifurcation pending)

COMPOSITIONAL NOTES:
- F minor and Db major share 4 flats, but their tonal centers conflict
- Shifting between them creates a "ground shifting" sensation
- Neither key is allowed to establish dominance (prevents premature routing)
- The tritone Ab-D creates fundamental instability throughout
- CC93=30 is CRITICAL: not 0 (fascist isolation) and not high (revolutionary solidarity)
- This represents the material conditions in flux BEFORE consciousness routes
- The piece must feel like standing on uncertain ground, not falling or rising
- Timpani hits are irregular - no march rhythm (prevents fascist association)
- Pitch bend adds literal "ground shifting" effect to strings
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 85  # Lurching, unstable tempo
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 170  # ~120 seconds at 85 BPM

# Note definitions (MIDI note numbers)
# F minor scale: F, G, Ab, Bb, C, Db, Eb
# Db major scale: Db, Eb, F, Gb, Ab, Bb, C
# Shared notes create the ambiguity
NOTES = {
    # Octave 1
    "C1": 24,
    "Db1": 25,
    "D1": 26,
    "Eb1": 27,
    "E1": 28,
    "F1": 29,
    "Gb1": 30,
    "G1": 31,
    "Ab1": 32,
    "A1": 33,
    "Bb1": 34,
    "B1": 35,
    # Octave 2
    "C2": 36,
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
    "B5": 83,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="Material Disruption - Conductor", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    # F minor key signature (4 flats) - but we'll oscillate with Db major
    track.append(MetaMessage("key_signature", key="Fm", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_piano_track() -> MidiTrack:
    """
    Track 1: Piano - Fragments (Program 0, Channel 0)
    Broken phrases that search for stability but never find it.
    Fragments that start in F minor, drift toward Db major, then dissolve.
    Short, questioning phrases - nothing completes itself.
    """
    track = MidiTrack()
    track.name = "Piano - Fragments"
    track.append(Message("program_change", program=0, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes: list[tuple[str, float, float, int]] = []

    # Section A (beats 0-42): Initial Instability
    # Fragment 1 - starts in F minor, questioning
    notes.extend(
        [
            ("F4", 2, 1.5, 52),
            ("Ab4", 3.5, 1, 48),
            ("C5", 5, 0.75, 45),  # Ascending... but...
            ("Bb4", 6, 1.5, 42),  # Falls back, incomplete
        ]
    )

    # Fragment 2 - drifts toward Db major
    notes.extend(
        [
            ("Db4", 10, 1.5, 50),
            ("F4", 11.5, 1, 47),
            ("Ab4", 13, 0.75, 44),  # Could be either key
            ("Gb4", 14.5, 1.5, 48),  # Db major flavor
        ]
    )

    # Fragment 3 - back to F minor, more unstable
    notes.extend(
        [
            ("C4", 18, 1, 55),
            ("F4", 19, 1, 52),
            ("Ab4", 20.5, 0.75, 48),
            ("D4", 22, 1.5, 58),  # Tritone! Disruptive
        ]
    )

    # Fragment 4 - dissolving phrases
    notes.extend(
        [
            ("Eb4", 26, 1, 50),
            ("Db4", 27.5, 0.75, 47),
            ("Bb3", 29, 1.5, 44),
            # Pause - searching
            ("Ab4", 34, 0.75, 52),
            ("F4", 35, 0.75, 48),
            ("Db4", 36.5, 1, 45),
            ("Eb4", 38, 1.5, 50),  # Neither resolves to F nor Db
        ]
    )

    # Section B (beats 43-100): Disruption - more fragmented
    # Fragment 5 - increasingly unstable
    notes.extend(
        [
            ("F5", 43, 0.75, 58),
            ("Eb5", 44, 0.75, 55),
            ("Db5", 45, 0.75, 52),
            ("C5", 46, 0.75, 50),
            # Drop down
            ("Ab4", 48, 1, 55),
            ("D4", 49.5, 1.5, 60),  # Tritone again
        ]
    )

    # Fragment 6 - searching, lost
    notes.extend(
        [
            ("Bb4", 54, 0.75, 52),
            ("Ab4", 55, 0.75, 50),
            ("Gb4", 56.5, 0.75, 48),  # Db major
            ("F4", 58, 1, 52),  # F minor
            # Which is it?
            ("E4", 60, 1.5, 58),  # Chromatic disruption!
        ]
    )

    # Fragment 7 - repeated questioning notes
    notes.extend(
        [
            ("F4", 64, 0.5, 55),
            ("F4", 65, 0.5, 52),
            ("F4", 66, 0.5, 50),  # Stuck, uncertain
            ("Gb4", 68, 0.75, 48),
            ("F4", 69.5, 0.75, 45),
            ("E4", 71, 0.75, 55),  # Chromatic - which way?
        ]
    )

    # Fragment 8 - descending fragments
    notes.extend(
        [
            ("Db5", 74, 0.75, 58),
            ("C5", 75, 0.75, 55),
            ("Bb4", 76.5, 0.75, 52),
            ("Ab4", 78, 1, 50),
            ("G4", 80, 1.5, 55),  # G natural - not in F minor!
        ]
    )

    # Fragment 9 - almost finding Db major...
    notes.extend(
        [
            ("Db4", 84, 1, 52),
            ("F4", 85, 1, 50),
            ("Ab4", 86.5, 1, 48),
            ("Db5", 88, 1.5, 55),  # Almost cadence...
            ("C5", 90, 1, 52),  # But no, falls to C
            ("Bb4", 92, 1.5, 48),
        ]
    )

    # Fragment 10 - disintegrating
    notes.extend(
        [
            ("Ab4", 96, 0.5, 50),
            ("F4", 97, 0.5, 48),
            ("Db4", 98, 0.5, 45),
            ("Ab3", 99.5, 1.5, 42),  # Fading down
        ]
    )

    # Section C (beats 101-170): Uncertainty - hovering, no resolution
    # Fragment 11 - sparse, questioning
    notes.extend(
        [
            ("F4", 103, 1.5, 48),
            ("Eb4", 106, 1.5, 45),
            ("Db4", 110, 2, 42),  # Long, suspended
        ]
    )

    # Fragment 12 - tritone again
    notes.extend(
        [
            ("Ab4", 116, 1, 50),
            ("D4", 118, 2, 55),  # Tritone - fundamental instability
            ("Eb4", 122, 1.5, 48),
        ]
    )

    # Fragment 13 - more questioning
    notes.extend(
        [
            ("C4", 128, 1, 52),
            ("F4", 129.5, 1, 50),
            ("Ab4", 132, 1.5, 48),
            ("Bb4", 135, 1.5, 45),  # Neither F nor Db
        ]
    )

    # Fragment 14 - wavering
    notes.extend(
        [
            ("Gb4", 140, 1, 50),  # Db major?
            ("F4", 142, 1, 52),  # F minor?
            ("Gb4", 144, 1, 48),  # Back to Db?
            ("F4", 146, 1.5, 45),  # Undecided
        ]
    )

    # Fragment 15 - final suspension, no resolution
    notes.extend(
        [
            ("Ab4", 152, 1.5, 48),
            ("Db4", 155, 2, 45),
            ("F4", 158, 2, 42),
            ("Eb4", 162, 3, 40),  # Held, uncertain
            ("Db4", 166, 4, 38),  # Fading, still unresolved
        ]
    )

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
    Track 2: Tremolo Strings - Trembling Ground (Program 44, Channel 1)
    The foundation trembles - nothing is stable anymore.
    Sustained tones with pitch bend creating wavering effect.
    Represents the material base in disruption.
    """
    track = MidiTrack()
    track.name = "Strings - Trembling Ground"
    track.append(Message("program_change", program=44, channel=1, time=0))

    # We'll add notes and pitch bend automation
    notes: list[tuple[str, float, float, int]] = []

    # Section A (beats 0-42): Ground begins to shake
    notes.extend(
        [
            # F minor foundation - but trembling
            ("F3", 0, 14, 45),
            ("Ab3", 0, 14, 42),
            ("C4", 0, 14, 40),
            # Shift toward Db
            ("Db3", 14, 14, 48),
            ("F3", 14, 14, 45),
            ("Ab3", 14, 14, 42),
            # Back to ambiguity
            ("Eb3", 28, 14, 50),
            ("G3", 28, 14, 47),  # G natural - unsettling
            ("Bb3", 28, 14, 44),
        ]
    )

    # Section B (beats 43-100): Full trembling
    notes.extend(
        [
            # Unstable cluster
            ("F3", 43, 12, 55),
            ("Gb3", 43, 12, 52),  # Minor second - grinding
            ("Ab3", 43, 12, 50),
            # Shift
            ("Db3", 55, 12, 58),
            ("Eb3", 55, 12, 55),
            ("Ab3", 55, 12, 52),
            # Tritone added
            ("D3", 67, 10, 60),  # Tritone against Ab
            ("F3", 67, 10, 55),
            ("Ab3", 67, 10, 52),
            # More unstable
            ("Eb3", 77, 12, 58),
            ("E3", 77, 12, 62),  # Chromatic cluster!
            ("F3", 77, 12, 55),
            # Building tension
            ("Db3", 89, 11, 55),
            ("F3", 89, 11, 52),
            ("Ab3", 89, 11, 50),
            ("D4", 89, 11, 58),  # Tritone high
        ]
    )

    # Section C (beats 101-170): Sustained uncertainty
    notes.extend(
        [
            # Hovering, unresolved
            ("F3", 101, 16, 50),
            ("Ab3", 101, 16, 48),
            ("Db4", 101, 16, 45),
            # Shift again
            ("Eb3", 117, 16, 48),
            ("Gb3", 117, 16, 45),
            ("Bb3", 117, 16, 42),
            # More ambiguity
            ("F3", 133, 14, 45),
            ("Ab3", 133, 14, 42),
            ("C4", 133, 14, 40),
            # Final sustained - neither F nor Db resolves
            ("Db3", 147, 12, 42),
            ("F3", 147, 12, 40),
            ("Ab3", 147, 12, 38),
            # Fading, still trembling
            ("F3", 159, 11, 35),
            ("Ab3", 159, 11, 33),
            ("Db4", 159, 11, 30),
        ]
    )

    # Build event list including pitch bend
    events: list[tuple[int, str, int, int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    # Add pitch bend events for wavering effect (ground shifting)
    # Pitch bend range: 0 = max down, 8192 = center, 16383 = max up
    # We'll create irregular wavering
    pitch_bend_points = [
        (0, 8192),  # Center
        (4, 8500),  # Slight up
        (8, 7800),  # Down
        (12, 8400),  # Up
        (18, 7600),  # More down
        (24, 8300),  # Back up
        (32, 7900),  # Down
        (40, 8192),  # Center
        # Section B - more dramatic wavering
        (46, 8700),  # Up
        (52, 7400),  # Down significantly
        (58, 8600),  # Up
        (64, 7200),  # Lower
        (72, 8800),  # Higher
        (80, 7000),  # Even lower
        (88, 8500),  # Back up
        (96, 7500),  # Down
        # Section C - sustained wavering
        (104, 8300),
        (112, 7700),
        (120, 8400),
        (130, 7600),
        (140, 8200),
        (150, 7800),
        (160, 8000),
        (168, 8192),  # Return to center for loop
    ]

    for beat, pitch_value in pitch_bend_points:
        events.append((beats_to_ticks(beat), "pitchwheel", pitch_value, 0))

    # Sort events - pitch bend events need special handling
    def event_sort_key(event: tuple[int, str, int, int]) -> tuple[int, int]:
        event_time, msg_type, _, _ = event
        # Sort order: pitchwheel before note_on, note_off last
        if msg_type == "pitchwheel":
            order = 0
        elif msg_type == "note_on":
            order = 1
        else:  # note_off
            order = 2
        return (event_time, order)

    events.sort(key=event_sort_key)

    last_time = 0
    for event in events:
        event_time, msg_type, value1, value2 = event
        delta = event_time - last_time
        if msg_type == "pitchwheel":
            track.append(Message("pitchwheel", pitch=value1 - 8192, channel=1, time=delta))
        else:
            track.append(Message(msg_type, note=value1, velocity=value2, channel=1, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_bass_track() -> MidiTrack:
    """
    Track 3: Contrabass - Shifting Base (Program 43, Channel 2)
    Irregular bass that represents the unstable material foundation.
    No steady rhythm - the ground is shifting.
    Irregular entries that destabilize rather than anchor.
    """
    track = MidiTrack()
    track.name = "Bass - Shifting Base"
    track.append(Message("program_change", program=43, channel=2, time=0))

    # Irregular bass notes - NOT a steady foundation
    notes: list[tuple[str, float, float, int]] = [
        # Section A: Initial instability
        ("F2", 3, 5, 55),  # Enters late - already unstable
        ("Db2", 11, 4, 52),
        ("Eb2", 18, 5, 58),
        ("Ab1", 26, 6, 55),
        ("D2", 35, 4, 62),  # Tritone - very unstable
        # Section B: More irregular
        ("F2", 44, 4, 58),
        ("Gb2", 51, 5, 55),  # Half step from F - grinding
        ("Eb2", 59, 4, 60),
        ("Ab1", 66, 5, 58),
        ("D2", 74, 4, 65),  # Tritone again
        ("F2", 81, 4, 55),
        ("Db2", 88, 5, 52),
        ("E2", 96, 4, 62),  # Chromatic disruption
        # Section C: Sparse and uncertain
        ("F2", 105, 6, 50),
        ("Eb2", 116, 5, 48),
        ("Ab1", 126, 6, 52),
        ("D2", 137, 5, 55),  # Tritone
        ("Db2", 146, 6, 48),
        ("F2", 157, 8, 45),  # Long, fading
        ("Ab1", 166, 4, 40),  # Final uncertainty
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


def create_synth_pad_track() -> MidiTrack:
    """
    Track 4: Synth Pad - Uncertainty (Program 89, Channel 3)
    Ambient unease - the fog of uncertainty about the future.
    Neither hopeful nor despairing - genuinely undetermined.
    This is where the CC automation is most critical.
    """
    track = MidiTrack()
    track.name = "Synth Pad - Uncertainty"
    track.append(Message("program_change", program=89, channel=3, time=0))

    # Ambient pad notes
    notes: list[tuple[str, float, float, int]] = [
        # Section A: Building unease
        ("F3", 4, 18, 35),
        ("Ab3", 4, 18, 32),
        ("C4", 4, 18, 30),
        ("Db3", 24, 18, 38),
        ("F3", 24, 18, 35),
        ("Ab3", 24, 18, 32),
        # Section B: Full uncertainty
        ("Eb3", 44, 16, 42),
        ("G3", 44, 16, 40),  # G natural - unsettling
        ("Bb3", 44, 16, 38),
        ("D4", 50, 10, 45),  # Tritone added
        ("F3", 62, 16, 45),
        ("Ab3", 62, 16, 42),
        ("Db4", 62, 16, 40),
        ("E4", 70, 8, 48),  # Chromatic!
        ("Db3", 80, 18, 42),
        ("F3", 80, 18, 40),
        ("Ab3", 80, 18, 38),
        ("D4", 86, 12, 45),  # Tritone again
        # Section C: Sustained uncertainty
        ("F3", 100, 20, 40),
        ("Ab3", 100, 20, 38),
        ("C4", 100, 20, 35),
        ("Eb3", 122, 20, 38),
        ("Gb3", 122, 20, 35),
        ("Bb3", 122, 20, 32),
        ("Db3", 144, 16, 35),
        ("F3", 144, 16, 32),
        ("Ab3", 144, 16, 30),
        # Final sustained - no resolution
        ("F3", 162, 8, 30),
        ("Ab3", 162, 8, 28),
        ("Db4", 162, 8, 25),
    ]

    events: list[tuple[int, str, int | tuple[int, int], int]] = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    # Add CC automation for this channel
    # CC11 (Expression): 40 -> 70 -> 50 (builds then recedes, NEVER resolves)
    expression_points = [
        (0, 40),
        (20, 45),
        (40, 55),
        (60, 65),
        (85, 70),  # Peak
        (100, 65),
        (120, 58),
        (140, 52),
        (160, 50),  # Receded but not resolved
    ]
    for beat, value in expression_points:
        events.append((beats_to_ticks(beat), "cc", (11, value), 0))

    # CC93 (Chorus): 30 CONSTANT - CRITICAL for neutral solidarity
    events.append((beats_to_ticks(0), "cc", (93, 30), 0))
    # Reinforce at intervals to ensure it stays
    events.append((beats_to_ticks(40), "cc", (93, 30), 0))
    events.append((beats_to_ticks(80), "cc", (93, 30), 0))
    events.append((beats_to_ticks(120), "cc", (93, 30), 0))
    events.append((beats_to_ticks(160), "cc", (93, 30), 0))

    # CC94 (Detune): 0 CONSTANT - not atomized yet
    events.append((beats_to_ticks(0), "cc", (94, 0), 0))

    # CC1 (Modulation): 30 -> 60 (mounting unease)
    modulation_points = [
        (0, 30),
        (30, 35),
        (60, 45),
        (90, 52),
        (120, 58),
        (150, 60),
        (165, 58),  # Slight pullback for loop
    ]
    for beat, value in modulation_points:
        events.append((beats_to_ticks(beat), "cc", (1, value), 0))

    # CC71 (Resonance): 50 -> 80 (grinding discomfort)
    resonance_points = [
        (0, 50),
        (25, 55),
        (50, 62),
        (75, 70),
        (100, 75),
        (130, 78),
        (160, 80),
    ]
    for beat, value in resonance_points:
        events.append((beats_to_ticks(beat), "cc", (71, value), 0))

    # Sort all events
    def event_sort_key(event: tuple[int, str, int | tuple[int, int], int]) -> tuple[int, int]:
        event_time, msg_type, _, _ = event
        if msg_type == "cc":
            return (event_time, 0)  # CC before notes
        elif msg_type == "note_on":
            return (event_time, 1)
        else:
            return (event_time, 2)

    events.sort(key=event_sort_key)

    last_time = 0
    for event in events:
        event_time, msg_type, value1, value2 = event
        delta = event_time - last_time
        if msg_type == "cc":
            assert isinstance(value1, tuple)
            cc_num, cc_val = value1
            track.append(
                Message("control_change", control=cc_num, value=cc_val, channel=3, time=delta)
            )
        else:
            assert isinstance(value1, int)
            track.append(Message(msg_type, note=value1, velocity=value2, channel=3, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 5: Timpani - Distant Rumble (Program 47, Channel 4)
    Occasional ominous thuds - irregular, NOT a march.
    These are warnings, not rhythm. The crisis approaches but timing is uncertain.
    Irregular spacing prevents fascist march association.
    """
    track = MidiTrack()
    track.name = "Timpani - Distant Rumble"
    track.append(Message("program_change", program=47, channel=4, time=0))

    # Irregular timpani hits - NOT a march rhythm
    notes: list[tuple[str, float, float, int]] = [
        # Section A: Sparse warnings
        ("F2", 8, 2.5, 45),
        ("Db2", 19, 3, 48),
        ("Ab1", 33, 3, 52),
        # Section B: More frequent but still irregular
        ("F2", 47, 2.5, 55),
        ("Eb2", 58, 2, 52),
        ("D2", 68, 3, 60),  # Tritone note - very unsettling
        ("Ab1", 79, 2.5, 55),
        ("F2", 93, 2, 50),
        # Section C: Ominous, sparse
        ("Db2", 108, 3, 52),
        ("Ab1", 122, 3.5, 55),
        ("D2", 138, 3, 58),  # Tritone rumble
        ("F2", 154, 3, 50),
        ("Ab1", 165, 4, 45),  # Final rumble, fading
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
        track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_expression_track() -> MidiTrack:
    """
    Dedicated automation track for global expression CC values.
    This applies CC automation to all channels for consistent expression.
    """
    track = MidiTrack()
    track.name = "Expression Automation"

    # We'll add CC automation for channels 0-4
    # CC11 (Expression): 40 -> 70 -> 50
    expression_curve = [
        (0, 40),
        (10, 42),
        (20, 45),
        (30, 50),
        (40, 55),
        (50, 60),
        (60, 65),
        (70, 68),
        (85, 70),  # Peak at approximately 60% through
        (95, 68),
        (105, 65),
        (115, 60),
        (125, 56),
        (135, 53),
        (145, 51),
        (155, 50),
        (165, 50),  # Settled but not resolved
    ]

    events: list[tuple[int, int, int, int]] = []  # (tick, channel, control, value)

    # Apply expression to channels 0, 1, 2, 4 (not 3, which has its own automation)
    for channel in [0, 1, 2, 4]:
        for beat, value in expression_curve:
            events.append((beats_to_ticks(beat), channel, 11, value))

    # Add CC93 (Chorus) = 30 to all channels for neutral solidarity
    for channel in range(5):
        events.append((beats_to_ticks(0), channel, 93, 30))
        events.append((beats_to_ticks(85), channel, 93, 30))  # Reinforce at midpoint

    # Add CC94 (Detune) = 0 to all channels (not atomized)
    for channel in range(5):
        events.append((beats_to_ticks(0), channel, 94, 0))

    # Add CC1 (Modulation): 30 -> 60 to channels 0, 1, 2
    modulation_curve = [
        (0, 30),
        (20, 33),
        (40, 38),
        (60, 45),
        (80, 52),
        (100, 55),
        (120, 58),
        (140, 60),
        (160, 58),
    ]
    for channel in [0, 1, 2]:
        for beat, value in modulation_curve:
            events.append((beats_to_ticks(beat), channel, 1, value))

    # Add CC71 (Resonance): 50 -> 80 to channels 0, 1, 3
    resonance_curve = [
        (0, 50),
        (20, 54),
        (40, 60),
        (60, 66),
        (80, 72),
        (100, 76),
        (120, 78),
        (140, 80),
        (160, 80),
    ]
    for channel in [0, 1, 3]:
        for beat, value in resonance_curve:
            events.append((beats_to_ticks(beat), channel, 71, value))

    # Sort by time
    events.sort(key=lambda x: x[0])

    last_time = 0
    for tick, channel, control, value in events:
        delta = tick - last_time
        track.append(
            Message("control_change", control=control, value=value, channel=channel, time=delta)
        )
        last_time = tick

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'Material Disruption'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_expression_track())  # Global automation
    mid.tracks.append(create_piano_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_bass_track())
    mid.tracks.append(create_synth_pad_track())
    mid.tracks.append(create_timpani_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/crisis/03_material_disruption.mid"

    print("Creating 'Material Disruption' - Crisis Suite 03")
    print("=" * 60)

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

    print("\n" + "=" * 60)
    print("CRITICAL EXPRESSION VALUES:")
    print("  CC11 (Expression): 40 -> 70 -> 50 (builds, recedes, NO resolution)")
    print("  CC93 (Chorus): 30 CONSTANT (NEUTRAL solidarity - bifurcation pending)")
    print("  CC94 (Detune): 0 (not atomized yet)")
    print("  CC1 (Modulation): 30 -> 60 (mounting unease)")
    print("  CC71 (Resonance): 50 -> 80 (grinding discomfort)")
    print("  Pitch Bend: Irregular wavering on strings (ground shifting)")
    print("=" * 60)
    print("\nMusical arc: Instability -> Disruption -> Uncertainty (NO RESOLUTION)")
    print("Key: Ambiguous (F minor / Db major oscillation)")
    print("Theme: Material base crumbling, bifurcation NOT YET determined")
    print("\nINTEGRATION NOTE: Play when material conditions destabilize")
    print("but before bifurcation event triggers. CC93=30 is NEUTRAL.")


if __name__ == "__main__":
    main()
