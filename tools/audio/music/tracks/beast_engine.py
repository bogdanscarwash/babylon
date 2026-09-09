#!/usr/bin/env python3
"""beast_engine — the pulsating capitalist death engine at idle (entity suite).

CONCEPTUAL BRIEF (#641):
The persistent bed under menus and idle screens: the beast never stops feeding.
Two pads state the same E drone, one riding a constant +35-cent bend, so the
bed's fundamental beats against itself — the machine is out of tune with
itself and never resolves. Seven organ voices (#639's organ map — the six
body organs plus the mask's glint — one per MIDI
channel — the channel↔organ binding is a cue-map CONTRACT) cycle at coprime
periods (16, 17, 23, 29, 31, 43, 47) so the surface never repeats, while a
strict 4-beat mechanical pulse holds the grid rigid underneath: the pulse is
the only honest thing in the piece. The breath is asymmetric — a 10-beat
inhale against a 6-beat exhale — so the organism always sounds like it is
over-feeding. Circulation idles on D–A–D and never PLAYS the F: the Φ motif
is gestured melodically, never completed. The F sounds in the piece only in
the body's OTHER organs — the nervous system's F2 tremolo grind and the
tissue's E3+F3 membrane cluster — and in six of the fourteen circulation
cycles one of them sounds under the idling D–A. In those bars the surplus
note is PRESENT AS TENSION, carried by the field and the flesh, never
collected as melody. That is the deliberate harmonic reading: the beast
cannot play its own surplus; it can only ache where the surplus would be.
The mask (channel 8, vibraphone) only glints here — a distant pure fifth every
47 beats; its full consonant surface lives in the_mask (Director-ruled Arm A).

INTENSITY GOVERNOR (menu-safe discipline, #641 Gate 5 — controller-adjudicated
under the Director's compass delegation, 2026-08-18): the bed's intensity is
governed by Φ, the imperial-rent pool — the beast's loudness IS its feeding
rate, so disrupting Φ audibly starves the machine. Assets ship one intensity;
the ladder is the (parked) mixer's contract, recorded in assets/CUE_MAP.md.

TECHNICAL: 60 BPM, 4/4, ~240 beats ≈ 4:00, loop-seamed (CC11 returns to its
beat-0 value; the final drone breath abuts beat 0). Warm Pad 89 (body),
New Age Pad 88 (the detuned twin, +35 ¢ ≈ bend +239), Timpani 47 (heartbeat),
Fretless 35 (circulation), Breath Noise 121 (metabolism), Choir 52 (tissue),
Tremolo Strings 44 (nervous system — F2 floor), Xylophone 13 (skeleton),
Vibraphone 11 (the mask's glint), percussion (kick/hat) as the grid.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

#: The channel↔organ contract (#639 organ map — mirrored in CUE_MAP.md).
#: 0/1 body drone + detuned twin; 2 heartbeat; 3 circulation; 4 metabolism;
#: 5 tissue; 6 nervous system; 7 skeleton; 8 the mask (glint only here).
_TWIN_BEND: int = 239  # +35 cents at ±12-semitone range (6.83 units/cent)


def compose() -> Score:
    score = Score(
        name="beast_engine",
        suite="entity",
        concept="The death engine idling: a self-beating E drone, six organs on "
        "coprime cycles, an over-feeding breath, and the only honest pulse in "
        "the game. D-A-D idles beneath and never finds the F.",
        bpm=60,
    )
    score.program(0, 89)  # warm pad — the body
    score.program(1, 88)  # new-age pad — the detuned twin
    score.program(2, 47)  # timpani — organ 1, heartbeat
    score.program(3, 35)  # fretless bass — organ 2, circulation
    score.program(4, 121)  # breath noise — organ 3, metabolism
    score.program(5, 52)  # choir — organ 4, tissue
    score.program(6, 44)  # tremolo strings — organ 5, nervous system
    score.program(7, 13)  # xylophone — organ 6, skeleton
    score.program(8, 11)  # vibraphone — organ 7, the mask's glint

    for channel, reverb in (
        (0, 82),
        (1, 82),
        (2, 60),
        (3, 55),
        (4, 74),
        (5, 78),
        (6, 66),
        (7, 48),
        (8, 90),
    ):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(3, 10, 0.0, 44)
    score.cc(4, 10, 0.0, 54)
    score.cc(5, 10, 0.0, 74)
    score.cc(6, 10, 0.0, 38)
    score.cc(7, 10, 0.0, 88)
    score.cc(8, 10, 0.0, 60)
    # Atomization as the bed's resting state — semantic record (CC94 is a
    # FluidR3 no-op); the audible device is the twin's constant detune bend.
    score.cc(1, 94, 0.0, 70)

    score.marker(0.0, "The Body")
    score.marker(60.0, "Feeding")
    score.marker(120.0, "Over-feeding")
    score.marker(180.0, "The Glint")

    # The body: E2+B2 in four abutting breaths on BOTH pads (never overlapping
    # — duration held under the 60-beat spacing; the kit's overlap sentinel
    # enforces this). The twin rides a constant +35-cent bend from beat 0,
    # re-asserted mid-piece: the fundamental beats against itself forever.
    score.bend(1, 0.0, _TWIN_BEND)
    score.bend(1, 120.0, _TWIN_BEND)
    for segment in range(4):
        start = segment * 60.0
        duration = 59.5 if segment < 3 else 60.5
        score.note(0, 40, start, duration, 56)
        score.note(0, 47, start, duration, 48)
        score.note(1, 40, start, duration, 40)
        score.note(1, 47, start, duration, 34)

    # The over-feeding breath: 15 asymmetric 16-beat cycles on the body's
    # expression — 10 beats in, 6 beats out. The organism takes more than
    # it returns; the loop seam closes at the beat-0 value below.
    for cycle in range(15):
        base = cycle * 16.0
        score.cc_ramp(0, 11, base, base + 10.0, 50, 85, steps=8)
        score.cc_ramp(0, 11, base + 10.0, base + 16.0, 85, 50, steps=6)

    # Organ 1 — heartbeat: timpani E1 every 16 beats, metronome-exact.
    for cycle in range(15):
        start = 8.0 + 16.0 * cycle
        score.note(2, 28, start, 1.2, 40 + 4 * (cycle % 3))

    # Organ 2 — circulation: D2-A2-D2 every 17 beats. The Phi motif's first
    # three notes, idling; the F never comes (SS2.12 — the beast gestures at
    # D-A-D-F and never completes it).
    for cycle in range(14):
        start = 3.0 + 17.0 * cycle
        score.note(3, 38, start, 1.1, 46)
        score.note(3, 45, start + 1.2, 1.1, 44)
        score.note(3, 38, start + 2.4, 1.1, 42)

    # Organ 3 — metabolism: a breath-noise exhalation every 23 beats.
    for cycle in range(11):
        start = 7.0 + 23.0 * cycle
        score.note(4, 64, start, 3.0, 36 + 6 * (cycle % 2))

    # Organ 5 — nervous system: the F2 tremolo grind every 29 beats (the
    # octave where FluidR3's tremolo patch actually speaks), creeping louder
    # across the piece — the field never quite settles.
    for cycle in range(9):
        start = 5.0 + 29.0 * cycle
        score.note(6, 41, start, 5.0, 34 + cycle)

    # Organ 4 — tissue: choir E3+F3, the flat-2 denial inside the membrane,
    # every 31 beats.
    for cycle in range(8):
        start = 11.0 + 31.0 * cycle
        score.note(5, 52, start, 4.0, 40)
        score.note(5, 53, start, 4.0, 36)

    # Organ 6 — skeleton: three dry xylophone taps every 43 beats — E5, the
    # B-flat crisis tritone, E5 again. The crisis is in the bones.
    for cycle in range(6):
        start = 13.0 + 43.0 * cycle
        score.note(7, 76, start, 0.3, 42)
        score.note(7, 70, start + 0.5, 0.3, 40)
        score.note(7, 76, start + 1.0, 0.3, 38)

    # Organ 7 — the mask's glint: a distant pure fifth every 47 beats. In the
    # bed the mask only glints; its full consonant surface is the_mask's.
    for cycle in range(5):
        start = 19.0 + 47.0 * cycle
        score.note(8, 64, start, 2.5, 34)
        score.note(8, 71, start, 2.5, 32)

    # The grid: the only honest thing in the piece. Kick every 4 beats, hat on
    # the off-pulse, quiet enough to live under a menu forever.
    for step in range(60):
        score.note(9, 36, 4.0 * step, 0.1, 50)
        score.note(9, 42, 4.0 * step + 2.0, 0.05, 28)

    # Seam: THE LOOP POINT IS 240.0, declared by marker — the breath ends
    # where it began so the menu loop closes without a level jump, and the
    # twin's detune is re-asserted across the seam. (The final drone breath
    # overhangs by 0.5 beats into the loop's head, the history_breathing
    # idiom; the cue map's mix row carries the same loop figure.)
    score.marker(240.0, "LOOP")
    score.cc(0, 11, 240.0, 50)
    score.bend(1, 239.5, _TWIN_BEND)
    return score
