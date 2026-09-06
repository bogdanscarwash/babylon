#!/usr/bin/env python3
"""the_ballot — electoral ritual as bourgeois waltz (superstructure suite).

CONCEPTUAL BRIEF:
The estate's planned-but-never-built "bourgeois waltz" absorbed into P25's
electoral machinery. A harpsichord waltz in A minor — elegant, ornamented
with chromatic grace notes, Viennese sophistication hiding rot — dances over
strings and soft organ. The campaign section moves to the relative major
with glockenspiel promises — but the melody is transposed BODILY, minor
shapes over major harmony: the promises are off key, and the Eb-against-B
grind is the point (Director's dissonance line). Every four bars a GAVEL
double-strike lands ON THE DOWNBEAT and drags that bar's harmony back to
A minor: the campaign audibly loses its ground four times.
Then the waltz resumes — and a timpani jackboot enters underneath, one
strike per bar, crescendo from inaudible to hammering, while the waltz keeps
its exact dynamics, POLITELY IGNORING IT. The coda cadences beautifully into
silence; the last sound in the hall is the gavel.

TECHNICAL: 96 BPM, 3/4, 64 bars = 192 beats ≈ 2:00. Harpsichord 6 (the
dance), Strings 48 + Rock Organ 19 (the ballroom), Timpani 47 (the boot),
Glockenspiel 9 (the promises), woodblock gavel on ch 9.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

# Four-bar harmonic loops: (bass_pitch, chord_voicing).
_LOOP_MINOR: tuple[tuple[int, tuple[int, int, int]], ...] = (
    (45, (57, 60, 64)),  # Am
    (45, (57, 60, 64)),  # Am
    (50, (57, 62, 65)),  # Dm
    (40, (56, 59, 62)),  # E7 (G#-B-D)
)
_LOOP_MAJOR: tuple[tuple[int, tuple[int, int, int]], ...] = (
    (48, (60, 64, 67)),  # C
    (43, (59, 62, 67)),  # G
    (45, (57, 60, 64)),  # Am
    (40, (56, 59, 62)),  # E7
)

# Eight-bar melody: per bar, (beat_in_bar, pitch, duration, velocity, grace).
_MELODY: tuple[tuple[tuple[float, int, float, int, bool], ...], ...] = (
    ((0.0, 76, 2.6, 96, False),),
    ((0.0, 72, 1.4, 88, False), (1.5, 71, 1.4, 84, False)),
    ((0.0, 69, 2.6, 90, True),),
    ((0.0, 71, 2.6, 86, True),),
    ((0.0, 74, 1.4, 92, False), (1.5, 72, 1.4, 86, False)),
    ((0.0, 72, 1.4, 84, False), (1.5, 69, 1.4, 80, False)),
    ((0.0, 68, 1.4, 88, True), (1.5, 71, 1.4, 84, False)),
    ((0.0, 69, 2.6, 92, False),),
)


def _accompaniment(
    score: Score, bar: int, loop: tuple[tuple[int, tuple[int, int, int]], ...]
) -> None:
    bass, chord = loop[bar % 4]
    start = bar * 3.0
    score.note(1, bass, start, 1.2, 74)
    for beat in (1.0, 2.0):
        score.chord(1, chord, start + beat, 0.85, 62)
        score.chord(2, chord, start + beat, 0.85, 46)


def _melody_bar(score: Score, bar: int, phrase_bar: int, transpose: int, sparkle: bool) -> None:
    start = bar * 3.0
    for beat, pitch, duration, velocity, grace in _MELODY[phrase_bar]:
        target = pitch + transpose
        if grace:
            score.note(0, target - 1, start + beat - 0.25, 0.22, velocity - 20)
        score.note(0, target, start + beat, duration, velocity)
        if sparkle and beat == 0.0:
            score.note(5, target + 12, start, 1.2, 66)


def compose() -> Score:
    score = Score(
        name="the_ballot",
        suite="superstructure",
        concept="A bourgeois waltz for the electoral ritual: campaign promises snapped "
        "back by the gavel, jackboots crescendoing under a dance that refuses to hear "
        "them, and the gavel as the hall's last word.",
        bpm=96,
        time_signature=(3, 4),
    )
    score.program(0, 6)  # harpsichord — the dance
    score.program(1, 48)  # strings — the ballroom
    score.program(2, 19)  # rock organ — gilt
    score.program(3, 47)  # timpani — the boot
    score.program(5, 9)  # glockenspiel — promises

    score.cc(0, 10, 0.0, 58)
    score.cc(1, 10, 0.0, 64)
    score.cc(2, 10, 0.0, 72)
    score.cc(3, 10, 0.0, 40)
    score.cc(5, 10, 0.0, 84)
    for channel, reverb in ((0, 46), (1, 52), (2, 56), (3, 44), (5, 60)):
        score.cc(channel, 91, 0.0, reverb)

    score.marker(0.0, "The Ballroom")
    score.marker(48.0, "The Campaign")
    score.marker(96.0, "Jackboots (unnoticed)")
    score.marker(168.0, "Adjournment")

    # Section A — bars 0-15: the waltz established.
    for bar in range(16):
        _accompaniment(score, bar, _LOOP_MINOR)
        _melody_bar(score, bar, bar % 8, 0, sparkle=False)

    # Section B — bars 16-31: the campaign, off-key promises over the
    # relative major; every fourth downbeat the gavel double-strikes and
    # forces the bar back onto A minor — ground audibly lost, four times.
    for bar in range(16, 32):
        snapped = bar % 4 == 0 and bar > 16
        _accompaniment(score, bar, _LOOP_MINOR if snapped else _LOOP_MAJOR)
        _melody_bar(score, bar, bar % 8, 3, sparkle=not snapped)
        if snapped:
            score.note(9, 37, bar * 3.0, 0.08, 112)
            score.note(9, 77, bar * 3.0 + 0.03, 0.08, 106)

    # Section A' — bars 32-55: one more gavel seals the campaign shut, the
    # waltz resumes; the jackboot enters and grows three velocity points a
    # bar; the dance's dynamics never move.
    score.note(9, 37, 96.0, 0.08, 116)
    score.note(9, 77, 96.03, 0.08, 108)
    for bar in range(32, 56):
        _accompaniment(score, bar, _LOOP_MINOR)
        _melody_bar(score, bar, bar % 8, 0, sparkle=False)
        boot = min(115, 46 + 3 * (bar - 32))
        score.note(3, 40, bar * 3.0, 0.5, boot)

    # Coda — bars 56-63: an elegant cadence, then the empty hall.
    for bar in range(56, 60):
        _accompaniment(score, bar, _LOOP_MINOR)
    score.chord(1, (57, 60, 64, 71), 180.0, 8.0, 68, spread=0.2)  # Am(add9)
    score.cc_ramp(1, 11, 180.0, 188.0, 100, 40, steps=8)
    for index, pitch in enumerate((76, 72, 71, 69, 64, 60)):  # harpsichord descends
        score.note(0, pitch, 180.0 + 0.7 * index, 0.6, 78 - 6 * index)
    score.note(9, 37, 190.0, 0.08, 118)  # the last word —
    score.note(9, 77, 190.03, 0.08, 110)  # — is the gavel's
    return score
