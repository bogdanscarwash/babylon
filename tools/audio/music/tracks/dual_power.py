#!/usr/bin/env python3
"""dual_power — RED_OGV (endgame suite).

CONCEPTUAL BRIEF:
Sovereignty, contested to the last bar. A contrabass B — the dominant of E,
the note that WANTS to resolve — pedals through the entire piece with a
timpani heartbeat, and never resolves. Above it, two musics alternate: the
red arrival (brass and choir, E major, warm and wide) and the old state
(harpsichord arpeggios and organ, F major — one poisoned semitone above
the pedal's home). Their entries begin politely separated, then the gap
closes: six beats, four, two, touching, overlapping — until both sound AT
ONCE, E major against F major over the B pedal, the loudest and densest
moment of the piece: interpenetration without synthesis. Both release; the
pedal hums on alone with its heartbeat; a final timpani stroke. Neither
yields. Victory with an asterisk, at soundtrack length.

TECHNICAL: 92 BPM, 4/4, ~180 beats ≈ 1:57. Contrabass 43 + Timpani 47 (the
pedal), Brass 61 + Choir 52 (red), Harpsichord 6 + Rock Organ 19 (the old
state).
"""

from __future__ import annotations

from tools.audio.music.composer import Score

#: (red_entry, old_entry) pairs — the gap between them closing to zero.
_ENTRIES: tuple[tuple[float, float], ...] = (
    (16.0, 30.0),
    (46.0, 58.0),
    (72.0, 82.0),
    (94.0, 102.0),
    (112.0, 118.0),
    (126.0, 130.0),
    (136.0, 136.0),
)


def _red_block(score: Score, start: float, duration: float, velocity: int) -> None:
    score.chord(2, (64, 68, 71), start, duration, velocity, spread=0.08)
    score.chord(3, (76, 80, 83), start + 0.5, duration - 0.5, velocity - 20, spread=0.3)
    score.cc_ramp(3, 11, start + 0.5, start + duration, 60, 105, steps=5)


def _old_block(
    score: Score,
    start: float,
    duration: float,
    velocity: int,
    organ: tuple[int, int, int] = (53, 57, 60),
) -> None:
    for cycle in range(int(duration // 2)):
        for index, pitch in enumerate((65, 69, 72, 77)):
            score.note(4, pitch, start + 2.0 * cycle + 0.4 * index, 0.35, velocity - 8)
    score.chord(5, organ, start, duration, velocity - 22, spread=0.1)


def compose() -> Score:
    score = Score(
        name="dual_power",
        suite="endgame",
        concept="A never-resolving dominant pedal under two alternating musics — E-major "
        "red arrival and F-major old state — whose entries converge from politely "
        "separated to fully simultaneous: interpenetration without synthesis.",
        bpm=92,
    )
    score.program(0, 43)  # contrabass — the pedal
    score.program(1, 47)  # timpani — the heartbeat
    score.program(2, 61)  # brass — red power
    score.program(3, 52)  # choir — red power's breadth
    score.program(4, 6)  # harpsichord — the old state's clerks
    score.program(5, 19)  # organ — the old state's stone

    score.cc(0, 91, 0.0, 55)
    score.cc(1, 91, 0.0, 50)
    score.cc(2, 91, 0.0, 62)
    score.cc(3, 91, 0.0, 82)
    score.cc(3, 93, 0.0, 80)
    score.cc(4, 91, 0.0, 18)
    score.cc(4, 93, 0.0, 0)
    score.cc(5, 91, 0.0, 46)
    score.cc(2, 10, 0.0, 48)
    score.cc(3, 10, 0.0, 56)
    score.cc(4, 10, 0.0, 84)
    score.cc(5, 10, 0.0, 78)

    score.marker(0.0, "The Pedal")
    score.marker(16.0, "Two Powers")
    score.marker(94.0, "Closer")
    score.marker(136.0, "Interpenetration")
    score.marker(148.0, "Neither Yields")

    # The pedal and its heartbeat — the heartbeat STOPS at 164 so the final
    # stroke lands in real silence instead of on its own grid.
    for strike in range(15):
        score.note(0, 35, 12.0 * strike, 11.5, 66)
    for pulse in range(42):
        score.note(1, 35, 4.0 * pulse, 0.4, 72)

    # The two powers: gap closing from fourteen beats to zero. The
    # penultimate old block is shortened so its note-offs cannot truncate
    # the climax chord (the overlap sentinel enforces this class now).
    for index, (red_start, old_start) in enumerate(_ENTRIES[:-1]):
        _red_block(score, red_start, 8.0, 96 + 2 * index)
        old_duration = 6.0 if index == 5 else 8.0
        _old_block(score, old_start, old_duration, 92 + 2 * index)
    # Full simultaneity: both at once, twelve beats, the densest moment —
    # the old state's organ voiced an octave up so the E-against-F grind
    # sits fully exposed above the pedal.
    _red_block(score, 136.0, 12.0, 110)
    _old_block(score, 136.0, 12.0, 102, organ=(65, 69, 72))

    # Release: the pedal alone, twelve beats of it. Then one stroke, louder
    # than the heartbeat ever was. Nothing resolves.
    score.cc_ramp(0, 11, 150.0, 172.0, 110, 55, steps=8)
    score.note(1, 35, 176.0, 1.0, 92)
    return score
