#!/usr/bin/env python3
"""
BABYLON - Fascist Suite
03_the_rally.mid - "The Rally"

CONCEPTUAL BRIEF:
This piece represents false solidarity - the mob energy of national identity
replacing class consciousness. Where revolutionary music features HARMONY
(different voices together), this track enforces UNISON (conformity). It is
a DARK MIRROR of revolutionary solidarity: same energy, same march tempo,
but twisted into something menacing. The crowd moves as one, but they move
toward darkness.

TECHNICAL SPECIFICATION:
- Key: E minor (triumphant but twisted - parallel to revolutionary tracks)
- Tempo: 115 BPM (marching, relentless, wrong)
- Time Signature: 4/4
- Duration: ~100 seconds (192 beats at 115 BPM)
- Loop Points: Beat 32 (after assembly) for looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Brass Section (Program 61) - "False Triumph" - Bold but hollow fanfares
- Channel 1: Orchestral Hit / Snare (Program 115) - "The March" - Mechanical, regimented
- Channel 2: Timpani (Program 47) - "Blood and Soil" - Heavy, earthbound thuds
- Channel 3: String Ensemble (Program 48) - "The Mob" - All in UNISON (conformity!)
- Channel 4: Church Organ (Program 19) - "Twisted Grandeur" - False sanctity

MUSICAL ARC (100 seconds = 192 beats at 115 BPM):
A. Assembly (beats 0-31): Timpani pulse, scattered brass calls gather the mob
B. Chanting (beats 32-95): Strings in UNISON with organ drone, building intensity
C. False Unity (beats 96-159): Full ensemble, triumphant but hollow, relentless march
D. Crescendo (beats 160-192): Overwhelming force, all voices merged into ONE

COMPOSITIONAL NOTES:
- E minor maintains thematic consistency with revolutionary tracks
- Unlike revolutionary harmony (voices in counterpoint), this uses parallel motion
- The organ provides false religious sanctity to secular nationalism
- Timpani represents "blood and soil" - heavy, earthbound, primitive
- Snare march is mechanical, inhuman, industrial
- Strings play in UNISON - no individual voice, only the collective mob
- The piece never resolves harmonically - it just ENDS, suggesting the void beneath

DARK MIRROR PRINCIPLE:
Where "The Internationale" (revolutionary) would have:
  - Multiple voices in harmony (class solidarity across difference)
  - Rising hope, building toward liberation
  - Organic rhythmic variation

This track has:
  - All voices in unison (conformity, erasure of difference)
  - Rising menace, building toward domination
  - Mechanical, lockstep precision
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480
BPM = 115
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 192  # ~100 seconds at 115 BPM

# Note definitions (MIDI note numbers)
NOTES = {
    # Low register (timpani, bass)
    "E1": 28,
    "B1": 35,
    "E2": 40,
    "F#2": 42,
    "G2": 43,
    "A2": 45,
    "B2": 47,
    # Mid register
    "C3": 48,
    "D3": 50,
    "E3": 52,
    "F#3": 54,
    "G3": 55,
    "A3": 57,
    "B3": 59,
    # Upper register
    "C4": 60,
    "D4": 62,
    "E4": 64,
    "F#4": 66,
    "G4": 67,
    "A4": 69,
    "B4": 71,
    # High register
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
    track.append(MetaMessage("track_name", name="The Rally - False Solidarity", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="Em", time=0))
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_brass_track() -> MidiTrack:
    """
    Track 1: Brass Section - False Triumph (Program 61, Channel 0)
    Bold fanfares that gather the mob, hollow triumphalism.
    """
    track = MidiTrack()
    track.name = "Brass - False Triumph"
    track.append(Message("program_change", program=61, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes = [
        # Section A (beats 0-31): Assembly - scattered calls
        ("E3", 0, 2, 70),
        ("E4", 0, 2, 70),  # Octave doubling - hollow
        ("B3", 4, 1.5, 65),
        ("B4", 4, 1.5, 65),
        ("E3", 8, 3, 72),
        ("E4", 8, 3, 72),
        ("G3", 12, 2, 68),
        ("G4", 12, 2, 68),
        ("F#3", 16, 2, 70),
        ("F#4", 16, 2, 70),
        ("E3", 20, 4, 75),
        ("E4", 20, 4, 75),
        ("B3", 24, 2, 72),
        ("B4", 24, 2, 72),
        ("E3", 28, 4, 78),
        ("E4", 28, 4, 78),
        # Section B (beats 32-95): Chanting - punctuating the mob
        ("E3", 32, 1, 80),
        ("E4", 32, 1, 80),
        ("E3", 36, 1, 80),
        ("E4", 36, 1, 80),
        ("B3", 40, 2, 78),
        ("B4", 40, 2, 78),
        ("E3", 48, 1, 82),
        ("E4", 48, 1, 82),
        ("E3", 52, 1, 82),
        ("E4", 52, 1, 82),
        ("G3", 56, 2, 80),
        ("G4", 56, 2, 80),
        ("E3", 64, 1, 85),
        ("E4", 64, 1, 85),
        ("F#3", 68, 1, 83),
        ("F#4", 68, 1, 83),
        ("G3", 72, 2, 85),
        ("G4", 72, 2, 85),
        ("E3", 80, 1, 88),
        ("E4", 80, 1, 88),
        ("B3", 84, 2, 85),
        ("B4", 84, 2, 85),
        ("E3", 88, 4, 88),
        ("E4", 88, 4, 88),
        # Section C (beats 96-159): False Unity - relentless march
        ("E3", 96, 2, 90),
        ("E4", 96, 2, 90),
        ("B3", 100, 2, 88),
        ("B4", 100, 2, 88),
        ("E3", 104, 2, 90),
        ("E4", 104, 2, 90),
        ("G3", 108, 2, 88),
        ("G4", 108, 2, 88),
        ("E3", 112, 2, 92),
        ("E4", 112, 2, 92),
        ("F#3", 116, 2, 90),
        ("F#4", 116, 2, 90),
        ("E3", 120, 4, 92),
        ("E4", 120, 4, 92),
        ("E3", 128, 2, 94),
        ("E4", 128, 2, 94),
        ("B3", 132, 2, 92),
        ("B4", 132, 2, 92),
        ("E3", 136, 2, 94),
        ("E4", 136, 2, 94),
        ("G3", 140, 2, 92),
        ("G4", 140, 2, 92),
        ("E3", 144, 4, 95),
        ("E4", 144, 4, 95),
        ("F#3", 150, 2, 93),
        ("F#4", 150, 2, 93),
        ("E3", 154, 6, 95),
        ("E4", 154, 6, 95),
        # Section D (beats 160-192): Crescendo - overwhelming
        ("E3", 160, 4, 100),
        ("E4", 160, 4, 100),
        ("E5", 160, 4, 100),  # Triple octave
        ("B3", 166, 2, 98),
        ("B4", 166, 2, 98),
        ("E3", 170, 2, 100),
        ("E4", 170, 2, 100),
        ("E5", 170, 2, 100),
        ("G3", 174, 2, 98),
        ("G4", 174, 2, 98),
        ("E3", 178, 4, 105),
        ("E4", 178, 4, 105),
        ("E5", 178, 4, 105),
        ("B3", 184, 2, 100),
        ("B4", 184, 2, 100),
        ("E3", 188, 4, 110),  # Final blast
        ("E4", 188, 4, 110),
        ("E5", 188, 4, 110),
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


def create_snare_track() -> MidiTrack:
    """
    Track 2: Snare - The March (Program 115, Channel 1)
    Mechanical, regimented percussion. Inhuman precision.
    Note: Using Woodblock (115) for sharp, mechanical quality.
    """
    track = MidiTrack()
    track.name = "Snare - The March"
    track.append(Message("program_change", program=115, channel=1, time=0))

    events = []
    snare_note = NOTES["E4"]  # Pitched percussion

    # Section A (beats 0-31): Assembly - sparse, gathering
    for beat in range(0, 32, 4):
        velocity = 55 + (beat // 4) * 2
        events.append((beats_to_ticks(beat), "note_on", snare_note, velocity))
        events.append((beats_to_ticks(beat + 0.25), "note_off", snare_note, 0))

    # Section B (beats 32-95): Chanting - every other beat
    for beat in range(32, 96, 2):
        velocity = 70 + ((beat - 32) // 8)
        events.append((beats_to_ticks(beat), "note_on", snare_note, velocity))
        events.append((beats_to_ticks(beat + 0.2), "note_off", snare_note, 0))

    # Section C (beats 96-159): False Unity - every beat (lockstep)
    for beat in range(96, 160):
        base_velocity = 80
        # Accent on beats 1 and 3
        accent = 10 if beat % 4 in (0, 2) else 0
        velocity = base_velocity + accent
        events.append((beats_to_ticks(beat), "note_on", snare_note, velocity))
        events.append((beats_to_ticks(beat + 0.15), "note_off", snare_note, 0))

    # Section D (beats 160-192): Crescendo - double-time, overwhelming
    for half_beat in range(160 * 2, 192 * 2):
        beat = half_beat / 2.0
        base_velocity = 85
        accent = 15 if half_beat % 4 == 0 else 5 if half_beat % 2 == 0 else 0
        velocity = min(base_velocity + accent + (half_beat - 320) // 4, 120)
        events.append((beats_to_ticks(beat), "note_on", snare_note, velocity))
        events.append((beats_to_ticks(beat + 0.1), "note_off", snare_note, 0))

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
    Track 3: Timpani - Blood and Soil (Program 47, Channel 2)
    Heavy, earthbound thuds. The primitive beneath the pageantry.
    """
    track = MidiTrack()
    track.name = "Timpani - Blood and Soil"
    track.append(Message("program_change", program=47, channel=2, time=0))

    events = []

    # Section A (beats 0-31): Assembly - slow, ominous heartbeat
    for beat in range(0, 32, 2):
        note = NOTES["E2"] if beat % 4 == 0 else NOTES["B1"]
        velocity = 50 + beat
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.8), "note_off", note, 0))

    # Section B (beats 32-95): Chanting - steady pulse
    for beat in range(32, 96):
        if beat % 4 == 0:
            note = NOTES["E2"]
            velocity = 75
        elif beat % 4 == 2:
            note = NOTES["B1"]
            velocity = 70
        else:
            continue

        velocity = velocity + ((beat - 32) // 8)
        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.6), "note_off", note, 0))

    # Section C (beats 96-159): False Unity - every beat, driving
    for beat in range(96, 160):
        if beat % 2 == 0:
            note = NOTES["E2"]
            velocity = 85
        else:
            note = NOTES["B1"]
            velocity = 80

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.5), "note_off", note, 0))

    # Section D (beats 160-192): Crescendo - thunderous, primitive
    for beat in range(160, 192):
        # E2 and B1 alternating rapidly
        note = NOTES["E2"] if beat % 2 == 0 else NOTES["B1"]
        velocity = min(90 + (beat - 160), 115)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.4), "note_off", note, 0))

        # Add sub-beat for intensity
        if beat >= 176:
            sub_note = NOTES["E1"]
            events.append((beats_to_ticks(beat + 0.5), "note_on", sub_note, velocity - 10))
            events.append((beats_to_ticks(beat + 0.7), "note_off", sub_note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=2, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_strings_track() -> MidiTrack:
    """
    Track 4: Strings - The Mob (Program 48, Channel 3)
    All in UNISON - no harmony, only conformity.
    This is the dark mirror of revolutionary polyphony.
    """
    track = MidiTrack()
    track.name = "Strings - The Mob (Unison)"
    track.append(Message("program_change", program=48, channel=3, time=0))

    # All strings play in UNISON - multiple octaves but same pitch class
    # This represents conformity, the erasure of individual voice

    notes = [
        # Section B (beats 32-95): Chanting melody - UNISON across octaves
        # First phrase
        ("E3", 32, 8, 65),
        ("E4", 32, 8, 65),
        ("E5", 32, 8, 55),  # Higher octave, softer
        ("F#3", 40, 4, 68),
        ("F#4", 40, 4, 68),
        ("F#5", 40, 4, 58),
        ("G3", 44, 4, 70),
        ("G4", 44, 4, 70),
        ("G5", 44, 4, 60),
        ("E3", 48, 8, 72),
        ("E4", 48, 8, 72),
        ("E5", 48, 8, 62),
        # Second phrase (repeat with variation)
        ("E3", 56, 4, 70),
        ("E4", 56, 4, 70),
        ("B3", 60, 4, 72),
        ("B4", 60, 4, 72),
        ("A3", 64, 4, 68),
        ("A4", 64, 4, 68),
        ("G3", 68, 4, 70),
        ("G4", 68, 4, 70),
        ("F#3", 72, 4, 72),
        ("F#4", 72, 4, 72),
        ("E3", 76, 8, 75),
        ("E4", 76, 8, 75),
        # Building
        ("E3", 84, 4, 78),
        ("E4", 84, 4, 78),
        ("E5", 84, 4, 68),
        ("G3", 88, 4, 80),
        ("G4", 88, 4, 80),
        ("G5", 88, 4, 70),
        ("B3", 92, 4, 82),
        ("B4", 92, 4, 82),
        # Section C (beats 96-159): False Unity - full unison power
        ("E3", 96, 8, 85),
        ("E4", 96, 8, 85),
        ("E5", 96, 8, 75),
        ("E3", 104, 4, 82),
        ("E4", 104, 4, 82),
        ("F#3", 108, 4, 84),
        ("F#4", 108, 4, 84),
        ("G3", 112, 8, 86),
        ("G4", 112, 8, 86),
        ("G5", 112, 8, 76),
        ("F#3", 120, 4, 84),
        ("F#4", 120, 4, 84),
        ("E3", 124, 4, 88),
        ("E4", 124, 4, 88),
        ("E5", 124, 4, 78),
        # March continues
        ("E3", 128, 8, 88),
        ("E4", 128, 8, 88),
        ("E5", 128, 8, 78),
        ("B3", 136, 4, 86),
        ("B4", 136, 4, 86),
        ("A3", 140, 4, 84),
        ("A4", 140, 4, 84),
        ("G3", 144, 4, 86),
        ("G4", 144, 4, 86),
        ("F#3", 148, 4, 88),
        ("F#4", 148, 4, 88),
        ("E3", 152, 8, 90),
        ("E4", 152, 8, 90),
        ("E5", 152, 8, 80),
        # Section D (beats 160-192): Crescendo - overwhelming unison
        ("E3", 160, 8, 95),
        ("E4", 160, 8, 95),
        ("E5", 160, 8, 85),
        ("G3", 168, 4, 98),
        ("G4", 168, 4, 98),
        ("G5", 168, 4, 88),
        ("B3", 172, 4, 100),
        ("B4", 172, 4, 100),
        ("E3", 176, 8, 102),
        ("E4", 176, 8, 102),
        ("E5", 176, 8, 92),
        ("E3", 184, 8, 110),  # Final sustained
        ("E4", 184, 8, 110),
        ("E5", 184, 8, 100),
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


def create_organ_track() -> MidiTrack:
    """
    Track 5: Church Organ - Twisted Grandeur (Program 19, Channel 4)
    False religious sanctity applied to secular nationalism.
    Sustained drones that provide hollow spiritual weight.
    """
    track = MidiTrack()
    track.name = "Organ - Twisted Grandeur"
    track.append(Message("program_change", program=19, channel=4, time=0))

    # Long sustained drones - false sanctity
    notes = [
        # Section A (beats 0-31): Assembly - ominous drone builds
        ("E2", 0, 16, 40),
        ("B2", 0, 16, 38),
        ("E2", 16, 16, 50),
        ("B2", 16, 16, 48),
        ("E3", 24, 8, 45),
        # Section B (beats 32-95): Chanting - sustained bed
        ("E2", 32, 32, 55),
        ("B2", 32, 32, 53),
        ("E3", 32, 32, 50),
        ("E2", 64, 32, 60),
        ("B2", 64, 32, 58),
        ("E3", 64, 32, 55),
        ("G3", 80, 16, 52),
        # Section C (beats 96-159): False Unity - full organ power
        ("E2", 96, 32, 70),
        ("B2", 96, 32, 68),
        ("E3", 96, 32, 65),
        ("G3", 96, 32, 60),
        ("E2", 128, 32, 75),
        ("B2", 128, 32, 73),
        ("E3", 128, 32, 70),
        ("G3", 128, 32, 65),
        ("B3", 144, 16, 68),
        # Section D (beats 160-192): Crescendo - overwhelming
        ("E2", 160, 32, 85),
        ("B2", 160, 32, 83),
        ("E3", 160, 32, 80),
        ("G3", 160, 32, 75),
        ("B3", 160, 32, 72),
        ("E4", 176, 16, 78),  # High note for final climax
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
    """Create the complete MIDI file for 'The Rally'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_brass_track())
    mid.tracks.append(create_snare_track())
    mid.tracks.append(create_timpani_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_organ_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = "/home/user/projects/game/babylon/assets/music/fascist/03_the_rally.mid"

    print("Creating 'The Rally' - Fascist Suite 03")
    print("=" * 50)
    print("Concept: False solidarity - mob energy replacing class consciousness")
    print("Dark Mirror: Unison (conformity) instead of Harmony (solidarity)")

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

    print("\nComposition complete.")
    print("Musical arc: Assembly -> Chanting -> False Unity -> Crescendo")
    print("Key: E minor (twisted triumph)")
    print("\nThematic notes:")
    print("  - All strings in UNISON = conformity, erasure of individual voice")
    print("  - Hollow octave doubling in brass = false triumph")
    print("  - Organ drone = false religious sanctity")
    print("  - Mechanical snare = inhuman precision")
    print("  - Heavy timpani = 'blood and soil' primitivism")


if __name__ == "__main__":
    main()
