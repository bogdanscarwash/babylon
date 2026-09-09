#!/usr/bin/env python3
"""the_reform_ceiling — the PASOK bleed as musical form (superstructure suite).

CONCEPTUAL BRIEF:
P25's reform ceiling and constant-ratio bleed, composed literally — with
counter-motion. A hopeful five-note ascent states itself twelve times, each
cycle transposed ONE SEMITONE HIGHER — reform always reaching upward — while
its velocity bleeds by a constant ratio (×0.94 per cycle: slow enough that
the theme survives to be deformed). There is a CEILING: any pitch that would
cross C5 is REFLECTED back down (p → 144 − p), so from cycle four onward the
ascent audibly flattens into a rise-and-fall arch — climbing and collapsing
in the same gesture. And the room fights back: the ceiling rings as a
TRITONE DYAD (C5+F#5 — the ceiling is not a note, it is a wrongness) that
strikes HARDER each cycle as the theme weakens, while the contrabass floor
rises underneath it. The theme also drifts progressively off key against
the floor, which never transposes: by the late cycles the reformer is
playing in a key the room no longer contains. The outro is the ceiling
ringing its tritone over the loudest floor note in the piece. The geometry
was the message; the geometry is dissonant.

TECHNICAL: 84 BPM, 4/4, ~166 beats ≈ 2:00. Piano 0 (the reformer), Strings
48 (the base, echoing fainter), Tubular Bells 14 (the ceiling, tritone),
Contrabass 43 (the floor, rising). First fold at cycle 4.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_CEILING: int = 72  # C5 — the reform ceiling
_ASCENT: tuple[int, ...] = (57, 60, 64, 67, 69)  # A3 C4 E4 G4 A4
_FALL: tuple[int, ...] = (64, 60)  # E4 C4


def _fold(pitch: int) -> int:
    """Reflect any pitch above the ceiling back below it."""
    return 2 * _CEILING - pitch if pitch > _CEILING else pitch


def compose() -> Score:
    score = Score(
        name="the_reform_ceiling",
        suite="superstructure",
        concept="A rising theme transposed up each cycle while its velocity bleeds at a "
        "constant ratio; pitches above C5 reflect downward; the ceiling rings as a "
        "hardening tritone dyad and the floor rises — the theme fades between them, "
        "increasingly off key against a room that never moves.",
        bpm=84,
    )
    score.program(0, 0)  # piano — the reformer
    score.program(1, 48)  # strings — the base, echoing
    score.program(2, 14)  # tubular bells — the ceiling
    score.program(3, 43)  # contrabass — the floor
    score.cc(0, 91, 0.0, 44)
    score.cc(1, 91, 0.0, 55)
    score.cc(1, 93, 0.0, 45)
    score.cc(2, 91, 0.0, 68)
    score.cc(3, 91, 0.0, 40)

    score.marker(0.0, "The Floor")
    score.marker(8.0, "Aspiration")
    score.marker(56.0, "First Contact")
    score.marker(104.0, "The Bleed")
    score.marker(156.0, "What Remains")

    # Intro: the ceiling names itself — as a tritone; the floor names itself.
    score.note(2, _CEILING, 0.0, 5.0, 74)
    score.note(2, _CEILING + 6, 0.0, 5.0, 62)
    score.note(3, 33, 0.0, 7.0, 56)

    for cycle in range(12):
        base = 8.0 + 12.0 * cycle
        velocity = round(102 * 0.94**cycle)
        folded_this_cycle = False
        # The ascent and fall, transposed and folded.
        for index, pitch in enumerate(_ASCENT):
            raw = pitch + cycle
            if raw > _CEILING and not folded_this_cycle:
                # The ceiling strikes HARDER as the theme weakens.
                bell_velocity = min(110, 60 + 7 * (cycle - 4))
                score.note(2, _CEILING, base + float(index), 4.0, bell_velocity)
                score.note(2, _CEILING + 6, base + float(index) + 0.02, 4.0, bell_velocity - 10)
                folded_this_cycle = True
            score.note(0, _fold(raw), base + float(index), 0.95, velocity)
        for index, pitch in enumerate(_FALL):
            score.note(0, _fold(pitch + cycle), base + 5.0 + index, 0.9, max(1, velocity - 6))
        # The base echoes the opening three notes, fainter than the reformer.
        # (Articulation stays under the 1-beat grid: the fold can map two
        # consecutive echo pitches onto the SAME note, and a longer duration
        # would truncate the repeat — the overlap sentinel caught this.)
        for index, pitch in enumerate(_ASCENT[:3]):
            score.note(1, _fold(pitch + cycle), base + 7.0 + index, 0.95, max(20, velocity - 18))
        # The floor, rising as the theme bleeds — never transposing with it.
        score.note(3, 33, base, 10.0, 52 + round(2.5 * cycle))

    # Outro: the ceiling rings its tritone over the loudest floor of the piece.
    score.note(2, _CEILING, 156.0, 9.0, 96)
    score.note(2, _CEILING + 6, 156.02, 9.0, 84)
    score.note(3, 33, 160.0, 5.5, 84)
    score.cc_ramp(2, 11, 157.0, 164.0, 110, 45, steps=6)
    return score
