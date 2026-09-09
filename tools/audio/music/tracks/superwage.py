#!/usr/bin/env python3
"""superwage — comfort on top of the grind (periphery suite).

CONCEPTUAL BRIEF:
The aristocracy of labour, W_c > V_c. On the surface: an electric piano and
strings turn a warm A-major loop with a gentle vibraphone motif — pleasant,
consumer, untroubled. Underneath: a contrabass/timpani/tremolo substrate in
E Phrygian, nearly inaudible at first, rising eight expression points per
loop. The surface's dynamics NEVER change — comfort does not acknowledge
what funds it. By late loops the substrate grinds a minor second (F against
E) and then the tritone under the pretty chords, fully surfaced, and the
music is revealed as two musics that were always playing. The surface's own
pretty motif carries a sustained minor second against the E chord every loop
(A4 held over G#4) — the comfort was never innocent, merely unexamined
(Director's dissonance line: the wrongness is structural, not decorative).
The surface is interrupted mid-bar; the substrate grinds alone once at full
strength — minor ninth plus tritone, in the octave where the patches
actually speak; then the opening chord returns over its release, soft, as
if nothing happened — loop-ready, which is the horror.

TECHNICAL: 72 BPM, 4/4, ~216 beats ≈ 3:00. EP 4, Strings 48, Vibraphone 11
(surface); Contrabass 43, Timpani 47, Tremolo Strings 44 (substrate).
"""

from __future__ import annotations

from tools.audio.music.composer import Score

# Surface loop: A — D — F#m — E, one chord per 4-beat bar.
_SURFACE: tuple[tuple[int, tuple[int, int, int]], ...] = (
    (45, (61, 64, 69)),  # A
    (50, (62, 66, 69)),  # D
    (54, (61, 66, 69)),  # F#m
    (52, (64, 68, 71)),  # E
)
_MOTIF: tuple[tuple[float, int, float], ...] = (
    (0.5, 76, 1.2),
    (2.0, 73, 0.8),
    (4.5, 74, 1.2),
    (8.5, 71, 1.6),
    (12.5, 69, 2.4),
)


def compose() -> Score:
    score = Score(
        name="superwage",
        suite="periphery",
        concept="A warm A-major surface with unchanging dynamics over an E-Phrygian "
        "substrate that rises eight expression points per loop until the grind is fully "
        "surfaced; interruption, one bare grind, then the comfort loop resumes as if "
        "nothing happened.",
        bpm=72,
    )
    score.program(0, 4)  # electric piano — the standard of living
    score.program(1, 48)  # strings — upholstery
    score.program(2, 11)  # vibraphone — the pleasant thought
    score.program(3, 43)  # contrabass — the substrate
    score.program(4, 47)  # timpani — the substrate's pulse
    score.program(5, 44)  # tremolo strings — the grind

    score.cc(0, 91, 0.0, 48)
    score.cc(1, 91, 0.0, 55)
    score.cc(1, 93, 0.0, 40)
    score.cc(2, 91, 0.0, 58)
    score.cc(3, 91, 0.0, 30)
    score.cc(4, 91, 0.0, 30)
    score.cc(5, 91, 0.0, 34)
    score.cc(5, 71, 0.0, 92)
    score.cc(0, 10, 0.0, 70)
    score.cc(2, 10, 0.0, 78)
    score.cc(3, 10, 0.0, 50)
    score.cc(5, 10, 0.0, 44)

    score.marker(0.0, "Comfort")
    score.marker(64.0, "Something Underneath")
    score.marker(128.0, "The Substrate Rises")
    score.marker(192.0, "Interruption")

    # Twelve 16-beat surface loops with constant dynamics.
    for loop in range(12):
        base = 16.0 * loop
        for bar, (bass, chord) in enumerate(_SURFACE):
            start = base + 4.0 * bar
            score.note(0, bass, start, 3.6, 72)
            score.chord(0, chord, start, 1.8, 78, spread=0.06)
            score.chord(0, chord, start + 2.0, 1.6, 70, spread=0.06)
            score.chord(1, chord, start, 3.8, 56, spread=0.2)
        for beat, pitch, duration in _MOTIF:
            score.note(2, pitch, base + beat, duration, 68)

    # The substrate: same twelve loops, expression rising +8 per loop.
    for loop in range(12):
        base = 16.0 * loop
        expression = min(113, 25 + 8 * loop)
        score.cc(3, 11, base, expression)
        score.cc(4, 11, base, expression)
        score.cc(5, 11, base, expression)
        score.note(3, 28, base, 15.0, 74)
        for pulse in range(4):
            score.note(4, 40, base + 4.0 * pulse, 0.4, 70)
        if loop >= 4:
            score.note(5, 41, base + 2.0, 6.0, 62)  # F2 against the surface's E
        if loop >= 8:
            score.note(5, 46, base + 10.0, 5.0, 66)  # Bb2 — the tritone surfaces

    # Interruption: the surface's next chord is cut mid-bar.
    score.note(0, 45, 192.0, 1.0, 72)
    score.chord(0, (61, 64, 69), 192.0, 0.9, 78, spread=0.06)
    # The substrate, alone, at full strength — minor ninth plus tritone, in
    # the octave where FluidR3's tremolo patch actually speaks.
    score.cc(3, 11, 194.0, 120)
    score.cc(5, 11, 194.0, 120)
    score.note(3, 28, 194.0, 8.0, 92)
    score.note(5, 41, 194.0, 8.0, 84)
    score.note(5, 46, 196.0, 6.0, 76)
    score.note(4, 40, 194.0, 0.5, 104)
    score.note(4, 40, 197.0, 0.5, 96)
    # ...and the comfort resumes over the grind's release — no dead air, no
    # acknowledgement. As if nothing happened.
    score.chord(0, (61, 64, 69), 202.0, 6.0, 66, spread=0.08)
    score.chord(1, (61, 64, 69), 202.0, 8.0, 48, spread=0.25)
    score.cc_ramp(1, 11, 204.0, 210.0, 100, 45, steps=6)
    return score
