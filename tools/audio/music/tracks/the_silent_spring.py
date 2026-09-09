#!/usr/bin/env python3
"""the_silent_spring — the fading of the living world (rift suite).

CONCEPTUAL BRIEF:
After overshoot, the quieter piece: not the crash but the thinning. A flute
sings birdsong fragments over a vast pad — and each fragment is SHORTER
than the last: seven notes, then six, five, four, two, and finally a single
note cut off mid-breath, exposed against a deliberately hushed swell. Three
string chords pass a register above the pad like seasons, each barer and
each WRONGER than the last — a true minor triad, then a lydian-flat smear
(C, E, F#, B: the season comes back with the wrong notes in it), then an
empty fifth. The landscape never goes fully silent until the end — the pad
entries overlap the gaps so the game never reads as hung — but from beat
140 the ground itself detunes in a continuous flatward slide while the
light fades. The last sound in the world is a single triangle stroke: one
drop of water. The final silence is not the absence of the piece; it is
the piece.

TECHNICAL: 56 BPM, 4/4, ~168 beats ≈ 3:00. Flute 73, Warm Pad 89,
Strings 48, triangle on ch 9. Reverb very high — an emptying landscape.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

#: Birdsong fragments: per fragment, (pitch, start_offset, duration) tuples.
_FRAGMENTS: tuple[tuple[tuple[int, float, float], ...], ...] = (
    (
        (76, 0.0, 0.5),
        (79, 0.6, 0.4),
        (83, 1.1, 0.6),
        (81, 1.9, 0.4),
        (79, 2.4, 0.5),
        (83, 3.1, 0.7),
        (76, 4.0, 1.2),
    ),
    (
        (79, 0.0, 0.5),
        (81, 0.6, 0.4),
        (84, 1.1, 0.7),
        (81, 2.0, 0.5),
        (79, 2.7, 0.9),
        (76, 3.8, 1.0),
    ),
    ((76, 0.0, 0.5), (79, 0.6, 0.5), (83, 1.2, 0.8), (79, 2.2, 0.6), (76, 3.0, 1.0)),
    ((79, 0.0, 0.6), (83, 0.8, 0.6), (81, 1.6, 0.7), (79, 2.5, 0.9)),
    ((76, 0.0, 0.7), (79, 0.9, 0.9)),
    ((84, 0.0, 0.3),),
)

#: Pad entries (start, duration, velocity) — sized so no mid-piece gap of
#: total silence exceeds ~4 beats; the true silence is reserved for the end.
_PAD: tuple[tuple[float, float, int], ...] = (
    (0.0, 24.0, 54),
    (40.0, 26.0, 48),
    (76.0, 32.0, 42),
    (132.0, 32.0, 36),
)

#: Expression peak per pad entry — the last swell is deliberately hushed so
#: the final one-note bird is exposed, not masked.
_PAD_PEAKS: tuple[int, ...] = (90, 90, 90, 50)

#: The seasons: each barer, the middle one wrong (C E F# B — lydian smear).
_SEASONS: tuple[tuple[tuple[int, ...], float, float, int], ...] = (
    ((52, 55, 59), 20.0, 16.0, 50),
    ((48, 52, 54, 59), 66.0, 14.0, 42),
    ((52, 59), 118.0, 12.0, 34),
)


def compose() -> Score:
    score = Score(
        name="the_silent_spring",
        suite="rift",
        concept="Birdsong shrinking seven notes to one over pad entries that keep the "
        "world from ever fully stopping mid-piece; seasons return barer and wronger; "
        "the ground slides continuously flat; one drop of water; then the real silence.",
        bpm=56,
    )
    score.program(0, 73)  # flute — the last singers
    score.program(1, 89)  # warm pad — the landscape
    score.program(2, 48)  # strings — the seasons

    score.cc(0, 91, 0.0, 88)
    score.cc(1, 91, 0.0, 100)
    score.cc(2, 91, 0.0, 92)
    score.cc(0, 10, 0.0, 76)
    score.cc(2, 10, 0.0, 54)

    score.marker(0.0, "Morning, Still")
    score.marker(60.0, "Fewer")
    score.marker(112.0, "Fewer Still")
    score.marker(140.0, "The Ground Detunes")
    score.marker(165.0, "One Drop")

    # The landscape: four entries, the last one hushed.
    for index, (start, duration, velocity) in enumerate(_PAD):
        score.note(1, 40, start, duration, velocity)
        score.note(1, 47, start, duration, velocity - 6)
        peak = _PAD_PEAKS[index]
        score.cc_ramp(1, 11, start, start + 8.0, 40, peak, steps=6)
        score.cc_ramp(1, 11, start + 10.0, start + duration, peak, 35, steps=6)

    # The seasons: a register above the pad, each barer, the middle one wrong.
    for pitches, start, duration, velocity in _SEASONS:
        score.chord(2, pitches, start, duration, velocity, spread=0.4)

    # The singers: six fragments, shrinking, spacing out, fading.
    for index, fragment in enumerate(_FRAGMENTS):
        base = 8.0 + 26.0 * index
        velocity = 72 - 6 * index
        for pitch, offset, duration in fragment:
            score.note(0, pitch, base + offset, duration, velocity)

    # The ground detunes — continuously, under a sounding pad — and the
    # light goes.
    for index in range(12):
        score.bend(1, 140.0 + 2.0 * index, -round(700 * index / 11))
    score.cc_ramp(1, 11, 150.0, 163.0, 45, 12, steps=8)
    # One drop.
    score.note(9, 81, 165.0, 0.5, 48)
    return score
