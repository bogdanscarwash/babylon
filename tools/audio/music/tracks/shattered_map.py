#!/usr/bin/env python3
"""shattered_map — FRAGMENTED_COLLAPSE (endgame suite).

CONCEPTUAL BRIEF:
A fugue that cannot work, for a country that no longer can. The subject
enters four times, as fugue subjects should — but each entry is in a key
UNRELATABLE to the others AND at its own tempo of speech: piano in E minor
at one note per beat on the far left; strings a tritone away in Bb,
slower (1.25 beats per note), far right, drifting flat; harpsichord in C#
at 0.875 beats per note in the centre; horn one poisoned semitone up in F,
slowest of all, at mid-left. Entries land OFF the shared grid (0 / 16.5 /
32.25 / 48.75), so the four voices phase against each other — real
polyphony, really incompatible: discordant, chaotic, off key, and each
voice strictly rhythmic within itself. Between beats 49 and 96 all four
grind at once — the whole map, all claims live. Then subtraction: the horn
stops, the piano stops, the strings stop, and the harpsichord is left
restating its subject whole, then five notes of it, then two, and finally
one note that stops on nothing — the last cartographer, drawing a border
no one is on the other side of.

TECHNICAL: 88 BPM, 4/4, ~176 beats ≈ 2:00. Piano 0 (E, pan 14), Strings 48
(Bb, pan 110), Harpsichord 6 (C#, pan 64), French Horn 60 (F, pan 40).
No percussion: this is cartography, not war.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_SUBJECT: tuple[int, ...] = (64, 67, 66, 64, 59, 60, 62, 64)
_COUNTER: tuple[int, ...] = (64, 61, 62, 64, 69, 68, 66, 64)  # inversion about E4

#: (channel, transpose, entry_beat, note_rate, loop_count) — each voice has
#: its own rate of speech and an off-grid entry, so nothing ever locks step.
_VOICES: tuple[tuple[int, int, float, float, int], ...] = (
    (0, 0, 0.0, 1.0, 7),  # piano, E minor — ends ~112
    (1, 6, 16.5, 1.25, 5),  # strings, Bb — the tritone country, ends ~116.5
    (2, 9, 32.25, 0.875, 7),  # harpsichord, C# — will remain, loops to ~130
    (3, 1, 48.75, 1.5, 2),  # horn, F — the poisoned neighbour, first to go (~97)
)


def compose() -> Score:
    score = Score(
        name="shattered_map",
        suite="endgame",
        concept="Four fugue entries in mutually unrelatable keys, each at its own note "
        "rate and off-grid entry, phase across a panned map; voices subtract until the "
        "harpsichord restates its subject in shrinking pieces and stops on one note.",
        bpm=88,
    )
    score.program(0, 0)
    score.program(1, 48)
    score.program(2, 6)
    score.program(3, 60)

    score.cc(0, 10, 0.0, 14)
    score.cc(1, 10, 0.0, 110)
    score.cc(2, 10, 0.0, 64)
    score.cc(3, 10, 0.0, 40)
    for channel, reverb in ((0, 45), (1, 55), (2, 14), (3, 50)):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(2, 93, 0.0, 0)

    score.marker(0.0, "One Country")
    score.marker(16.5, "Two")
    score.marker(32.25, "Three")
    score.marker(48.75, "Four")
    score.marker(96.0, "Subtraction")
    score.marker(132.0, "The Last Cartographer")

    # Each voice loops subject + inverted counter at its own rate.
    for channel, transpose, entry, rate, loop_count in _VOICES:
        for loop_index in range(loop_count):
            loop_start = entry + 16.0 * rate * loop_index
            for index, pitch in enumerate(_SUBJECT):
                score.note(channel, pitch + transpose, loop_start + rate * index, 0.9 * rate, 84)
            for index, pitch in enumerate(_COUNTER):
                score.note(
                    channel,
                    pitch + transpose,
                    loop_start + rate * (8.0 + index),
                    0.9 * rate,
                    60,
                )

    # The strings drift flat while they last.
    score.bend(1, 80.0, -350)

    # The last cartographer: whole, then five notes, then two, then one.
    for index, pitch in enumerate(_SUBJECT):
        score.note(2, pitch + 9, 132.0 + 0.875 * index, 0.8, 78)
    for index, pitch in enumerate(_SUBJECT[:5]):
        score.note(2, pitch + 9, 148.0 + 0.875 * index, 0.8, 66)
    for index, pitch in enumerate(_SUBJECT[:2]):
        score.note(2, pitch + 9, 164.0 + 0.875 * index, 0.8, 54)
    score.note(2, 61, 172.0, 1.0, 44)
    return score
