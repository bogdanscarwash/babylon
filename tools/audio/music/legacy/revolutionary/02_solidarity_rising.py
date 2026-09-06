#!/usr/bin/env python3
"""
BABYLON - Revolutionary Suite
02_solidarity_rising.mid - "Solidarity Rising"

CONCEPTUAL BRIEF:
This piece represents the forging of bonds between workers - the moment when
isolated individuals recognize their shared struggle and begin to act as one.
The call-and-response between cello and violin embodies the dialectical process
of consciousness spreading: one voice speaks, another answers, and through this
dialogue emerges collective understanding. The piece builds from tentative
exchanges to a thunderous unified chorus.

TECHNICAL SPECIFICATION:
- Key: A minor -> C major (relative major resolution)
- Tempo: 105 BPM (building momentum)
- Time Signature: 4/4
- Duration: ~100 seconds (175 beats at 105 BPM)
- Loop Points: Beat 32 (post-intro) for looping

INSTRUMENT ASSIGNMENTS:
- Channel 0: Cello (Program 42) - "The Masses" - Call melody, grounded voice
- Channel 1: Violin (Program 40) - "Response" - Answer melody, aspiring voice
- Channel 2: French Horn (Program 60) - "Collective Power" - Rising fanfares
- Channel 3: Strings (Program 48) - "Unity" - Thick harmonies, collective weight
- Channel 4: Timpani (Program 47) - "March" - Driving rhythm, insistent pulse

MUSICAL ARC (100 seconds = 175 beats at 105 BPM):
A. Individual Voices (beats 0-31): Cello alone, tentative melody
B. First Dialogue (beats 32-63): Violin answers, call and response begins
C. Growing Chorus (beats 64-111): French horn joins, strings swell, dialogue intensifies
D. United Front (beats 112-175): All voices in unison, triumphant C major climax

COMPOSITIONAL NOTES:
- A minor represents the weight of struggle, the tension of unresolved contradictions
- C major (relative major) represents achieved solidarity, collective strength
- Call-and-response pattern mirrors the dialectical spread of class consciousness
- French horn fanfares represent moments of breakthrough, collective recognition
- Timpani march pattern grows more insistent as solidarity strengthens
- The transition from individual voices to unified chorus mirrors the transition
  from atomized workers to organized class
"""

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-not-found]

# Constants
TICKS_PER_BEAT = 480  # Standard resolution
BPM = 105
MICROSECONDS_PER_BEAT = int(60_000_000 / BPM)
TOTAL_BEATS = 176  # ~100 seconds at 105 BPM

# Note definitions (MIDI note numbers)
NOTES = {
    # Octave 1
    "A1": 33,
    "B1": 35,
    # Octave 2
    "C2": 36,
    "D2": 38,
    "E2": 40,
    "F2": 41,
    "G2": 43,
    "A2": 45,
    "B2": 47,
    # Octave 3
    "C3": 48,
    "D3": 50,
    "E3": 52,
    "F3": 53,
    "G3": 55,
    "A3": 57,
    "B3": 59,
    # Octave 4
    "C4": 60,
    "D4": 62,
    "E4": 64,
    "F4": 65,
    "G4": 67,
    "A4": 69,
    "B4": 71,
    # Octave 5
    "C5": 72,
    "D5": 74,
    "E5": 76,
    "F5": 77,
    "G5": 79,
    "A5": 81,
    "B5": 83,
    # Octave 6
    "C6": 84,
}


def beats_to_ticks(beats: float) -> int:
    """Convert beats to MIDI ticks."""
    return int(beats * TICKS_PER_BEAT)


def create_conductor_track() -> MidiTrack:
    """Create the conductor track with tempo and time signature."""
    track = MidiTrack()
    track.append(MetaMessage("track_name", name="Solidarity Rising", time=0))
    track.append(MetaMessage("set_tempo", tempo=MICROSECONDS_PER_BEAT, time=0))
    track.append(MetaMessage("time_signature", numerator=4, denominator=4, time=0))
    track.append(MetaMessage("key_signature", key="A", time=0))  # A minor
    track.append(MetaMessage("end_of_track", time=beats_to_ticks(TOTAL_BEATS)))
    return track


def create_cello_track() -> MidiTrack:
    """
    Track 1: Cello - The Masses (Program 42, Channel 0)
    The calling voice - speaks first, grounded in material reality.
    Presents themes that violin will answer.
    """
    track = MidiTrack()
    track.name = "Cello - The Masses"
    track.append(Message("program_change", program=42, channel=0, time=0))

    # (note_name, start_beat, duration_beats, velocity)
    notes = [
        # Section A (beats 0-31): Individual Voices - Cello alone
        # First phrase - questioning, tentative
        ("A2", 0, 3, 60),
        ("C3", 3, 2, 62),
        ("E3", 5, 2, 65),
        ("D3", 7, 1, 60),
        # Second phrase - more assertive
        ("A2", 10, 2, 65),
        ("B2", 12, 2, 68),
        ("C3", 14, 3, 70),
        ("B2", 17, 1, 65),
        # Third phrase - reaching upward
        ("A2", 20, 2, 68),
        ("E3", 22, 3, 72),
        ("D3", 25, 2, 68),
        ("C3", 27, 2, 65),
        ("A2", 29, 3, 62),
        # Section B (beats 32-63): First Dialogue - CALL patterns
        # Call 1 (violin will answer)
        ("A2", 32, 2, 72),
        ("C3", 34, 2, 75),
        ("E3", 36, 2, 78),
        ("A3", 38, 2, 80),
        # Call 2 (after violin response)
        ("E3", 48, 2, 75),
        ("D3", 50, 2, 72),
        ("C3", 52, 2, 70),
        ("B2", 54, 2, 72),
        ("A2", 56, 4, 75),
        # Section C (beats 64-111): Growing Chorus - intensifying calls
        # Call 3 - stronger
        ("A2", 64, 2, 80),
        ("C3", 66, 2, 82),
        ("E3", 68, 2, 85),
        ("G3", 70, 2, 88),
        # Call 4 - C major hint
        ("C3", 80, 2, 82),
        ("E3", 82, 2, 85),
        ("G3", 84, 2, 88),
        ("C4", 86, 2, 90),
        # Call 5 - building
        ("A2", 96, 2, 85),
        ("C3", 98, 2, 88),
        ("E3", 100, 2, 90),
        ("A3", 102, 3, 92),
        ("G3", 105, 2, 88),
        ("E3", 107, 2, 85),
        # Section D (beats 112-175): United Front - unison with all voices
        # C major triumph
        ("C3", 112, 4, 95),
        ("E3", 116, 4, 95),
        ("G3", 120, 4, 98),
        ("C4", 124, 4, 100),
        # Sustained power
        ("G2", 128, 4, 95),
        ("C3", 132, 4, 98),
        ("E3", 136, 4, 100),
        ("G3", 140, 4, 100),
        # Final statement
        ("C3", 144, 4, 100),
        ("G3", 148, 4, 98),
        ("E3", 152, 4, 95),
        ("C3", 156, 4, 92),
        # Cadence
        ("G2", 160, 4, 95),
        ("C3", 164, 8, 100),
        ("C2", 172, 4, 85),  # Final low C
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


def create_violin_track() -> MidiTrack:
    """
    Track 2: Violin - Response (Program 40, Channel 1)
    The answering voice - responds to cello, higher register representing
    the aspirational quality of collective consciousness.
    """
    track = MidiTrack()
    track.name = "Violin - Response"
    track.append(Message("program_change", program=40, channel=1, time=0))

    # Violin enters at Section B (beat 32) with responses
    notes = [
        # Section B (beats 32-63): First Dialogue - RESPONSE patterns
        # Response 1 to cello's call (beat 40)
        ("E4", 40, 2, 72),
        ("G4", 42, 2, 75),
        ("A4", 44, 2, 78),
        ("E5", 46, 2, 80),
        # Response 2 (beat 58) - echoing and extending
        ("A4", 58, 1.5, 75),
        ("G4", 59.5, 1.5, 72),
        ("E4", 61, 2, 75),
        # Section C (beats 64-111): Growing Chorus - intensifying responses
        # Response 3 (beat 72) - matching cello's energy
        ("E4", 72, 2, 82),
        ("G4", 74, 2, 85),
        ("A4", 76, 2, 88),
        ("C5", 78, 2, 90),
        # Response 4 (beat 88) - C major echo
        ("G4", 88, 2, 85),
        ("C5", 90, 2, 88),
        ("E5", 92, 2, 90),
        ("G5", 94, 2, 92),
        # Response 5 (beat 106) - soaring
        ("C5", 106, 2, 90),
        ("E5", 108, 2, 92),
        ("G5", 110, 2, 95),
        # Section D (beats 112-175): United Front - parallel with cello
        # C major triumph - octave above cello
        ("C5", 112, 4, 95),
        ("E5", 116, 4, 95),
        ("G5", 120, 4, 98),
        ("C6", 124, 4, 100),
        # Sustained power
        ("G4", 128, 4, 95),
        ("C5", 132, 4, 98),
        ("E5", 136, 4, 100),
        ("G5", 140, 4, 100),
        # Final statement
        ("C5", 144, 4, 100),
        ("G5", 148, 4, 98),
        ("E5", 152, 4, 95),
        ("C5", 156, 4, 92),
        # Cadence - high sustain
        ("G5", 160, 4, 95),
        ("E5", 164, 8, 98),
        ("C5", 172, 4, 90),
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
        track.append(Message(msg_type, note=note, velocity=velocity, channel=1, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_french_horn_track() -> MidiTrack:
    """
    Track 3: French Horn - Collective Power (Program 60, Channel 2)
    Enters at Section C with heroic fanfares representing
    breakthrough moments of collective recognition.
    """
    track = MidiTrack()
    track.name = "French Horn - Collective Power"
    track.append(Message("program_change", program=60, channel=2, time=0))

    # French horn enters at Section C (beat 64)
    notes = [
        # Section C (beats 64-111): Rising fanfares
        # Fanfare 1 - A minor power
        ("A3", 64, 3, 78),
        ("E4", 67, 3, 80),
        ("A4", 70, 2, 82),
        # Fanfare 2 - C major arrival
        ("C4", 80, 3, 82),
        ("G4", 83, 3, 85),
        ("C5", 86, 2, 88),
        # Fanfare 3 - sustained call
        ("E4", 96, 4, 85),
        ("G4", 100, 4, 88),
        ("C5", 104, 4, 90),
        # Section D (beats 112-175): United Front - full power fanfares
        # Triumphant C major fanfare
        ("C4", 112, 4, 95),
        ("E4", 112, 4, 95),
        ("G4", 116, 4, 98),
        ("C5", 116, 4, 98),
        ("E5", 120, 4, 100),
        ("G5", 120, 4, 100),
        # Sustained power chords
        ("C4", 128, 8, 95),
        ("G4", 128, 8, 95),
        ("C4", 136, 8, 98),
        ("E4", 136, 8, 98),
        ("G4", 136, 8, 98),
        # Final fanfare
        ("G4", 148, 4, 100),
        ("C5", 152, 4, 100),
        ("E5", 156, 4, 98),
        ("G5", 160, 4, 100),
        ("C5", 164, 12, 100),  # Final sustained note
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


def create_strings_track() -> MidiTrack:
    """
    Track 4: Strings - Unity (Program 48, Channel 3)
    Thick harmonies representing the collective weight of organized labor.
    Enters gradually, building to full power in Section D.
    """
    track = MidiTrack()
    track.name = "Strings - Unity"
    track.append(Message("program_change", program=48, channel=3, time=0))

    # Strings enter at end of Section B, build through C, full power in D
    notes = [
        # Section B (beats 56-63): First hint of unity
        ("A2", 56, 8, 55),
        ("E3", 56, 8, 55),
        ("A3", 56, 8, 55),
        # Section C (beats 64-111): Growing unity
        # A minor swell
        ("A2", 64, 8, 65),
        ("C3", 64, 8, 65),
        ("E3", 64, 8, 65),
        ("A3", 64, 8, 65),
        # Building
        ("E3", 72, 8, 70),
        ("G3", 72, 8, 70),
        ("B3", 72, 8, 70),
        ("E4", 72, 8, 70),
        # C major hint
        ("C3", 80, 8, 75),
        ("E3", 80, 8, 75),
        ("G3", 80, 8, 75),
        ("C4", 80, 8, 75),
        # G major (dominant)
        ("G2", 88, 8, 78),
        ("B2", 88, 8, 78),
        ("D3", 88, 8, 78),
        ("G3", 88, 8, 78),
        # A minor return
        ("A2", 96, 8, 80),
        ("C3", 96, 8, 80),
        ("E3", 96, 8, 80),
        ("A3", 96, 8, 80),
        # G major (building to resolution)
        ("G2", 104, 8, 82),
        ("B2", 104, 8, 82),
        ("D3", 104, 8, 82),
        ("G3", 104, 8, 82),
        # Section D (beats 112-175): United Front - full harmonic weight
        # C major triumph
        ("C3", 112, 16, 92),
        ("E3", 112, 16, 92),
        ("G3", 112, 16, 92),
        ("C4", 112, 16, 92),
        # Second phrase
        ("G2", 128, 16, 95),
        ("C3", 128, 16, 95),
        ("E3", 128, 16, 95),
        ("G3", 128, 16, 95),
        # Third phrase
        ("C3", 144, 16, 98),
        ("E3", 144, 16, 98),
        ("G3", 144, 16, 98),
        ("C4", 144, 16, 98),
        # Final resolution
        ("C2", 160, 16, 100),
        ("G2", 160, 16, 100),
        ("C3", 160, 16, 100),
        ("E3", 160, 16, 100),
        ("G3", 160, 16, 100),
        ("C4", 160, 16, 100),
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


def create_timpani_track() -> MidiTrack:
    """
    Track 5: Timpani - March (Program 47, Channel 4)
    Driving rhythm representing the inexorable march of solidarity.
    More insistent than "The Spark" - this is a march, not just a heartbeat.
    """
    track = MidiTrack()
    track.name = "Timpani - March"
    track.append(Message("program_change", program=47, channel=4, time=0))

    events = []

    # Section A (beats 0-31): No timpani - solo cello
    # Timpani enters at Section B

    # Section B (beats 32-63): March pattern begins - quarter notes with accents
    for beat in range(32, 64):
        position_in_measure = beat % 4

        # March pattern: strong-weak-medium-weak
        if position_in_measure == 0:
            note = NOTES["A1"]
            velocity = 70 + min(beat - 32, 15)  # Crescendo
        elif position_in_measure == 2:
            note = NOTES["E2"]
            velocity = 60 + min(beat - 32, 12)
        else:
            note = NOTES["A1"]
            velocity = 50 + min(beat - 32, 10)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.35), "note_off", note, 0))

    # Section C (beats 64-111): March intensifies
    for beat in range(64, 112):
        position_in_measure = beat % 4

        # More aggressive pattern
        if position_in_measure == 0:
            note = NOTES["A1"]
            velocity = min(85 + (beat - 64) // 4, 95)
        elif position_in_measure == 2:
            note = NOTES["E2"]
            velocity = min(75 + (beat - 64) // 4, 88)
        elif position_in_measure == 1:
            note = NOTES["A1"]
            velocity = min(65 + (beat - 64) // 5, 78)
        else:  # position 3
            note = NOTES["E2"]
            velocity = min(60 + (beat - 64) // 5, 75)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.3), "note_off", note, 0))

    # Section D (beats 112-175): Full power march - C pedal
    for beat in range(112, 176):
        position_in_measure = beat % 4

        # Thunderous march on C
        if position_in_measure == 0:
            note = NOTES["C2"]
            velocity = 100
        elif position_in_measure == 2:
            note = NOTES["G2"]
            velocity = 92
        elif position_in_measure == 1:
            note = NOTES["C2"]
            velocity = 85
        else:  # position 3
            note = NOTES["G2"]
            velocity = 80

        # Final diminuendo in last 8 beats
        if beat >= 168:
            velocity = max(70, velocity - (beat - 168) * 3)

        events.append((beats_to_ticks(beat), "note_on", note, velocity))
        events.append((beats_to_ticks(beat + 0.25), "note_off", note, 0))

    events.sort(key=lambda x: (x[0], x[1] == "note_off"))

    last_time = 0
    for event_time, msg_type, note, velocity in events:
        delta = event_time - last_time
        track.append(Message(msg_type, note=note, velocity=velocity, channel=4, time=delta))
        last_time = event_time

    track.append(MetaMessage("end_of_track", time=beats_to_ticks(2)))
    return track


def create_midi_file() -> MidiFile:
    """Create the complete MIDI file for 'Solidarity Rising'."""
    mid = MidiFile(ticks_per_beat=TICKS_PER_BEAT)

    mid.tracks.append(create_conductor_track())
    mid.tracks.append(create_cello_track())
    mid.tracks.append(create_violin_track())
    mid.tracks.append(create_french_horn_track())
    mid.tracks.append(create_strings_track())
    mid.tracks.append(create_timpani_track())

    return mid


def main() -> None:
    """Generate and save the MIDI file."""
    output_path = (
        "/home/user/projects/game/babylon/assets/music/revolutionary/02_solidarity_rising.mid"
    )

    print("Creating 'Solidarity Rising' - Revolutionary Suite 02")
    print("=" * 55)

    mid = create_midi_file()
    mid.save(output_path)

    print(f"Saved to: {output_path}")
    print(f"Ticks per beat: {mid.ticks_per_beat}")
    print(f"Tempo: {BPM} BPM")
    print(f"Track count: {len(mid.tracks)}")
    for i, track in enumerate(mid.tracks):
        name = track.name if track.name else "(conductor)"
        print(f"  Track {i}: {name}")

    # Use mido's built-in length calculation
    length = mid.length
    print(f"Duration: {length:.1f} seconds ({length / 60:.2f} minutes)")

    print("\nComposition complete.")
    print("Musical arc: Individual Voices -> First Dialogue -> Growing Chorus -> United Front")
    print("Key progression: A minor -> C major")
    print("\nCall-and-Response Structure:")
    print("  - Cello (The Masses) states themes in lower register")
    print("  - Violin (Response) answers in upper register")
    print("  - French Horn (Collective Power) punctuates with fanfares")
    print("  - Strings (Unity) provide harmonic foundation")
    print("  - Timpani (March) drives the inexorable forward momentum")


if __name__ == "__main__":
    main()
