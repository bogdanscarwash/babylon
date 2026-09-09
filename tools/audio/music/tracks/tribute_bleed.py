#!/usr/bin/env python3
"""tribute_bleed — extraction heard as bleeding (entity suite).

CONCEPTUAL BRIEF (#641):
The unequal-exchange velocity transfer turned vertical: a warm human voice
(choir over low strings) and a cold machine voice (the estate's harpsichord)
exchange a fixed loudness budget by the symmetric ±11-per-round rule — five
rounds, the human phrase stated 11 points quieter each time as the machine's
WRONG copy grows 11 points louder. The copy is a mis-transcription (two of its
eight notes shifted a semitone, the `_MISCOPIED` device) and it RUSHES — the
machine states the phrase in three-quarter time because it does not breathe.
Under every round the string body is drawn down a four-point glide: at each
round's entry the wheel resets shallow — the body keeps trying to rise — and
is pulled to a floor 344 units deeper than the last (−1024 at the first
round's floor to −2400 at the fifth's; the coda holds −2570, the deepest of
all). The value does not merely get quieter as it is drawn off, it goes FLAT
as it leaves — and it never gets back to pitch. CC93 solidarity falls with
it, and the bend makes the flattening audible. Coda: the giver breaks off
mid-phrase and is left as one low string at velocity 38 while the taker loops
its copy twice, intact, at 102.

Cue binding (non-event): the Φ / tribute lens, the imperial-rent view, any
sustained extraction-focused screen — see assets/CUE_MAP.md.

TECHNICAL: 60 BPM, 4/4, ~161 beats ≈ 2:41. Choir 52 (the voice), Strings 48
(the body), Harpsichord 6 (the machine), low tom (the ledger's drum turn).
Human panned centre-left, machine hard right — the geography of the transfer.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

#: The phrase: E minor descending from E4 to E3 — a whole octave given away.
_PHRASE: tuple[int, ...] = (64, 62, 60, 59, 57, 55, 53, 52)
#: The machine's mis-transcription: indices shifted a semitone in the copy.
_MISCOPIED: tuple[int, ...] = (2, 5)
#: The body's sag per round: −1024 at round 0 deepening to −2400 at round 4.
_SAG_BASE: int = -1024
_SAG_STEP: int = -344


def _human_phrase(score: Score, start: float, velocity: int) -> None:
    for index, pitch in enumerate(_PHRASE):
        score.note(0, pitch, start + 1.0 * index, 0.9, velocity)


def _machine_copy(score: Score, start: float, velocity: int) -> None:
    for index, pitch in enumerate(_PHRASE):
        wrong = pitch + 1 if index in _MISCOPIED else pitch
        score.note(2, wrong + 12, start + 0.75 * index, 0.6, velocity)


def compose() -> Score:
    score = Score(
        name="tribute_bleed",
        suite="entity",
        concept="Unequal exchange, vertical: a choir gives eleven points of "
        "velocity per round to a harpsichord copy that is wrong in two places "
        "and rushes, while the string body under it goes flat as the value "
        "leaves. The giver ends broken off mid-phrase; the taker loops.",
        bpm=60,
    )
    score.program(0, 52)  # choir — the voice
    score.program(1, 48)  # strings — the body it is drawn from
    score.program(2, 6)  # harpsichord — the machine

    score.cc(0, 91, 0.0, 70)
    score.cc(1, 91, 0.0, 62)
    score.cc(2, 91, 0.0, 22)
    score.cc(0, 10, 0.0, 50)
    score.cc(1, 10, 0.0, 46)
    score.cc(2, 10, 0.0, 84)

    score.marker(0.0, "The Wage")
    score.marker(62.0, "The Rate Rises")
    score.marker(146.0, "The Ledger Closes")

    # Five rounds of exchange. Each round: the body enters, the voice states
    # the phrase, the ledger's drum turns, the machine states its wrong copy.
    for round_index in range(5):
        base = 6.0 + 28.0 * round_index
        human_velocity = 92 - 11 * round_index
        machine_velocity = 56 + 11 * round_index

        # The body: E2+B2 under the whole round. At each round's entry the
        # wheel resets shallow (the body keeps trying to rise), then four
        # points glide it down to the round's floor — a sag the ear follows,
        # not a staircase (SS2.1). Every bend lands under sounding strings.
        floor = _SAG_BASE + _SAG_STEP * round_index
        score.note(1, 40, base - 1.0, 24.0, 42)
        score.note(1, 47, base - 1.0, 24.0, 36)
        score.bend(1, base - 0.5, -341)
        score.bend(1, base + 4.0, -341 + (floor + 341) // 3)
        score.bend(1, base + 9.0, -341 + 2 * (floor + 341) // 3)
        score.bend(1, base + 14.0, floor)

        # Solidarity falls as the value leaves — CC93 renders on FluidR3, and
        # the bend above makes the same flattening audible in pitch.
        score.cc(0, 93, base, max(0, 55 - 11 * round_index))

        _human_phrase(score, base, human_velocity)

        # The ledger's drum turn: four low-tom strokes, fading with the giver.
        for beat in range(4):
            score.note(9, 41, base + 10.0 + 2.0 * beat, 0.5, max(30, 88 - 10 * round_index))

        _machine_copy(score, base + 18.0, machine_velocity)

    # Coda: the giver breaks off mid-phrase — four notes and silence — at 38;
    # one low string is all that remains of the body. The taker loops its
    # wrong copy twice, intact, at 102. The bend under the last string is
    # driven to −2570 — deeper than any round reached: what is left is the
    # flattest thing in the piece, and it stays there.
    coda = 146.0
    for index, pitch in enumerate(_PHRASE[:4]):
        score.note(0, pitch, coda + 1.0 * index, 0.8, 38)
    score.note(1, 40, coda + 2.0, 8.0, 38)
    score.bend(1, coda + 3.0, -2570)
    _machine_copy(score, coda + 2.0, 102)
    _machine_copy(score, coda + 8.5, 102)
    return score
