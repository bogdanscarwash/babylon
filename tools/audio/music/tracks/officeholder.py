#!/usr/bin/env python3
"""officeholder — capture, note by note (superstructure suite).

CONCEPTUAL BRIEF:
P25's officeholder-capture mechanic as counterpoint. A warm eight-note
workers' theme in the cello (chorus high — it sings WITH others) states
itself nine times. Each statement, the harpsichord — the Machine, chorus
zero, bone dry, fixed velocity, staccato — takes ONE MORE NOTE, from the
FRONT of the phrase: the public opening is captured first, the private
cadence last. The cello's remaining notes tire (velocity falls two points
per statement) but keep their warmth. By the ninth statement the Machine
plays the whole theme — and deletes the ornament: the leading-tone D that
gave the cadence its life is flattened into a straight repeat. In the coda
the cello tries to begin again, manages two notes, stops; the harpsichord
finishes the phrase without it. The contrabass floor and the timpani clock
never react. That is what capture sounds like: the same notes, and nothing
left inside them.

TECHNICAL: 76 BPM, 4/4, ~160 beats ≈ 2:06. Cello 42 (the class), Harpsichord
6 (the office), Contrabass 43, Timpani 47.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_THEME: tuple[int, ...] = (52, 55, 57, 59, 57, 55, 52, 50)  # E3 G3 A3 B3 A3 G3 E3 D3


def compose() -> Score:
    score = Score(
        name="officeholder",
        suite="superstructure",
        concept="A workers' theme captured one note per statement by the Machine, from "
        "the public front to the private cadence; the ninth statement is all Machine "
        "with the ornament deleted; the coda is the class failing to restart its own song.",
        bpm=76,
    )
    score.program(0, 42)  # cello — the class
    score.program(1, 6)  # harpsichord — the office
    score.program(2, 43)  # contrabass — the floor
    score.program(3, 47)  # timpani — the clock
    score.cc(0, 93, 0.0, 72)
    score.cc(0, 91, 0.0, 55)
    score.cc(1, 93, 0.0, 0)
    score.cc(1, 91, 0.0, 12)
    score.cc(2, 91, 0.0, 40)
    score.cc(3, 91, 0.0, 42)
    score.cc(0, 10, 0.0, 54)
    score.cc(1, 10, 0.0, 74)

    score.marker(4.0, "The Theme, Whole")
    score.marker(20.0, "First Capture")
    score.marker(68.0, "Majority Captured")
    score.marker(132.0, "The Machine Speaks")
    score.marker(148.0, "Aphasia")

    # The floor rises two points a statement as the class tires — the only
    # thing in the room gaining weight — while the clock stays indifferent.
    for segment in range(10):
        score.note(2, 28, 16.0 * segment, 15.0, 56 + 2 * segment)
    for tick in range(20):
        score.note(3, 40, 8.0 * tick, 0.4, 56)

    # Nine statements; the Machine takes one more note each time, from the front.
    for statement in range(9):
        base = 4.0 + 16.0 * statement
        cello_velocity = 92 - 2 * statement
        for index, pitch in enumerate(_THEME):
            start = base + 2.0 * index
            if index < statement or statement == 8:
                # Captured: staccato, mechanical; the Machine warms to its role
                # (+2 velocity per statement past the majority), and deletes
                # the ornament outright at full capture.
                sounded = 52 if (statement == 8 and index == 7) else pitch
                machine_velocity = 88 + max(0, 2 * (statement - 4))
                score.note(1, sounded, start, 0.5, machine_velocity)
            else:
                score.note(0, pitch, start, 1.8, cello_velocity)
        # The cello still phrases — while it has anything left to phrase with.
        if statement < 8:
            score.cc_ramp(0, 11, base, base + 8.0, 84, 104, steps=6)
            score.cc_ramp(0, 11, base + 8.0, base + 15.5, 104, 84, steps=6)

    # Coda: the class tries to begin again; the Machine completes the sentence.
    score.note(0, 52, 148.0, 1.8, 66)
    score.note(0, 55, 150.0, 1.4, 58)
    for index, pitch in enumerate((57, 59, 57, 55, 52, 52)):
        score.note(1, pitch, 153.0 + 0.8 * index, 0.45, 84)
    return score
