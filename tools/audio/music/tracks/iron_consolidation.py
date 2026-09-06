#!/usr/bin/env python3
"""iron_consolidation — FASCIST_CONSOLIDATION (endgame suite).

CONCEPTUAL BRIEF:
False order at full length. Brass, strings and organ hammer the same
Phrygian line in strict unison across three octaves — conformity given
mass — over timpani, taiko and a regimented snare, chorus zero, in a dry
airless room. At the centre, the machine idles and a SOLO VIOLIN plays the
solidarity call from the revolutionary suite... and nothing answers. Four
beats of percussion where the response should be. It tries once more,
smaller. Silence again — the atomized nation cannot answer a call; that is
the whole diagnosis in eight beats. The column returns larger (a tuba
octave below), marches to the end of the harmony, and then the FULL
percussion machine — snare included — keeps marching, bar after identical
bar, and is cut mid-pattern. The machine does not resolve; it is
interrupted.

TECHNICAL: 104 BPM, 4/4, ~174 beats ≈ 1:40. Brass 61, Strings 48, Rock
Organ 19, Tuba 58, Violin 40, Timpani 47, Taiko 116, snare/kick on ch 9.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

# Two-bar unison units (pitch, beat, duration) rooted on E3.
_UNIT_A: tuple[tuple[int, float], ...] = ((52, 0.0), (53, 1.0), (52, 2.0), (50, 3.0), (52, 4.0))
_UNIT_B: tuple[tuple[int, float], ...] = ((52, 0.0), (53, 1.0), (55, 2.0), (53, 3.0), (52, 4.0))


def _column(
    score: Score, base: float, unit: tuple[tuple[int, float], ...], velocity: int, tuba: bool
) -> None:
    for pitch, beat in unit:
        start = base + beat
        score.note(0, pitch, start, 0.9, velocity)
        score.note(1, pitch - 12, start, 0.9, velocity - 12)
        score.note(2, pitch + 12, start, 0.9, velocity - 18)
        if tuba:
            score.note(6, pitch - 24, start, 0.9, velocity - 8)


def _machine(score: Score, base: float, velocity: int) -> None:
    for pulse in (0.0, 2.0, 4.0, 6.0):
        score.note(3, 40, base + pulse, 0.4, velocity)
    score.note(4, 48, base, 0.4, velocity - 4)
    score.note(4, 48, base + 4.0, 0.4, velocity - 4)
    for offbeat in (0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5):
        accent = offbeat in (0.5, 2.5, 4.5, 6.5)
        score.note(9, 38, base + offbeat, 0.1, (velocity + 2) if accent else (velocity - 34))
    score.note(9, 36, base, 0.1, velocity - 2)
    score.note(9, 36, base + 4.0, 0.1, velocity - 2)


def compose() -> Score:
    score = Score(
        name="iron_consolidation",
        suite="endgame",
        concept="A three-octave Phrygian unison column over a regimented percussion "
        "machine; at the centre a solo violin's solidarity call receives no answer, "
        "twice; the column returns larger, the harmony ends, and the machine marches "
        "on identically until it is cut mid-pattern.",
        bpm=104,
    )
    score.program(0, 61)  # brass — the column
    score.program(1, 48)  # strings — the column, below
    score.program(2, 19)  # organ — the column, above
    score.program(3, 47)  # timpani — the boot
    score.program(4, 116)  # taiko — the drum of state
    score.program(5, 40)  # violin — the unanswered call
    score.program(6, 58)  # tuba — mass

    for channel in (0, 1, 2, 5):
        score.cc(channel, 93, 0.0, 0)
    for channel, reverb in ((0, 12), (1, 12), (2, 14), (3, 8), (4, 8), (5, 30), (6, 10)):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(0, 71, 0.0, 96)
    # False unity, audibly false: the organ rides a constant +40-cent bend, so
    # the strict unison is permanently out of tune with itself (Director's
    # dissonance line); brass vibrato rises through the return (CC1 renders
    # where CC71 does not on FluidR3 — the CC71 row above stays as the
    # semantic record).
    score.bend(2, 0.0, 273)
    score.cc_ramp(0, 1, 84.0, 148.0, 20, 80, steps=8)

    score.marker(0.0, "The Column")
    score.marker(64.0, "The Question")
    score.marker(72.0, "No Answer")
    score.marker(84.0, "The Column, Larger")
    score.marker(148.0, "The Machine Alone")
    score.marker(173.5, "Cut")

    # Section A: the column, units AABB, machine underneath.
    for section, base in enumerate((0.0, 8.0, 16.0, 24.0)):
        unit = _UNIT_A if section < 2 else _UNIT_B
        _column(score, base, unit, 106, tuba=False)
        _machine(score, base, 108)
    for section, base in enumerate((32.0, 40.0, 48.0, 56.0)):
        unit = _UNIT_A if section % 2 == 0 else _UNIT_B
        _column(score, base, unit, 110, tuba=False)
        _machine(score, base, 110)

    # The question: the machine idles; the call goes out. Nothing answers.
    score.cc(3, 11, 64.0, 55)
    score.cc(4, 11, 64.0, 55)
    for pulse in (64.0, 66.0, 68.0, 70.0, 72.0, 74.0, 76.0, 78.0, 80.0, 82.0):
        score.note(3, 40, pulse, 0.3, 58)
    for index, pitch in enumerate((64, 67, 69)):
        score.note(5, pitch, 66.0 + 1.5 * index, 1.3, 66)
    # ...four beats of nothing where the answer should be...
    score.note(5, 64, 76.0, 1.0, 52)
    score.note(5, 67, 77.2, 0.8, 46)
    # ...and nothing again.

    # The column returns, GENUINELY larger — the tuba enters here for the
    # first time, and from beat 116 the organ doubles a second octave up.
    score.cc(3, 11, 84.0, 127)
    score.cc(4, 11, 84.0, 127)
    for section, base in enumerate((84.0, 92.0, 100.0, 108.0, 116.0, 124.0, 132.0, 140.0)):
        unit = _UNIT_A if section % 2 == 0 else _UNIT_B
        _column(score, base, unit, 116, tuba=True)
        _machine(score, base, 114)
        if base >= 116.0:
            for pitch, beat in unit:
                score.note(2, pitch + 24, base + beat, 0.9, 84)

    # The harmony has stopped. The machine has not — and it is still growing.
    for base, machine_velocity in ((148.0, 114), (156.0, 118), (164.0, 122)):
        _machine(score, base, machine_velocity)
    score.note(3, 40, 172.0, 0.4, 112)
    score.note(4, 48, 172.0, 0.4, 108)
    score.note(9, 36, 172.0, 0.1, 110)
    score.note(9, 38, 172.5, 0.1, 114)
    score.note(9, 38, 173.5, 0.1, 114)  # cut mid-pattern: no downbeat follows
    return score
