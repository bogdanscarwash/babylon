#!/usr/bin/env python3
"""history_breathing — the world map at rest (ambient suite).

CONCEPTUAL BRIEF:
The menu/idle music the estate never had: "the world map — history breathing."
Five voices cycle at COPRIME periods (16, 17, 23, 31, 47 beats), so no two
phrases ever realign inside the piece — a generative, never-repeating surface
from fully deterministic material. The pad inhales and exhales on a 16-beat
breath; the harp turns a 17-beat wheel; strings shade Dorian hope on a
23-beat cycle; the piano Observer speaks every 31 beats; and every 47 beats
the cello sounds just the first interval of the Phi motif (D–A) — the engine
of history idling under the map. Twice, at prime-numbered minutes, a flute
overflies. E Dorian: darkness with the raised sixth of possibility.

TECHNICAL: 60 BPM, 4/4, ~244 beats ≈ 4:04. Warm Pad 89 (breath), Strings 48,
Harp 46, Piano 0, Cello 42, Flute 73. High shared reverb (the map is vast);
spatial spread: harp left, piano right, cello just left of centre.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_HARP_ARP: tuple[int, ...] = (52, 55, 59, 61, 64)  # E3 G3 B3 C#4 E4 — Dorian
#: Every third turn the wheel comes up WRONG — minor third, tritone, flat
#: seventh, flat second: the map is not at peace (Director's dissonance line).
_HARP_SOUR: tuple[int, ...] = (52, 55, 58, 62, 65)  # E3 G3 Bb3 D4 F4


def compose() -> Score:
    score = Score(
        name="history_breathing",
        suite="ambient",
        concept="The world map idling: five coprime cycles breathe over an E drone; "
        "the Phi interval idles beneath; nothing ever aligns twice.",
        bpm=60,
    )
    score.program(0, 89)  # warm pad — the breath
    score.program(1, 48)  # strings — Dorian shade
    score.program(2, 46)  # harp — the wheel
    score.program(3, 0)  # piano — the Observer
    score.program(4, 42)  # cello — the Phi engine, idling
    score.program(5, 73)  # flute — the overflight

    for channel, reverb in ((0, 88), (1, 84), (2, 80), (3, 74), (4, 72), (5, 86)):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(1, 93, 0.0, 35)
    score.cc(2, 10, 0.0, 44)
    score.cc(3, 10, 0.0, 82)
    score.cc(4, 10, 0.0, 54)
    score.cc(5, 10, 0.0, 90)

    score.marker(0.0, "Inhale")
    score.marker(61.0, "The Map")
    score.marker(122.0, "Memory")
    score.marker(183.0, "Exhale")

    # The drone: E2+B2, four abutting breaths (never overlapping — a same-pitch
    # overlap lets the earlier note_off truncate the newer breath; the kit's
    # overlap sentinel now enforces this).
    for segment in range(4):
        start = segment * 61.0
        duration = 60.5 if segment < 3 else 61.5
        score.note(0, 40, start, duration, 58)
        score.note(0, 47, start, duration, 50)
    # The 16-beat breath cycle on the pad's expression.
    for cycle in range(15):
        base = cycle * 16.0
        score.cc_ramp(0, 11, base, base + 8.0, 50, 85, steps=8)
        score.cc_ramp(0, 11, base + 8.0, base + 16.0, 85, 50, steps=8)

    # Strings: 23-beat cycle — Dorian hope alternating with an E-F-B grind
    # (flat-2 against the drone, tritone F-B inside the chord): the map's
    # calm carries its contradiction.
    for cycle in range(10):
        start = 6.0 + 23.0 * cycle
        voicing = (55, 59, 61) if cycle % 2 == 0 else (52, 53, 59)
        score.chord(1, voicing, start, 14.0, 46 + 3 * (cycle % 3), spread=0.15)

    # Harp: 17-beat wheel; every third turn the wheel comes up sour, every
    # fourth inverted — rhythm constant, harmony refusing to settle.
    for cycle in range(14):
        start = 2.0 + 17.0 * cycle
        arp = _HARP_SOUR if cycle % 3 == 2 else _HARP_ARP
        pitches = arp if cycle % 4 != 3 else tuple(reversed(arp))
        velocity = 56 + 6 * (cycle % 3)
        for index, pitch in enumerate(pitches):
            score.note(2, pitch, start + 0.55 * index, 2.2, velocity)

    # Piano Observer: 31-beat cycle — a statement, then a question.
    for cycle in range(7):
        start = 11.0 + 31.0 * cycle
        if cycle % 2 == 0:
            score.note(3, 71, start, 2.5, 58)
            score.note(3, 76, start + 1.5, 3.5, 52)
        else:
            score.note(3, 67, start, 2.5, 56)
            score.note(3, 66, start + 1.5, 3.5, 48)

    # Cello: the Phi interval idling every 47 beats — D2 then A2, unresolved.
    for cycle in range(5):
        start = 5.0 + 47.0 * cycle
        score.note(4, 38, start, 1.2, 66)
        score.note(4, 45, start + 1.2, 2.8, 62)

    # Two flute overflights at prime beats — the world is still alive up there.
    for start in (89.0, 179.0):
        score.note(5, 88, start, 3.5, 42)
        score.cc_ramp(5, 11, start, start + 3.0, 45, 95, steps=6)

    # Seam: the breath ends where it began (CC11 at 50, as at beat 0) so the
    # menu loop closes without a level jump; one last harp tone rings across it.
    score.cc(0, 11, 240.0, 50)
    score.note(2, 64, 238.0, 3.0, 60)
    return score
