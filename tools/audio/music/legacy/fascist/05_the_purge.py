#!/usr/bin/env python3
"""
BABYLON - Fascist Suite
05_the_purge.mid - "The Purge"

CONCEPTUAL BRIEF:
This is the DARKEST track in the fascist suite. It represents the scapegoat mechanism
reaching its violent conclusion - the moment when dehumanization permits murder.
Where "The Scapegoat" (02) was about FINDING a target, this is about DESTROYING it.
There is no mercy, no resolution, no redemption. Only relentless, state-sanctioned
violence against "the other."

The music captures:
- The terrifying efficiency of organized violence
- The mechanical nature of industrial-scale persecution
- The complete absence of solidarity (CC93 Chorus = ZERO)
- The fragmentation of victims (increasing detune/chaos)
- The bloodlust frenzy of the perpetrators

TECHNICAL SPECIFICATION:
- Key: C minor (brutal, no escape - the darkest minor key)
- Tempo: 130 BPM (aggressive, relentless, hunting)
- Time Signature: 4/4
- Duration: ~85 seconds (184 beats at 130 BPM)
- Loop Points: None - this track should NOT loop. Violence has a beginning and end.

INSTRUMENT ASSIGNMENTS:
- Channel 0: Brass Section (Program 61) - "Violence" - Stabbing, aggressive fanfares
- Channel 1: Tremolo Strings (Program 44) - "The Hunt" - Relentless pursuit
- Channel 2: Timpani (Program 47) - "Jackboots" - Marching, crushing
- Channel 9: Snare/Percussion - "Execution" - Military precision (GM Percussion)
- Channel 4: Tuba (Program 58) - "Doom" - Inescapable fate

EXPRESSION AUTOMATION (CRITICAL):
- CC11 (Expression): 80 -> 127 (maximum aggression throughout)
- CC93 (Chorus): 0 (ZERO SOLIDARITY - complete isolation)
- CC94 (Detune): 50 -> 80 (increasing chaos, fragmentation)
- CC1 (Modulation): 60 -> 100 (building frenzy, bloodlust)
- CC71 (Resonance): 90 -> 127 (maximum harshness, no warmth)
- Pitch bend: Erratic, violent sweeps (±4096 to ±8192)

MUSICAL ARC (85 seconds = 184 beats at 130 BPM):
A. Pursuit (beats 0-63): The hunt begins - quarry identified, chase commences
B. Capture (beats 64-127): Cornered, no escape - the net closes
C. Violence (beats 128-184): The killing - mechanical, efficient, merciless

COMPOSITIONAL NOTES:
- C minor is the darkest, most unforgiving key - there is no hope here
- 130 BPM is aggressive but controlled - this is ORGANIZED violence, not chaos
- Tremolo strings create the sense of constant, vibrating menace
- Timpani as "jackboots" - the crushing weight of state power
- Snare as "execution" - military precision in killing
- Tuba as "doom" - the bass voice of inescapable fate
- NO CHORUS (CC93=0) because there is NO solidarity for the victims
- Increasing detune represents the fragmentation of the victims' world
- The track does NOT resolve - it simply ENDS. Violence leaves only silence.

DARK MIRROR PRINCIPLE:
This is the anti-revolutionary moment. Where revolutionary music builds toward
liberation and collective action, this builds toward destruction and isolation.
The victims have been atomized, dehumanized, and now eliminated.

WARNING: This is intentionally disturbing music. It represents something horrific
precisely because fascism IS horrific. The discomfort is the point.
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
BPM = 130
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 184  # ~85 seconds at 130 BPM

# MIDI Control Change numbers
CC_MODULATION = 1
CC_EXPRESSION = 11
CC_RESONANCE = 71
CC_CHORUS = 93
CC_DETUNE = 94

# Note definitions (MIDI note numbers) - C minor scale and chromatic neighbors
NOTES = {
    # Sub-bass (for tuba doom)
    "C1": 24,
    "D1": 26,
    "Eb1": 27,
    "F1": 29,
    "G1": 31,
    "Ab1": 32,
    "Bb1": 34,
    # Low register
    "C2": 36,
    "D2": 38,
    "Eb2": 39,
    "F2": 41,
    "G2": 43,
    "Ab2": 44,
    "Bb2": 46,
    # Bass register
    "C3": 48,
    "D3": 50,
    "Eb3": 51,
    "F3": 53,
    "G3": 55,
    "Ab3": 56,
    "Bb3": 58,
    # Mid register
    "C4": 60,
    "D4": 62,
    "Eb4": 63,
    "F4": 65,
    "G4": 67,
    "Ab4": 68,
    "Bb4": 70,
    # Upper register
    "C5": 72,
    "D5": 74,
    "Eb5": 75,
    "F5": 77,
    "G5": 79,
    "Ab5": 80,
    "Bb5": 82,
    # High register
    "C6": 84,
}

# GM Percussion notes (Channel 9)
PERC_BASS_DRUM = 36
PERC_SNARE = 38
PERC_SIDE_STICK = 37
PERC_TOM_LOW = 41
PERC_TOM_MID = 45
PERC_TOM_HIGH = 48
PERC_CRASH = 49
PERC_RIDE = 51


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_cc_automation(
    channel: int,
    cc_number: int,
    start_value: int,
    end_value: int,
    start_beat: float,
    end_beat: float,
    resolution: int = 8,
) -> list[tuple[int, int, int, int]]:
    """
    Create smooth CC automation between two values.
    Returns list of (tick, channel, cc_number, value) tuples.

    Args:
        channel: MIDI channel (0-15)
        cc_number: CC number to automate
        start_value: Starting CC value (0-127)
        end_value: Ending CC value (0-127)
        start_beat: Beat to start automation
        end_beat: Beat to end automation
        resolution: Number of steps in automation curve
    """
    events = []
    total_ticks = beats_to_ticks(end_beat - start_beat)
    value_range = end_value - start_value

    for i in range(resolution + 1):
        progress = i / resolution
        tick = beats_to_ticks(start_beat) + int(total_ticks * progress)
        value = int(start_value + value_range * progress)
        value = max(0, min(127, value))  # Clamp to valid range
        events.append((tick, channel, cc_number, value))

    return events


def create_pitch_bend_sweep(
    channel: int,
    start_value: int,
    end_value: int,
    start_beat: float,
    duration_beats: float,
    resolution: int = 8,
) -> list[tuple[int, str, int, int]]:
    """
    Create pitch bend sweep.
    Values are in range -8192 to +8191 (center is 0, MIDI sends 0-16383 with 8192 as center).
    Returns list of (tick, 'pitchwheel', channel, value) tuples.
    """
    events = []
    total_ticks = beats_to_ticks(duration_beats)
    value_range = end_value - start_value

    for i in range(resolution + 1):
        progress = i / resolution
        tick = beats_to_ticks(start_beat) + int(total_ticks * progress)
        value = int(start_value + value_range * progress)
        value = max(-8192, min(8191, value))  # Clamp to valid range
        events.append((tick, "pitchwheel", channel, value))

    return events


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo, time signature, and global CC setup."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="The Purge - State Violence", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    # C minor - using Eb major signature (3 flats), minor mode
    track.append(MetaMessage("key_signature", key="Eb", time=0))

    # Marker for sections
    track.append(MetaMessage("marker", text="A: Pursuit", time=0))
    track.append(MetaMessage("marker", text="B: Capture", time=beats_to_ticks(64)))
    track.append(
        MetaMessage("marker", text="C: Violence", time=beats_to_ticks(64))
    )  # Delta from previous

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS - 128)))
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 1: Brass Section - Violence (Program 61, Channel 0)
    Stabbing, aggressive fanfares. Short, sharp attacks like blows.
    """
    track = MidiTrack()
    track.name = "Brass - Violence"
    track.append(Message("program_change", program=61, channel=0, time=0))

    # Initialize expression CCs
    track.append(Message("control_change", control=CC_EXPRESSION, value=80, channel=0, time=0))
    track.append(Message("control_change", control=CC_CHORUS, value=0, channel=0, time=0))
    track.append(Message("control_change", control=CC_MODULATION, value=60, channel=0, time=0))
    track.append(Message("control_change", control=CC_RESONANCE, value=90, channel=0, time=0))

    notes = []
    cc_events = []
    pitch_events = []

    # Section A (beats 0-63): Pursuit - hunting fanfares
    pursuit_brass = [
        # Sharp staccato hunting calls
        (0, "C4", 0.5, 85),
        (0.5, "G4", 0.5, 88),
        (2, "C4", 0.5, 85),
        (2.5, "Eb4", 0.5, 82),
        (4, "G4", 0.75, 90),
        (5, "C5", 0.5, 92),
        (8, "C4", 0.5, 88),
        (8.5, "G4", 0.5, 90),
        (9, "C5", 0.75, 92),
        (12, "Eb4", 0.5, 85),
        (12.5, "G4", 0.5, 88),
        (13, "Bb4", 0.5, 90),
        (16, "C4", 0.5, 92),
        (16.5, "Eb4", 0.5, 90),
        (17, "G4", 0.75, 95),
        (20, "Ab4", 0.5, 88),
        (20.5, "G4", 0.5, 90),
        (21, "F4", 0.75, 85),
        (24, "C4", 0.5, 95),
        (24.5, "G4", 0.5, 98),
        (25, "C5", 1, 100),
        (28, "Bb4", 0.5, 92),
        (28.5, "Ab4", 0.5, 90),
        (29, "G4", 1, 95),
        (32, "C4", 0.5, 95),
        (32.5, "C4", 0.5, 98),
        (33, "G4", 0.75, 100),
        (34, "C5", 0.5, 100),
        (36, "Eb4", 0.5, 95),
        (36.5, "G4", 0.5, 98),
        (37, "C5", 0.75, 100),
        (40, "C4", 0.5, 100),
        (40.5, "Eb4", 0.5, 100),
        (41, "G4", 0.5, 102),
        (41.5, "C5", 0.75, 105),
        (44, "Bb4", 0.5, 98),
        (44.5, "G4", 0.5, 95),
        (45, "Eb4", 0.75, 92),
        (48, "C4", 0.5, 100),
        (48.5, "G4", 0.5, 102),
        (49, "C5", 1, 105),
        (52, "D5", 0.5, 100),
        (52.5, "Eb5", 0.5, 102),
        (53, "D5", 0.75, 100),
        (56, "C4", 0.5, 105),
        (56.5, "G4", 0.5, 105),
        (57, "C5", 0.5, 108),
        (57.5, "G5", 1, 110),
        (60, "F5", 0.5, 105),
        (60.5, "Eb5", 0.5, 102),
        (61, "D5", 0.5, 100),
        (61.5, "C5", 1, 105),
    ]

    for beat, note_name, duration, velocity in pursuit_brass:
        notes.append((note_name, beat, duration, velocity))

    # Section B (beats 64-127): Capture - closing in, stabbing chords
    capture_brass = [
        # Aggressive chord stabs - the net closes
        (64, "C4", 0.5, 108),
        (64, "Eb4", 0.5, 108),
        (64, "G4", 0.5, 108),
        (65, "C4", 0.5, 105),
        (65, "Eb4", 0.5, 105),
        (65, "G4", 0.5, 105),
        (66, "C4", 0.75, 110),
        (66, "Eb4", 0.75, 110),
        (66, "G4", 0.75, 110),
        (68, "Ab3", 0.5, 102),
        (68, "C4", 0.5, 102),
        (68, "Eb4", 0.5, 102),
        (70, "G3", 0.5, 105),
        (70, "C4", 0.5, 105),
        (70, "Eb4", 0.5, 105),
        (72, "C4", 0.5, 112),
        (72, "Eb4", 0.5, 112),
        (72, "G4", 0.5, 112),
        (72.5, "C5", 0.75, 115),
        (76, "Bb3", 0.5, 108),
        (76, "D4", 0.5, 108),
        (76, "F4", 0.5, 108),
        (78, "Ab3", 0.5, 105),
        (78, "C4", 0.5, 105),
        (78, "Eb4", 0.5, 105),
        (80, "G3", 0.5, 115),
        (80, "C4", 0.5, 115),
        (80, "Eb4", 0.5, 115),
        (80, "G4", 0.5, 115),
        (81, "G3", 0.5, 112),
        (81, "C4", 0.5, 112),
        (81, "Eb4", 0.5, 112),
        (82, "G3", 0.75, 118),
        (82, "C4", 0.75, 118),
        (82, "Eb4", 0.75, 118),
        (82, "G4", 0.75, 118),
        (84, "F3", 0.5, 110),
        (84, "Ab3", 0.5, 110),
        (84, "C4", 0.5, 110),
        (86, "Eb3", 0.5, 108),
        (86, "G3", 0.5, 108),
        (86, "Bb3", 0.5, 108),
        (88, "C4", 0.5, 118),
        (88, "Eb4", 0.5, 118),
        (88, "G4", 0.5, 118),
        (88, "C5", 0.5, 118),
        (89, "C4", 0.5, 115),
        (89, "Eb4", 0.5, 115),
        (89, "G4", 0.5, 115),
        (90, "C4", 1, 120),
        (90, "Eb4", 1, 120),
        (90, "G4", 1, 120),
        (90, "C5", 1, 120),
        # Cornered - rapid stabs
        (92, "C4", 0.25, 115),
        (92.5, "C4", 0.25, 118),
        (93, "C4", 0.25, 120),
        (93.5, "G4", 0.5, 122),
        (96, "C4", 0.5, 120),
        (96, "G4", 0.5, 120),
        (96, "C5", 0.5, 120),
        (97, "Eb4", 0.5, 118),
        (97, "G4", 0.5, 118),
        (97, "Bb4", 0.5, 118),
        (98, "C4", 0.75, 122),
        (98, "Eb4", 0.75, 122),
        (98, "G4", 0.75, 122),
        (98, "C5", 0.75, 122),
        (100, "Ab3", 0.5, 115),
        (100, "C4", 0.5, 115),
        (100, "Eb4", 0.5, 115),
        (102, "G3", 0.5, 118),
        (102, "C4", 0.5, 118),
        (102, "Eb4", 0.5, 118),
        (104, "C4", 0.5, 122),
        (104, "Eb4", 0.5, 122),
        (104, "G4", 0.5, 122),
        (104, "C5", 0.5, 122),
        (105, "C4", 0.5, 120),
        (105, "Eb4", 0.5, 120),
        (105, "G4", 0.5, 120),
        (106, "C4", 1, 125),
        (106, "Eb4", 1, 125),
        (106, "G4", 1, 125),
        (106, "C5", 1, 125),
        (108, "Bb3", 0.5, 118),
        (108, "D4", 0.5, 118),
        (108, "F4", 0.5, 118),
        (110, "Ab3", 0.5, 115),
        (110, "C4", 0.5, 115),
        (110, "Eb4", 0.5, 115),
        (112, "G3", 0.5, 122),
        (112, "C4", 0.5, 122),
        (112, "Eb4", 0.5, 122),
        (112, "G4", 0.5, 122),
        (113, "G3", 0.5, 120),
        (113, "C4", 0.5, 120),
        (113, "Eb4", 0.5, 120),
        (114, "G3", 1, 125),
        (114, "C4", 1, 125),
        (114, "Eb4", 1, 125),
        (114, "G4", 1, 125),
        (116, "C4", 0.5, 125),
        (116, "Eb4", 0.5, 125),
        (116, "G4", 0.5, 125),
        (116, "C5", 0.5, 125),
        (117, "D4", 0.5, 122),
        (117, "F4", 0.5, 122),
        (117, "Ab4", 0.5, 122),
        (118, "Eb4", 0.5, 125),
        (118, "G4", 0.5, 125),
        (118, "Bb4", 0.5, 125),
        (119, "C4", 0.5, 127),
        (119, "Eb4", 0.5, 127),
        (119, "G4", 0.5, 127),
        (119, "C5", 0.5, 127),
        # Final capture
        (120, "C4", 2, 127),
        (120, "Eb4", 2, 127),
        (120, "G4", 2, 127),
        (120, "C5", 2, 127),
        (124, "G4", 2, 125),
        (124, "C5", 2, 125),
        (124, "Eb5", 2, 125),
    ]

    for beat, note_name, duration, velocity in capture_brass:
        notes.append((note_name, beat, duration, velocity))

    # Section C (beats 128-184): Violence - maximum aggression
    violence_brass = [
        # Pure violence - relentless stabbing
        (128, "C4", 0.5, 127),
        (128, "Eb4", 0.5, 127),
        (128, "G4", 0.5, 127),
        (128, "C5", 0.5, 127),
        (129, "C4", 0.5, 125),
        (129, "Eb4", 0.5, 125),
        (129, "G4", 0.5, 125),
        (130, "C4", 0.5, 127),
        (130, "Eb4", 0.5, 127),
        (130, "G4", 0.5, 127),
        (130, "C5", 0.5, 127),
        (131, "Eb4", 0.5, 125),
        (131, "G4", 0.5, 125),
        (131, "Bb4", 0.5, 125),
        (132, "C4", 0.75, 127),
        (132, "Eb4", 0.75, 127),
        (132, "G4", 0.75, 127),
        (132, "C5", 0.75, 127),
        (134, "Ab3", 0.5, 122),
        (134, "C4", 0.5, 122),
        (134, "Eb4", 0.5, 122),
        (135, "G3", 0.5, 125),
        (135, "C4", 0.5, 125),
        (135, "Eb4", 0.5, 125),
        (136, "C4", 0.5, 127),
        (136, "Eb4", 0.5, 127),
        (136, "G4", 0.5, 127),
        (136, "C5", 0.5, 127),
        (137, "C4", 0.5, 127),
        (137, "Eb4", 0.5, 127),
        (137, "G4", 0.5, 127),
        (138, "C4", 0.5, 127),
        (138, "Eb4", 0.5, 127),
        (138, "G4", 0.5, 127),
        (138, "C5", 0.5, 127),
        (139, "D4", 0.5, 125),
        (139, "F4", 0.5, 125),
        (139, "Ab4", 0.5, 125),
        (140, "Eb4", 0.5, 127),
        (140, "G4", 0.5, 127),
        (140, "Bb4", 0.5, 127),
        (140, "Eb5", 0.5, 127),
        (141, "C4", 0.5, 127),
        (141, "Eb4", 0.5, 127),
        (141, "G4", 0.5, 127),
        (142, "C4", 0.5, 127),
        (142, "Eb4", 0.5, 127),
        (142, "G4", 0.5, 127),
        (142, "C5", 0.5, 127),
        (143, "C4", 0.5, 127),
        (143, "Eb4", 0.5, 127),
        (143, "G4", 0.5, 127),
        (144, "C4", 1, 127),
        (144, "Eb4", 1, 127),
        (144, "G4", 1, 127),
        (144, "C5", 1, 127),
        # Continuing violence
        (146, "Bb3", 0.5, 125),
        (146, "D4", 0.5, 125),
        (146, "F4", 0.5, 125),
        (147, "Ab3", 0.5, 122),
        (147, "C4", 0.5, 122),
        (147, "Eb4", 0.5, 122),
        (148, "G3", 0.5, 127),
        (148, "C4", 0.5, 127),
        (148, "Eb4", 0.5, 127),
        (148, "G4", 0.5, 127),
        (149, "G3", 0.5, 127),
        (149, "C4", 0.5, 127),
        (149, "Eb4", 0.5, 127),
        (150, "G3", 0.5, 127),
        (150, "C4", 0.5, 127),
        (150, "Eb4", 0.5, 127),
        (150, "G4", 0.5, 127),
        (151, "Ab3", 0.5, 125),
        (151, "C4", 0.5, 125),
        (151, "Eb4", 0.5, 125),
        (152, "C4", 1, 127),
        (152, "Eb4", 1, 127),
        (152, "G4", 1, 127),
        (152, "C5", 1, 127),
        (154, "D4", 0.5, 125),
        (154, "F4", 0.5, 125),
        (154, "Ab4", 0.5, 125),
        (155, "Eb4", 0.5, 127),
        (155, "G4", 0.5, 127),
        (155, "Bb4", 0.5, 127),
        (156, "C4", 0.5, 127),
        (156, "Eb4", 0.5, 127),
        (156, "G4", 0.5, 127),
        (156, "C5", 0.5, 127),
        (157, "C4", 0.5, 127),
        (157, "Eb4", 0.5, 127),
        (157, "G4", 0.5, 127),
        (158, "C4", 0.5, 127),
        (158, "Eb4", 0.5, 127),
        (158, "G4", 0.5, 127),
        (158, "C5", 0.5, 127),
        (159, "C4", 0.5, 127),
        (159, "Eb4", 0.5, 127),
        (159, "G4", 0.5, 127),
        (160, "C4", 2, 127),
        (160, "Eb4", 2, 127),
        (160, "G4", 2, 127),
        (160, "C5", 2, 127),
        # Final blows
        (164, "G4", 1, 127),
        (164, "C5", 1, 127),
        (164, "Eb5", 1, 127),
        (166, "F4", 0.5, 125),
        (166, "Ab4", 0.5, 125),
        (166, "C5", 0.5, 125),
        (167, "Eb4", 0.5, 127),
        (167, "G4", 0.5, 127),
        (167, "Bb4", 0.5, 127),
        (168, "C4", 0.5, 127),
        (168, "Eb4", 0.5, 127),
        (168, "G4", 0.5, 127),
        (168, "C5", 0.5, 127),
        (169, "C4", 0.5, 127),
        (169, "Eb4", 0.5, 127),
        (169, "G4", 0.5, 127),
        (170, "C4", 0.5, 127),
        (170, "Eb4", 0.5, 127),
        (170, "G4", 0.5, 127),
        (170, "C5", 0.5, 127),
        (171, "C4", 0.5, 127),
        (171, "Eb4", 0.5, 127),
        (171, "G4", 0.5, 127),
        (172, "C4", 2, 127),
        (172, "Eb4", 2, 127),
        (172, "G4", 2, 127),
        (172, "C5", 2, 127),
        (176, "C4", 2, 127),
        (176, "Eb4", 2, 127),
        (176, "G4", 2, 127),
        (176, "C5", 2, 127),
        (176, "G5", 2, 127),
        # The end - abrupt silence
        (180, "C4", 2, 127),
        (180, "Eb4", 2, 127),
        (180, "G4", 2, 127),
        (180, "C5", 2, 127),
    ]

    for beat, note_name, duration, velocity in violence_brass:
        notes.append((note_name, beat, duration, velocity))

    # CC Automation - Expression builds to maximum
    cc_events.extend(create_cc_automation(0, CC_EXPRESSION, 80, 100, 0, 64, 16))
    cc_events.extend(create_cc_automation(0, CC_EXPRESSION, 100, 120, 64, 128, 16))
    cc_events.extend(create_cc_automation(0, CC_EXPRESSION, 120, 127, 128, 184, 16))

    # Modulation - building frenzy
    cc_events.extend(create_cc_automation(0, CC_MODULATION, 60, 80, 0, 64, 12))
    cc_events.extend(create_cc_automation(0, CC_MODULATION, 80, 95, 64, 128, 12))
    cc_events.extend(create_cc_automation(0, CC_MODULATION, 95, 100, 128, 184, 12))

    # Resonance - increasing harshness
    cc_events.extend(create_cc_automation(0, CC_RESONANCE, 90, 110, 0, 64, 8))
    cc_events.extend(create_cc_automation(0, CC_RESONANCE, 110, 120, 64, 128, 8))
    cc_events.extend(create_cc_automation(0, CC_RESONANCE, 120, 127, 128, 184, 8))

    # Pitch bend - erratic violent sweeps during Violence section
    pitch_events.extend(create_pitch_bend_sweep(0, 0, 4096, 130, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, 4096, -2048, 134, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, -2048, 6000, 140, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, 6000, -4096, 146, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, -4096, 8000, 154, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, 8000, 0, 160, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, 0, -8000, 168, 2, 8))
    pitch_events.extend(create_pitch_bend_sweep(0, -8000, 0, 176, 2, 8))

    # Combine all events
    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity, 0))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0, 0))

    for tick, _channel, cc, value in cc_events:
        events.append((tick, "cc", cc, value, channel))

    for tick, msg_type, channel, value in pitch_events:
        events.append((tick, msg_type, channel, value, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off", x[1] == "cc"))

    last_time = 0
    for event in events:
        delta = event[0] - last_time
        if event[1] == "note_on":
            track.append(
                Message("note_on", note=event[2], velocity=event[3], channel=0, time=delta)
            )
        elif event[1] == "note_off":
            track.append(Message("note_off", note=event[2], velocity=0, channel=0, time=delta))
        elif event[1] == "cc":
            track.append(
                Message("control_change", control=event[2], value=event[3], channel=0, time=delta)
            )
        elif event[1] == "pitchwheel":
            track.append(Message("pitchwheel", pitch=event[3], channel=0, time=delta))
        last_time = event[0]

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 2: Tremolo Strings - The Hunt (Program 44, Channel 1)
    Relentless, vibrating pursuit. Never stops, never rests.
    """
    track = MidiTrack()
    track.name = "Strings - The Hunt"
    # Program 44 = Tremolo Strings
    track.append(Message("program_change", program=44, channel=1, time=0))

    # Initialize CCs
    track.append(Message("control_change", control=CC_EXPRESSION, value=80, channel=1, time=0))
    track.append(Message("control_change", control=CC_CHORUS, value=0, channel=1, time=0))
    track.append(Message("control_change", control=CC_DETUNE, value=50, channel=1, time=0))
    track.append(Message("control_change", control=CC_RESONANCE, value=90, channel=1, time=0))

    notes = []
    cc_events = []

    # Section A (beats 0-63): Pursuit - hunting tremolo
    # Sustained tremolo notes creating constant menace
    pursuit_strings = [
        # Low sustained tremolo - the hunt begins
        ("G3", 0, 8, 70),
        ("C4", 0, 8, 68),
        ("Eb4", 4, 6, 72),
        ("G3", 8, 8, 75),
        ("C4", 8, 8, 73),
        ("Eb4", 8, 8, 70),
        ("F3", 16, 6, 72),
        ("Ab3", 16, 6, 70),
        ("G3", 20, 4, 78),
        ("C4", 20, 4, 75),
        ("G3", 24, 8, 80),
        ("C4", 24, 8, 78),
        ("Eb4", 24, 8, 75),
        ("D4", 28, 4, 78),
        ("G3", 32, 8, 82),
        ("C4", 32, 8, 80),
        ("Eb4", 32, 8, 78),
        ("G4", 36, 6, 85),
        ("F3", 40, 4, 80),
        ("Ab3", 40, 4, 78),
        ("C4", 40, 4, 75),
        ("G3", 44, 4, 85),
        ("Bb3", 44, 4, 82),
        ("G3", 48, 8, 88),
        ("C4", 48, 8, 85),
        ("Eb4", 48, 8, 82),
        ("G4", 52, 6, 90),
        ("Ab3", 56, 4, 85),
        ("C4", 56, 4, 82),
        ("Eb4", 56, 4, 80),
        ("G3", 60, 4, 92),
        ("C4", 60, 4, 90),
        ("Eb4", 60, 4, 88),
    ]

    for note_name, beat, duration, velocity in pursuit_strings:
        notes.append((note_name, beat, duration, velocity))

    # Section B (beats 64-127): Capture - intensifying tremolo
    capture_strings = [
        ("G3", 64, 8, 95),
        ("C4", 64, 8, 92),
        ("Eb4", 64, 8, 90),
        ("G4", 64, 8, 88),
        ("Ab3", 72, 4, 90),
        ("C4", 72, 4, 88),
        ("Eb4", 72, 4, 85),
        ("G3", 76, 4, 95),
        ("Bb3", 76, 4, 92),
        ("D4", 76, 4, 90),
        ("G3", 80, 8, 100),
        ("C4", 80, 8, 98),
        ("Eb4", 80, 8, 95),
        ("G4", 80, 8, 92),
        ("F3", 88, 4, 95),
        ("Ab3", 88, 4, 92),
        ("C4", 88, 4, 90),
        ("G3", 92, 4, 100),
        ("C4", 92, 4, 98),
        ("Eb4", 92, 4, 95),
        ("G3", 96, 8, 105),
        ("C4", 96, 8, 102),
        ("Eb4", 96, 8, 100),
        ("G4", 96, 8, 98),
        ("Ab3", 104, 4, 100),
        ("C4", 104, 4, 98),
        ("Eb4", 104, 4, 95),
        ("G3", 108, 4, 105),
        ("Bb3", 108, 4, 102),
        ("D4", 108, 4, 100),
        ("G3", 112, 8, 108),
        ("C4", 112, 8, 105),
        ("Eb4", 112, 8, 102),
        ("G4", 112, 8, 100),
        ("F3", 120, 4, 105),
        ("Ab3", 120, 4, 102),
        ("C4", 120, 4, 100),
        ("G3", 124, 4, 110),
        ("C4", 124, 4, 108),
        ("Eb4", 124, 4, 105),
        ("G4", 124, 4, 102),
    ]

    for note_name, beat, duration, velocity in capture_strings:
        notes.append((note_name, beat, duration, velocity))

    # Section C (beats 128-184): Violence - maximum intensity tremolo
    violence_strings = [
        ("G3", 128, 8, 115),
        ("C4", 128, 8, 112),
        ("Eb4", 128, 8, 110),
        ("G4", 128, 8, 108),
        ("C5", 132, 6, 115),
        ("Ab3", 136, 4, 110),
        ("C4", 136, 4, 108),
        ("Eb4", 136, 4, 105),
        ("Ab4", 136, 4, 102),
        ("G3", 140, 4, 118),
        ("Bb3", 140, 4, 115),
        ("D4", 140, 4, 112),
        ("G4", 140, 4, 110),
        ("G3", 144, 8, 120),
        ("C4", 144, 8, 118),
        ("Eb4", 144, 8, 115),
        ("G4", 144, 8, 112),
        ("C5", 148, 6, 118),
        ("F3", 152, 4, 115),
        ("Ab3", 152, 4, 112),
        ("C4", 152, 4, 110),
        ("F4", 152, 4, 108),
        ("G3", 156, 4, 120),
        ("C4", 156, 4, 118),
        ("Eb4", 156, 4, 115),
        ("G4", 156, 4, 112),
        ("G3", 160, 8, 122),
        ("C4", 160, 8, 120),
        ("Eb4", 160, 8, 118),
        ("G4", 160, 8, 115),
        ("C5", 164, 6, 120),
        ("Ab3", 168, 4, 118),
        ("C4", 168, 4, 115),
        ("Eb4", 168, 4, 112),
        ("Ab4", 168, 4, 110),
        ("G3", 172, 4, 125),
        ("Bb3", 172, 4, 122),
        ("D4", 172, 4, 120),
        ("G4", 172, 4, 118),
        # Final sustained terror
        ("G3", 176, 8, 127),
        ("C4", 176, 8, 125),
        ("Eb4", 176, 8, 122),
        ("G4", 176, 8, 120),
        ("C5", 176, 8, 118),
    ]

    for note_name, beat, duration, velocity in violence_strings:
        notes.append((note_name, beat, duration, velocity))

    # CC Automation - Detune increases (victim fragmentation)
    cc_events.extend(create_cc_automation(1, CC_DETUNE, 50, 60, 0, 64, 16))
    cc_events.extend(create_cc_automation(1, CC_DETUNE, 60, 72, 64, 128, 16))
    cc_events.extend(create_cc_automation(1, CC_DETUNE, 72, 80, 128, 184, 16))

    # Expression
    cc_events.extend(create_cc_automation(1, CC_EXPRESSION, 80, 100, 0, 64, 12))
    cc_events.extend(create_cc_automation(1, CC_EXPRESSION, 100, 115, 64, 128, 12))
    cc_events.extend(create_cc_automation(1, CC_EXPRESSION, 115, 127, 128, 184, 12))

    # Resonance
    cc_events.extend(create_cc_automation(1, CC_RESONANCE, 90, 105, 0, 64, 8))
    cc_events.extend(create_cc_automation(1, CC_RESONANCE, 105, 118, 64, 128, 8))
    cc_events.extend(create_cc_automation(1, CC_RESONANCE, 118, 127, 128, 184, 8))

    # Combine events
    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0))

    for tick, _channel, cc, value in cc_events:
        events.append((tick, "cc", cc, value))

    events.sort(key=lambda x: (x[0], x[1] == "note_off", x[1] == "cc"))

    last_time = 0
    for event in events:
        delta = event[0] - last_time
        if event[1] == "note_on":
            track.append(
                Message("note_on", note=event[2], velocity=event[3], channel=1, time=delta)
            )
        elif event[1] == "note_off":
            track.append(Message("note_off", note=event[2], velocity=0, channel=1, time=delta))
        elif event[1] == "cc":
            track.append(
                Message("control_change", control=event[2], value=event[3], channel=1, time=delta)
            )
        last_time = event[0]

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_timpani_track() -> MidiTrack:
    """
    Track 3: Timpani - Jackboots (Program 47, Channel 2)
    Marching, crushing rhythm. The boots of state power.
    """
    track = MidiTrack()
    track.name = "Timpani - Jackboots"
    track.append(Message("program_change", program=47, channel=2, time=0))

    # Initialize CCs
    track.append(Message("control_change", control=CC_EXPRESSION, value=85, channel=2, time=0))
    track.append(Message("control_change", control=CC_CHORUS, value=0, channel=2, time=0))

    events = []
    cc_events = []

    # Section A (beats 0-63): Pursuit - marching timpani
    for beat in range(0, 64):
        beat_in_measure = beat % 4

        if beat_in_measure == 0:
            # Downbeat - heavy C
            velocity = min(75 + beat // 2, 100)
            events.append((beats_to_ticks(beat), "note_on", NOTES["C2"], velocity))
            events.append((beats_to_ticks(beat + 0.6), "note_off", NOTES["C2"], 0))
        elif beat_in_measure == 2:
            # Beat 3 - G
            velocity = min(70 + beat // 2, 95)
            events.append((beats_to_ticks(beat), "note_on", NOTES["G2"], velocity))
            events.append((beats_to_ticks(beat + 0.5), "note_off", NOTES["G2"], 0))

        # Add off-beat accents in later pursuit
        if beat >= 32 and beat_in_measure == 1:
            velocity = min(60 + beat // 3, 85)
            events.append((beats_to_ticks(beat), "note_on", NOTES["Eb2"], velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", NOTES["Eb2"], 0))

    # Section B (beats 64-127): Capture - intensifying march
    for beat in range(64, 128):
        beat_in_measure = beat % 4

        if beat_in_measure == 0:
            velocity = min(95 + (beat - 64) // 4, 115)
            events.append((beats_to_ticks(beat), "note_on", NOTES["C2"], velocity))
            events.append((beats_to_ticks(beat + 0.5), "note_off", NOTES["C2"], 0))
        elif beat_in_measure == 1:
            velocity = min(80 + (beat - 64) // 4, 100)
            events.append((beats_to_ticks(beat), "note_on", NOTES["Eb2"], velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", NOTES["Eb2"], 0))
        elif beat_in_measure == 2:
            velocity = min(90 + (beat - 64) // 4, 110)
            events.append((beats_to_ticks(beat), "note_on", NOTES["G2"], velocity))
            events.append((beats_to_ticks(beat + 0.5), "note_off", NOTES["G2"], 0))
        elif beat_in_measure == 3:
            velocity = min(75 + (beat - 64) // 4, 95)
            events.append((beats_to_ticks(beat), "note_on", NOTES["Bb1"], velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", NOTES["Bb1"], 0))

    # Section C (beats 128-184): Violence - crushing march
    for beat in range(128, 184):
        beat_in_measure = beat % 4

        # Every beat now - relentless crushing
        if beat_in_measure == 0:
            velocity = min(110 + (beat - 128) // 4, 127)
            events.append((beats_to_ticks(beat), "note_on", NOTES["C2"], velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", NOTES["C2"], 0))
            # Double hit
            events.append((beats_to_ticks(beat + 0.5), "note_on", NOTES["C1"], velocity - 10))
            events.append((beats_to_ticks(beat + 0.8), "note_off", NOTES["C1"], 0))
        elif beat_in_measure == 1:
            velocity = min(100 + (beat - 128) // 4, 120)
            events.append((beats_to_ticks(beat), "note_on", NOTES["Eb2"], velocity))
            events.append((beats_to_ticks(beat + 0.35), "note_off", NOTES["Eb2"], 0))
        elif beat_in_measure == 2:
            velocity = min(105 + (beat - 128) // 4, 125)
            events.append((beats_to_ticks(beat), "note_on", NOTES["G2"], velocity))
            events.append((beats_to_ticks(beat + 0.4), "note_off", NOTES["G2"], 0))
            # Double hit
            events.append((beats_to_ticks(beat + 0.5), "note_on", NOTES["G1"], velocity - 10))
            events.append((beats_to_ticks(beat + 0.8), "note_off", NOTES["G1"], 0))
        elif beat_in_measure == 3:
            velocity = min(95 + (beat - 128) // 4, 115)
            events.append((beats_to_ticks(beat), "note_on", NOTES["Bb1"], velocity))
            events.append((beats_to_ticks(beat + 0.35), "note_off", NOTES["Bb1"], 0))

    # CC Automation
    cc_events.extend(create_cc_automation(2, CC_EXPRESSION, 85, 100, 0, 64, 8))
    cc_events.extend(create_cc_automation(2, CC_EXPRESSION, 100, 118, 64, 128, 8))
    cc_events.extend(create_cc_automation(2, CC_EXPRESSION, 118, 127, 128, 184, 8))

    # Add CC events
    for tick, _channel, cc, value in cc_events:
        events.append((tick, "cc", cc, value))

    events.sort(key=lambda x: (x[0], x[1] == "note_off", x[1] == "cc"))

    last_time = 0
    for event in events:
        delta = event[0] - last_time
        if event[1] == "note_on":
            track.append(
                Message("note_on", note=event[2], velocity=event[3], channel=2, time=delta)
            )
        elif event[1] == "note_off":
            track.append(Message("note_off", note=event[2], velocity=0, channel=2, time=delta))
        elif event[1] == "cc":
            track.append(
                Message("control_change", control=event[2], value=event[3], channel=2, time=delta)
            )
        last_time = event[0]

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_snare_track() -> MidiTrack:
    """
    Track 4: Snare/Percussion - Execution (Channel 9, GM Percussion)
    Military precision in killing. Mechanical, inhuman.
    """
    track = MidiTrack()
    track.name = "Snare - Execution"
    # Channel 9 is GM percussion - no program change needed

    events = []
    cc_events = []

    # Initialize - NO CHORUS (isolation of victims)
    cc_events.append((0, 9, CC_CHORUS, 0))
    cc_events.append((0, 9, CC_EXPRESSION, 85))

    # Section A (beats 0-63): Pursuit - military snare pattern
    for beat in range(0, 64):
        beat_in_measure = beat % 4

        # Basic military pattern
        if beat_in_measure == 0:
            velocity = min(75 + beat, 100)
            events.append((beats_to_ticks(beat), "note_on", PERC_BASS_DRUM, velocity))
            events.append((beats_to_ticks(beat + 0.2), "note_off", PERC_BASS_DRUM, 0))
        elif beat_in_measure == 2:
            velocity = min(80 + beat, 105)
            events.append((beats_to_ticks(beat), "note_on", PERC_SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.25), "note_off", PERC_SNARE, 0))

        # 16th note hi-hat/ride pattern
        for sixteenth in range(4):
            tick = beats_to_ticks(beat + sixteenth * 0.25)
            velocity = 50 + (beat // 4) if sixteenth % 2 == 0 else 40 + (beat // 4)
            events.append((tick, "note_on", PERC_SIDE_STICK, min(velocity, 80)))
            events.append((tick + beats_to_ticks(0.1), "note_off", PERC_SIDE_STICK, 0))

    # Section B (beats 64-127): Capture - intensifying pattern
    for beat in range(64, 128):
        beat_in_measure = beat % 4

        # More aggressive pattern
        if beat_in_measure == 0:
            velocity = min(100 + (beat - 64) // 2, 120)
            events.append((beats_to_ticks(beat), "note_on", PERC_BASS_DRUM, velocity))
            events.append((beats_to_ticks(beat + 0.15), "note_off", PERC_BASS_DRUM, 0))
            # Double kick
            events.append((beats_to_ticks(beat + 0.5), "note_on", PERC_BASS_DRUM, velocity - 15))
            events.append((beats_to_ticks(beat + 0.65), "note_off", PERC_BASS_DRUM, 0))
        elif beat_in_measure == 1:
            velocity = min(85 + (beat - 64) // 2, 105)
            events.append((beats_to_ticks(beat), "note_on", PERC_SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.2), "note_off", PERC_SNARE, 0))
        elif beat_in_measure == 2:
            velocity = min(100 + (beat - 64) // 2, 118)
            events.append((beats_to_ticks(beat), "note_on", PERC_SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.25), "note_off", PERC_SNARE, 0))
            # Flam
            events.append((beats_to_ticks(beat + 0.5), "note_on", PERC_SNARE, velocity - 20))
            events.append((beats_to_ticks(beat + 0.65), "note_off", PERC_SNARE, 0))
        elif beat_in_measure == 3:
            velocity = min(90 + (beat - 64) // 2, 110)
            events.append((beats_to_ticks(beat), "note_on", PERC_SNARE, velocity))
            events.append((beats_to_ticks(beat + 0.2), "note_off", PERC_SNARE, 0))

        # Continuous 16ths on rim
        for sixteenth in range(4):
            tick = beats_to_ticks(beat + sixteenth * 0.25)
            velocity = min(65 + (beat - 64) // 3, 90)
            events.append((tick, "note_on", PERC_SIDE_STICK, velocity))
            events.append((tick + beats_to_ticks(0.08), "note_off", PERC_SIDE_STICK, 0))

    # Section C (beats 128-184): Violence - execution rhythm
    for beat in range(128, 184):
        beat_in_measure = beat % 4

        # Brutal, relentless pattern
        velocity_base = min(110 + (beat - 128) // 3, 127)

        # Snare on every beat - execution rhythm
        events.append((beats_to_ticks(beat), "note_on", PERC_SNARE, velocity_base))
        events.append((beats_to_ticks(beat + 0.2), "note_off", PERC_SNARE, 0))

        # 8th note snare doubles
        events.append((beats_to_ticks(beat + 0.5), "note_on", PERC_SNARE, velocity_base - 15))
        events.append((beats_to_ticks(beat + 0.65), "note_off", PERC_SNARE, 0))

        # Bass drum pattern
        if beat_in_measure == 0 or beat_in_measure == 2:
            events.append((beats_to_ticks(beat), "note_on", PERC_BASS_DRUM, velocity_base))
            events.append((beats_to_ticks(beat + 0.15), "note_off", PERC_BASS_DRUM, 0))
            events.append(
                (beats_to_ticks(beat + 0.25), "note_on", PERC_BASS_DRUM, velocity_base - 10)
            )
            events.append((beats_to_ticks(beat + 0.4), "note_off", PERC_BASS_DRUM, 0))

        # Crash accents on strong beats in violence section
        if beat % 8 == 0 and beat >= 128:
            events.append((beats_to_ticks(beat), "note_on", PERC_CRASH, min(velocity_base, 120)))
            events.append((beats_to_ticks(beat + 1), "note_off", PERC_CRASH, 0))

        # Tom fills every 4 bars
        if beat % 16 == 15:
            # Tom fill leading into next phrase
            events.append((beats_to_ticks(beat), "note_on", PERC_TOM_HIGH, velocity_base - 5))
            events.append((beats_to_ticks(beat + 0.15), "note_off", PERC_TOM_HIGH, 0))
            events.append((beats_to_ticks(beat + 0.25), "note_on", PERC_TOM_MID, velocity_base - 5))
            events.append((beats_to_ticks(beat + 0.4), "note_off", PERC_TOM_MID, 0))
            events.append((beats_to_ticks(beat + 0.5), "note_on", PERC_TOM_LOW, velocity_base))
            events.append((beats_to_ticks(beat + 0.75), "note_off", PERC_TOM_LOW, 0))

    # CC Automation
    cc_events.extend(create_cc_automation(9, CC_EXPRESSION, 85, 105, 0, 64, 8))
    cc_events.extend(create_cc_automation(9, CC_EXPRESSION, 105, 120, 64, 128, 8))
    cc_events.extend(create_cc_automation(9, CC_EXPRESSION, 120, 127, 128, 184, 8))

    # Add CC events
    for tick, _channel, cc, value in cc_events:
        events.append((tick, "cc", cc, value))

    events.sort(key=lambda x: (x[0], x[1] == "note_off", x[1] == "cc"))

    last_time = 0
    for event in events:
        delta = event[0] - last_time
        if event[1] == "note_on":
            track.append(
                Message("note_on", note=event[2], velocity=event[3], channel=9, time=delta)
            )
        elif event[1] == "note_off":
            track.append(Message("note_off", note=event[2], velocity=0, channel=9, time=delta))
        elif event[1] == "cc":
            track.append(
                Message("control_change", control=event[2], value=event[3], channel=9, time=delta)
            )
        last_time = event[0]

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_tuba_track() -> MidiTrack:
    """
    Track 5: Tuba - Doom (Program 58, Channel 4)
    The bass voice of inescapable fate. Deep, crushing, final.
    """
    track = MidiTrack()
    track.name = "Low Brass - Doom"
    track.append(Message("program_change", program=58, channel=4, time=0))

    # Initialize CCs
    track.append(Message("control_change", control=CC_EXPRESSION, value=80, channel=4, time=0))
    track.append(Message("control_change", control=CC_CHORUS, value=0, channel=4, time=0))
    track.append(Message("control_change", control=CC_MODULATION, value=60, channel=4, time=0))

    notes = []
    cc_events = []
    pitch_events = []

    # Section A (beats 0-63): Pursuit - ominous bass
    doom_pursuit = [
        ("C2", 0, 4, 70),
        ("G1", 4, 4, 68),
        ("C2", 8, 4, 72),
        ("Eb2", 12, 2, 70),
        ("D2", 14, 2, 68),
        ("C2", 16, 4, 75),
        ("G1", 20, 4, 72),
        ("Ab1", 24, 4, 70),
        ("G1", 28, 4, 75),
        ("C2", 32, 4, 78),
        ("G1", 36, 4, 75),
        ("C2", 40, 4, 80),
        ("Bb1", 44, 2, 78),
        ("Ab1", 46, 2, 75),
        ("G1", 48, 4, 82),
        ("C2", 52, 4, 80),
        ("F1", 56, 4, 78),
        ("G1", 60, 4, 85),
    ]

    for note_name, beat, duration, velocity in doom_pursuit:
        notes.append((note_name, beat, duration, velocity))

    # Section B (beats 64-127): Capture - closing doom
    doom_capture = [
        ("C2", 64, 4, 90),
        ("G2", 64, 4, 88),
        ("G1", 68, 4, 88),
        ("C2", 68, 4, 85),
        ("Ab1", 72, 2, 85),
        ("G1", 74, 2, 88),
        ("C2", 76, 4, 92),
        ("C2", 80, 4, 95),
        ("G2", 80, 4, 92),
        ("Eb2", 84, 2, 90),
        ("D2", 86, 2, 88),
        ("C2", 88, 4, 95),
        ("G1", 92, 4, 92),
        ("C2", 92, 4, 90),
        ("C2", 96, 4, 100),
        ("G2", 96, 4, 98),
        ("F1", 100, 2, 95),
        ("G1", 102, 2, 98),
        ("Ab1", 104, 4, 100),
        ("G1", 108, 4, 102),
        ("C2", 108, 4, 100),
        ("C2", 112, 4, 105),
        ("G2", 112, 4, 102),
        ("Bb1", 116, 2, 100),
        ("Ab1", 118, 2, 102),
        ("G1", 120, 4, 108),
        ("C2", 124, 4, 110),
        ("G2", 124, 4, 108),
    ]

    for note_name, beat, duration, velocity in doom_capture:
        notes.append((note_name, beat, duration, velocity))

    # Section C (beats 128-184): Violence - crushing doom
    doom_violence = [
        ("C2", 128, 2, 115),
        ("G2", 128, 2, 112),
        ("C2", 130, 2, 118),
        ("Eb2", 130, 2, 115),
        ("C2", 132, 4, 120),
        ("G2", 132, 4, 118),
        ("C3", 132, 4, 115),
        ("Ab1", 136, 2, 115),
        ("G1", 138, 2, 118),
        ("C2", 140, 4, 122),
        ("G2", 140, 4, 120),
        ("C2", 144, 2, 125),
        ("G2", 144, 2, 122),
        ("C2", 146, 2, 125),
        ("Eb2", 146, 2, 122),
        ("C2", 148, 4, 127),
        ("G2", 148, 4, 125),
        ("C3", 148, 4, 122),
        ("F1", 152, 2, 120),
        ("G1", 154, 2, 122),
        ("Ab1", 156, 2, 125),
        ("G1", 158, 2, 127),
        ("C2", 160, 4, 127),
        ("G2", 160, 4, 125),
        ("C3", 160, 4, 122),
        ("C2", 164, 2, 127),
        ("Eb2", 164, 2, 125),
        ("C2", 166, 2, 127),
        ("G2", 166, 2, 125),
        ("C2", 168, 4, 127),
        ("G2", 168, 4, 127),
        ("C3", 168, 4, 125),
        ("Bb1", 172, 2, 125),
        ("Ab1", 174, 2, 127),
        ("G1", 176, 4, 127),
        ("C2", 176, 4, 127),
        ("G2", 176, 4, 125),
        # Final doom
        ("C1", 180, 4, 127),
        ("C2", 180, 4, 127),
        ("G2", 180, 4, 125),
    ]

    for note_name, beat, duration, velocity in doom_violence:
        notes.append((note_name, beat, duration, velocity))

    # CC Automation
    cc_events.extend(create_cc_automation(4, CC_EXPRESSION, 80, 100, 0, 64, 12))
    cc_events.extend(create_cc_automation(4, CC_EXPRESSION, 100, 118, 64, 128, 12))
    cc_events.extend(create_cc_automation(4, CC_EXPRESSION, 118, 127, 128, 184, 12))

    cc_events.extend(create_cc_automation(4, CC_MODULATION, 60, 80, 0, 64, 8))
    cc_events.extend(create_cc_automation(4, CC_MODULATION, 80, 95, 64, 128, 8))
    cc_events.extend(create_cc_automation(4, CC_MODULATION, 95, 100, 128, 184, 8))

    # Pitch bend - ominous drops during violence
    pitch_events.extend(create_pitch_bend_sweep(4, 0, -4096, 132, 2, 6))
    pitch_events.extend(create_pitch_bend_sweep(4, -4096, 0, 136, 2, 6))
    pitch_events.extend(create_pitch_bend_sweep(4, 0, -6000, 148, 2, 6))
    pitch_events.extend(create_pitch_bend_sweep(4, -6000, 0, 152, 2, 6))
    pitch_events.extend(create_pitch_bend_sweep(4, 0, -8000, 168, 2, 6))
    pitch_events.extend(create_pitch_bend_sweep(4, -8000, 0, 172, 2, 6))

    # Combine events
    events = []
    for note_name, start, duration, velocity in notes:
        note = NOTES[note_name]
        events.append((beats_to_ticks(start), "note_on", note, velocity, 4))
        events.append((beats_to_ticks(start + duration), "note_off", note, 0, 4))

    for tick, _channel, cc, value in cc_events:
        events.append((tick, "cc", cc, value, channel))

    for tick, msg_type, channel, value in pitch_events:
        events.append((tick, msg_type, channel, value, 4))

    events.sort(key=lambda x: (x[0], x[1] == "note_off", x[1] == "cc"))

    last_time = 0
    for event in events:
        delta = event[0] - last_time
        if event[1] == "note_on":
            track.append(
                Message("note_on", note=event[2], velocity=event[3], channel=4, time=delta)
            )
        elif event[1] == "note_off":
            track.append(Message("note_off", note=event[2], velocity=0, channel=4, time=delta))
        elif event[1] == "cc":
            track.append(
                Message("control_change", control=event[2], value=event[3], channel=4, time=delta)
            )
        elif event[1] == "pitchwheel":
            track.append(Message("pitchwheel", pitch=event[3], channel=4, time=delta))
        last_time = event[0]

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'The Purge'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_snare_track())
    mid.tracks.append(create_tuba_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/fascist/05_the_purge.mid"

    print("Creating 'The Purge' - Fascist Suite 05")
    print("=" * 60)
    print("WARNING: This is the DARKEST track in the fascist suite.")
    print("It represents state violence at its most horrific peak.")
    print()
    print("Concept: The scapegoat mechanism reaches its violent conclusion")
    print("         - pursuit, capture, and execution of 'the other'")
    print()

    mid = create_midi_file()
    mid.save(output_path)

    print(f"Saved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Tempo: {BPM} BPM (aggressive, relentless)")
    print("Key: C minor (brutal, no escape)")
    print(f"Track count: {len(mid.tracks)}")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    length = mid.length
    print(f"\nDuration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\nEXPRESSION AUTOMATION:")
    print("  CC11 (Expression): 80 -> 127 (building to maximum aggression)")
    print("  CC93 (Chorus): 0 (ZERO - no solidarity for victims)")
    print("  CC94 (Detune): 50 -> 80 (increasing chaos, victim fragmentation)")
    print("  CC1 (Modulation): 60 -> 100 (building frenzy, bloodlust)")
    print("  CC71 (Resonance): 90 -> 127 (maximum harshness)")
    print("  Pitch bend: Erratic violent sweeps during Violence section")

    print("\nMUSICAL ARC:")
    print("  A. Pursuit (beats 0-63): The hunt begins")
    print("  B. Capture (beats 64-127): The net closes, no escape")
    print("  C. Violence (beats 128-184): The killing - merciless, mechanical")

    print("\nINTEGRATION GUIDANCE:")
    print("  - Trigger when: Fascist faction initiates pogrom/purge event")
    print("  - Trigger when: Ethnic cleansing mechanic activates")
    print("  - Trigger when: Maximum repression + scapegoat target selected")
    print("  - DO NOT LOOP: This track should play once and END")
    print("  - Follow with silence or ambient dread track")
    print()
    print("THEMATIC NOTES:")
    print("  - This is the logical conclusion of the fascist arc")
    print("  - Scapegoating (02) -> Rally (03) -> Purge (05)")
    print("  - The absence of chorus (solidarity) is critical")
    print("  - The increasing detune represents victim fragmentation")
    print("  - The track does NOT resolve - violence leaves only silence")


if __name__ == "__main__":
    main()
