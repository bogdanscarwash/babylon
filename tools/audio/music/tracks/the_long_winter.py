#!/usr/bin/env python3
"""the_long_winter — ECOLOGICAL_COLLAPSE (endgame suite).

CONCEPTUAL BRIEF:
Nature does not lose; it stops answering. A whole-tone ladder — no tonal
gravity, because the biosphere has no stake in our cadences — descends in
long overlapping string tones, each softer, while a contrabass E1 returns
at longer intervals like breath slowing. Once, at the centre, the flute
tries the opening of the silent-spring birdsong and is cut off
mid-gesture: a memory of a memory. Two tubular-bell strikes pass like a
clock somewhere still running, slower each time. The strings drift flat in
three slow bends — the ground itself detuning — and the piece does not end
so much as stop being audible. Snow.

TECHNICAL: 50 BPM, 4/4, ~125 beats ≈ 2:30. Strings 48, Contrabass 43,
Flute 73, Tubular Bells 14. Reverb near-maximum; almost no attack anywhere.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_LADDER: tuple[int, ...] = (64, 62, 60, 58, 56, 54, 52)  # whole-tone, wrapping home


def compose() -> Score:
    score = Score(
        name="the_long_winter",
        suite="endgame",
        concept="A whole-tone ladder descends in overlapping softening strings over a "
        "slowing contrabass breath; one cut-off birdsong memory, two slowing bell "
        "strikes, three flatward bends. Snow.",
        bpm=50,
    )
    score.program(0, 48)  # strings — the ladder
    score.program(1, 43)  # contrabass — breath
    score.program(2, 73)  # flute — a memory of song
    score.program(3, 14)  # tubular bells — a clock somewhere

    score.cc(0, 91, 0.0, 96)
    score.cc(1, 91, 0.0, 90)
    score.cc(2, 91, 0.0, 92)
    score.cc(3, 91, 0.0, 100)
    score.cc(0, 94, 40.0, 35)
    score.cc(0, 94, 80.0, 70)

    score.marker(0.0, "First Frost")
    score.marker(12.0, "The Ladder Down")
    score.marker(60.0, "A Memory of Song")
    score.marker(90.0, "The Clock Slows")
    score.marker(118.0, "Snow")

    # The ladder: overlapping 16-beat tones, each softer, then a dying held
    # tone that carries the detune and the fade to the end of the piece.
    for index, pitch in enumerate(_LADDER):
        score.note(0, pitch, 12.0 * index, 16.0, 84 - 7 * index)
    score.note(0, 50, 88.0, 34.0, 38)

    # Breath, slowing: entries at widening intervals, each softer.
    for start, duration, velocity in (
        (0.0, 30.0, 60),
        (35.0, 25.0, 52),
        (65.0, 25.0, 44),
        (95.0, 20.0, 36),
    ):
        score.note(1, 28, start, duration, velocity)

    # A memory of the silent spring — cut off mid-gesture.
    score.note(2, 76, 60.0, 0.5, 46)
    score.note(2, 79, 60.6, 0.4, 42)
    score.note(2, 83, 61.1, 0.35, 38)

    # A clock somewhere, slower each time.
    score.note(3, 64, 90.0, 6.0, 44)
    score.note(3, 64, 110.0, 6.0, 36)

    # The ground detunes — continuously, under sounding strings — and the
    # light goes. CC1 vibrato rises alongside (CC94 is kept as the semantic
    # record; FluidR3 does not render it, so the audible device is here).
    for index in range(14):
        score.bend(0, 40.0 + 6.0 * index, -round(1100 * index / 13))
    score.cc_ramp(0, 1, 40.0, 110.0, 15, 70, steps=8)
    score.cc_ramp(0, 11, 100.0, 122.0, 100, 18, steps=10)
    score.cc_ramp(1, 11, 105.0, 115.0, 90, 25, steps=5)
    return score
