#!/usr/bin/env python3
"""red_dawn — REVOLUTIONARY_VICTORY (endgame suite).

CONCEPTUAL BRIEF:
The one full resolution in the estate, and it settles an eight-year-old
debt: the Phi theme has always ended on the dominant A — "unresolved,
waiting". Here the waiting ends — and the transformation is AUDIBLE,
because the untransformed motif is stated first: twice under the struggle,
the cello grinds the original D–A–D–F beneath the solidarity
call-and-response and the accelerating timpani roll. After the break into
E major and a hymn whose trumpet descant genuinely climbs pass by pass,
the coda re-states the motif — and the F, the surplus note, tries ONCE to
enter and is cut off mid-attack; from then on the loop runs D–A–D–A,
extraction closed on itself. The bass holds the old dominant one last
time... and resolves to E. The final sonority keeps its promises the hard
way: the top voices are an OPEN fifth-and-octave (E5–B5–E6, no third), the
third lives only down in the brass, a flat-seventh D smoulders in the
middle of the chord (triumph with the grit still in it — the work is only
now possible), and the root is planted in the cello's E2.

TECHNICAL: 112 BPM, 4/4, ~184 beats ≈ 1:39. Cello 42, Violin 40, Brass 61,
Choir 52, Trumpet 56, Tremolo 44, Glockenspiel 9, Timpani 47, Orch Hit 55.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_HYMN: tuple[tuple[int, int, int], ...] = (
    (64, 68, 71),  # E
    (61, 66, 69),  # A (first inversion)
    (61, 64, 68),  # C#m
    (59, 63, 66),  # B
)
_ROLL: tuple[tuple[float, int], ...] = (
    (24.0, 60),
    (30.0, 68),
    (35.5, 76),
    (40.0, 84),
    (43.5, 94),
    (45.5, 102),
    (46.75, 110),
    (47.5, 116),
)
#: Trumpet descant per hymn pass — the line CLIMBS as it loudens.
_DESCANTS: tuple[tuple[int, ...], ...] = (
    (71, 73, 76),
    (71, 73, 76),
    (71, 73, 76, 79),
    (71, 73, 76, 79),
    (71, 73, 76, 79, 83),
)


def compose() -> Score:
    score = Score(
        name="red_dawn",
        suite="endgame",
        concept="The Phi motif stated raw, then transformed — the surplus F cut off "
        "mid-attack, the loop closing D-A-D-A — and the theme's eternal dominant "
        "finally resolving to an E chord whose top is an open fifth-and-octave with "
        "a flat-seventh smouldering inside: triumph, grit included.",
        bpm=112,
    )
    score.program(0, 42)  # cello — the call, the motif, the root
    score.program(1, 40)  # violin — the answer
    score.program(2, 61)  # brass — arrival
    score.program(3, 52)  # choir — the many
    score.program(4, 56)  # trumpet — the descant
    score.program(5, 44)  # tremolo strings — the struggle
    score.program(6, 9)  # glockenspiel — light
    score.program(7, 47)  # timpani — history
    score.program(8, 55)  # orchestra hit — the break

    for channel, reverb in (
        (0, 55),
        (1, 55),
        (2, 62),
        (3, 85),
        (4, 70),
        (5, 55),
        (6, 75),
        (7, 50),
        (8, 60),
    ):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(0, 93, 0.0, 70)
    score.cc(1, 93, 0.0, 70)
    score.cc(3, 93, 0.0, 100)
    score.cc(0, 10, 0.0, 46)
    score.cc(1, 10, 0.0, 82)

    score.marker(0.0, "The Struggle (Phi, Raw)")
    score.marker(48.0, "The Break / The Hymn")
    score.marker(128.0, "Phi, Transformed")
    score.marker(160.0, "The Waiting Ends")

    # The struggle: tremolo bed, rising.
    score.note(5, 52, 0.0, 48.0, 74)
    score.note(5, 55, 0.0, 48.0, 70)
    score.cc_ramp(5, 11, 0.0, 47.0, 45, 118, steps=12)
    # Call and response, twice, then together — with the RAW Phi motif
    # (D-A-D-F) grinding underneath each answer: the antecedent the coda
    # will transform.
    for base, velocity in ((4.0, 88), (20.0, 98)):
        for index, pitch in enumerate((52, 55, 57)):
            score.note(0, pitch, base + 2.0 * index, 1.8, velocity)
        for index, pitch in enumerate((71, 74, 76)):
            score.note(1, pitch, base + 8.0 + 2.0 * index, 1.8, velocity - 4)
    for motif_base, motif_velocity in ((12.0, 80), (28.0, 84)):
        for index, pitch in enumerate((38, 45, 50, 41)):
            score.note(0, pitch, motif_base + 2.0 * index, 1.6, motif_velocity)
    for index, (low, high) in enumerate(((52, 67), (55, 71), (57, 76))):
        score.note(0, low, 36.0 + 2.0 * index, 1.8, 104)
        score.note(1, high, 36.0 + 2.0 * index, 1.8, 100)
    # The accelerating roll.
    for beat, velocity in _ROLL:
        score.note(7, 40, beat, 0.3, velocity)

    # The break.
    score.note(8, 52, 48.0, 0.5, 124)
    score.note(8, 64, 48.0, 0.5, 110)
    score.note(9, 49, 48.0, 0.5, 102)

    # The hymn: five passes; the descant climbs, the timpani swells.
    for hymn_pass in range(5):
        base = 48.0 + 16.0 * hymn_pass
        for bar, chord in enumerate(_HYMN):
            score.chord(2, chord, base + 4.0 * bar, 3.8, 106, spread=0.05)
        score.chord(3, (76, 80, 83), base, 15.5, 78 + 3 * hymn_pass, spread=0.4)
        score.cc_ramp(3, 11, base, base + 14.0, 70, 100 + 5 * hymn_pass, steps=6)
        descant = _DESCANTS[hymn_pass]
        for index, pitch in enumerate(descant):
            duration = 2.5 if index == len(descant) - 1 else 1.8
            score.note(4, pitch, base + 8.0 + 1.5 * index, duration, 96 + 4 * hymn_pass)
        score.note(6, 88, base, 2.0, 78 + 5 * hymn_pass)
        for pulse in range(4):
            score.note(7, 40, base + 4.0 * pulse, 0.4, 84 + 5 * hymn_pass)

    # Phi, transformed: the F tries once to enter — and is cut off mid-attack;
    # from then on the loop closes on itself: D-A-D-A, nothing skimmed.
    score.note(0, 38, 128.0, 1.8, 92)
    score.note(0, 45, 130.0, 1.8, 92)
    score.note(0, 50, 132.0, 1.8, 92)
    score.note(0, 41, 134.0, 0.35, 74)  # the surplus note, cut
    score.note(0, 45, 134.5, 1.3, 94)
    score.chord(3, (69, 73, 76), 128.0, 7.5, 62, spread=0.3)
    for cycle in range(1, 4):
        base = 128.0 + 8.0 * cycle
        for index, pitch in enumerate((38, 45, 50, 45)):
            score.note(0, pitch, base + 2.0 * index, 1.8, 92 + 2 * cycle)
        score.chord(3, (69, 73, 76), base, 7.5, 62 + 4 * cycle, spread=0.3)

    # The old dominant, held one last time... and resolved — with the root
    # planted, the top open, and the flat-seventh smouldering inside.
    score.note(0, 33, 160.0, 4.0, 96)
    score.cc_ramp(3, 11, 160.0, 164.0, 80, 95, steps=4)
    score.cc(3, 93, 164.0, 60)
    score.cc(2, 11, 164.0, 105)
    score.note(0, 40, 164.0, 16.0, 96)  # the root, E2
    score.chord(2, (52, 56, 59, 64, 68, 71), 164.0, 16.0, 100, spread=0.06)
    score.note(2, 62, 164.3, 15.7, 72)  # D — the flat-seventh grit
    score.chord(3, (76, 83, 88), 164.0, 16.0, 84, spread=0.3)
    score.note(5, 76, 164.0, 16.0, 76)
    score.note(5, 83, 164.0, 16.0, 72)  # the open fifth on top
    score.note(6, 88, 164.5, 2.5, 88)
    score.note(6, 88, 172.0, 3.0, 84)
    score.note(7, 40, 164.0, 1.2, 104)
    score.note(7, 40, 172.0, 1.2, 96)
    score.note(9, 49, 164.0, 0.6, 94)
    score.cc_ramp(2, 11, 172.0, 180.0, 120, 60, steps=8)
    return score
