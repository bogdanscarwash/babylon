#!/usr/bin/env python3
"""unequal_exchange — the sigma-gradient, audible (periphery suite).

CONCEPTUAL BRIEF:
The planned-but-never-built "periphery lament", grounded in the Spectrum of
Unequal Exchange. A modal D-Mixolydian melody lives on the LEFT of the
stereo field — sitar and shanai, richly ornamented, human — over a taiko
heartbeat. After each statement, the RIGHT side answers with the SAME
melody, stripped and pressed down into the ledger's register: ornaments
gone, rhythm stiffened, harpsichord and muted trumpet — and MIS-TRANSCRIBED:
two of its notes are copied a semitone wrong, and stay wrong in every
round. The copy is not the song; it is the invoice. Round by round,
velocity TRANSFERS — the periphery states its phrase 11 points quieter as
the core's wrong copy grows 11 points louder. In the coda the periphery
breaks off mid-phrase; the core keeps looping its off-key copy and drifts
from the right edge INTO THE CENTRE — the wrong version becomes "the"
version. Then, at last, alone: one low sitar D at the edge of hearing,
with nothing left to accompany it.

TECHNICAL: 88 BPM, 4/4, ~258 beats ≈ 2:56. Sitar 104 + Shanai 111 (pan
20/28), Harpsichord 6 + Muted Trumpet 59 (pan 100/108), Taiko 116 (pan 24).
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_PHRASE: tuple[int, ...] = (62, 65, 67, 69, 67, 65, 64, 62)  # D Mixolydian
_ORNAMENTED: tuple[int, ...] = (0, 3, 6)  # indices that receive grace notes
_MISCOPIED: tuple[int, ...] = (2, 5)  # indices the ledger transcribes a semitone wrong


def _periphery_phrase(score: Score, base: float, velocity: int, notes: int) -> None:
    for index in range(notes):
        pitch = _PHRASE[index]
        start = base + 2.0 * index
        if index in _ORNAMENTED:
            score.note(0, pitch + 2, start - 0.2, 0.18, max(1, velocity - 22))
        score.note(0, pitch, start, 1.7, velocity)
        if index % 2 == 0:
            score.note(1, pitch + 12, start + 0.1, 1.4, max(1, velocity - 26))


def _core_copy(score: Score, base: float, velocity: int) -> None:
    for index, pitch in enumerate(_PHRASE):
        start = base + 2.0 * index
        copied = pitch + 1 if index in _MISCOPIED else pitch
        score.note(2, copied, start, 0.9, velocity)
        if index in (0, 4):
            score.note(3, copied, start, 1.6, max(1, velocity - 18))


def compose() -> Score:
    score = Score(
        name="unequal_exchange",
        suite="periphery",
        concept="A periphery melody and its mis-transcribed core copy trade velocity "
        "round by round — value transferring left to right — until the wrong copy "
        "takes the centre and the original is one low string at the edge of hearing.",
        bpm=88,
    )
    score.program(0, 104)  # sitar — the periphery voice
    score.program(1, 111)  # shanai — its shadow
    score.program(2, 6)  # harpsichord — the core ledger
    score.program(3, 59)  # muted trumpet — the core's polite fanfare
    score.program(4, 116)  # taiko — the heartbeat

    score.cc(0, 10, 0.0, 20)
    score.cc(1, 10, 0.0, 28)
    score.cc(2, 10, 0.0, 100)
    score.cc(3, 10, 0.0, 108)
    score.cc(4, 10, 0.0, 24)
    for channel, reverb in ((0, 62), (1, 66), (2, 24), (3, 40), (4, 55)):
        score.cc(channel, 91, 0.0, reverb)
    score.cc(0, 93, 0.0, 45)
    score.cc(2, 93, 0.0, 0)

    score.marker(0.0, "The Village")
    score.marker(24.0, "First Extraction")
    score.marker(104.0, "The Ledger Turns")
    score.marker(168.0, "Terms of Trade")
    score.marker(208.0, "The Center Moves")
    score.marker(252.0, "What Is Left")

    # Intro: heartbeat and a drone fifth.
    for beat in range(4):
        score.note(4, 48, 2.0 * beat, 0.5, 92)
    score.note(0, 50, 0.0, 8.0, 70)

    # Five rounds of exchange: periphery statement, drum turn, wrong copy.
    for round_index in range(5):
        base = 8.0 + 40.0 * round_index
        periphery_velocity = 96 - 11 * round_index
        core_velocity = 58 + 11 * round_index
        _periphery_phrase(score, base, periphery_velocity, notes=8)
        for beat in range(4):
            score.note(4, 48, base + 16.0 + 2.0 * beat, 0.5, max(30, 92 - 10 * round_index))
        _core_copy(score, base + 24.0, core_velocity)

    # Coda: the periphery breaks off mid-phrase; the wrong copy loops and
    # takes the centre; then — alone — the last string.
    _periphery_phrase(score, 208.0, 41, notes=6)
    _core_copy(score, 222.0, 102)
    _core_copy(score, 234.0, 102)
    score.cc(2, 10, 222.0, 88)
    score.cc(2, 10, 230.0, 76)
    score.cc(2, 10, 238.0, 64)
    score.note(0, 50, 252.0, 6.0, 38)
    return score
