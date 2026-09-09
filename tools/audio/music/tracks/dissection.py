#!/usr/bin/env python3
"""dissection — the beast taken apart and put back together (entity suite).

CONCEPTUAL BRIEF (#641):
The exploded-view companion (#639 organ 8 — dissection is the interaction,
not a voice): each of the SEVEN sounding organs states its own material ALONE,
in the fixed order of the organ list, for sixteen bare beats inside an
eighteen-beat slot — a listener learns which timbre is which — then they
re-enter cumulatively in the same order, and the piece ends with the full
bed, drone, twin-detune and grid included: audibly MORE WRONG assembled than
any part was alone. That is the thesis. The pathology is relational; no organ
carries it by itself.

The mask (organ 7, vibraphone — Director-ruled Arm A) IS dissected here: its
solo is the last one you learn, a pure fifth glinting alone — and learning to
pick that voice out of the bed is exactly what makes toggling to the ticker
land as a lifting of the veil.

The channel↔organ binding is the same CONTRACT as beast_engine (cue-map row:
"solo organ N" in the client is a mixer operation on bed channel N; this track
is the reference recording for which timbre is which).

TECHNICAL: 60 BPM, 4/4, 206 beats ≈ 3:26. Solos at 18·k for k in 0..6;
cumulative re-entry at 126 + 10·k; the body (pads + grid) enters at 196 and
the piece ends assembled. Programs identical to beast_engine: Timpani 47,
Fretless 35, Breath Noise 121, Choir 52, Tremolo Strings 44 (F2 floor),
Xylophone 13, Vibraphone 11, Warm Pad 89 + New Age Pad 88 (+35-cent twin),
percussion grid. All note generation is fixed-count `for` loops — the count
per section is stated at the call site, per the house loop-bound rule.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_TWIN_BEND: int = 239  # +35 cents — the same wrongness as beast_engine's


def _heartbeat(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        score.note(2, 28, start + 4.0 * index, 1.2, velocity)


def _circulation(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        base = start + 6.0 * index
        score.note(3, 38, base, 1.1, velocity)
        score.note(3, 45, base + 1.2, 1.1, velocity - 2)
        score.note(3, 38, base + 2.4, 1.1, velocity - 4)


def _metabolism(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        score.note(4, 64, start + 6.0 * index, 3.0, velocity)


def _tissue(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        base = start + 7.0 * index
        score.note(5, 52, base, 5.0, velocity)
        score.note(5, 53, base, 5.0, velocity - 4)


def _nervous(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        score.note(6, 41, start + 8.0 * index, 6.0, velocity)


def _skeleton(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        base = start + 4.0 * index
        score.note(7, 76, base, 0.3, velocity)
        score.note(7, 70, base + 0.5, 0.3, velocity - 2)
        score.note(7, 76, base + 1.0, 0.3, velocity - 4)


def _mask(score: Score, start: float, count: int, velocity: int) -> None:
    for index in range(count):
        base = start + 6.0 * index
        score.note(8, 64, base, 2.5, velocity)
        score.note(8, 71, base, 2.5, velocity - 2)


#: (solo function, organ name, solo count, reassembly count) in the organ-list
#: order — the fixed order a listener learns them in, and the order they
#: return. Solo counts fill the 16-beat material window (slot start + 1 to
#: + 17); reassembly counts fill from each voice's re-entry to beat ~206.
_ORGANS = (
    (_heartbeat, "Organ 1: heartbeat", 4, 20),
    (_circulation, "Organ 2: circulation", 3, 12),
    (_metabolism, "Organ 3: metabolism", 3, 10),
    (_tissue, "Organ 4: tissue", 2, 7),
    (_nervous, "Organ 5: nervous system", 2, 5),
    (_skeleton, "Organ 6: skeleton", 4, 8),
    (_mask, "Organ 7: the mask", 3, 4),
)


def compose() -> Score:
    score = Score(
        name="dissection",
        suite="entity",
        concept="Seven organ solos in fixed order, bare, then cumulative "
        "reassembly into the full bed: more wrong together than any part "
        "alone. The pathology is relational, not located. The face is the "
        "last voice you learn.",
        bpm=60,
    )
    score.program(0, 89)
    score.program(1, 88)
    score.program(2, 47)
    score.program(3, 35)
    score.program(4, 121)
    score.program(5, 52)
    score.program(6, 44)
    score.program(7, 13)
    score.program(8, 11)

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

    # The solos: sixteen bare beats of material in each eighteen-beat slot,
    # in the organ order. Resist the urge to accompany — the bareness is the
    # pedagogy.
    for index, (solo, label, solo_count, _reentry_count) in enumerate(_ORGANS):
        start = 18.0 * index
        score.marker(start, label)
        solo(score, start + 1.0, solo_count, 46)

    # Reassembly: the organs return cumulatively in the same order, each
    # holding on once entered. The wrongness accumulates relation by relation.
    score.marker(126.0, "Reassembly")
    for index, (solo, _label, _solo_count, reentry_count) in enumerate(_ORGANS):
        solo(score, 126.0 + 10.0 * index, reentry_count, 40)

    # The body arrives last: drone, detuned twin, breath, grid — the full bed
    # stands for the final ten beats, and it is the wrongest thing in the
    # piece precisely because every part of it was innocent alone.
    score.marker(196.0, "The Whole Beast")
    score.bend(1, 196.0, _TWIN_BEND)
    score.note(0, 40, 196.0, 9.5, 56)
    score.note(0, 47, 196.0, 9.5, 48)
    score.note(1, 40, 196.0, 9.5, 40)
    score.note(1, 47, 196.0, 9.5, 34)
    score.cc_ramp(0, 11, 196.0, 201.0, 50, 85, steps=5)
    score.cc_ramp(0, 11, 201.0, 206.0, 85, 50, steps=5)
    for step in range(3):
        score.note(9, 36, 196.0 + 4.0 * step, 0.1, 50)
    for step in range(2):
        score.note(9, 42, 198.0 + 4.0 * step, 0.05, 28)
    return score
